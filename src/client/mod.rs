//! The Client contains information about a single bot or user's token, as well
//! as event handlers. Dispatching events to configured handlers and starting
//! the shards' connections are handled directly via the client. In addition,
//! the [`rest`] module and [`Cache`] are also automatically handled by the
//! Client module for you.
//!
//! A [`Context`] is provided for every handler. The context is an ergonomic
//! method of accessing the lower-level HTTP functions.
//!
//! The `rest` module is the lower-level method of interacting with the Discord
//! REST API. Realistically, there should be little reason to use this yourself,
//! as the Context will do this for you. A possible use case of using the `rest`
//! module is if you do not have a Cache, for purposes such as low memory
//! requirements.
//!
//! Click [here][Client examples] for an example on how to use a `Client`.
//!
//! [`Client`]: struct.Client.html#examples
//! [`Context`]: struct.Context.html
//! [`Cache`]: ../ext/cache/index.html
//! [`rest`]: rest/index.html
//! [Client examples]: struct.Client.html#examples

pub mod gateway;
pub mod rest;

mod context;
mod dispatch;
mod error;
mod event_store;
mod login_type;

pub use self::context::Context;
pub use self::error::Error as ClientError;
pub use self::login_type::LoginType;

use self::dispatch::dispatch;
use self::event_store::EventStore;
use self::gateway::Shard;
use serde_json::builder::ObjectBuilder;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use websocket::client::Receiver;
use websocket::stream::WebSocketStream;
use ::internal::prelude::{Error, Result, Value};
use ::internal::ws_impl::ReceiverExt;
use ::model::event::{
    ChannelPinsAckEvent,
    ChannelPinsUpdateEvent,
    Event,
    GatewayEvent,
    GuildSyncEvent,
    MessageUpdateEvent,
    PresenceUpdateEvent,
    ResumedEvent,
    TypingStartEvent,
    VoiceServerUpdateEvent,
};
use ::model::*;

#[cfg(feature = "framework")]
use ::ext::framework::Framework;

#[cfg(feature = "cache")]
use ::ext::cache::Cache;

#[cfg(not(feature = "cache"))]
use ::model::event::{
    CallUpdateEvent,
    GuildMemberUpdateEvent,
    UserSettingsUpdateEvent,
};

#[cfg(feature = "cache")]
lazy_static! {
    /// A mutable and lazily-initialized static binding. It can be accessed
    /// across any function and in any context.
    ///
    /// This [`Cache`] instance is updated for every event received, so you do
    /// not need to maintain your own cache.
    ///
    /// See the [cache module documentation] for more details.
    ///
    /// The Cache itself is wrapped within an `RwLock`, which allows for
    /// multiple readers or at most one writer at a time across threads. This
    /// means that you may have multiple commands reading from the Cache
    /// concurrently.
    ///
    /// # Examples
    ///
    /// Retrieve the [current user][`CurrentUser`]'s Id, by opening a Read
    /// guard:
    ///
    /// ```rust,ignore
    /// use serenity::client::CACHE;
    ///
    /// println!("{}", CACHE.read().unwrap().user.id);
    /// ```
    ///
    /// By `unwrap()`ing, the thread managing an event dispatch will be blocked
    /// until the guard can be opened.
    ///
    /// If you do not want to block the current thread, you may instead use
    /// `RwLock::try_read`. Refer to `RwLock`'s documentation in the stdlib for
    /// more information.
    ///
    /// [`CurrentUser`]: ../model/struct.CurrentUser.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    /// [cache module documentation]: ../ext/cache/index.html
    pub static ref CACHE: RwLock<Cache> = RwLock::new(Cache::default());
}

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
/// let mut client = Client::login_bot("my token here");
///
/// client.on_message(|context, message| {
///     if message.content == "!ping" {
///         context.say("Pong!");
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
    /// A vector of all active shards that have received their [`Event::Ready`]
    /// payload, and have dispatched to [`on_ready`] if an event handler was
    /// configured.
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`on_ready`]: #method.on_ready
    event_store: Arc<Mutex<EventStore>>,
    #[cfg(feature="framework")]
    framework: Arc<Mutex<Framework>>,
    login_type: LoginType,
    pub shards: Vec<Arc<Mutex<Shard>>>,
    token: String,
}

#[allow(type_complexity)]
impl Client {
    /// Creates a Client for a bot user.
    ///
    /// Discord has a requirement of prefixing bot tokens with `"Bot "`, which
    /// this function will automatically do for you.
    pub fn login_bot(bot_token: &str) -> Client {
        let token = format!("Bot {}", bot_token);

        login(&token, LoginType::Bot)
    }

    /// Create an instance from "raw values". This allows you to manually
    /// specify whether to login as a [`Bot`] or [`User`], and does not modify
    /// the token in any way regardless.
    ///
    /// [`Bot`]: enum.LoginType.html#variant.Bot
    /// [`User`]: enum.LoginType.html#variant.User
    #[doc(hidden)]
    pub fn login_raw(token: &str, login_type: LoginType) -> Client {
        login(&token.to_owned(), login_type)
    }

    /// Creates a Client for a user.
    ///
    /// **Note**: Read the notes for [`LoginType::User`] prior to using this, as
    /// there are restrictions on usage.
    ///
    /// [`LoginType::User`]: enum.LoginType.html#variant.User
    pub fn login_user(user_token: &str) -> Client {
        login(&user_token.to_owned(), LoginType::User)
    }

    /// Logout from the Discord API. This theoretically is supposed to
    /// invalidate the current token, but currently does not do anything. This
    /// is an issue on Discord's side.
    ///
    /// **Note**: This can only be used by users.
    pub fn logout(self) -> Result<()> {
        if self.login_type == LoginType::Bot {
            return Err(Error::Client(ClientError::InvalidOperationAsBot));
        }

        let map = ObjectBuilder::new()
            .insert("provider", Value::Null)
            .insert("token", Value::Null)
            .build();

        rest::logout(map)
    }

    /// Sets a framework to be used with the client. All message events will be
    /// passed through the framework _after_ being passed to the [`on_message`]
    /// event handler.
    ///
    /// See the [framework module-level documentation][framework docs] for more
    /// information on usage.
    ///
    /// [`on_message`]: #method.on_message
    /// [framework docs]: ../ext/framework/index.html
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
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start(&mut self) -> Result<()> {
        self.start_connection(None)
    }

    /// Establish the connection(s) and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the
    /// events to your registered handlers.
    ///
    /// This will retrieve an automatically determined number of shards to use
    /// from the API - determined by Discord - and then open a number of shards
    /// equivilant to that amount.
    ///
    /// Refer to the [Gateway documentation][gateway docs] for more information
    /// on effectively using sharding.
    ///
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start_autosharded(&mut self) -> Result<()> {
        let res = try!(rest::get_bot_gateway());

        self.start_connection(Some([0, res.shards as u8 - 1, res.shards as u8]))
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
    /// [gateway docs]: gateway/index.html#sharding
    pub fn start_shard(&mut self, shard: u8, shards: u8) -> Result<()> {
        self.start_connection(Some([shard, shard, shards]))
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
    /// [`start_shard`]: #method.start_shard
    /// [`start_shard_range`]: #method.start_shards
    /// [Gateway docs]: gateway/index.html#sharding
    pub fn start_shards(&mut self, total_shards: u8) -> Result<()> {
        self.start_connection(Some([0, total_shards - 1, total_shards]))
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
    /// let mut client = Client::login_bot(&token);
    ///
    /// let _ = client.start_shard_range([4, 7], 10);
    /// ```
    ///
    /// [`start_shard`]: #method.start_shard
    /// [`start_shards`]: #method.start_shards
    /// [Gateway docs]: gateway/index.html#sharding
    pub fn start_shard_range(&mut self, range: [u8; 2], total_shards: u8)
        -> Result<()> {
        self.start_connection(Some([range[0], range[1], total_shards]))
    }

    /// Attaches a handler for when a [`CallCreate`] is received.
    ///
    /// [`CallCreate`]: ../model/event/enum.Event.html#variant.CallCreate
    pub fn on_call_create<F>(&mut self, handler: F)
        where F: Fn(Context, Call) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelCreate`] is received.
    ///
    /// [`ChannelCreate`]: ../model/event/enum.Event.html#variant.ChannelCreate
    pub fn on_channel_create<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelDelete`] is received.
    ///
    /// [`ChannelDelete`]: ../model/event/enum.Event.html#variant.ChannelDelete
    pub fn on_channel_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelPinsAck`] is received.
    ///
    /// [`ChannelPinsAck`]: ../model/event/enum.Event.html#variant.ChannelPinsAck
    pub fn on_channel_pins_ack<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelPinsAckEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_pins_ack = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelPinsUpdate`] is received.
    ///
    /// [`ChannelPinsUpdate`]: ../model/event/enum.Event.html#variant.ChannelPinsUpdate
    pub fn on_channel_pins_update<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelPinsUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_pins_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`FriendSuggestionCreate`] is received.
    ///
    /// [`FriendSuggestionCreate`]: ../model/event/enum.Event.html#variant.FriendSuggestionCreate
    pub fn on_friend_suggestion_create<F>(&mut self, handler: F)
        where F: Fn(Context, User, Vec<SuggestionReason>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_friend_suggestion_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`FriendSuggestionDelete`] is received.
    ///
    /// [`FriendSuggestionDelete`]: ../model/event/enum.Event.html#variant.FriendSuggestionDelete
    pub fn on_friend_suggestion_delete<F>(&mut self, handler: F)
        where F: Fn(Context, UserId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_friend_suggestion_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildCreate`] is received.
    ///
    /// [`GuildCreate`]: ../model/event/enum.Event.html#variant.GuildCreate
    pub fn on_guild_create<F>(&mut self, handler: F)
        where F: Fn(Context, Guild) + Send + Sync + 'static {
        self.event_store.lock()
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
        self.event_store.lock()
            .unwrap()
            .on_guild_emojis_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildIntegrationsUpdate`] is received.
    ///
    /// [`GuildIntegrationsUpdate`]: ../model/event/enum.Event.html#variant.GuildIntegrationsUpdate
    pub fn on_guild_integrations_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_integrations_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberAdd`] is received.
    ///
    /// [`GuildMemberAdd`]: ../model/event/enum.Event.html#variant.GuildMemberAdd
    pub fn on_guild_member_add<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Member) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMembersChunk`] is received.
    ///
    /// [`GuildMembersChunk`]: ../model/event/enum.Event.html#variant.GuildMembersChunk
    pub fn on_guild_members_chunk<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, HashMap<UserId, Member>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_members_chunk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleCreate`] is received.
    ///
    /// [`GuildRoleCreate`]: ../model/event/enum.Event.html#variant.GuildRoleCreate
    pub fn on_guild_role_create<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Role) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleSync`] is received.
    ///
    /// [`GuildRoleSync`]: ../model/event/enum.Event.html#variant.GuildRoleSync
    pub fn on_guild_sync<F>(&mut self, handler: F)
        where F: Fn(Context, GuildSyncEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_sync = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnavailable`] is received.
    ///
    /// [`GuildUnavailable`]: ../model/event/enum.Event.html#variant.GuildUnavailable
    pub fn on_guild_unavailable<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_unavailable = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildBan`] is received.
    ///
    /// [`GuildBan`]: ../model/event/enum.Event.html#variant.GuildBan
    pub fn on_member_ban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_ban_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnban`] is received.
    ///
    /// [`GuildUnban`]: ../model/event/enum.Event.html#variant.GuildUnban
    pub fn on_member_unban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_ban_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageCreate`] is received.
    ///
    /// [`MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    pub fn on_message<F>(&mut self, handler: F)
        where F: Fn(Context, Message) + Send + Sync + 'static {

        self.event_store.lock()
            .unwrap()
            .on_message = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageAck`] is received.
    ///
    /// [`MessageAck`]: ../model/event/enum.Event.html#variant.MessageAck
    pub fn on_message_ack<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Option<MessageId>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_ack = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDelete`] is received.
    ///
    /// [`MessageDelete`]: ../model/event/enum.Event.html#variant.MessageDelete
    pub fn on_message_delete<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDeleteBulk`] is received.
    ///
    /// [`MessageDeleteBulk`]: ../model/event/enum.Event.html#variant.MessageDeleteBulk
    pub fn on_message_delete_bulk<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Vec<MessageId>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_delete_bulk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageUpdate`] is received.
    ///
    /// [`MessageUpdate`]: ../model/event/enum.Event.html#variant.MessageUpdate
    pub fn on_message_update<F>(&mut self, handler: F)
        where F: Fn(Context, MessageUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresencesReplace`] is received.
    ///
    /// [`PresencesReplace`]: ../model/event/enum.Event.html#variant.PresencesReplace
    pub fn on_presence_replace<F>(&mut self, handler: F)
        where F: Fn(Context, Vec<Presence>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_presence_replace = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresenceUpdate`] is received.
    ///
    /// [`PresenceUpdate`]: ../model/event/enum.Event.html#variant.PresenceUpdate
    pub fn on_presence_update<F>(&mut self, handler: F)
        where F: Fn(Context, PresenceUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_presence_update = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionAdd`] is received.
    ///
    /// [`ReactionAdd`]: ../model/event/enum.Event.html#variant.ReactionAdd
    pub fn on_reaction_add<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_reaction_add = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemove`] is received.
    ///
    /// [`ReactionRemove`]: ../model/event/enum.Event.html#variant.ReactionRemove
    pub fn on_reaction_remove<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_reaction_remove = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemoveAll`] is received.
    ///
    /// [`ReactionRemoveAll`]: ../model/event/enum.Event.html#variant.ReactionRemoveAll
    pub fn on_reaction_remove_all<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.lock()
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
    /// let mut client = Client::login_bot(&token);
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
        self.event_store.lock()
            .unwrap()
            .on_ready = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientAdd`] is received.
    ///
    /// [`ChannelRecipientAdd`]: ../model/event/enum.Event.html#variant.ChannelRecipientAdd
    pub fn on_recipient_add<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_recipient_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientRemove`] is received.
    ///
    /// [`ChannelRecipientRemove`]: ../model/event/enum.Event.html#variant.ChannelRecipientRemove
    pub fn on_recipient_remove<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_recipient_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`RelationshipAdd`] is received.
    ///
    /// [`RelationshipAdd`]: ../model/event/enum.Event.html#variant.RelationshipAdd
    pub fn on_relationship_add<F>(&mut self, handler: F)
        where F: Fn(Context, Relationship) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_relationship_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`RelationshipRemove`] is received.
    ///
    /// [`RelationshipRemove`]: ../model/event/enum.Event.html#variant.RelationshipRemove
    pub fn on_relationship_remove<F>(&mut self, handler: F)
        where F: Fn(Context, UserId, RelationshipType) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_relationship_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`Resumed`] is received.
    ///
    /// [`Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    pub fn on_resume<F>(&mut self, handler: F)
        where F: Fn(Context, ResumedEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_resume = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`TypingStart`] is received.
    ///
    /// [`TypingStart`]: ../model/event/enum.Event.html#variant.TypingStart
    pub fn on_typing_start<F>(&mut self, handler: F)
        where F: Fn(Context, TypingStartEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_typing_start = Some(Arc::new(handler));
    }

    /// Attaches a handler for when an [`Unknown`] is received.
    ///
    /// [`Unknown`]: ../model/event/enum.Event.html#variant.Unknown
    pub fn on_unknown<F>(&mut self, handler: F)
        where F: Fn(Context, String, BTreeMap<String, Value>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_unknown = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceServerUpdate`] is received.
    ///
    /// [`VoiceServerUpdate`]: ../model/event/enum.Event.html#variant.VoiceServerUpdate
    pub fn on_voice_server_update<F>(&mut self, handler: F)
        where F: Fn(Context, VoiceServerUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_voice_server_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceStateUpdate`] is received.
    ///
    /// [`VoiceStateUpdate`]: ../model/event/enum.Event.html#variant.VoiceStateUpdate
    pub fn on_voice_state_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<GuildId>, VoiceState) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_voice_state_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`WebhookUpdate`] is received.
    ///
    /// [`WebhookUpdate`]: ../model/event/enum.Event.html#variant.WebhookUpdate
    pub fn on_webhook_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, ChannelId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_webhook_update = Some(Arc::new(handler));
    }

    // Shard data layout is:
    // 0: first shard number to initialize
    // 1: shard number to initialize up to and including
    // 2: total number of shards the bot is sharding for
    //
    // Not all shards need to be initialized in this process.
    fn start_connection(&mut self, shard_data: Option<[u8; 3]>) -> Result<()> {
        let gateway_url = try!(rest::get_gateway()).url;

        for i in 0..shard_data.map_or(1, |x| x[1] + 1) {
            let shard = Shard::new(&gateway_url,
                                        &self.token,
                                        shard_data.map(|s| [i, s[2]]),
                                        self.login_type);
            match shard {
                Ok((shard, ready, receiver)) => {
                    self.shards.push(Arc::new(Mutex::new(shard)));

                    feature_cache_enabled! {{
                        CACHE.write()
                            .unwrap()
                            .update_with_ready(&ready);
                    }}

                    match self.shards.last() {
                        Some(shard) => {
                            feature_framework! {{
                                dispatch(Event::Ready(ready),
                                         shard.clone(),
                                         self.framework.clone(),
                                         self.login_type,
                                         self.event_store.clone());
                            } else {
                                dispatch(Event::Ready(ready),
                                         shard.clone(),
                                         self.login_type,
                                         self.event_store.clone());
                            }}

                            let shard_clone = shard.clone();
                            let event_store = self.event_store.clone();
                            let login_type = self.login_type;

                            feature_framework! {{
                                let framework = self.framework.clone();

                                thread::spawn(move || {
                                    handle_shard(shard_clone,
                                                 framework,
                                                 login_type,
                                                 event_store,
                                                 receiver)
                                });
                            } else {
                                thread::spawn(move || {
                                    handle_shard(shard_clone,
                                                 login_type,
                                                 event_store,
                                                 receiver)
                                });
                            }}
                        },
                        None => return Err(Error::Client(ClientError::ShardUnknown)),
                    }
                },
                Err(why) => return Err(why),
            }
        }

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

#[cfg(feature = "cache")]
impl Client {
    /// Attaches a handler for when a [`CallDelete`] is received.
    ///
    /// The `ChannelId` is the Id of the channel hosting the call. Returns the
    /// call from the cache - optionally - if the call was in it.
    ///
    /// [`CallDelete`]: ../model/event/enum.Event.html#variant.CallDelete
    pub fn on_call_delete<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Option<Call>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`CallUpdate`] is received.
    ///
    /// [`CallUpdate`]: ../model/event/enum.Event.html#variant.CallUpdate
    pub fn on_call_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Call>, Option<Call>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelUpdate`] is received.
    ///
    /// Optionally provides the version of the channel before the update.
    ///
    /// [`ChannelUpdate`]: ../model/event/enum.Event.html#variant.ChannelUpdate
    pub fn on_channel_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Channel>, Channel) + Send + Sync + 'static {
        self.event_store.lock()
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
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild, Option<Guild>) + Send + Sync + 'static {
        self.event_store.lock()
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
        self.event_store.lock()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/event/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Member>, Member) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/event/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId, Option<Role>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleUpdate`] is received.
    ///
    /// The optional `Role` is the role prior to updating. This can be `None` if
    /// it did not exist in the [`Cache`] before the update.
    ///
    /// [`GuildRoleUpdate`]: ../model/event/enum.Event.html#variant.GuildRoleUpdate
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    pub fn on_guild_role_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Option<Role>, Role) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserGuildSettingsUpdate`] is received.
    ///
    /// [`UserGuildSettingsUpdate`]: ../model/event/enum.Event.html#variant.UserGuildSettingsUpdate
    pub fn on_user_guild_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<UserGuildSettings>, UserGuildSettings) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_guild_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUpdate`] is received.
    ///
    /// [`GuildUpdate`]: ../model/event/enum.Event.html#variant.GuildUpdate
    pub fn on_guild_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Guild>, PartialGuild) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserNoteUpdate`] is received.
    ///
    /// Optionally returns the old note for the [`User`], if one existed.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [`UserNoteUpdate`]: ../model/event/enum.Event.html#variant.UserNoteUpdate
    pub fn on_note_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserId, Option<String>, String) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_note_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserSettingsUpdate`] is received.
    ///
    /// The old user settings will be provided as well.
    ///
    /// [`UserSettingsUpdate`]: ../model/event/enum.Event.html#variant.UserSettingsUpdate
    pub fn on_user_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserSettings, UserSettings) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserUpdate`] is received.
    ///
    /// The old current user will be provided as well.
    ///
    /// [`UserUpdate`]: ../model/event/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser, CurrentUser) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

#[cfg(not(feature = "cache"))]
impl Client {
    /// Attaches a handler for when a [`CallDelete`] is received.
    ///
    /// [`CallDelete`]: ../model/event/enum.Event.html#variant.CallDelete
    pub fn on_call_delete<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`CallUpdate`] is received.
    ///
    /// [`CallUpdate`]: ../model/event/enum.Event.html#variant.CallUpdate
    pub fn on_call_update<F>(&mut self, handler: F)
        where F: Fn(Context, CallUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelUpdate`] is received.
    ///
    /// [`ChannelUpdate`]: ../model/event/enum.Event.html#variant.ChannelUpdate
    pub fn on_channel_update<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildDelete`] is received.
    ///
    /// [`GuildDelete`]: ../model/event/enum.Event.html#variant.GuildDelete
    /// [`Role`]: ../model/struct.Role.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild) + Send + Sync + 'static {
        self.event_store.lock()
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
        self.event_store.lock()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/event/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildMemberUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/event/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleUpdate`] is received.
    ///
    /// [`GuildRoleUpdate`]: ../model/event/enum.Event.html#variant.GuildRoleUpdate
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    pub fn on_guild_role_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Role) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserGuildSettingsUpdate`] is received.
    ///
    /// [`UserGuildSettingsUpdate`]: ../model/event/enum.Event.html#variant.UserGuildSettingsUpdate
    pub fn on_user_guild_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserGuildSettings) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_guild_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUpdate`] is received.
    ///
    /// [`GuildUpdate`]: ../model/event/enum.Event.html#variant.GuildUpdate
    pub fn on_guild_update<F>(&mut self, handler: F)
        where F: Fn(Context, PartialGuild) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserNoteUpdate`] is received.
    ///
    /// Optionally returns the old note for the [`User`], if one existed.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [`UserNoteUpdate`]: ../model/event/enum.Event.html#variant.UserNoteUpdate
    pub fn on_note_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserId, String) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_note_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserSettingsUpdate`] is received.
    ///
    /// [`UserSettingsUpdate`]: ../model/event/enum.Event.html#variant.UserSettingsUpdate
    pub fn on_user_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserSettingsUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserUpdate`] is received.
    ///
    /// [`UserUpdate`]: ../model/event/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

#[cfg(feature="framework")]
fn handle_shard(shard: Arc<Mutex<Shard>>,
                framework: Arc<Mutex<Framework>>,
                login_type: LoginType,
                event_store: Arc<Mutex<EventStore>>,
                mut receiver: Receiver<WebSocketStream>) {
    loop {
        let event = receiver.recv_json(GatewayEvent::decode);

        let event = match shard.lock().unwrap().handle_event(event, &mut receiver) {
            Ok(Some(x)) => match x {
                (event, Some(new_receiver)) => {
                    receiver = new_receiver;

                    event
                },
                (event, None) => event,
            },
            _ => continue,
        };

        dispatch(event,
                 shard.clone(),
                 framework.clone(),
                 login_type,
                 event_store.clone());
    }
}

#[cfg(not(feature="framework"))]
fn handle_shard(shard: Arc<Mutex<Shard>>,
                     login_type: LoginType,
                     event_store: Arc<Mutex<EventStore>>,
                     mut receiver: Receiver<WebSocketStream>) {
    loop {
        let event = receiver.recv_json(GatewayEvent::decode);

        let event = match shard.lock().unwrap().handle_event(event, &mut receiver) {
            Ok(Some(x)) => match x {
                (event, Some(new_receiver)) => {
                    receiver = new_receiver;

                    event
                },
                (event, None) => event,
            },
            _ => continue,
        };

        dispatch(event,
                 shard.clone(),
                 login_type,
                 event_store.clone());
    }
}

fn login(token: &str, login_type: LoginType) -> Client {
    let token = token.to_owned();

    rest::set_token(&token);

    feature_framework! {{
        Client {
            shards: Vec::default(),
            event_store: Arc::new(Mutex::new(EventStore::default())),
            framework: Arc::new(Mutex::new(Framework::default())),
            login_type: login_type,
            token: token.to_owned(),
        }
    } else {
        Client {
            shards: Vec::default(),
            event_store: Arc::new(Mutex::new(EventStore::default())),
            login_type: login_type,
            token: token.to_owned(),
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
