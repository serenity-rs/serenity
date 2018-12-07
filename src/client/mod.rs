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
#![allow(clippy::zero_ptr)]

pub mod bridge;

mod context;
mod dispatch;
mod error;
mod event_handler;

pub use self::{
    context::Context,
    error::Error as ClientError,
    event_handler::EventHandler
};

// Note: the following re-exports are here for backwards compatibility
pub use crate::gateway;
pub use crate::http as rest;

#[cfg(feature = "cache")]
pub use crate::CACHE;

use crate::http;
use crate::internal::prelude::*;
use parking_lot::Mutex;
use self::bridge::gateway::{ShardManager, ShardManagerMonitor, ShardManagerOptions};
use std::sync::Arc;
use threadpool::ThreadPool;
use typemap::ShareMap;

#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::model::id::UserId;
#[cfg(feature = "voice")]
use self::bridge::voice::ClientVoiceManager;

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
/// ```rust,ignore
/// use serenity::prelude::*;
/// use serenity::model::*;
///
/// struct Handler;
///
/// impl EventHandler for Handler {
///     fn on_message(&self, _: Context, msg: Message) {
///         if msg.content == "!ping" {
///             let _ = msg.channel_id.say("Pong!");
///         }
///     }
/// }
///
/// let mut client = Client::new("my token here", Handler);
///
/// client.start();
/// ```
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`EventHandler::message`]: trait.EventHandler.html#tymethod.message
/// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
/// [sharding docs]: ../index.html#sharding
pub struct Client {
    /// A ShareMap which requires types to be Send + Sync. This is a map that
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
    /// extern crate serenity;
    /// extern crate typemap;
    ///
    /// use serenity::prelude::*;
    /// use serenity::model::*;
    /// use std::collections::HashMap;
    /// use std::env;
    /// use typemap::Key;
    ///
    /// struct MessageEventCounter;
    ///
    /// impl Key for MessageEventCounter {
    ///     type Value = HashMap<String, u64>;
    /// }
    ///
    /// macro_rules! reg {
    ///     ($ctx:ident $name:expr) => {
    ///         {
    ///             let mut data = $ctx.data.lock();
    ///             let counter = data.get_mut::<MessageEventCounter>().unwrap();
    ///             let entry = counter.entry($name).or_insert(0);
    ///             *entry += 1;
    ///         }
    ///     };
    /// }
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, _: Message) { reg!(ctx "MessageCreate") }
    ///     fn on_message_delete(&self, ctx: Context, _: ChannelId, _: MessageId) {
    ///         reg!(ctx "MessageDelete") }
    ///     fn on_message_delete_bulk(&self, ctx: Context, _: ChannelId, _: Vec<MessageId>) {
    ///         reg!(ctx "MessageDeleteBulk") }
    ///     fn on_message_update(&self, ctx: Context, _: ChannelId, _: MessageId) {
    ///         reg!(ctx "MessageUpdate") }
    /// }
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);
    ///
    /// {
    ///     let mut data = client.data.lock();
    ///     data.insert::<MessageEventCounter>(HashMap::default());
    /// }
    ///
    /// client.start().unwrap();
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
    pub data: Arc<Mutex<ShareMap>>,
    /// A vector of all active shards that have received their [`Event::Ready`]
    /// payload, and have dispatched to [`on_ready`] if an event handler was
    /// configured.
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`on_ready`]: #method.on_ready
    #[cfg(feature = "framework")] framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
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
    /// # use std::error::Error;
    /// # use std::time::Duration;
    /// # use std::{env, thread};
    ///
    /// # fn try_main() -> Result<(), Box<Error>> {
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler { }
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
    ///
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// thread::spawn(move || {
    ///     loop {
    ///         println!("Shard count instantiated: {}",
    ///                  shard_manager.lock().shards_instantiated().len());
    ///
    ///         thread::sleep(Duration::from_millis(5000));
    ///     }
    /// });
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Shutting down all connections after one minute of operation:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::{Client, EventHandler};
    /// use std::time::Duration;
    /// use std::{env, thread};
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler { }
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
    ///
    /// // Create a clone of the `Arc` containing the shard manager.
    /// let shard_manager = client.shard_manager.clone();
    ///
    /// // Create a thread which will sleep for 60 seconds and then have the
    /// // shard manager shutdown.
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_secs(60));
    ///
    ///     shard_manager.lock().shutdown_all();
    ///
    ///     println!("Shutdown shard manager!");
    /// });
    ///
    /// println!("Client shutdown: {:?}", client.start());
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`Client::start_shard`]: #method.start_shard
    /// [`Client::start_shards`]: #method.start_shards
    pub shard_manager: Arc<Mutex<ShardManager>>,
    shard_manager_worker: ShardManagerMonitor,
    /// The threadpool shared by all shards.
    ///
    /// Defaults to 5 threads, which should suffice small bots. Consider
    /// increasing this number as your bot grows.
    pub threadpool: ThreadPool,
    /// The token in use by the client.
    pub token: Arc<Mutex<String>>,
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
}

impl Client {
    /// Creates a Client for a bot user.
    ///
    /// Discord has a requirement of prefixing bot tokens with `"Bot "`, which
    /// this function will automatically do for you if not already included.
    ///
    /// # Examples
    ///
    /// Create a Client, using a token from an environment variable:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let client = Client::new(&token, Handler)?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #    try_main().unwrap();
    /// # }
    /// ```
    pub fn new<H>(token: &str, handler: H) -> Result<Self>
        where H: EventHandler + Send + Sync + 'static {
        let token = token.trim();

        let token = if token.starts_with("Bot ") {
            token.to_string()
        } else {
            format!("Bot {}", token)
        };

        http::set_token(&token);
        let locked = Arc::new(Mutex::new(token));

        let name = "serenity client".to_owned();
        let threadpool = ThreadPool::with_name(name, 5);
        let url = Arc::new(Mutex::new(http::get_gateway()?.url));
        let data = Arc::new(Mutex::new(ShareMap::custom()));
        let event_handler = Arc::new(handler);

        #[cfg(feature = "framework")]
        let framework = Arc::new(Mutex::new(None));
        #[cfg(feature = "voice")]
        let voice_manager = Arc::new(Mutex::new(ClientVoiceManager::new(
            0,
            UserId(0),
        )));

        let (shard_manager, shard_manager_worker) = {
            ShardManager::new(ShardManagerOptions {
                data: &data,
                event_handler: &event_handler,
                #[cfg(feature = "framework")]
                framework: &framework,
                shard_index: 0,
                shard_init: 0,
                shard_total: 0,
                threadpool: threadpool.clone(),
                token: &locked,
                #[cfg(feature = "voice")]
                voice_manager: &voice_manager,
                ws_url: &url,
            })
        };

        Ok(Client {
            token: locked,
            ws_uri: url,
            #[cfg(feature = "framework")]
            framework,
            data,
            shard_manager,
            shard_manager_worker,
            threadpool,
            #[cfg(feature = "voice")]
            voice_manager,
        })
    }

    /// Sets a framework to be used with the client. All message events will be
    /// passed through the framework _after_ being passed to the [`message`]
    /// event handler.
    ///
    /// See the [framework module-level documentation][framework docs] for more
    /// information on usage.
    ///
    /// # Examples
    ///
    /// Create a simple framework that responds to a `~ping` command:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c.prefix("~"))
    ///     .on("ping", |_, msg, _| {
    ///         msg.channel_id.say("Pong!")?;
    ///
    ///         Ok(())
    ///      }));
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Using your own framework:
    ///
    /// ```rust,ignore
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// use serenity::Framework;
    /// use serenity::client::Context;
    /// use serenity::model::*;
    /// use tokio_core::reactor::Handle;
    /// use std::collections::HashMap;
    ///
    ///
    /// struct MyFramework {
    ///     commands: HashMap<String, Box<Fn(Message, Vec<String>)>>,
    /// }
    ///
    /// impl Framework for MyFramework {
    ///     fn dispatch(&mut self, _: Context, msg: Message, tokio_handle: &Handle) {
    ///         let args = msg.content.split_whitespace();
    ///         let command = match args.next() {
    ///             Some(command) => {
    ///                 if !command.starts_with('*') { return; }
    ///                 command
    ///             },
    ///             None => return,
    ///         };
    ///
    ///         let command = match self.commands.get(&command) {
    ///             Some(command) => command, None => return,
    ///         };
    ///
    ///         tokio_handle.spawn_fn(move || { (command)(msg, args); Ok() });
    ///     }
    /// }
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::new(&token, Handler).unwrap();
    /// client.with_framework(MyFramework { commands: {
    ///     let mut map = HashMap::new();
    ///     map.insert("ping".to_string(), Box::new(|msg, _| msg.channel_id.say("pong!")));
    ///     map
    /// }});
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    /// Refer to the documentation for the `framework` module for more in-depth
    /// information.
    ///
    /// [`message`]: trait.EventHandler.html#method.message
    /// [framework docs]: ../framework/index.html
    #[cfg(feature = "framework")]
    pub fn with_framework<F: Framework + Send + 'static>(&mut self, f: F) {
        *self.framework.lock() = Some(Box::new(f));
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
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// if let Err(why) = client.start() {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [gateway docs]: ../gateway/index.html#sharding
    pub fn start(&mut self) -> Result<()> {
        self.start_connection([0, 0, 1])
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
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// if let Err(why) = client.start_autosharded() {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
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
    pub fn start_autosharded(&mut self) -> Result<()> {
        let (x, y) = {
            let res = http::get_bot_gateway()?;

            (res.shards as u64 - 1, res.shards as u64)
        };

        self.start_connection([0, x, y])
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
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// if let Err(why) = client.start_shard(3, 5) {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Start shard 0 of 1 (you may also be interested in [`start`] or
    /// [`start_autosharded`]):
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
    ///
    /// if let Err(why) = client.start_shard(0, 1) {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
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
    pub fn start_shard(&mut self, shard: u64, shards: u64) -> Result<()> {
        self.start_connection([shard, shard, shards])
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
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// if let Err(why) = client.start_shards(8) {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
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
    pub fn start_shards(&mut self, total_shards: u64) -> Result<()> {
        self.start_connection([0, total_shards - 1, total_shards])
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
    /// ```rust,ignore
    /// # use serenity::prelude::EventHandler;
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN").unwrap();
    /// let mut client = Client::new(&token, Handler);
    ///
    /// let _ = client.start_shard_range([4, 7], 10);
    /// ```
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// # use std::error::Error;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// if let Err(why) = client.start_shard_range([4, 7], 10) {
    ///     println!("Err with client: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
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
    pub fn start_shard_range(&mut self, range: [u64; 2], total_shards: u64) -> Result<()> {
        self.start_connection([range[0], range[1], total_shards])
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
    fn start_connection(&mut self, shard_data: [u64; 3]) -> Result<()> {
        #[cfg(feature = "voice")]
        self.voice_manager.lock().set_shard_count(shard_data[2]);

        // This is kind of gross, but oh well.
        //
        // Both the framework and voice bridge need the user's ID, so we'll only
        // retrieve it over REST if at least one of those are enabled.
        #[cfg(any(all(feature = "standard_framework", feature = "framework"),
                  feature = "voice"))]
        {
            let user = http::get_current_user()?;

            // Update the framework's current user if the feature is enabled.
            //
            // This also acts as a form of check to ensure the token is correct.
            #[cfg(all(feature = "standard_framework", feature = "framework"))]
            {
                if let Some(ref mut framework) = *self.framework.lock() {
                    framework.update_current_user(user.id);
                }
            }

            #[cfg(feature = "voice")]
            {
                self.voice_manager.lock().set_user_id(user.id);
            }
        }

        {
            let mut manager = self.shard_manager.lock();

            let init = shard_data[1] - shard_data[0] + 1;

            manager.set_shards(shard_data[0], init, shard_data[2]);

            debug!(
                "Initializing shard info: {} - {}/{}",
                shard_data[0],
                init,
                shard_data[2],
            );

            if let Err(why) = manager.initialize() {
                error!("Failed to boot a shard: {:?}", why);
                info!("Shutting down all shards");

                manager.shutdown_all();

                return Err(Error::Client(ClientError::ShardBootFailure));
            }
        }

        self.shard_manager_worker.run();

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
pub fn validate_token(token: &str) -> Result<()> {
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
