//! The Client contains information about a single bot's token, as well
//! as event handlers. Dispatching events to configured handlers and starting
//! the shards' connections are handled directly via the client. In addition,
//! the `http` module and `Cache` are also automatically handled by the
//! Client module for you.
//!
//! A [`Context`] is provided for every handler.
//!
//! The `http` module is the lower-level method of interacting with the Discord
//! REST API. Realistically, there should be little reason to use this yourself,
//! as the Context will do this for you. A possible use case of using the `http`
//! module is if you do not have a Cache, for purposes such as low memory
//! requirements.
//!
//! Click [here][Client examples] for an example on how to use a `Client`.
//!
//! [`Client`]: struct.Client.html#examples
//! [`Context`]: struct.Context.html
//! [Client examples]: struct.Client.html#examples

pub mod bridge;

mod context;
#[cfg(feature = "gateway")]
mod dispatch;
mod error;
#[cfg(feature = "gateway")]
mod event_handler;
#[cfg(feature = "gateway")]
mod extras;

pub use self::{
    context::Context,
    error::Error as ClientError,
};
#[cfg(feature = "gateway")]
pub use self::{
    event_handler::{EventHandler, RawEventHandler},
    extras::Extras,
};

pub use crate::CacheAndHttp;
#[cfg(feature = "cache")]
pub use crate::cache::Cache;
use crate::internal::prelude::*;
use tokio::sync::{Mutex, RwLock};
#[cfg(feature = "gateway")]
use super::gateway::GatewayError;
use log::{error, debug, info};
#[cfg(feature = "gateway")]
use self::bridge::gateway::{GatewayIntents, ShardManager, ShardManagerMonitor, ShardManagerOptions, ShardManagerError};
use std::{
    boxed::Box,
    sync::Arc,
    future::Future,
    pin::Pin,
    task::{Context as FutContext, Poll},
};
#[cfg(any(feature = "framework", feature = "gateway"))]
use std::time::Duration;
#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::model::id::UserId;
#[cfg(feature = "voice")]
use self::bridge::voice::ClientVoiceManager;
use crate::http::Http;
use crate::utils::{TypeMap, TypeMapKey};
use futures::future::BoxFuture;

/// A builder implementing [`Future`] building a [`Client`] to interact with Discord.
///
/// [`Client`]: #struct.Client.html
/// [`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
#[cfg(feature = "gateway")]
pub struct ClientBuilder<'a> {
    data: Option<TypeMap>,
    http: Option<Http>,
    fut: Option<BoxFuture<'a, Result<Client>>>,
    guild_subscriptions: bool,
    intents: Option<GatewayIntents>,
    #[cfg(feature = "cache")]
    timeout: Option<Duration>,
    #[cfg(feature = "framework")]
    framework: Option<Arc<Box<dyn Framework + Send + Sync + 'static>>>,
    event_handler: Option<Arc<dyn EventHandler>>,
    raw_event_handler: Option<Arc<dyn RawEventHandler>>,
}

#[cfg(feature = "gateway")]
impl<'a> ClientBuilder<'a> {
    /// Construct a new builder to call methods on for the client construction.
    /// The `token` will automatically be prefixed "Bot " if not already.
    ///
    /// **Panic**:
    /// If you enabled the `framework`-feature (on by default), you must specify
    /// a framework via the [`framework`] or [`framework_arc`] method,
    /// otherwise awaiting the builder will cause a panic.
    ///
    /// [`framework`]: #method.framework
    /// [`framework_arc`]: #method.framework_arc
    pub fn new(token: impl AsRef<str>) -> Self {
        Self {
            data: Some(TypeMap::new()),
            http: None,
            fut: None,
            guild_subscriptions: true,
            intents: None,
            #[cfg(feature = "cache")]
            timeout: None,
            #[cfg(feature = "framework")]
            framework: None,
            event_handler: None,
            raw_event_handler: None,
        }.token(token)
    }

    /// Sets a token for the bot. If the token is not prefixed "Bot ",
    /// this method will automatically do so.
    pub fn token(mut self, token: impl AsRef<str>) -> Self {
        let token = token.as_ref().trim();

        let token = if token.starts_with("Bot ") {
            token.to_string()
        } else {
            format!("Bot {}", token)
        };

        self.http = Some(Http::new_with_token(&token));

        self
    }

    /// Sets the entire [`TypeMap`] that will be available in [`Context`]s.
    /// A `TypeMap` must not be constructed manually: [`type_map_insert`]
    /// can be used to insert one type at a time.
    ///
    /// [`TypeMap`]: ../utils/struct.TypeMap.html
    pub fn type_map(mut self, type_map: TypeMap) -> Self {
        self.data = Some(type_map);

        self
    }

    /// Insert a single `value` into the internal [`TypeMap`] that will
    /// be available in [`Context::data`].
    /// This method can be called multiple times in order to populate the
    /// [`TypeMap`] with `value`s.
    ///
    /// [`Context::data`]: #struct.Context
    /// [`TypeMap`]: ../utils/struct.TypeMap.html
    pub fn type_map_insert<T: TypeMapKey>(mut self, value: T::Value) -> Self {
        if let Some(ref mut data) = self.data {
            data.insert::<T>(value);
        } else {
            let mut type_map = TypeMap::new();
            type_map.insert::<T>(value);

            self.data = Some(type_map);
        }

        self
    }

    /// Sets how long - if wanted to begin with - a cache update shall
    /// be attempted for. After the `timeout` ran out, the update will be
    /// skipped.
    ///
    /// By default, a cache update will never timeout and potentially
    /// cause a deadlock.
    /// A timeout however, will invalidate the cache.
    #[cfg(feature = "cache")]
    pub fn cache_update_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);

        self
    }

    /// Whether presence or typing events shall be received over the gateway.
    ///
    /// **Info**:
    /// Prefer to use `intents`, as this may be replaced by them in the future.**
    pub fn guild_subscriptions(mut self, is_enabled: bool) -> Self {
        self.guild_subscriptions = is_enabled;

        self
    }

    /// Sets the command framework to be used. It will receive messages sent
    /// over the gateway and then consider - based on its settings - whether to
    /// dispatch a command.
    ///
    /// *Info*:
    /// If a reference to the framework is required for manual dispatch,
    /// use the [`framework_arc`]-method instead.
    ///
    /// [`framework_arc`]: #method.framework_arc
    #[cfg(feature = "framework")]
    pub fn framework<F>(mut self, framework: F) -> Self
    where F: Framework + Send + Sync + 'static,
    {
        self.framework = Some(Arc::new(Box::new(framework)));

        self
    }

    /// This method allows to pass an `Arc`'ed `framework` - this step is
    /// done for you in the [`framework`]-method, if you don't need the
    /// extra control.
    /// You can provide a clone and keep the original to manually dispatch.
    ///
    /// [`framework`]: #method.framework
    #[cfg(feature = "framework")]
    pub fn framework_arc(mut self, framework: Arc<Box<dyn Framework + Send + Sync + 'static>>) -> Self {
        self.framework = Some(framework);

        self
    }

    /// Sets all intents directly, replacing already set intents.
    ///
    /// *See also*:
    /// If visually preferred, you can use [`add_intent`] and chain it
    /// in order to add intent after intent.
    ///
    /// *Info*:
    /// Intents are a bitflag, you can combine them by performing the
    /// `|`-operator.
    ///
    /// [`add_intent`]: #method.add_intent
    pub fn intents(mut self, intents: GatewayIntents) -> Self {
        self.intents = Some(intents);

        self
    }

    /// Adds a single `intent`, this method can be called
    /// repetitively to add multiple intents.
    ///
    /// *See also*:
    /// If visually preferred, you can use [`intents`] and specify all
    /// intents at once. In theory you could also achieve the same result
    /// by passing the combined `intents`-bitflag to this method.
    ///
    /// [`intents`]: #method.intents
    pub fn add_intent(mut self, intent: GatewayIntents) -> Self {
        if let Some(ref mut intents) = self.intents {
            intents.insert(intent);
        } else {
            self.intents = Some(intent);
        }

        self
    }

    /// Sets an event handler with multiple methods for each possible event.
    pub fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Self {
        self.event_handler = Some(Arc::new(event_handler));

        self
    }

    /// Sets an event handler with a single method where all received gateway
    /// events will be dispatched.
    pub fn raw_event_handler<H: RawEventHandler + 'static>(mut self, raw_event_handler: H) -> Self {
        self.raw_event_handler = Some(Arc::new(raw_event_handler));

        self
    }
}

#[cfg(feature = "gateway")]
impl<'a> Future for ClientBuilder<'a> {
    type Output = Result<Client>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let data = Arc::new(RwLock::new(self.data.take().unwrap()));
            #[cfg(feature = "framework")]
            let framework = self.framework.take()
                .expect("The `framework`-feature is enabled (it's on by default), but no framework was provided.\n\
                If you don't want to use the command framework, disable default features and specify all features you want to use.");
            let event_handler = self.event_handler.take();
            let raw_event_handler = self.raw_event_handler.take();
            let guild_subscriptions = self.guild_subscriptions;
            let intents = self.intents;
            let http = Arc::new(self.http.take().unwrap());
            #[cfg(feature = "voice")]
            let voice_manager = Arc::new(Mutex::new(ClientVoiceManager::new(
                0,
                UserId(0),
            )));

            let cache_and_http = Arc::new(CacheAndHttp {
                #[cfg(feature = "cache")]
                cache: Arc::new(Cache::default()),
                #[cfg(feature = "cache")]
                update_cache_timeout: self.timeout.take(),
                http: Arc::clone(&http),
                __nonexhaustive: (),
            });

            self.fut = Some(Box::pin(async move {
                let url = Arc::new(Mutex::new(http.get_gateway().await?.url));

                let (shard_manager, shard_manager_worker) = {
                    ShardManager::new(ShardManagerOptions {
                        data: &data,
                        event_handler: &event_handler,
                        raw_event_handler: &raw_event_handler,
                        #[cfg(feature = "framework")]
                        framework: &framework,
                        shard_index: 0,
                        shard_init: 0,
                        shard_total: 0,
                        #[cfg(feature = "voice")]
                        voice_manager: &voice_manager,
                        ws_url: &url,
                        cache_and_http: &cache_and_http,
                        guild_subscriptions,
                        intents,
                    }).await
                };

                Ok(Client {
                    ws_uri: url,
                    data,
                    shard_manager,
                    shard_manager_worker,
                    #[cfg(feature = "voice")]
                    voice_manager,
                    cache_and_http,
                })
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

/// The Client is the way to be able to start sending authenticated requests
/// over the REST API, as well as initializing a WebSocket connection through
/// [`Shard`]s. Refer to the [documentation on using sharding][sharding docs]
/// for more information.
///
/// # Event Handlers
///
/// Event handlers can be configured. For example, the event handler
/// [`EventHandler::message`] will be dispatched to whenever a
/// [`Event::MessageCreate`] is received over the connection.
///
/// Note that you do not need to manually handle events, as they are handled
/// internally and then dispatched to your event handlers.
///
/// # Examples
///
/// Creating a Client instance and adding a handler on every message
/// receive, acting as a "ping-pong" bot is simple:
///
/// ```no_run
/// use serenity::prelude::*;
/// use serenity::model::prelude::*;
/// use serenity::Client;
///
/// struct Handler;
///
/// #[serenity::async_trait]
/// impl EventHandler for Handler {
///     async fn message(&self, context: Context, msg: Message) {
///         if msg.content == "!ping" {
///             let _ = msg.channel_id.say(&context, "Pong!");
///         }
///     }
/// }
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let mut client = Client::new("my token here").event_handler(Handler).await?;
///
/// client.start().await?;
/// #   Ok(())
/// # }
/// ```
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`EventHandler::message`]: trait.EventHandler.html#tymethod.message
/// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
/// [sharding docs]: ../index.html#sharding
#[cfg(feature = "gateway")]
pub struct Client {
    /// A TypeMap which requires types to be Send + Sync. This is a map that
    /// can be safely shared across contexts.
    ///
    /// The purpose of the data field is to be accessible and persistent across
    /// contexts; that is, data can be modified by one context, and will persist
    /// through the future and be accessible through other contexts. This is
    /// useful for anything that should "live" through the program: counters,
    /// database connections, custom user caches, etc.
    ///
    /// In the meaning of a context, this data can be accessed through
    /// [`Context::data`].
    ///
    /// # Examples
    ///
    /// Create a `MessageEventCounter` to track the following events:
    ///
    /// - [`Event::MessageCreate`]
    /// - [`Event::MessageDelete`]
    /// - [`Event::MessageDeleteBulk`]
    /// - [`Event::MessageUpdate`]
    ///
    /// ```rust,ignore
    /// use serenity::prelude::*;
    /// use serenity::model::prelude::*;
    /// use std::collections::HashMap;
    /// use std::env;
    ///
    /// struct MessageEventCounter;
    ///
    /// impl TypeMapKey for MessageEventCounter {
    ///     type Value = HashMap<String, u64>;
    /// }
    ///
    /// async fn reg<S: Into<String>>(ctx: Context, name: S) {
    ///     let mut data = ctx.data.write().await;
    ///     let counter = data.get_mut::<MessageEventCounter>().unwrap();
    ///     let entry = counter.entry(name.into()).or_insert(0);
    ///     *entry += 1;
    /// }
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, _: Message) {
    ///         reg(ctx, "MessageCreate").await
    ///     }
    ///     async fn message_delete(&self, ctx: Context, _: ChannelId, _: MessageId) {
    ///         reg(ctx, "MessageDelete").await
    ///     }
    ///     async fn message_delete_bulk(&self, ctx: Context, _: ChannelId, _: Vec<MessageId>) {
    ///         reg(ctx, "MessageDeleteBulk").await
    ///     }
    ///
    ///     #[cfg(feature = "cache")]
    ///     async fn message_update(&self, ctx: Context, _old: Option<Message>, _new: Option<Message>, _: MessageUpdateEvent) {
    ///         reg(ctx, "MessageUpdate").await
    ///     }
    ///
    ///     #[cfg(not(feature = "cache"))]
    ///     async fn message_update(&self, ctx: Context, _new_data: MessageUpdateEvent) {
    ///         reg(ctx, "MessageUpdate").await
    ///     }
    /// }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    /// {
    ///     let mut data = client.data.write().await;
    ///     data.insert::<MessageEventCounter>(HashMap::default());
    /// }
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Refer to [example 05] for an example on using the `data` field.
    ///
    /// [`Context::data`]: struct.Context.html#structfield.data
    /// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    /// [`Event::MessageDelete`]: ../model/event/enum.Event.html#variant.MessageDelete
    /// [`Event::MessageDeleteBulk`]: ../model/event/enum.Event.html#variant.MessageDeleteBulk
    /// [`Event::MessageUpdate`]: ../model/event/enum.Event.html#variant.MessageUpdate
    /// [example 05]: https://github.com/serenity-rs/serenity/tree/current/examples/05_command_framework
    pub data: Arc<RwLock<TypeMap>>,
    /// A HashMap of all shards instantiated by the Client.
    ///
    /// The key is the shard ID and the value is the shard itself.
    ///
    /// # Examples
    ///
    /// If you call [`client.start_shard(3, 5)`][`Client::start_shard`], this
    /// HashMap will only ever contain a single key of `3`, as that's the only
    /// Shard the client is responsible for.
    ///
    /// If you call [`client.start_shards(10)`][`Client::start_shards`], this
    /// HashMap will contain keys 0 through 9, one for each shard handled by the
    /// client.
    ///
    /// Printing the number of shards currently instantiated by the client every
    /// 5 seconds:
    ///
    /// ```rust,no_run
    /// # use serenity::client::{Client, EventHandler};
    /// # use std::time::Duration;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// struct Handler;
    ///
    /// impl EventHandler for Handler { }
    ///
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// tokio::spawn(async move {
    ///     loop {
    ///         let sm = shard_manager.lock().await;
    ///         let count = sm.shards_instantiated().await.len();
    ///         println!("Shard count instantiated: {}", count);
    ///
    ///         tokio::time::delay_for(Duration::from_millis(5000)).await;
    ///     }
    /// });
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Shutting down all connections after one minute of operation:
    ///
    /// ```rust,no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::client::{Client, EventHandler};
    /// use std::time::Duration;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler { }
    ///
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// // Create a clone of the `Arc` containing the shard manager.
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// // Create a thread which will sleep for 60 seconds and then have the
    /// // shard manager shutdown.
    /// tokio::spawn(async move {
    ///     tokio::time::delay_for(Duration::from_secs(60));
    ///
    ///     shard_manager.lock().await.shutdown_all().await;
    ///
    ///     println!("Shutdown shard manager!");
    /// });
    ///
    /// println!("Client shutdown: {:?}", client.start().await);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Client::start_shard`]: #method.start_shard
    /// [`Client::start_shards`]: #method.start_shards
    pub shard_manager: Arc<Mutex<ShardManager>>,
    shard_manager_worker: ShardManagerMonitor,
    /// The voice manager for the client.
    ///
    /// This is an ergonomic structure for interfacing over shards' voice
    /// connections.
    #[cfg(feature = "voice")]
    pub voice_manager: Arc<Mutex<ClientVoiceManager>>,
    /// URI that the client's shards will use to connect to the gateway.
    ///
    /// This is likely not important for production usage and is, at best, used
    /// for debugging.
    ///
    /// This is wrapped in an `Arc<Mutex<T>>` so all shards will have an updated
    /// value available.
    pub ws_uri: Arc<Mutex<String>>,
    /// A container for an optional cache and HTTP client.
    /// It also contains the cache update timeout.
    pub cache_and_http: Arc<CacheAndHttp>,
}

#[cfg(feature = "gateway")]
impl Client {
    /// Returns a builder implementing [`Future`]. You can chain the builder methods and then await
    /// in order to finish the [`Client`].
    ///
    /// [`Client`]: #struct.Client.html
    /// [`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
    pub fn new<'a>(token: impl AsRef<str>) -> ClientBuilder<'a> {
        ClientBuilder::new(token)
    }

    /// Establish the connection and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the
    /// events to your registered handlers.
    ///
    /// Note that this should be used only for users and for bots which are in
    /// less than 2500 guilds. If you have a reason for sharding and/or are in
    /// more than 2500 guilds, use one of these depending on your use case:
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information
    /// on effectively using sharding.
    ///
    /// # Examples
    ///
    /// Starting a Client with only 1 shard, out of 1 total:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start().await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [gateway docs]: ../gateway/index.html#sharding
    pub async fn start(&mut self) -> Result<()> {
        self.start_connection([0, 0, 1]).await
    }

    /// Establish the connection(s) and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the
    /// events to your registered handlers.
    ///
    /// This will retrieve an automatically determined number of shards to use
    /// from the API - determined by Discord - and then open a number of shards
    /// equivalent to that amount.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information
    /// on effectively using sharding.
    ///
    /// # Examples
    ///
    /// Start as many shards as needed using autosharding:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start_autosharded().await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::Shutdown`] when all shards have shutdown due to
    /// an error.
    ///
    /// [`ClientError::Shutdown`]: enum.ClientError.html#variant.Shutdown
    /// [gateway docs]: ../gateway/index.html#sharding
    pub async fn start_autosharded(&mut self) -> Result<()> {
        let (x, y) = {
            let res = self.cache_and_http.http.get_bot_gateway().await?;

            (res.shards as u64 - 1, res.shards as u64)
        };

        self.start_connection([0, x, y]).await
    }

    /// Establish a sharded connection and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered
    /// handlers.
    ///
    /// This will create a single shard by ID. If using one shard per process,
    /// you will need to start other processes with the other shard IDs in some
    /// way.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information
    /// on effectively using sharding.
    ///
    /// # Examples
    ///
    /// Start shard 3 of 5:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start_shard(3, 5).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Start shard 0 of 1 (you may also be interested in [`start`] or
    /// [`start_autosharded`]):
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start_shard(0, 1).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::Shutdown`] when all shards have shutdown due to
    /// an error.
    ///
    /// [`ClientError::Shutdown`]: enum.ClientError.html#variant.Shutdown
    /// [`start`]: #method.start
    /// [`start_autosharded`]: #method.start_autosharded
    /// [gateway docs]: ../gateway/index.html#sharding
    pub async fn start_shard(&mut self, shard: u64, shards: u64) -> Result<()> {
        self.start_connection([shard, shard, shards]).await
    }

    /// Establish sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered
    /// handlers.
    ///
    /// This will create and handle all shards within this single process. If
    /// you only need to start a single shard within the process, or a range of
    /// shards, use [`start_shard`] or [`start_shard_range`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information
    /// on effectively using sharding.
    ///
    /// # Examples
    ///
    /// Start all of 8 shards:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start_shards(8).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::Shutdown`] when all shards have shutdown due to
    /// an error.
    ///
    /// [`ClientError::Shutdown`]: enum.ClientError.html#variant.Shutdown
    /// [`start_shard`]: #method.start_shard
    /// [`start_shard_range`]: #method.start_shard_range
    /// [Gateway docs]: ../gateway/index.html#sharding
    pub async fn start_shards(&mut self, total_shards: u64) -> Result<()> {
        self.start_connection([0, total_shards - 1, total_shards]).await
    }

    /// Establish a range of sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered
    /// handlers.
    ///
    /// This will create and handle all shards within a given range within this
    /// single process. If you only need to start a single shard within the
    /// process, or all shards within the process, use [`start_shard`] or
    /// [`start_shards`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more
    /// information on effectively using sharding.
    ///
    /// # Examples
    ///
    /// For a bot using a total of 10 shards, initialize shards 4 through 7:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::EventHandler;
    /// use serenity::Client;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token).event_handler(Handler).await?;
    ///
    /// if let Err(why) = client.start_shard_range([4, 7], 10).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::Shutdown`] when all shards have shutdown due to
    /// an error.
    ///
    ///
    /// [`ClientError::Shutdown`]: enum.ClientError.html#variant.Shutdown
    /// [`start_shard`]: #method.start_shard
    /// [`start_shards`]: #method.start_shards
    /// [Gateway docs]: ../gateway/index.html#sharding
    pub async fn start_shard_range(&mut self, range: [u64; 2], total_shards: u64) -> Result<()> {
        self.start_connection([range[0], range[1], total_shards]).await
    }

    // Shard data layout is:
    // 0: first shard number to initialize
    // 1: shard number to initialize up to and including
    // 2: total number of shards the bot is sharding for
    //
    // Not all shards need to be initialized in this process.
    //
    // # Errors
    //
    // Returns a [`ClientError::Shutdown`] when all shards have shutdown due to
    // an error.
    //
    // [`ClientError::Shutdown`]: enum.ClientError.html#variant.Shutdown
    async fn start_connection(&mut self, shard_data: [u64; 3]) -> Result<()> {
        #[cfg(feature = "voice")]
        self.voice_manager.lock().await.set_shard_count(shard_data[2]);

        #[cfg(feature = "voice")]
        {
            let user = self.cache_and_http.http.get_current_user().await?;

            self.voice_manager.lock().await.set_user_id(user.id);
        }

        {

            let mut manager = self.shard_manager.lock().await;

            let init = shard_data[1] - shard_data[0] + 1;

            manager.set_shards(shard_data[0], init, shard_data[2]).await;

            debug!(
                "Initializing shard info: {} - {}/{}",
                shard_data[0],
                init,
                shard_data[2],
            );

            if let Err(why) = manager.initialize() {
                error!("Failed to boot a shard: {:?}", why);
                info!("Shutting down all shards");

                manager.shutdown_all().await;

                return Err(Error::Client(ClientError::ShardBootFailure));
            }
        }

        if let Err(why) = self.shard_manager_worker.run().await {
            let err =  match why {
                ShardManagerError::DisallowedGatewayIntents => GatewayError::DisallowedGatewayIntents,
                ShardManagerError::InvalidGatewayIntents => GatewayError::InvalidGatewayIntents,
                ShardManagerError::InvalidToken => GatewayError::InvalidAuthentication,
            };
            return Err(Error::Gateway(err));
        }

        Ok(())
    }
}

/// Validates that a token is likely in a valid format.
///
/// This performs the following checks on a given token:
///
/// - At least one character long;
/// - Contains 3 parts (split by the period char `'.'`);
/// - The second part of the token is at least 6 characters long;
/// - The token does not contain any whitespace prior to or after the token.
///
/// # Examples
///
/// Validate that a token is valid and that a number of invalid tokens are
/// actually invalid:
///
/// ```rust,no_run
/// use serenity::client::validate_token;
///
/// // ensure a valid token is in fact valid:
/// assert!(validate_token("Mjg4NzYwMjQxMzYzODc3ODg4.C_ikow.j3VupLBuE1QWZng3TMGH0z_UAwg").is_ok());
///
/// // "cat" isn't a valid token:
/// assert!(validate_token("cat").is_err());
///
/// // tokens must have three parts, separated by periods (this is still
/// // actually an invalid token):
/// assert!(validate_token("aaa.abcdefgh.bbb").is_ok());
///
/// // the second part must be _at least_ 6 characters long:
/// assert!(validate_token("a.abcdef.b").is_ok());
/// assert!(validate_token("a.abcde.b").is_err());
/// ```
///
/// # Errors
///
/// Returns a [`ClientError::InvalidToken`] when one of the above checks fail.
/// The type of failure is not specified.
///
/// [`ClientError::InvalidToken`]: enum.ClientError.html#variant.InvalidToken
pub fn validate_token(token: impl AsRef<str>) -> Result<()> {
    let token = token.as_ref();

    if token.is_empty() {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    let parts: Vec<&str> = token.split('.').collect();

    // Check that the token has a total of 3 parts.
    if parts.len() != 3 {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    // Check that the second part is at least 6 characters long.
    if parts[1].len() < 6 {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    // Check that there is no whitespace before/after the token.
    if token.trim() != token {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    Ok(())
}
