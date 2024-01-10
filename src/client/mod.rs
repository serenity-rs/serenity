//! A module for [`Client`] and supporting types.
//!
//! The Client contains information about a single bot's token, as well as event handlers.
//! Dispatching events to configured handlers and starting the shards' connections are handled
//! directly via the client. In addition, the `http` module and `Cache` are also automatically
//! handled by the Client module for you.
//!
//! A [`Context`] is provided for every handler.
//!
//! The `http` module is the lower-level method of interacting with the Discord REST API.
//! Realistically, there should be little reason to use this yourself, as the Context will do this
//! for you. A possible use case of using the `http` module is if you do not have a Cache, for
//! purposes such as low memory requirements.
//!
//! Click [here][Client examples] for an example on how to use a `Client`.
//!
//! [Client examples]: Client#examples

mod context;
#[cfg(feature = "gateway")]
pub(crate) mod dispatch;
mod error;
#[cfg(feature = "gateway")]
mod event_handler;

use std::future::IntoFuture;
use std::num::NonZeroU16;
use std::ops::Range;
use std::sync::Arc;
#[cfg(feature = "framework")]
use std::sync::OnceLock;

use futures::channel::mpsc::UnboundedReceiver as Receiver;
use futures::future::BoxFuture;
use futures::StreamExt as _;
use tokio::sync::RwLock;
use tracing::debug;
use typemap_rev::{TypeMap, TypeMapKey};

pub use self::context::Context;
pub use self::error::Error as ClientError;
#[cfg(feature = "gateway")]
pub use self::event_handler::{EventHandler, FullEvent, RawEventHandler};
#[cfg(feature = "gateway")]
use super::gateway::GatewayError;
#[cfg(feature = "cache")]
pub use crate::cache::Cache;
#[cfg(feature = "cache")]
use crate::cache::Settings as CacheSettings;
#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::gateway::VoiceGatewayManager;
use crate::gateway::{ActivityData, PresenceData};
#[cfg(feature = "gateway")]
use crate::gateway::{ShardManager, ShardManagerOptions};
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "gateway")]
use crate::model::gateway::GatewayIntents;
use crate::model::id::ApplicationId;
#[cfg(feature = "voice")]
use crate::model::id::UserId;
use crate::model::user::OnlineStatus;
use crate::utils::check_shard_total;

/// A builder implementing [`IntoFuture`] building a [`Client`] to interact with Discord.
#[cfg(feature = "gateway")]
#[must_use = "Builders do nothing unless they are awaited"]
pub struct ClientBuilder {
    data: TypeMap,
    http: Http,
    intents: GatewayIntents,
    #[cfg(feature = "cache")]
    cache_settings: CacheSettings,
    #[cfg(feature = "framework")]
    framework: Option<Box<dyn Framework>>,
    #[cfg(feature = "voice")]
    voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    presence: PresenceData,
}

#[cfg(feature = "gateway")]
impl ClientBuilder {
    fn new_(http: Http, intents: GatewayIntents) -> Self {
        Self {
            data: TypeMap::new(),
            http,
            intents,
            #[cfg(feature = "cache")]
            cache_settings: CacheSettings::default(),
            #[cfg(feature = "framework")]
            framework: None,
            #[cfg(feature = "voice")]
            voice_manager: None,
            event_handlers: vec![],
            raw_event_handlers: vec![],
            presence: PresenceData::default(),
        }
    }

    /// Construct a new builder to call methods on for the client construction. The `token` will
    /// automatically be prefixed "Bot " if not already.
    ///
    /// **Panic**: If you have enabled the `framework`-feature (on by default), you must specify a
    /// framework via the [`Self::framework`] method, otherwise awaiting the builder will cause a
    /// panic.
    pub fn new(token: impl AsRef<str>, intents: GatewayIntents) -> Self {
        Self::new_(Http::new(token.as_ref()), intents)
    }

    /// Construct a new builder with a [`Http`] instance to calls methods on for the client
    /// construction.
    ///
    /// **Panic**: If you have enabled the `framework`-feature (on by default), you must specify a
    /// framework via the [`Self::framework`] method, otherwise awaiting the builder will cause a
    /// panic.
    pub fn new_with_http(http: Http, intents: GatewayIntents) -> Self {
        Self::new_(http, intents)
    }

    /// Sets a token for the bot. If the token is not prefixed "Bot ", this method will
    /// automatically do so.
    pub fn token(mut self, token: impl AsRef<str>) -> Self {
        self.http = Http::new(token.as_ref());

        self
    }

    /// Gets the current token used for the [`Http`] client.
    pub fn get_token(&self) -> &str {
        self.http.token()
    }

    /// Sets the application id.
    pub fn application_id(self, application_id: ApplicationId) -> Self {
        self.http.set_application_id(application_id);

        self
    }

    /// Gets the application ID, if already initialized. See [`Self::application_id`] for more
    /// info.
    pub fn get_application_id(&self) -> Option<ApplicationId> {
        self.http.application_id()
    }

    /// Sets the entire [`TypeMap`] that will be available in [`Context`]s. A [`TypeMap`] must not
    /// be constructed manually: [`Self::type_map_insert`] can be used to insert one type at a
    /// time.
    pub fn type_map(mut self, type_map: TypeMap) -> Self {
        self.data = type_map;

        self
    }

    /// Gets the type map. See [`Self::type_map`] for more info.
    pub fn get_type_map(&self) -> &TypeMap {
        &self.data
    }

    /// Insert a single `value` into the internal [`TypeMap`] that will be available in
    /// [`Context::data`]. This method can be called multiple times in order to populate the
    /// [`TypeMap`] with `value`s.
    pub fn type_map_insert<T: TypeMapKey>(mut self, value: T::Value) -> Self {
        self.data.insert::<T>(value);

        self
    }

    /// Sets the settings of the cache. Refer to [`Settings`] for more information.
    ///
    /// [`Settings`]: CacheSettings
    #[cfg(feature = "cache")]
    pub fn cache_settings(mut self, settings: CacheSettings) -> Self {
        self.cache_settings = settings;
        self
    }

    /// Gets the cache settings. See [`Self::cache_settings`] for more info.
    #[cfg(feature = "cache")]
    pub fn get_cache_settings(&self) -> &CacheSettings {
        &self.cache_settings
    }

    /// Sets the command framework to be used. It will receive messages sent over the gateway and
    /// then consider - based on its settings - whether to dispatch a command.
    ///
    /// *Info*: If a reference to the framework is required for manual dispatch, you can implement
    /// [`Framework`] on [`Arc<YourFrameworkType>`] instead of `YourFrameworkType`.
    #[cfg(feature = "framework")]
    pub fn framework<F>(mut self, framework: F) -> Self
    where
        F: Framework + 'static,
    {
        self.framework = Some(Box::new(framework));

        self
    }

    /// Gets the framework, if already initialized. See [`Self::framework`] for more info.
    #[cfg(feature = "framework")]
    pub fn get_framework(&self) -> Option<&dyn Framework> {
        self.framework.as_deref()
    }

    /// Sets the voice gateway handler to be used. It will receive voice events sent over the
    /// gateway and then consider - based on its settings - whether to dispatch a command.
    #[cfg(feature = "voice")]
    pub fn voice_manager<V>(mut self, voice_manager: impl Into<Arc<V>>) -> Self
    where
        V: VoiceGatewayManager + 'static,
    {
        self.voice_manager = Some(voice_manager.into());
        self
    }

    /// Gets the voice manager, if already initialized. See [`Self::voice_manager`] for more info.
    #[cfg(feature = "voice")]
    pub fn get_voice_manager(&self) -> Option<Arc<dyn VoiceGatewayManager>> {
        self.voice_manager.clone()
    }

    /// Sets all intents directly, replacing already set intents. Intents are a bitflag, you can
    /// combine them by performing the `|`-operator.
    ///
    /// # What are Intents
    ///
    /// A [gateway intent] sets the types of gateway events (e.g. member joins, guild integrations,
    /// guild emoji updates, ...) the bot shall receive. Carefully picking the needed intents
    /// greatly helps the bot to scale, as less intents will result in less events to be received
    /// hence less processed by the bot.
    ///
    /// # Privileged Intents
    ///
    /// The intents [`GatewayIntents::GUILD_PRESENCES`], [`GatewayIntents::GUILD_MEMBERS`] and
    /// [`GatewayIntents::MESSAGE_CONTENT`] are *privileged*. [Privileged intents] need to be
    /// enabled in the *developer portal*. Once the bot is in 100 guilds or more, [the bot must be
    /// verified] in order to use privileged intents.
    ///
    /// [gateway intent]: https://discord.com/developers/docs/topics/gateway#privileged-intents
    /// [Privileged intents]: https://discord.com/developers/docs/topics/gateway#privileged-intents
    /// [the bot must be verified]: https://support.discord.com/hc/en-us/articles/360040720412-Bot-Verification-and-Data-Whitelisting
    pub fn intents(mut self, intents: GatewayIntents) -> Self {
        self.intents = intents;

        self
    }

    /// Gets the intents. See [`Self::intents`] for more info.
    pub fn get_intents(&self) -> GatewayIntents {
        self.intents
    }

    /// Adds an event handler with multiple methods for each possible event.
    pub fn event_handler<H>(mut self, event_handler: impl Into<Arc<H>>) -> Self
    where
        H: EventHandler + 'static,
    {
        self.event_handlers.push(event_handler.into());

        self
    }

    /// Gets the added event handlers. See [`Self::event_handler`] for more info.
    pub fn get_event_handlers(&self) -> &[Arc<dyn EventHandler>] {
        &self.event_handlers
    }

    /// Adds an event handler with a single method where all received gateway events will be
    /// dispatched.
    pub fn raw_event_handler<H: RawEventHandler + 'static>(mut self, raw_event_handler: H) -> Self {
        self.raw_event_handlers.push(Arc::new(raw_event_handler));

        self
    }

    /// Gets the added raw event handlers. See [`Self::raw_event_handler`] for more info.
    pub fn get_raw_event_handlers(&self) -> &[Arc<dyn RawEventHandler>] {
        &self.raw_event_handlers
    }

    /// Sets the initial activity.
    pub fn activity(mut self, activity: ActivityData) -> Self {
        self.presence.activity = Some(activity);

        self
    }

    /// Sets the initial status.
    pub fn status(mut self, status: OnlineStatus) -> Self {
        self.presence.status = status;

        self
    }

    /// Gets the initial presence. See [`Self::activity`] and [`Self::status`] for more info.
    pub fn get_presence(&self) -> &PresenceData {
        &self.presence
    }
}

#[cfg(feature = "gateway")]
impl IntoFuture for ClientBuilder {
    type Output = Result<Client>;

    type IntoFuture = BoxFuture<'static, Result<Client>>;

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn into_future(self) -> Self::IntoFuture {
        let data = Arc::new(RwLock::new(self.data));
        #[cfg(feature = "framework")]
        let framework = self.framework;
        let event_handlers = self.event_handlers;
        let raw_event_handlers = self.raw_event_handlers;
        let intents = self.intents;
        let presence = self.presence;

        let mut http = self.http;

        if let Some(ratelimiter) = &mut http.ratelimiter {
            let event_handlers_clone = event_handlers.clone();
            ratelimiter.set_ratelimit_callback(Box::new(move |info| {
                for event_handler in event_handlers_clone.iter().map(Arc::clone) {
                    let info = info.clone();
                    tokio::spawn(async move { event_handler.ratelimit(info).await });
                }
            }));
        }

        let http = Arc::new(http);

        #[cfg(feature = "voice")]
        let voice_manager = self.voice_manager;

        #[cfg(feature = "cache")]
        let cache = Arc::new(Cache::new_with_settings(self.cache_settings));

        Box::pin(async move {
            let (ws_url, shard_total) = match http.get_bot_gateway().await {
                Ok(response) => (Arc::from(response.url), response.shards),
                Err(err) => {
                    tracing::warn!("HTTP request to get gateway URL failed: {err}");
                    (Arc::from("wss://gateway.discord.gg"), NonZeroU16::MIN)
                },
            };

            #[cfg(feature = "framework")]
            let framework_cell = Arc::new(OnceLock::new());
            let (shard_manager, shard_manager_ret_value) = ShardManager::new(ShardManagerOptions {
                data: Arc::clone(&data),
                event_handlers,
                raw_event_handlers,
                #[cfg(feature = "framework")]
                framework: Arc::clone(&framework_cell),
                #[cfg(feature = "voice")]
                voice_manager: voice_manager.clone(),
                ws_url: Arc::clone(&ws_url),
                shard_total,
                #[cfg(feature = "cache")]
                cache: Arc::clone(&cache),
                http: Arc::clone(&http),
                intents,
                presence: Some(presence),
            });

            let client = Client {
                data,
                shard_manager,
                shard_manager_return_value: shard_manager_ret_value,
                #[cfg(feature = "voice")]
                voice_manager,
                ws_url,
                #[cfg(feature = "cache")]
                cache,
                http,
            };
            #[cfg(feature = "framework")]
            if let Some(mut framework) = framework {
                framework.init(&client).await;
                if let Err(_existing) = framework_cell.set(framework.into()) {
                    tracing::warn!("overwrote existing contents of framework OnceLock");
                }
            }
            Ok(client)
        })
    }
}

/// A wrapper for HTTP and gateway connections.
///
/// The Client is the way to be able to start sending authenticated requests over the REST API, as
/// well as initializing a WebSocket connection through [`Shard`]s. Refer to the [documentation on
/// using sharding][sharding docs] for more information.
///
/// # Event Handlers
///
/// Event handlers can be configured. For example, the event handler [`EventHandler::message`] will
/// be dispatched to whenever a [`Event::MessageCreate`] is received over the connection.
///
/// Note that you do not need to manually handle events, as they are handled internally and then
/// dispatched to your event handlers.
///
/// # Examples
///
/// Creating a Client instance and adding a handler on every message receive, acting as a
/// "ping-pong" bot is simple:
///
/// ```no_run
/// use serenity::model::prelude::*;
/// use serenity::prelude::*;
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
/// let mut client =
///     Client::builder("my token here", GatewayIntents::default()).event_handler(Handler).await?;
///
/// client.start().await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Shard`]: crate::gateway::Shard
/// [`Event::MessageCreate`]: crate::model::event::Event::MessageCreate
/// [sharding docs]: crate::gateway#sharding
#[cfg(feature = "gateway")]
pub struct Client {
    /// A TypeMap which requires types to be Send + Sync. This is a map that can be safely shared
    /// across contexts.
    ///
    /// The purpose of the data field is to be accessible and persistent across contexts; that is,
    /// data can be modified by one context, and will persist through the future and be accessible
    /// through other contexts. This is useful for anything that should "live" through the program:
    /// counters, database connections, custom user caches, etc.
    ///
    /// In the meaning of a context, this data can be accessed through [`Context::data`].
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
    /// use std::collections::HashMap;
    /// use std::env;
    ///
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
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
    ///     async fn message_update(
    ///         &self,
    ///         ctx: Context,
    ///         _old: Option<Message>,
    ///         _new: Option<Message>,
    ///         _: MessageUpdateEvent,
    ///     ) {
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
    /// let mut client = Client::builder(&token, GatewayIntents::default()).event_handler(Handler).await?;
    /// {
    ///     let mut data = client.data.write().await;
    ///     data.insert::<MessageEventCounter>(HashMap::default());
    /// }
    ///
    /// client.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Refer to [example 05] for an example on using the [`Self::data`] field.
    ///
    /// [`Event::MessageCreate`]: crate::model::event::Event::MessageCreate
    /// [`Event::MessageDelete`]: crate::model::event::Event::MessageDelete
    /// [`Event::MessageDeleteBulk`]: crate::model::event::Event::MessageDeleteBulk
    /// [`Event::MessageUpdate`]: crate::model::event::Event::MessageUpdate
    /// [example 05]: https://github.com/serenity-rs/serenity/tree/current/examples/e05_command_framework
    pub data: Arc<RwLock<TypeMap>>,
    /// A HashMap of all shards instantiated by the Client.
    ///
    /// The key is the shard ID and the value is the shard itself.
    ///
    /// # Examples
    ///
    /// If you call [`client.start_shard(3, 5)`][`Client::start_shard`], this HashMap will only
    /// ever contain a single key of `3`, as that's the only Shard the client is responsible for.
    ///
    /// If you call [`client.start_shards(10)`][`Client::start_shards`], this HashMap will contain
    /// keys 0 through 9, one for each shard handled by the client.
    ///
    /// Printing the number of shards currently instantiated by the client every 5 seconds:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use std::time::Duration;
    /// #
    /// # fn run(client: Client) {
    /// // Create a clone of the `Arc` containing the shard manager.
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// tokio::spawn(async move {
    ///     loop {
    ///         let count = shard_manager.shards_instantiated().await.len();
    ///         println!("Shard count instantiated: {}", count);
    ///
    ///         tokio::time::sleep(Duration::from_millis(5000)).await;
    ///     }
    /// });
    /// # }
    /// ```
    ///
    /// Shutting down all connections after one minute of operation:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use std::time::Duration;
    /// #
    /// # fn run(client: Client) {
    /// // Create a clone of the `Arc` containing the shard manager.
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// // Create a thread which will sleep for 60 seconds and then have the shard manager
    /// // shutdown.
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(Duration::from_secs(60)).await;
    ///
    ///     shard_manager.shutdown_all().await;
    ///
    ///     println!("Shutdown shard manager!");
    /// });
    /// # }
    /// ```
    pub shard_manager: Arc<ShardManager>,
    shard_manager_return_value: Receiver<Result<(), GatewayError>>,
    /// The voice manager for the client.
    ///
    /// This is an ergonomic structure for interfacing over shards' voice
    /// connections.
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    /// URL that the client's shards will use to connect to the gateway.
    pub ws_url: Arc<str>,
    /// The cache for the client.
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    /// An HTTP client.
    pub http: Arc<Http>,
}

impl Client {
    pub fn builder(token: impl AsRef<str>, intents: GatewayIntents) -> ClientBuilder {
        ClientBuilder::new(token, intents)
    }

    /// Establish the connection and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the events to your
    /// registered handlers.
    ///
    /// Note that this should be used only for users and for bots which are in less than 2500
    /// guilds. If you have a reason for sharding and/or are in more than 2500 guilds, use one of
    /// these depending on your use case:
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Starting a Client with only 1 shard, out of 1 total:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
    ///
    /// if let Err(why) = client.start().await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    /// Returns [`Error::Http`] if fetching the current User fails when initialising a voice
    /// manager.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start(&mut self) -> Result<()> {
        self.start_connection(0, 0, NonZeroU16::MIN).await
    }

    /// Establish the connection(s) and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the events to your
    /// registered handlers.
    ///
    /// This will retrieve an automatically determined number of shards to use from the API -
    /// determined by Discord - and then open a number of shards equivalent to that amount.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start as many shards as needed using autosharding:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
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
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    /// Returns [`Error::Http`] if fetching the current User fails when initialising a voice
    /// manager.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_autosharded(&mut self) -> Result<()> {
        let (end, total) = {
            let res = self.http.get_bot_gateway().await?;
            (res.shards.get() - 1, res.shards)
        };

        self.start_connection(0, end, total).await
    }

    /// Establish a sharded connection and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create a single shard by ID. If using one shard per process, you will need to
    /// start other processes with the other shard IDs in some way.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start shard 3 of 5:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
    ///
    /// if let Err(why) = client.start_shard(3, 5).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Start shard 0 of 1 (you may also be interested in [`Self::start`] or
    /// [`Self::start_autosharded`]):
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
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
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    /// Returns [`Error::Http`] if fetching the current User fails when initialising a voice
    /// manager.
    ///
    /// [gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shard(&mut self, shard: u16, shards: u16) -> Result<()> {
        self.start_connection(shard, shard, check_shard_total(shards)).await
    }

    /// Establish sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create and handle all shards within this single process. If you only need to
    /// start a single shard within the process, or a range of shards, use [`Self::start_shard`] or
    /// [`Self::start_shard_range`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// Start all of 8 shards:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
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
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    /// Returns [`Error::Http`] if fetching the current User fails when initialising a voice
    /// manager.
    ///
    /// [Gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shards(&mut self, total_shards: u16) -> Result<()> {
        self.start_connection(0, total_shards - 1, check_shard_total(total_shards)).await
    }

    /// Establish a range of sharded connections and start listening for events.
    ///
    /// This will start receiving events and dispatch them to your registered handlers.
    ///
    /// This will create and handle all shards within a given range within this single process. If
    /// you only need to start a single shard within the process, or all shards within the process,
    /// use [`Self::start_shard`] or [`Self::start_shards`], respectively.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information on effectively
    /// using sharding.
    ///
    /// # Examples
    ///
    /// For a bot using a total of 10 shards, initialize shards 4 through 7:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// # use serenity::prelude::*;
    /// use serenity::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token, GatewayIntents::default()).await?;
    ///
    /// if let Err(why) = client.start_shard_range(4..7, 10).await {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Gateway`] when all shards have shutdown due to an error.
    /// Returns [`Error::Http`] if fetching the current User fails when initialising a voice
    /// manager.
    ///
    /// [Gateway docs]: crate::gateway#sharding
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_shard_range(&mut self, range: Range<u16>, total_shards: u16) -> Result<()> {
        self.start_connection(range.start, range.end, check_shard_total(total_shards)).await
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn start_connection(
        &mut self,
        start_shard: u16,
        end_shard: u16,
        total_shards: NonZeroU16,
    ) -> Result<()> {
        #[cfg(feature = "voice")]
        if let Some(voice_manager) = &self.voice_manager {
            #[cfg(feature = "cache")]
            let cache_user_id = {
                let cache_user = self.cache.current_user();
                if cache_user.id == UserId::default() {
                    Some(cache_user.id)
                } else {
                    None
                }
            };

            #[cfg(not(feature = "cache"))]
            let cache_user_id: Option<UserId> = None;

            let user_id = match cache_user_id {
                Some(u) => u,
                None => self.http.get_current_user().await?.id,
            };

            voice_manager.initialise(total_shards, user_id).await;
        }

        let init = end_shard - start_shard + 1;

        debug!("Initializing shard info: {} - {}/{}", start_shard, init, total_shards);

        self.shard_manager.initialize(start_shard, init, total_shards);
        if let Some(Err(err)) = self.shard_manager_return_value.next().await {
            return Err(Error::Gateway(err));
        }

        Ok(())
    }
}
