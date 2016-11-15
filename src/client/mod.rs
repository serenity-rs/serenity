//! The Client contains information about a single bot or user's "session" with
//! Discord. Event handers and starting the connection are handled directly via
//! the client. In addition, the [http module] and [`State`] are also
//! automatically handled by the Client module for you.
//!
//! A [`Context`] is provided for every handler. The
//! context is an ergonomic way of accessing the lower-level Http struct's
//! methods.
//!
//! The Http struct is the lower-level method of accessing the Discord REST API.
//! Realistically there should be little reason to use this yourself, as the
//! Context will do this for you. A possible use case of using the Http struct
//! is if you do not have a state for purposes such as low memory requirements.
//!
//! Creating a Client instance and adding a handler on every message
//! receive, acting as a "ping-pong" bot is simple:
//!
//! ```rust,ignore
//! use serenity::Client;
//!
//! let mut client = Client::login_bot("my token here");
//!
//! client.on_message(|context, message| {
//!     if message.content == "!ping" {
//!         context.say("Pong!");
//!     }
//! });
//!
//! client.start();
//! ```
//!
//! [`Context`]: struct.Context.html
//! [`State`]: ext/state/index.html
//! [http module]: client/http/index.html

pub mod http;

mod connection;
mod context;
mod dispatch;
mod event_store;
mod login_type;

pub use self::connection::{
    Connection,
    ConnectionError,
    Status as ConnectionStatus
};
pub use self::context::Context;
pub use self::login_type::LoginType;

use hyper::status::StatusCode;
use self::dispatch::dispatch;
use self::event_store::EventStore;
use serde_json::builder::ObjectBuilder;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature = "framework")]
use ::ext::framework::Framework;

#[cfg(feature = "state")]
use ::ext::state::State;

#[cfg(feature = "state")]
lazy_static! {
    /// The STATE is a mutable lazily-initialized static binding. It can be
    /// accessed across any function and in any context.
    ///
    /// This [`State`] instance is updated for every event received, so you do
    /// not need to maintain your own state.
    ///
    /// See the [state module documentation] for more details.
    ///
    /// # Examples
    ///
    /// Retrieve the [current user][`CurrentUser`]'s Id:
    ///
    /// ```rust,ignore
    /// use serenity::client::STATE;
    ///
    /// println!("{}", STATE.lock().unwrap().user.id);
    /// ```
    ///
    /// [`CurrentUser`]: ../model/struct.CurrentUser.html
    /// [`State`]: ../ext/state/struct.State.html
    /// [state module documentation]: ../ext/state/index.html
    pub static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));
}

/// An error returned from the [`Client`] or the [`Context`], or model instance.
///
/// This is always wrapped within the library's generic [`Error::Client`]
/// variant.
///
/// # Examples
///
/// Matching an [`Error`] with this variant may look something like the
/// following for the [`Context::ban_user`] method:
///
/// ```rust,ignore
/// use serenity::client::ClientError;
/// use serenity::Error;
///
/// // assuming you are in a context and a `guild_id` has been bound
///
/// match context.ban_user(context.guild_id, context.message.author, 8) {
///     Ok(()) => {
///         // Ban successful.
///     },
///     Err(Error::Client(ClientError::DeleteMessageDaysAmount(amount))) => {
///         println!("Tried deleting {} days' worth of messages", amount);
///     },
///     Err(why) => {
///         println!("Unexpected error: {:?}", why);
///     },
/// }
/// ```
///
/// [`Client`]: struct.Client.html
/// [`Context`]: struct.Context.html
/// [`Context::ban_user`]: struct.Context.html#method.ban_user
/// [`Error::Client`]: ../enum.Error.html#variant.Client
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ClientError {
    /// When attempting to delete below or above the minimum and maximum allowed
    /// number of messages.
    BulkDeleteAmount,
    /// When the connection being retrieved from within the Client could not be
    /// found after being inserted into the Client's internal vector of
    /// [`Connection`]s.
    ///
    /// This can be returned from one of the options for starting one or
    /// multiple connections.
    ///
    /// **This should never be received.**
    ///
    /// [`Connection`]: struct.Connection.html
    ConnectionUnknown,
    /// When attempting to delete a number of days' worth of messages that is
    /// not allowed.
    DeleteMessageDaysAmount(u8),
    /// When there was an error retrieving the gateway URI from the REST API.
    Gateway,
    /// An indication that a [guild][`LiveGuild`] could not be found by
    /// [Id][`GuildId`] in the [`State`].
    ///
    /// [`GuildId`]: ../model/struct.GuildId.html
    /// [`LiveGuild`]: ../model/struct.LiveGuild.html
    /// [`State`]: ../ext/state/struct.State.html
    GuildNotFound,
    InvalidOpCode,
    /// When attempting to perform an action which is only available to user
    /// accounts.
    InvalidOperationAsBot,
    /// When attempting to perform an action which is only available to bot
    /// accounts.
    InvalidOperationAsUser,
    /// Indicates that you do not have the required permissions to perform an
    /// operation.
    ///
    /// The provided [`Permission`]s is the set of required permissions
    /// required.
    ///
    /// [`Permission`]: ../model/permissions/struct.Permissions.html
    InvalidPermissions(Permissions),
    /// An indicator that the shard data received from the gateway is invalid.
    InvalidShards,
    /// When the token provided is invalid. This is returned when validating a
    /// token through the [`validate_token`] function.
    ///
    /// [`validate_token`]: fn.validate_token.html
    InvalidToken,
    /// An indicator that the [current user] can not perform an action.
    ///
    /// [current user]: ../model/struct.CurrentUser.html
    InvalidUser,
    /// An indicator that an item is missing from the [`State`], and the action
    /// can not be continued.
    ///
    /// [`State`]: ../ext/state/struct.State.html
    ItemMissing,
    /// Indicates that a [`Message`]s content was too long and will not
    /// successfully send, as the length is over 2000 codepoints, or 4000 bytes.
    ///
    /// The number of bytes larger than the limit is provided.
    ///
    /// [`Message`]: ../model/struct.Message.html
    MessageTooLong(u64),
    /// When attempting to use a [`Context`] helper method which requires a
    /// contextual [`ChannelId`], but the current context is not appropriate for
    /// the action.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`Context`]: struct.Context.html
    NoChannelId,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64`.
    RateLimitI64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When attempting to find a required record from the State could not be
    /// found. This is required in methods such as [`Context::edit_role`].
    ///
    /// [`Context::edit_role`]: struct.Context.html#method.edit_role
    RecordNotFound,
    /// When a function such as [`Context::edit_channel`] did not expect the
    /// received [`ChannelType`].
    ///
    /// [`ChannelType`]: ../model/enum.ChannelType.html
    /// [`Context::edit_channel`]: struct.Context.html#method.edit_channel
    UnexpectedChannelType(ChannelType),
    /// When a status code was unexpectedly received for a request's status.
    UnexpectedStatusCode(StatusCode),
    /// When a status is received, but the verification to ensure the response
    /// is valid does not recognize the status.
    UnknownStatus(u16),
}

pub struct Client {
    pub connections: Vec<Arc<Mutex<Connection>>>,
    event_store: Arc<Mutex<EventStore>>,
    #[cfg(feature="framework")]
    framework: Arc<Mutex<Framework>>,
    login_type: LoginType,
    token: String,
}

#[allow(type_complexity)]
impl Client {
    /// Creates a Client for a bot.
    pub fn login_bot(bot_token: &str) -> Client {
        let token = format!("Bot {}", bot_token);

        login(&token, LoginType::Bot)
    }
    /// Create an instance from "raw values"
    #[doc(hidden)]
    pub fn login_raw(token: &str, login_type: LoginType) -> Client {
        login(&token.to_owned(), login_type)
    }

    /// Creates a Client for a user.
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

        http::logout(map)
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
    /// Refer to the [module-level documentation][connection docs] for more
    /// information on effectively using sharding.
    ///
    /// [connection docs]: struct.Connection.html#sharding
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
    /// Refer to the [module-level documentation][connection docs] for more
    /// information on effectively using sharding.
    ///
    /// [connection docs]: struct.Connection.html#sharding
    pub fn start_autosharded(&mut self) -> Result<()> {
        let res = try!(http::get_bot_gateway());

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
    /// Refer to the [module-level documentation][connection docs] for more
    /// information on effectively using sharding.
    ///
    /// [connection docs]: struct.Connection.html#sharding
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
    /// Refer to the [module-level documentation][connection docs] for more
    /// information on effectively using sharding.
    ///
    /// [`start_shard`]: #method.start_shard
    /// [`start_shard_range`]: #method.start_shards
    /// [connection docs]: struct.Connection.html#sharding
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
    /// Refer to the [module-level documentation][connection docs] for more
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
    /// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN")
    ///     .unwrap());
    ///
    /// let _ = client.start_shard_range([4, 7], 10);
    /// ```
    ///
    /// [`start_shard`]: #method.start_shard
    /// [`start_shards`]: #method.start_shards
    /// [connection docs]: struct.Connection.html#sharding
    pub fn start_shard_range(&mut self, range: [u8; 2], total_shards: u8)
        -> Result<()> {
        self.start_connection(Some([range[0], range[1], total_shards]))
    }

    /// Attaches a handler for when a [`CallCreate`] is received.
    ///
    /// [`CallCreate`]: ../model/enum.Event.html#variant.CallCreate
    pub fn on_call_create<F>(&mut self, handler: F)
        where F: Fn(Context, Call) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`CallDelete`] is received.
    ///
    /// [`CallDelete`]: ../model/enum.Event.html#variant.CallDelete
    pub fn on_call_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Call>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelCreate`] is received.
    ///
    /// [`ChannelCreate`]: ../model/enum.Event.html#variant.ChannelCreate
    pub fn on_channel_create<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelDelete`] is received.
    ///
    /// [`ChannelDelete`]: ../model/enum.Event.html#variant.ChannelDelete
    pub fn on_channel_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelPinsAck`] is received.
    ///
    /// [`ChannelPinsAck`]: ../model/enum.Event.html#variant.ChannelPinsAck
    pub fn on_channel_pins_ack<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelPinsAckEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_pins_ack = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelPinsUpdate`] is received.
    ///
    /// [`ChannelPinsUpdate`]: ../model/enum.Event.html#variant.ChannelPinsUpdate
    pub fn on_channel_pins_update<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelPinsUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_pins_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelUpdate`] is received.
    ///
    /// [`ChannelUpdate`]: ../model/enum.Event.html#variant.ChannelUpdate
    pub fn on_channel_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Channel>, Channel) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildCreate`] is received.
    ///
    /// [`GuildCreate`]: ../model/enum.Event.html#variant.GuildCreate
    pub fn on_guild_create<F>(&mut self, handler: F)
        where F: Fn(Context, LiveGuild) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildEmojisUpdate`] is received.
    ///
    /// The `HashMap` of emojis is the new full list of emojis.
    ///
    /// [`GuildEmojisUpdate`]: ../model/enum.Event.html#variant.GuildEmojisUpdate
    pub fn on_guild_emojis_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, HashMap<EmojiId, Emoji>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_emojis_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildIntegrationsUpdate`] is received.
    ///
    /// [`GuildIntegrationsUpdate`]: ../model/enum.Event.html#variant.GuildIntegrationsUpdate
    pub fn on_guild_integrations_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_integrations_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberAdd`] is received.
    ///
    /// [`GuildMemberAdd`]: ../model/enum.Event.html#variant.GuildMemberAdd
    pub fn on_guild_member_add<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Member) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMembersChunk`] is received.
    ///
    /// [`GuildMembersChunk`]: ../model/enum.Event.html#variant.GuildMembersChunk
    pub fn on_guild_members_chunk<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, HashMap<UserId, Member>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_members_chunk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleCreate`] is received.
    ///
    /// [`GuildRoleCreate`]: ../model/enum.Event.html#variant.GuildRoleCreate
    pub fn on_guild_role_create<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Role) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_create = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleUpdate`] is received.
    ///
    /// The optional `Role` is the role prior to updating. This can be `None` if
    /// it did not exist in the [`State`] before the update.
    ///
    /// [`GuildRoleUpdate`]: ../model/enum.Event.html#variant.GuildRoleUpdate
    /// [`State`]: ../ext/state/struct.State.html
    pub fn on_guild_role_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, Option<Role>, Role) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleSync`] is received.
    ///
    /// [`GuildRoleSync`]: ../model/enum.Event.html#variant.GuildRoleSync
    pub fn on_guild_sync<F>(&mut self, handler: F)
        where F: Fn(Context, GuildSyncEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_sync = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnavailable`] is received.
    ///
    /// [`GuildUnavailable`]: ../model/enum.Event.html#variant.GuildUnavailable
    pub fn on_guild_unavailable<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_unavailable = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUpdate`] is received.
    ///
    /// [`GuildUpdate`]: ../model/enum.Event.html#variant.GuildUpdate
    pub fn on_guild_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<LiveGuild>, Guild) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildBan`] is received.
    ///
    /// [`GuildBan`]: ../model/enum.Event.html#variant.GuildBan
    pub fn on_member_ban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_ban_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildUnban`] is received.
    ///
    /// [`GuildUnban`]: ../model/enum.Event.html#variant.GuildUnban
    pub fn on_member_unban<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_ban_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageCreate`] is received.
    ///
    /// [`MessageCreate`]: ../model/enum.Event.html#variant.MessageCreate
    pub fn on_message<F>(&mut self, handler: F)
        where F: Fn(Context, Message) + Send + Sync + 'static {

        self.event_store.lock()
            .unwrap()
            .on_message = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageAck`] is received.
    ///
    /// [`MessageAck`]: ../model/enum.Event.html#variant.MessageAck
    pub fn on_message_ack<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Option<MessageId>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_ack = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDelete`] is received.
    ///
    /// [`MessageDelete`]: ../model/enum.Event.html#variant.MessageDelete
    pub fn on_message_delete<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageDeleteBulk`] is received.
    ///
    /// [`MessageDeleteBulk`]: ../model/enum.Event.html#variant.MessageDeleteBulk
    pub fn on_message_delete_bulk<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, Vec<MessageId>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_delete_bulk = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`MessageUpdate`] is received.
    ///
    /// [`MessageUpdate`]: ../model/enum.Event.html#variant.MessageUpdate
    pub fn on_message_update<F>(&mut self, handler: F)
        where F: Fn(Context, MessageUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_message_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresencesReplace`] is received.
    ///
    /// [`PresencesReplace`]: ../model/enum.Event.html#variant.PresencesReplace
    pub fn on_presence_replace<F>(&mut self, handler: F)
        where F: Fn(Context, Vec<Presence>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_presence_replace = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`PresenceUpdate`] is received.
    ///
    /// [`PresenceUpdate`]: ../model/enum.Event.html#variant.PresenceUpdate
    pub fn on_presence_update<F>(&mut self, handler: F)
        where F: Fn(Context, PresenceUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_presence_update = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionAdd`] is received.
    ///
    /// [`ReactionAdd`]: ../model/enum.Event.html#variant.ReactionAdd
    pub fn on_reaction_add<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_reaction_add = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemove`] is received.
    ///
    /// [`ReactionRemove`]: ../model/enum.Event.html#variant.ReactionRemove
    pub fn on_reaction_remove<F>(&mut self, handler: F)
        where F: Fn(Context, Reaction) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_reaction_remove = Some(Arc::new(handler));
    }

    /// Attached a handler for when a [`ReactionRemoveAll`] is received.
    ///
    /// [`ReactionRemoveAll`]: ../model/enum.Event.html#variant.ReactionRemoveAll
    pub fn on_reaction_remove_all<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, MessageId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_reaction_remove_all = Some(Arc::new(handler));
    }

    /// Register an event to be called whenever a Ready event is received.
    ///
    /// Registering a handler for the ready event is good for noting when your
    /// bot has established a connection to the gateway.
    ///
    /// **Note**: The Ready event is not guarenteed to be the first event you
    /// will receive by Discord. Do not actively rely on it.
    ///
    /// # Examples
    ///
    /// Print the [current user][`CurrentUser`]'s name on ready:
    ///
    /// ```rust,ignore
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN")
    ///     .unwrap());
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected", ready.user.name);
    /// });
    /// ```
    ///
    /// [`CurrentUser`]: ../model/struct.CurrentUser.html
    pub fn on_ready<F>(&mut self, handler: F)
        where F: Fn(Context, Ready) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_ready = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientAdd`] is received.
    ///
    /// [`ChannelRecipientAdd`]: ../model/enum.Event.html#variant.ChannelRecipientAdd
    pub fn on_recipient_add<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_recipient_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`ChannelRecipientRemove`] is received.
    ///
    /// [`ChannelRecipientRemove`]: ../model/enum.Event.html#variant.ChannelRecipientRemove
    pub fn on_recipient_remove<F>(&mut self, handler: F)
        where F: Fn(Context, ChannelId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_channel_recipient_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`RelationshipAdd`] is received.
    ///
    /// [`RelationshipAdd`]: ../model/enum.Event.html#variant.RelationshipAdd
    pub fn on_relationship_add<F>(&mut self, handler: F)
        where F: Fn(Context, Relationship) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_relationship_addition = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`RelationshipRemove`] is received.
    ///
    /// [`RelationshipRemove`]: ../model/enum.Event.html#variant.RelationshipRemove
    pub fn on_relationship_remove<F>(&mut self, handler: F)
        where F: Fn(Context, UserId, RelationshipType) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_relationship_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`Resumed`] is received.
    ///
    /// [`Resumed`]: ../model/enum.Event.html#variant.Resumed
    pub fn on_resume<F>(&mut self, handler: F)
        where F: Fn(Context, ResumedEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_resume = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`TypingStart`] is received.
    ///
    /// [`TypingStart`]: ../model/enum.Event.html#variant.TypingStart
    pub fn on_typing_start<F>(&mut self, handler: F)
        where F: Fn(Context, TypingStartEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_typing_start = Some(Arc::new(handler));
    }

    /// Attaches a handler for when an [`Unknown`] is received.
    ///
    /// [`Unknown`]: ../model/enum.Event.html#variant.Unknown
    pub fn on_unknown<F>(&mut self, handler: F)
        where F: Fn(Context, String, BTreeMap<String, Value>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_unknown = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserGuildSettingsUpdate`] is received.
    ///
    /// [`UserGuildSettingsUpdate`]: ../model/enum.Event.html#variant.UserGuildSettingsUpdate
    pub fn on_user_guild_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<UserGuildSettings>, UserGuildSettings) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_guild_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceServerUpdate`] is received.
    ///
    /// [`VoiceServerUpdate`]: ../model/enum.Event.html#variant.VoiceServerUpdate
    pub fn on_voice_server_update<F>(&mut self, handler: F)
        where F: Fn(Context, VoiceServerUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_voice_server_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`VoiceStateUpdate`] is received.
    ///
    /// [`VoiceStateUpdate`]: ../model/enum.Event.html#variant.VoiceStateUpdate
    pub fn on_voice_state_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<GuildId>, VoiceState) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_voice_state_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`WebhookUpdate`] is received.
    ///
    /// [`WebhookUpdate`]: ../model/enum.Event.html#variant.WebhookUpdate
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
        let gateway_url = try!(http::get_gateway()).url;

        for i in 0..shard_data.map_or(1, |x| x[1] + 1) {
            let connection = Connection::new(&gateway_url,
                                             &self.token,
                                             shard_data.map(|s| [i, s[2]]),
                                             self.login_type);
            match connection {
                Ok((connection, ready)) => {
                    self.connections.push(Arc::new(Mutex::new(connection)));

                    feature_state_enabled! {{
                        STATE.lock()
                            .unwrap()
                            .update_with_ready(&ready);
                    }}

                    match self.connections.last() {
                        Some(connection) => {
                            feature_framework! {{
                                dispatch(Ok(Event::Ready(ready)),
                                         connection.clone(),
                                         self.framework.clone(),
                                         self.login_type,
                                         self.event_store.clone());
                            } {
                                dispatch(Ok(Event::Ready(ready)),
                                         connection.clone(),
                                         self.login_type,
                                         self.event_store.clone());
                            }}

                            let connection_clone = connection.clone();
                            let event_store = self.event_store.clone();
                            let login_type = self.login_type;

                            feature_framework! {{
                                let framework = self.framework.clone();

                                thread::spawn(move || {
                                    handle_connection(connection_clone,
                                                      framework,
                                                      login_type,
                                                      event_store)
                                });
                            } {
                                thread::spawn(move || {
                                    handle_connection(connection_clone,
                                                      login_type,
                                                      event_store)
                                });
                            }}
                        },
                        None => return Err(Error::Client(ClientError::ConnectionUnknown)),
                    }
                },
                Err(why) => return Err(why),
            }
        }

        // How to avoid the problem while still working on other parts of the
        // library 101
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }

    // Boot up a new connection. This is used primarily in the scenario of
    // re-instantiating a connection in the reconnect logic in another
    // Connection.
    #[doc(hidden)]
    pub fn boot_connection(&mut self,
                           shard_info: Option<[u8; 2]>)
                           -> Result<(Connection, ReadyEvent)> {
        let gateway_url = try!(http::get_gateway()).url;

        Connection::new(&gateway_url, &self.token, shard_info, self.login_type)
    }
}

#[cfg(feature = "state")]
impl Client {
    /// Attaches a handler for when a [`CallUpdate`] is received.
    ///
    /// [`CallUpdate`]: ../model/enum.Event.html#variant.CallUpdate
    pub fn on_call_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Call>, Option<Call>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildDelete`] is received.
    ///
    /// Returns a partial guild as well as - optionally - the full guild, with
    /// data like [`Role`]s. This can be `None` in the event that it was not in
    /// the [`State`].
    ///
    /// **Note**: The relevant guild is _removed_ from the State when this event
    /// is received. If you need to keep it, you can either re-insert it
    /// yourself back into the State or manage it in another way.
    ///
    /// [`GuildDelete`]: ../model/enum.Event.html#variant.GuildDelete
    /// [`Role`]: ../model/struct.Role.html
    /// [`State`]: ../ext/state/struct.State.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Guild, Option<LiveGuild>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberRemove`] is received.
    ///
    /// Returns the user's associated `Member` object, _if_ it existed in the
    /// state.
    ///
    /// [`GuildMemberRemove`]: ../model/enum.Event.html#variant.GuildMemberRemove
    pub fn on_guild_member_remove<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User, Option<Member>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, Option<Member>, Member) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId, Option<Role>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserNoteUpdate`] is received.
    ///
    /// Optionally returns the old note for the [`User`], if one existed.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [`UserNoteUpdate`]: ../model/enum.Event.html#variant.UserNoteUpdate
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
    /// [`UserSettingsUpdate`]: ../model/enum.Event.html#variant.UserSettingsUpdate
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
    /// [`UserUpdate`]: ../model/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser, CurrentUser) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

#[cfg(not(feature = "state"))]
impl Client {
    /// Attaches a handler for when a [`CallUpdate`] is received.
    ///
    /// [`CallUpdate`]: ../model/enum.Event.html#variant.CallUpdate
    pub fn on_call_update<F>(&mut self, handler: F)
        where F: Fn(Context, CallUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_call_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildDelete`] is received.
    ///
    /// [`GuildDelete`]: ../model/enum.Event.html#variant.GuildDelete
    /// [`Role`]: ../model/struct.Role.html
    /// [`State`]: ../ext/state/struct.State.html
    pub fn on_guild_delete<F>(&mut self, handler: F)
        where F: Fn(Context, Guild, Option<LiveGuild>) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberRemove`] is received.
    ///
    /// Returns the user's associated `Member` object, _if_ it existed in the
    /// state.
    ///
    /// [`GuildMemberRemove`]: ../model/enum.Event.html#variant.GuildMemberRemove
    pub fn on_guild_member_remove<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, User) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_removal = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildMemberUpdate`] is received.
    ///
    /// [`GuildMemberUpdate`]: ../model/enum.Event.html#variant.GuildMemberUpdate
    pub fn on_guild_member_update<F>(&mut self, handler: F)
        where F: Fn(Context, GuildMemberUpdateEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_member_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`GuildRoleDelete`] is received.
    ///
    /// [`GuildRoleDelete`]: ../model/enum.Event.html#variant.GuildRoleDelete
    pub fn on_guild_role_delete<F>(&mut self, handler: F)
        where F: Fn(Context, GuildId, RoleId) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_guild_role_delete = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserNoteUpdate`] is received.
    ///
    /// Optionally returns the old note for the [`User`], if one existed.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [`UserNoteUpdate`]: ../model/enum.Event.html#variant.UserNoteUpdate
    pub fn on_note_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserId, String) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_note_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserSettingsUpdate`] is received.
    ///
    /// [`UserSettingsUpdate`]: ../model/enum.Event.html#variant.UserSettingsUpdate
    pub fn on_user_settings_update<F>(&mut self, handler: F)
        where F: Fn(Context, UserSettingsEvent) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_settings_update = Some(Arc::new(handler));
    }

    /// Attaches a handler for when a [`UserUpdate`] is received.
    ///
    /// [`UserUpdate`]: ../model/enum.Event.html#variant.UserUpdate
    pub fn on_user_update<F>(&mut self, handler: F)
        where F: Fn(Context, CurrentUser) + Send + Sync + 'static {
        self.event_store.lock()
            .unwrap()
            .on_user_update = Some(Arc::new(handler));
    }
}

#[cfg(feature="framework")]
fn handle_connection(connection: Arc<Mutex<Connection>>,
                     framework: Arc<Mutex<Framework>>,
                     login_type: LoginType,
                     event_store: Arc<Mutex<EventStore>>) {
    loop {
        let event = {
            let mut connection = connection.lock().unwrap();

            connection.receive()
        };

        dispatch(event,
                 connection.clone(),
                 framework.clone(),
                 login_type,
                 event_store.clone());
    }
}

#[cfg(not(feature="framework"))]
fn handle_connection(connection: Arc<Mutex<Connection>>,
                     login_type: LoginType,
                     event_store: Arc<Mutex<EventStore>>) {
    loop {
        let event = {
            let mut connection = connection.lock().unwrap();

            connection.receive()
        };

        dispatch(event,
                 connection.clone(),
                 login_type,
                 event_store.clone());
    }
}

fn login(token: &str, login_type: LoginType) -> Client {
    let token = token.to_owned();

    http::set_token(&token);

    feature_framework! {{
        Client {
            connections: Vec::default(),
            event_store: Arc::new(Mutex::new(EventStore::default())),
            framework: Arc::new(Mutex::new(Framework::default())),
            login_type: login_type,
            token: token.to_owned(),
        }
    } {
        Client {
            connections: Vec::default(),
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
    if parts.get(1).unwrap().len() < 6 {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    // Check that there is no whitespace before/after the token.
    if token.trim() != token {
        return Err(Error::Client(ClientError::InvalidToken));
    }

    Ok(())
}
