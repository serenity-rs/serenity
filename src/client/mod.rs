//! The Client contains information about a single bot or user's token, as well
//! as event handlers. Dispatching events to configured handlers and starting
//! the shards' connections are handled directly via the client. In addition,
//! the `http` module and `Cache` are also automatically handled by the
//! Client module for you.
//!
//! A [`Context`] is provided for every handler. The context is a method of
//! accessing the lower-level HTTP functions relevant to the contextual channel.
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
#![allow(zero_ptr)]

mod context;
mod dispatch;
mod error;
mod event_store;

pub use self::context::Context;
pub use self::error::Error as ClientError;

// Note: the following re-exports are here for backwards compatibility
pub use ::gateway;
pub use ::http as rest;

#[cfg(feature="cache")]
pub use ::CACHE;

use self::dispatch::dispatch;
use self::event_store::EventStore;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{mem, thread};
use super::gateway::Shard;
use typemap::ShareMap;
use websocket::client::Receiver;
use websocket::result::WebSocketError;
use websocket::stream::WebSocketStream;
use ::http;
use ::internal::prelude::*;
use ::internal::ws_impl::ReceiverExt;
use ::model::event::*;
use ::model::*;

#[cfg(feature="framework")]
use ::framework::Framework;

/// The Client is the way to "login" and be able to start sending authenticated
/// requests over the REST API, as well as initializing a WebSocket connection
/// through [`Shard`]s. Refer to the
/// [documentation on using sharding][sharding docs] for more information.
///
/// # Event Handlers
///
/// Event handlers can be configured. For example, the event handler
/// [`on_message`] will be dispatched to whenever a [`Event::MessageCreate`] is
/// received over the connection.
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
/// use serenity::Client;
///
/// let mut client = Client::login("my token here");
///
/// client.on_message(|context, message| {
///     if message.content == "!ping" {
///         message.channel_id.say("Pong!");
///     }
/// });
///
/// client.start();
/// ```
///
/// [`Shard`]: gateway/struct.Shard.html
/// [`on_message`]: #method.on_message
/// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
/// [sharding docs]: gateway/index.html#sharding
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
    /// use serenity::Client;
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
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN").unwrap());
    ///
    /// {
    ///     let mut data = client.data.lock().unwrap();
    ///     data.insert::<MessageEventCounter>(HashMap::default());
    /// }
    ///
    /// macro_rules! reg {
    ///     ($ctx:ident $name:expr) => {
    ///         {
    ///             let mut data = $ctx.data.lock().unwrap();
    ///             let counter = data.get_mut::<MessageEventCounter>().unwrap();
    ///             let entry = counter.entry($name).or_insert(0);
    ///             *entry += 1;
    ///         }
    ///     };
    /// }
    ///
    /// client.on_message(|ctx, _| reg!(ctx "MessageCreate"));
    /// client.on_message_delete(|ctx, _| reg!(ctx "MessageDelete"));
    /// client.on_message_delete_bulk(|ctx, _| reg!(ctx "MessageDeleteBulk"));
    /// client.on_message_update(|ctx, _| reg!(ctx "MessageUpdate"));
    /// ```
    ///
    /// Refer to [example 05] for an example on using the `data` field.
    ///
    /// [`Context::data`]: struct.Context.html#method.data
    /// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    /// [`Event::MessageDelete`]: ../model/event/enum.Event.html#variant.MessageDelete
    /// [`Event::MessageDeleteBulk`]: ../model/event/enum.Event.html#variant.MessageDeleteBulk
    /// [`Event::MessageUpdate`]: ../model/event/enum.Event.html#variant.MessageUpdate
    /// [example 05]: https://github.com/zeyla/serenity/tree/master/examples/05_command_framework
    pub data: Arc<Mutex<ShareMap>>,
    /// A vector of all active shards that have received their [`Event::Ready`]
    /// payload, and have dispatched to [`on_ready`] if an event handler was
    /// configured.
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`on_ready`]: #method.on_ready
    event_store: Arc<RwLock<EventStore>>,
    #[cfg(feature="framework")]
    framework: Arc<Mutex<Framework>>,
    token: String,
}

#[allow(type_complexity)]
impl Client {
    /// Alias of [`login`].
    ///
    /// [`login`]: #method.login
    #[deprecated(since="0.1.5", note="Use `login` instead")]
    #[inline]
    pub fn login_bot(token: &str) -> Self {
        Self::login(token)
    }

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
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN")?;
    /// let client = Client::login(&token);
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #    try_main().unwrap();
    /// # }
    /// ```
    pub fn login(bot_token: &str) -> Self {
        let token = if bot_token.starts_with("Bot ") {
            bot_token.to_owned()
        } else {
            format!("Bot {}", bot_token)
        };

        login(token)
    }

    /// Sets a framework to be used with the client. All message events will be
    /// passed through the framework _after_ being passed to the [`on_message`]
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
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
    /// client.with_framework(|f| f
    ///     .configure(|c| c.prefix("~"))
    ///     .command("ping", |c| c.exec_str("Pong!")));
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Refer to the documentation for the `framework` module for more in-depth
    /// information.
    ///
    /// [`on_message`]: #method.on_message
    /// [framework docs]: ../framework/index.html
    #[cfg(feature="framework")]
    pub fn with_framework<F>(&mut self, f: F)
        where F: FnOnce(Framework) -> Framework + Send + Sync + 'static {
        self.framework = Arc::new(Mutex::new(f(Framework::default())));
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
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start(&mut self) -> Result<()> {
        self.start_connection(None, http::get_gateway()?.url)
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
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start_autosharded(&mut self) -> Result<()> {
        let mut res = http::get_bot_gateway()?;

        let x = res.shards as u64 - 1;
        let y = res.shards as u64;
        let url = mem::replace(&mut res.url, String::default());

        drop(res);

        self.start_connection(Some([0, x, y]), url)
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
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// [`start`]: #method.start
    /// [`start_autosharded`]: #method.start_autosharded
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start_shard(&mut self, shard: u64, shards: u64) -> Result<()> {
        self.start_connection(Some([shard, shard, shards]), http::get_gateway()?.url)
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
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// [`start_shard`]: #method.start_shard
    /// [`start_shard_range`]: #method.start_shard_range
    /// [Gateway docs]: gateway/index.html#sharding
    pub fn start_shards(&mut self, total_shards: u64) -> Result<()> {
        self.start_connection(Some([0, total_shards - 1, total_shards]), http::get_gateway()?.url)
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
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
    /// let mut client = Client::login(&token);
    ///
    /// let _ = client.start_shard_range([4, 7], 10);
    /// ```
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN")?);
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
    /// [`start_shard`]: #method.start_shard
    /// [`start_shards`]: #method.start_shards
    /// [Gateway docs]: gateway/index.html#sharding
    pub fn start_shard_range(&mut self, range: [u64; 2], total_shards: u64) -> Result<()> {
        self.start_connection(Some([range[0], range[1], total_shards]), http::get_gateway()?.url)
    }

    /// Attaches a handler for when a [`ChannelCreate`] is received.
    ///
    /// # Examples
    ///
    /// If the channel is a guild channel, send `"first"` to the channel when
    /// one is created:
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// use serenity::model::Channel;
    ///
    /// client.on_channel_create(|ctx, channel| {
    ///     if let Channel::Guild(ch) = channel {
    ///         if let Err(why) = ch.read().unwrap().say("first") {
    ///             println!("Err sending first message: {:?}", why);
    ///         }
    ///     }
    /// });
    /// ```
    ///
    /// [`ChannelCreate`]: ../model/event/enum.Event.html#variant.ChannelCreate
    pub fn on_channel_create<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelDelete`] is received.
    ///
    /// # Examples
    ///
    /// If the channel is a guild channel, send the name of the channel to the
    /// guild's default channel.
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// use serenity::model::{Channel, ChannelId};
    ///
    /// client.on_channel_delete(|ctx, channel| {
    ///     if let Channel::Guild(channel) = channel {
    ///         let (content, default_channel_id) = {
    ///             let reader = channel.read().unwrap();
    ///             let content = format!("A channel named '{}' was deleted.", reader.name);
    ///             let id = ChannelId(reader.guild_id.0);
    ///
    ///             (content, id)
    ///         };
    ///
    ///         if let Err(why) = default_channel_id.say(&content) {
    ///             println!("Err sending message to default channel: {:?}", why);
    ///         }
    ///     }
    /// });
    /// ```
    ///
    /// [`ChannelDelete`]: ../model/event/enum.Event.html#variant.ChannelDelete
    pub fn on_channel_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelPinsUpdate`] is received.
    ///
    /// [`ChannelPinsUpdate`]: ../model/event/enum.Event.html#variant.ChannelPinsUpdate
    pub fn on_channel_pins_update<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelPinsUpdateEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_pins_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildCreate`] is received.
    ///
    /// [`GuildCreate`]: ../model/event/enum.Event.html#variant.GuildCreate
    pub fn on_guild_create<F>(&mut self, handler: F)
        where F: Fn(Context, Guild) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildEmojisUpdate`] is received.
    ///
    /// The `HashMap` of emojis is the new full list of emojis.
    ///
    /// [`GuildEmojisUpdate`]: ../model/event/enum.Event.html#variant.GuildEmojisUpdate
    pub fn on_guild_emojis_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, HashMap<EmojiId, Emoji>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_emojis_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildIntegrationsUpdate`] is received.
    ///
    /// [`GuildIntegrationsUpdate`]: ../model/event/enum.Event.html#variant.GuildIntegrationsUpdate
    pub fn on_guild_integrations_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_integrations_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberAdd`] is received.
    ///
    /// [`GuildMemberAdd`]: ../model/event/enum.Event.html#variant.GuildMemberAdd
    pub fn on_guild_member_add<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Member) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_member_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMembersChunk`] is received.
    ///
    /// [`GuildMembersChunk`]: ../model/event/enum.Event.html#variant.GuildMembersChunk
    pub fn on_guild_members_chunk<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, HashMap<UserId, Member>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_members_chunk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleCreate`] is received.
    ///
    /// [`GuildRoleCreate`]: ../model/event/enum.Event.html#variant.GuildRoleCreate
    pub fn on_guild_role_create<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Role) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_role_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnavailable`] is received.
    ///
    /// [`GuildUnavailable`]: ../model/event/enum.Event.html#variant.GuildUnavailable
    pub fn on_guild_unavailable<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_unavailable = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildBan`] is received.
    ///
    /// [`GuildBan`]: ../model/event/enum.Event.html#variant.GuildBan
    pub fn on_member_ban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_ban_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnban`] is received.
    ///
    /// [`GuildUnban`]: ../model/event/enum.Event.html#variant.GuildUnban
    pub fn on_member_unban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_ban_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageCreate`] is received.
    ///
    /// # Examples
    ///
    /// Print the contents of every received message:
    ///
    /// ```rust,ignore
    /// use serenity::Client;
    ///
    /// let mut client = Client::login("bot token here");
    ///
    /// client.on_message(|_context, message| {
    ///     println!("{}", message.content);
    /// });
    ///
    /// let _ = client.start();
    /// ```
    ///
    /// [`MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    pub fn on_message<F>(&mut self, handler: F)
        where F: Fn(Context, Message) + Send + Sync + 'static {

        self.event_store.write()
            .unwrap()
            .on_message = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDelete`] is received.
    ///
    /// [`MessageDelete`]: ../model/event/enum.Event.html#variant.MessageDelete
    pub fn on_message_delete<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_message_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDeleteBulk`] is received.
    ///
    /// [`MessageDeleteBulk`]: ../model/event/enum.Event.html#variant.MessageDeleteBulk
    pub fn on_message_delete_bulk<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Vec<MessageId>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_message_delete_bulk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageUpdate`] is received.
    ///
    /// [`MessageUpdate`]: ../model/event/enum.Event.html#variant.MessageUpdate
    pub fn on_message_update<F>(&mut self, handler: F)
        where F: Fn(Context, MessageUpdateEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_message_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresencesReplace`] is received.
    ///
    /// [`PresencesReplace`]: ../model/event/enum.Event.html#variant.PresencesReplace
    pub fn on_presence_replace<F>(&mut self, handler: F)
        where F: Fn(Context, Vec<Presence>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_presence_replace = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresenceUpdate`] is received.
    ///
    /// [`PresenceUpdate`]: ../model/event/enum.Event.html#variant.PresenceUpdate
    pub fn on_presence_update<F>(&mut self, handler: F)
        where F: Fn(Context, PresenceUpdateEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_presence_update = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionAdd`] is received.
    ///
    /// [`ReactionAdd`]: ../model/event/enum.Event.html#variant.ReactionAdd
    pub fn on_reaction_add<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_reaction_add = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemove`] is received.
    ///
    /// [`ReactionRemove`]: ../model/event/enum.Event.html#variant.ReactionRemove
    pub fn on_reaction_remove<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_reaction_remove = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemoveAll`] is received.
    ///
    /// [`ReactionRemoveAll`]: ../model/event/enum.Event.html#variant.ReactionRemoveAll
    pub fn on_reaction_remove_all<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_reaction_remove_all = Some(Arc::new(handler));
    }

    /// Register an event to be called whenever a Ready event is received.
    ///
    /// Registering a handler for the ready event is good for noting when your
    /// bot has established a connection to the gateway through a [`Shard`].
    ///
    /// **Note**: The Ready event is not guarenteed to be the first event you
    /// will receive by Discord. Do not actively rely on it.
    ///
    /// # Examples
    ///
    /// Print the [current user][`CurrentUser`]'s name on ready:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
    /// let mut client = Client::login(&token);
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected", ready.user.name);
    /// });
    /// ```
    ///
    /// [`CurrentUser`]: ../model/struct.CurrentUser.html
    /// [`Shard`]: gateway/struct.Shard.html
    pub fn on_ready<F>(&mut self, handler: F)
        where F: Fn(Context, Ready) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_ready = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientAdd`] is received.
    ///
    /// [`ChannelRecipientAdd`]: ../model/event/enum.Event.html#variant.ChannelRecipientAdd
    pub fn on_recipient_add<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_recipient_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientRemove`] is received.
    ///
    /// [`ChannelRecipientRemove`]: ../model/event/enum.Event.html#variant.ChannelRecipientRemove
    pub fn on_recipient_remove<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_recipient_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`Resumed`] is received.
    ///
    /// [`Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    pub fn on_resume<F>(&mut self, handler: F)
        where F: Fn(Context, ResumedEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_resume = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`TypingStart`] is received.
    ///
    /// [`TypingStart`]: ../model/event/enum.Event.html#variant.TypingStart
    pub fn on_typing_start<F>(&mut self, handler: F)
        where F: Fn(Context, TypingStartEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_typing_start = Some(Arc::new(handler));
    }

    /// Attaches a handler for when an [`Unknown`] is received.
    ///
    /// [`Unknown`]: ../model/event/enum.Event.html#variant.Unknown
    pub fn on_unknown<F>(&mut self, handler: F)
        where F: Fn(Context, String, Value) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_unknown = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceServerUpdate`] is received.
    ///
    /// [`VoiceServerUpdate`]: ../model/event/enum.Event.html#variant.VoiceServerUpdate
    pub fn on_voice_server_update<F>(&mut self, handler: F)
        where F: Fn(Context, VoiceServerUpdateEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_voice_server_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceStateUpdate`] is received.
    ///
    /// [`VoiceStateUpdate`]: ../model/event/enum.Event.html#variant.VoiceStateUpdate
    pub fn on_voice_state_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<GuildId>, VoiceState) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_voice_state_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`WebhookUpdate`] is received.
    ///
    /// [`WebhookUpdate`]: ../model/event/enum.Event.html#variant.WebhookUpdate
    pub fn on_webhook_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, ChannelId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_webhook_update = Some(Arc::new(handler));
    }

    // Shard data layout is:
    // 0: first shard number to initialize
    // 1: shard number to initialize up to and including
    // 2: total number of shards the bot is sharding for
    //
    // Not all shards need to be initialized in this process.
    fn start_connection(&mut self, shard_data: Option<[u64; 3]>, url: String) -> Result<()> {
        // Update the framework's current user if the feature is enabled.
        //
        // This also acts as a form of check to ensure the token is correct.
        #[cfg(feature="framework")]
        {
            let user = http::get_current_user()?;

            self.framework.lock()
                .unwrap()
                .update_current_user(user.id, user.bot);
        }

        let gateway_url = Arc::new(Mutex::new(url));

        let shards_index = shard_data.map_or(0, |x| x[0]);
        let shards_total = shard_data.map_or(1, |x| x[1] + 1);

        for shard_number in shards_index..shards_total {
            let shard_info = shard_data.map(|s| [shard_number, s[2]]);

            let boot = boot_shard(&BootInfo {
                gateway_url: gateway_url.clone(),
                shard_info: shard_info,
                token: self.token.clone(),
            });

            match boot {
                Ok((shard, ready, receiver)) => {
                    #[cfg(feature="cache")]
                    {
                        CACHE.write()
                            .unwrap()
                            .update_with_ready(&ready);
                    }

                    let shard = Arc::new(Mutex::new(shard));

                    feature_framework! {{
                        dispatch(Event::Ready(ready),
                                 &shard,
                                 &self.framework,
                                 &self.data,
                                 &self.event_store);
                    } else {
                        dispatch(Event::Ready(ready),
                                 &shard,
                                 &self.data,
                                 &self.event_store);
                    }}

                    let monitor_info = feature_framework! {{
                        MonitorInfo {
                            data: self.data.clone(),
                            event_store: self.event_store.clone(),
                            framework: self.framework.clone(),
                            gateway_url: gateway_url.clone(),
                            receiver: receiver,
                            shard: shard,
                            shard_info: shard_info,
                            token: self.token.clone(),
                        }
                    } else {
                        MonitorInfo {
                            data: self.data.clone(),
                            event_store: self.event_store.clone(),
                            gateway_url: gateway_url.clone(),
                            receiver: receiver,
                            shard: shard,
                            shard_info: shard_info,
                            token: self.token.clone(),
                        }
                    }};

                    thread::spawn(move || {
                        monitor_shard(monitor_info);
                    });
                },
                Err(why) => warn!("Error starting shard {:?}: {:?}", shard_info, why),
            }

            // Wait 5 seconds between shard boots.
            //
            // We need to wait at least 5 seconds between READYs.
            thread::sleep(Duration::from_secs(5));
        }

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

#[cfg(feature="cache")]
impl Client {
    /// Attaches a handler for when a [`ChannelUpdate`] is received.
    ///
    /// Optionally provides the version of the channel before the update.
    ///
    /// [`ChannelUpdate`]: ../model/event/enum.Event.html#variant.ChannelUpdate
    pub fn on_channel_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Channel>, Channel) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildDelete`] is received.
    ///
    /// Returns a partial guild as well as - optionally - the full guild, with
    /// data like [`Role`]s. This can be `None` in the event that it was not in
    /// the [`Cache`].
    ///
    /// **Note**: The relevant guild is _removed_ from the Cache when this event
    /// is received. If you need to keep it, you can either re-insert it
    /// yourself back into the Cache or manage it in another way.
    ///
    /// [`GuildDelete`]: ../model/event/enum.Event.html#variant.GuildDelete
    /// [`Role`]: ../model/struct.Role.html
    /// [`Cache`]: ../cache/struct.Cache.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild, Option<Arc<RwLock<Guild>>>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberRemove`] is received.
    ///
    /// Returns the user's associated `Member` object, _if_ it existed in the
    /// cache.
    ///
    /// [`GuildMemberRemove`]: ../model/event/enum.Event.html#variant.GuildMemberRemove
    pub fn on_guild_member_remove<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User, Option<Member>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/event/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Member>, Member) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/event/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId, Option<Role>) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleUpdate`] is received.
    ///
    /// The optional `Role` is the role prior to updating. This can be `None` if
    /// it did not exist in the [`Cache`] before the update.
    ///
    /// [`GuildRoleUpdate`]: ../model/event/enum.Event.html#variant.GuildRoleUpdate
    /// [`Cache`]: ../cache/struct.Cache.html
    pub fn on_guild_role_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Option<Role>, Role) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_role_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUpdate`] is received.
    ///
    /// [`GuildUpdate`]: ../model/event/enum.Event.html#variant.GuildUpdate
    pub fn on_guild_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Arc<RwLock<Guild>>>, PartialGuild) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserUpdate`] is received.
    ///
    /// The old current user will be provided as well.
    ///
    /// [`UserUpdate`]: ../model/event/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser, CurrentUser) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

#[cfg(not(feature="cache"))]
impl Client {
    /// Attaches a handler for when a [`ChannelUpdate`] is received.
    ///
    /// [`ChannelUpdate`]: ../model/event/enum.Event.html#variant.ChannelUpdate
    pub fn on_channel_update<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_channel_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildDelete`] is received.
    ///
    /// [`GuildDelete`]: ../model/event/enum.Event.html#variant.GuildDelete
    /// [`Role`]: ../model/struct.Role.html
    /// [`Cache`]: ../cache/struct.Cache.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberRemove`] is received.
    ///
    /// Returns the user's associated `Member` object, _if_ it existed in the
    /// cache.
    ///
    /// [`GuildMemberRemove`]: ../model/event/enum.Event.html#variant.GuildMemberRemove
    pub fn on_guild_member_remove<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/event/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildMemberUpdateEvent) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/event/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleUpdate`] is received.
    ///
    /// [`GuildRoleUpdate`]: ../model/event/enum.Event.html#variant.GuildRoleUpdate
    /// [`Cache`]: ../cache/struct.Cache.html
    pub fn on_guild_role_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Role) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_role_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUpdate`] is received.
    ///
    /// [`GuildUpdate`]: ../model/event/enum.Event.html#variant.GuildUpdate
    pub fn on_guild_update<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_guild_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserUpdate`] is received.
    ///
    /// [`UserUpdate`]: ../model/event/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser) + Send + Sync + 'static {
        self.event_store.write()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

struct BootInfo {
    gateway_url: Arc<Mutex<String>>,
    shard_info: Option<[u64; 2]>,
    token: String,
}

#[cfg(feature="framework")]
struct MonitorInfo {
    data: Arc<Mutex<ShareMap>>,
    event_store: Arc<RwLock<EventStore>>,
    framework: Arc<Mutex<Framework>>,
    gateway_url: Arc<Mutex<String>>,
    receiver: Receiver<WebSocketStream>,
    shard: Arc<Mutex<Shard>>,
    shard_info: Option<[u64; 2]>,
    token: String,
}

#[cfg(not(feature="framework"))]
struct MonitorInfo {
    data: Arc<Mutex<ShareMap>>,
    event_store: Arc<RwLock<EventStore>>,
    gateway_url: Arc<Mutex<String>>,
    receiver: Receiver<WebSocketStream>,
    shard: Arc<Mutex<Shard>>,
    shard_info: Option<[u64; 2]>,
    token: String,
}

fn boot_shard(info: &BootInfo) -> Result<(Shard, ReadyEvent, Receiver<WebSocketStream>)> {
    // Make ten attempts to boot the shard, exponentially backing off; if it
    // still doesn't boot after that, accept it as a failure.
    //
    // After three attempts, start re-retrieving the gateway URL. Before that,
    // use the cached one.
    for attempt_number in 1..11u64 {
        // If we've tried over 3 times so far, get a new gateway URL.
        //
        // If doing so fails, count this as a boot attempt.
        if attempt_number > 3 {
            match http::get_gateway() {
                Ok(g) => *info.gateway_url.lock().unwrap() = g.url,
                Err(why) => {
                    warn!("Failed to retrieve gateway URL: {:?}", why);

                    // Failed -- start over.
                    continue;
                },
            }
        }

        let attempt = Shard::new(&info.gateway_url.lock().unwrap(),
                                 &info.token,
                                 info.shard_info);

        match attempt {
            Ok((shard, ready, receiver)) => {
                #[cfg(feature="cache")]
                {
                    CACHE.write()
                        .unwrap()
                        .update_with_ready(&ready);
                }

                info!("Successfully booted shard: {:?}", info.shard_info);

                return Ok((shard, ready, receiver));
            },
            Err(why) => warn!("Failed to boot shard: {:?}", why),
        }
    }

    // Hopefully _never_ happens?
    Err(Error::Client(ClientError::ShardBootFailure))
}

fn monitor_shard(mut info: MonitorInfo) {
    handle_shard(&mut info);

    loop {
        let mut boot_successful = false;

        for _ in 0..3 {
            let boot = boot_shard(&BootInfo {
                gateway_url: info.gateway_url.clone(),
                shard_info: info.shard_info,
                token: info.token.clone(),
            });

            match boot {
                Ok((new_shard, ready, new_receiver)) => {
                    #[cfg(feature="cache")]
                    {
                        CACHE.write().unwrap().update_with_ready(&ready);
                    }

                    *info.shard.lock().unwrap() = new_shard;
                    info.receiver = new_receiver;

                    boot_successful = true;

                    feature_framework! {{
                        dispatch(Event::Ready(ready),
                                 &info.shard,
                                 &info.framework,
                                 &info.data,
                                 &info.event_store);
                    } else {
                        dispatch(Event::Ready(ready),
                                 &info.shard,
                                 &info.data,
                                 &info.event_store);
                    }}

                    break;
                },
                Err(why) => warn!("Failed to boot shard: {:?}", why),
            }
        }

        if boot_successful {
            handle_shard(&mut info);
        } else {
            break;
        }

        // The shard died: redo the cycle.
    }

    error!("Completely failed to reboot shard");
}

fn handle_shard(info: &mut MonitorInfo) {
    loop {
        let event = match info.receiver.recv_json(GatewayEvent::decode) {
            Err(Error::WebSocket(WebSocketError::NoDataAvailable)) => {
                debug!("Attempting to shutdown receiver/sender");

                match info.shard.lock().unwrap().resume(&mut info.receiver) {
                    Ok((_, receiver)) => {
                        debug!("Successfully resumed shard");

                        info.receiver = receiver;

                        continue;
                    },
                    Err(why) => {
                        warn!("Err resuming shard: {:?}", why);

                        return;
                    },
                }
            },
            other => other,
        };

        trace!("Received event on shard handler: {:?}", event);

        // This will only lock when _updating_ the shard, resuming, etc. Most
        // of the time, this won't be locked (i.e. when receiving an event over
        // the receiver, separate from the shard itself).
        let event = match info.shard.lock().unwrap().handle_event(event, &mut info.receiver) {
            Ok(Some((event, Some(new_receiver)))) => {
                info.receiver = new_receiver;

                event
            },
            Ok(Some((event, None))) => event,
            Ok(None) => continue,
            Err(why) => {
                error!("Shard handler received err: {:?}", why);

                continue;
            },
        };

        feature_framework! {{
            dispatch(event,
                     &info.shard,
                     &info.framework,
                     &info.data,
                     &info.event_store);
        } else {
            dispatch(event,
                     &info.shard,
                     &info.data,
                     &info.event_store);
        }}
    }
}

fn login(token: String) -> Client {
    http::set_token(&token);

    feature_framework! {{
        Client {
            data: Arc::new(Mutex::new(ShareMap::custom())),
            event_store: Arc::new(RwLock::new(EventStore::default())),
            framework: Arc::new(Mutex::new(Framework::default())),
            token: token,
        }
    } else {
        Client {
            data: Arc::new(Mutex::new(ShareMap::custom())),
            event_store: Arc::new(RwLock::new(EventStore::default())),
            token: token,
        }
    }}
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
