//! A cache of events received over a `Shard`, where storing at least some
//! data from the event is possible.
//!
//! This acts as a cache, to avoid making requests over the REST API through
//! the [`http`] module where possible. All fields are public, and do not have
//! getters, to allow you more flexibility with the stored data. However, this
//! allows data to be "corrupted", and _may or may not_ cause misfunctions
//! within the library. Mutate data at your own discretion.
//!
//! # Use by Models
//!
//! Most models of Discord objects, such as the [`Message`], [`GuildChannel`],
//! or [`Emoji`], have methods for interacting with that single instance. This
//! feature is only compiled if the `methods` feature is enabled. An example of
//! this is [`Guild::edit`], which performs a check to ensure that the current
//! user is the owner of the guild, prior to actually performing the HTTP
//! request. The cache is involved due to the function's use of unlocking the
//! cache and retrieving the Id of the current user, and comparing it to the Id
//! of the user that owns the guild. This is an inexpensive method of being able
//! to access data required by these sugary methods.
//!
//! # Do I need the Cache?
//!
//! If you're asking this, the answer is likely "definitely yes" or
//! "definitely no"; any in-between tends to be "yes". If you are low on RAM,
//! and need to run on only a couple MB, then the answer is "definitely no". If
//! you do not care about RAM and want your bot to be able to access data
//! while needing to hit the REST API as little as possible, then the answer
//! is "yes".
//!
//! [`Emoji`]: ../model/guild/struct.Emoji.html
//! [`Group`]: ../model/channel/struct.Group.html
//! [`Guild`]: ../model/guild/struct.Guild.html
//! [`Guild::edit`]: ../model/guild/struct.Guild.html#method.edit
//! [`Message`]: ../model/channel/struct.Message.html
//! [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
//! [`Role`]: ../model/guild/struct.Role.html
//! [`http`]: ../http/index.html

use std::str::FromStr;
use crate::model::prelude::*;
use parking_lot::RwLock;
use std::collections::{
    hash_map::Entry,
    HashMap,
    HashSet,
    VecDeque,
};
use std::{
    default::Default,
    ops::Deref,
    sync::Arc,
};

mod cache_update;
mod settings;

pub use self::cache_update::CacheUpdate;
pub use self::settings::Settings;

type MessageCache = HashMap<ChannelId, HashMap<MessageId, Message>>;

pub trait FromStrAndCache: Sized {
    type Err;

    fn from_str(cache: impl AsRef<CacheRwLock>, s: &str) -> Result<Self, Self::Err>;
}

pub trait StrExt: Sized {
    fn parse_cached<F: FromStrAndCache>(&self, cache: impl AsRef<CacheRwLock>) -> Result<F, F::Err>;
}

impl<'a> StrExt for &'a str {
    fn parse_cached<F: FromStrAndCache>(&self, cache: impl AsRef<CacheRwLock>) -> Result<F, F::Err> {
        F::from_str(&cache, &self)
    }
}

impl<F: FromStr> FromStrAndCache for F {
    type Err = F::Err;

    fn from_str(_cache: impl AsRef<CacheRwLock>, s: &str) -> Result<Self, Self::Err> {
        s.parse::<F>()
    }
}

/// A cache of all events received over a [`Shard`], where storing at least
/// some data from the event is possible.
///
/// This acts as a cache, to avoid making requests over the REST API through the
/// [`http`] module where possible. All fields are public, and do not have
/// getters, to allow you more flexibility with the stored data. However, this
/// allows data to be "corrupted", and _may or may not_ cause misfunctions
/// within the library. Mutate data at your own discretion.
///
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`http`]: ../http/index.html
#[derive(Clone, Debug)]
pub struct Cache {
    /// A map of channels in [`Guild`]s that the current user has received data
    /// for.
    ///
    /// When a [`Event::GuildDelete`] or [`Event::GuildUnavailable`] is
    /// received and processed by the cache, the relevant channels are also
    /// removed from this map.
    ///
    /// [`Event::GuildDelete`]: ../model/event/struct.GuildDeleteEvent.html
    /// [`Event::GuildUnavailable`]: ../model/event/struct.GuildUnavailableEvent.html
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub channels: HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
    /// A map of channel categories.
    pub categories: HashMap<ChannelId, Arc<RwLock<ChannelCategory>>>,
    /// A map of the groups that the current user is in.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [special cases]: index.html#special-cases-in-the-cache
    pub groups: HashMap<ChannelId, Arc<RwLock<Group>>>,
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    ///
    /// [`Emoji`]: ../model/guild/struct.Emoji.html
    /// [`Role`]: ../model/guild/struct.Role.html
    pub guilds: HashMap<GuildId, Arc<RwLock<Guild>>>,
    /// A map of channels to messages.
    ///
    /// This is a map of channel IDs to another map of message IDs to messages.
    ///
    /// This keeps only the ten most recent messages.
    pub messages: MessageCache,
    /// A map of notes that a user has made for individual users.
    ///
    /// An empty note is equivalent to having no note, and creating an empty
    /// note is equivalent to deleting a note.
    ///
    /// This will always be empty for bot users.
    pub notes: HashMap<UserId, String>,
    /// A map of users' presences. This is updated in real-time. Note that
    /// status updates are often "eaten" by the gateway, and this should not
    /// be treated as being entirely 100% accurate.
    pub presences: HashMap<UserId, Presence>,
    /// A map of direct message channels that the current user has open with
    /// other users.
    pub private_channels: HashMap<ChannelId, Arc<RwLock<PrivateChannel>>>,
    /// The total number of shards being used by the bot.
    pub shard_count: u64,
    /// A list of guilds which are "unavailable". Refer to the documentation for
    /// [`Event::GuildUnavailable`] for more information on when this can occur.
    ///
    /// Additionally, guilds are always unavailable for bot users when a Ready
    /// is received. Guilds are "sent in" over time through the receiving of
    /// [`Event::GuildCreate`]s.
    ///
    /// [`Event::GuildCreate`]: ../model/event/enum.Event.html#variant.GuildCreate
    /// [`Event::GuildUnavailable`]: ../model/event/enum.Event.html#variant.GuildUnavailable
    pub unavailable_guilds: HashSet<GuildId>,
    /// The current user "logged in" and for which events are being received
    /// for.
    ///
    /// The current user contains information that a regular [`User`] does not,
    /// such as whether it is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    ///
    /// [`CurrentUser`]: ../model/user/struct.CurrentUser.html
    /// [`User`]: ../model/user/struct.User.html
    pub user: CurrentUser,
    /// A map of users that the current user sees.
    ///
    /// Users are added to - and updated from - this map via the following
    /// received events:
    ///
    /// - [`ChannelRecipientAdd`][`ChannelRecipientAddEvent`]
    /// - [`GuildMemberAdd`][`GuildMemberAddEvent`]
    /// - [`GuildMemberRemove`][`GuildMemberRemoveEvent`]
    /// - [`GuildMembersChunk`][`GuildMembersChunkEvent`]
    /// - [`PresenceUpdate`][`PresenceUpdateEvent`]
    /// - [`Ready`][`ReadyEvent`]
    ///
    /// Note, however, that users are _not_ removed from the map on removal
    /// events such as [`GuildMemberRemove`][`GuildMemberRemoveEvent`], as other
    /// structs such as members or recipients may still exist.
    ///
    /// [`ChannelRecipientAddEvent`]: ../model/event/struct.ChannelRecipientAddEvent.html
    /// [`GuildMemberAddEvent`]: ../model/event/struct.GuildMemberAddEvent.html
    /// [`GuildMemberRemoveEvent`]: ../model/event/struct.GuildMemberRemoveEvent.html
    /// [`GuildMemberUpdateEvent`]: ../model/event/struct.GuildMemberUpdateEvent.html
    /// [`GuildMembersChunkEvent`]: ../model/event/struct.GuildMembersChunkEvent.html
    /// [`PresenceUpdateEvent`]: ../model/event/struct.PresenceUpdateEvent.html
    /// [`ReadyEvent`]: ../model/event/struct.ReadyEvent.html
    pub users: HashMap<UserId, Arc<RwLock<User>>>,
    /// Queue of message IDs for each channel.
    ///
    /// This is simply a vecdeque so we can keep track of the order of messages
    /// inserted into the cache. When a maximum number of messages are in a
    /// channel's cache, we can pop the front and remove that ID from the cache.
    pub(crate) message_queue: HashMap<ChannelId, VecDeque<MessageId>>,
    /// The settings for the cache.
    settings: Settings,
    __nonexhaustive: (),
}

impl Cache {
    /// Creates a new cache.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new cache instance with settings applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::cache::{Cache, Settings};
    ///
    /// let mut settings = Settings::new();
    /// settings.max_messages(10);
    ///
    /// let cache = Cache::new_with_settings(settings);
    /// ```
    pub fn new_with_settings(settings: Settings) -> Self {
        Self {
            settings,
            ..Default::default()
        }
    }

    /// Fetches the number of [`Member`]s that have not had data received.
    ///
    /// The important detail to note here is that this is the number of
    /// _member_s that have not had data received. A single [`User`] may have
    /// multiple associated member objects that have not been received.
    ///
    /// This can be used in combination with [`Shard::chunk_guilds`], and can be
    /// used to determine how many members have not yet been received.
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// # #[cfg(feature = "client")]
    /// # fn main() {
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, ctx: Context, _: Ready) {
    ///          // Wait some time for guilds to be received.
    ///         //
    ///         // You should keep track of this in a better fashion by tracking how
    ///         // many guilds each `ready` has, and incrementing a counter on
    ///         // GUILD_CREATEs. Once the number is equal, print the number of
    ///         // unknown members.
    ///         //
    ///         // For demonstrative purposes we're just sleeping the thread for 5
    ///         // seconds.
    ///         thread::sleep(Duration::from_secs(5));
    ///
    ///         println!("{} unknown members", ctx.cache.read().unknown_members());
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// # }
    /// #
    /// # #[cfg(not(feature = "client"))]
    /// # fn main() { }
    /// ```
    ///
    /// [`Member`]: ../model/guild/struct.Member.html
    /// [`Shard::chunk_guilds`]: ../gateway/struct.Shard.html#method.chunk_guilds
    /// [`User`]: ../model/user/struct.User.html
    pub fn unknown_members(&self) -> u64 {
        let mut total = 0;

        for guild in self.guilds.values() {
            let guild = guild.read();

            let members = guild.members.len() as u64;

            if guild.member_count > members {
                total += guild.member_count - members;
            }
        }

        total
    }

    /// Fetches a vector of all [`PrivateChannel`] and [`Group`] Ids that are
    /// stored in the cache.
    ///
    /// # Examples
    ///
    /// If there are 6 private channels and 2 groups in the cache, then `8` Ids
    /// will be returned.
    ///
    /// Printing the count of all private channels and groups:
    ///
    /// ```rust,no_run
    /// # use serenity::{cache::{Cache, CacheRwLock}};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// let amount = cache.read().all_private_channels().len();
    ///
    /// println!("There are {} private channels", amount);
    /// ```
    ///
    /// [`Group`]: ../model/channel/struct.Group.html
    /// [`PrivateChannel`]: ../model/channel/struct.PrivateChannel.html
    pub fn all_private_channels(&self) -> Vec<&ChannelId> {
        self.groups
            .keys()
            .chain(self.private_channels.keys())
            .collect()
    }

    /// Fetches a vector of all [`Guild`]s' Ids that are stored in the cache.
    ///
    /// Note that if you are utilizing multiple [`Shard`]s, then the guilds
    /// retrieved over all shards are included in this count -- not just the
    /// current [`Context`]'s shard, if accessing from one.
    ///
    /// # Examples
    ///
    /// Print all of the Ids of guilds in the Cache:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # fn main() {
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, context: Context, _: Ready) {
    ///         let guilds = context.cache.read().guilds.len();
    ///
    ///         println!("Guilds in the Cache: {}", guilds);
    ///     }
    /// }
    /// # }
    /// #
    /// # #[cfg(not(feature = "client"))]
    /// # fn main() { }
    /// ```
    ///
    /// [`Context`]: ../client/struct.Context.html
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`Shard`]: ../gateway/struct.Shard.html
    pub fn all_guilds(&self) -> Vec<&GuildId> {
        self.guilds
            .keys()
            .chain(self.unavailable_guilds.iter())
            .collect()
    }

    /// Retrieves a [`Channel`] from the cache based on the given Id.
    ///
    /// This will search the [`channels`] map, the [`private_channels`] map, and
    /// then the map of [`groups`] to find the channel.
    ///
    /// If you know what type of channel you're looking for, you should instead
    /// manually retrieve from one of the respective maps or methods:
    ///
    /// - [`GuildChannel`]: [`guild_channel`] or [`channels`]
    /// - [`PrivateChannel`]: [`private_channel`] or [`private_channels`]
    /// - [`Group`]: [`group`] or [`groups`]
    ///
    /// [`Channel`]: ../model/channel/enum.Channel.html
    /// [`Group`]: ../model/channel/struct.Group.html
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`channels`]: #structfield.channels
    /// [`group`]: #method.group
    /// [`guild_channel`]: #method.guild_channel
    /// [`private_channel`]: #method.private_channel
    /// [`groups`]: #structfield.groups
    /// [`private_channels`]: #structfield.private_channels
    #[inline]
    pub fn channel<C: Into<ChannelId>>(&self, id: C) -> Option<Channel> {
        self._channel(id.into())
    }

    fn _channel(&self, id: ChannelId) -> Option<Channel> {
        if let Some(channel) = self.channels.get(&id) {
            return Some(Channel::Guild(Arc::clone(channel)));
        }

        if let Some(private_channel) = self.private_channels.get(&id) {
            return Some(Channel::Private(Arc::clone(private_channel)));
        }

        if let Some(group) = self.groups.get(&id) {
            return Some(Channel::Group(Arc::clone(group)));
        }

        None
    }

    /// Retrieves a guild from the cache based on the given Id.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`GuildId`].
    ///
    /// [`GuildId`]: ../model/id/struct.GuildId.html
    ///
    /// # Examples
    ///
    /// Retrieve a guild from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::{cache::{Cache, CacheRwLock}};
    /// # use parking_lot::RwLock;
    /// # use std::{error::Error, sync::Arc};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(guild) = cache.read().guild(7) {
    ///     println!("Guild name: {}", guild.read().name);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn guild<G: Into<GuildId>>(&self, id: G) -> Option<Arc<RwLock<Guild>>> {
        self._guild(id.into())
    }

    fn _guild(&self, id: GuildId) -> Option<Arc<RwLock<Guild>>> {
        self.guilds.get(&id).cloned()
    }

    /// Retrieves a reference to a [`Guild`]'s channel. Unlike [`channel`],
    /// this will only search guilds for the given channel.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// # Examples
    ///
    /// Getting a guild's channel via the Id of the message received through a
    /// [`Client::on_message`] event dispatch:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # fn main() {
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, context: Context, message: Message) {
    ///         let cache = context.cache.read();
    ///
    ///         let channel = match cache.guild_channel(message.channel_id) {
    ///             Some(channel) => channel,
    ///             None => {
    /// if let Err(why) = message.channel_id.say(&context.http, "Could not find guild's
    /// channel data") {
    ///                     println!("Error sending message: {:?}", why);
    ///                 }
    ///
    ///                 return;
    ///             },
    ///         };
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// # }
    /// #
    /// # #[cfg(not(feature = "client"))]
    /// # fn main() { }
    /// ```
    ///
    /// [`ChannelId`]: ../model/id/struct.ChannelId.html
    /// [`Client::on_message`]: ../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`channel`]: #method.channel
    #[inline]
    pub fn guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<GuildChannel>>> {
        self._guild_channel(id.into())
    }

    fn _guild_channel(&self, id: ChannelId) -> Option<Arc<RwLock<GuildChannel>>> {
        self.channels.get(&id).cloned()
    }

    /// Retrieves a reference to a [`Group`] from the cache based on the given
    /// associated channel Id.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// [`ChannelId`]: ../model/id/struct.ChannelId.html
    /// [`Group`]: ../model/channel/struct.Group.html
    ///
    /// # Examples
    ///
    /// Retrieve a group from the cache and print its owner's id:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::{Cache, CacheRwLock};
    /// # use parking_lot::RwLock;
    /// # use std::{error::Error, sync::Arc};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// if let Some(group) = cache.read().group(7) {
    ///     println!("Owner Id: {}", group.read().owner_id);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn group<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<Group>>> {
        self._group(id.into())
    }

    fn _group(&self, id: ChannelId) -> Option<Arc<RwLock<Group>>> {
        self.groups.get(&id).cloned()
    }

    /// Retrieves a [`Guild`]'s member from the cache based on the guild's and
    /// user's given Ids.
    ///
    /// **Note**: This will clone the entire member. Instead, retrieve the guild
    /// and retrieve from the guild's [`members`] map to avoid this.
    ///
    /// # Examples
    ///
    /// Retrieving the member object of the user that posted a message, in a
    /// [`Client::on_message`] context:
    ///
    /// ```rust,ignore
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// let cache = cache.read();
    ///
    /// let member = {
    ///     let channel = match cache.guild_channel(message.channel_id) {
    ///         Some(channel) => channel,
    ///         None => {
    ///             if let Err(why) = message.channel_id.say("Error finding channel data") {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///         },
    ///     };
    ///
    ///     match cache.member(channel.guild_id, message.author.id) {
    ///         Some(member) => member,
    ///         None => {
    ///             if let Err(why) = message.channel_id.say("Error finding member data") {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///         },
    ///     }
    /// };
    ///
    /// let msg = format!("You have {} roles", member.roles.len());
    ///
    /// if let Err(why) = message.channel_id.say(&msg) {
    ///     println!("Error sending message: {:?}", why);
    /// }
    /// ```
    ///
    /// [`Client::on_message`]: ../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`members`]: ../model/guild/struct.Guild.html#structfield.members
    #[inline]
    pub fn member<G, U>(&self, guild_id: G, user_id: U) -> Option<Member>
        where G: Into<GuildId>, U: Into<UserId> {
        self._member(guild_id.into(), user_id.into())
    }

    fn _member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        self.guilds.get(&guild_id).and_then(|guild| {
            guild.read().members.get(&user_id).cloned()
        })
    }

    /// Retrieves a [`Channel`]'s message from the cache based on the channel's and
    /// message's given Ids.
    ///
    /// **Note**: This will clone the entire message.
    ///
    /// # Examples
    ///
    /// Retrieving the message object from a channel, in a
    /// [`EventHandler::message`] context:
    ///
    /// ```rust,no_run
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::id::{ChannelId, MessageId}};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let http = Arc::new(Http::new_with_token("DISCORD_TOKEN"));
    /// # let message = ChannelId(0).message(&http, MessageId(1)).unwrap();
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #
    /// let cache = cache.read();
    /// let fetched_message = cache.message(message.channel_id, message.id);
    ///
    /// match fetched_message {
    ///     Some(m) => {
    ///         assert_eq!(message.content, m.content);
    ///     },
    ///     None => {
    ///         println!("No message found in cache.");
    ///     },
    /// }
    /// ```
    ///
    /// [`EventHandler::message`]: ../client/trait.EventHandler.html#method.message
    /// [`Channel`]: ../model/channel/struct.Channel.html
    #[inline]
    pub fn message<C, M>(&self, channel_id: C, message_id: M) -> Option<Message>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self._message(channel_id.into(), message_id.into())
    }

    fn _message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        self.messages.get(&channel_id).and_then(|messages| {
            messages.get(&message_id).cloned()
        })
    }

    /// Retrieves a [`PrivateChannel`] from the cache's [`private_channels`]
    /// map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// # Examples
    ///
    /// Retrieve a private channel from the cache and print its recipient's
    /// name:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #   let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #   let cache = cache.read();
    /// // assuming the cache has been unlocked
    ///
    /// if let Some(channel) = cache.private_channel(7) {
    ///     let channel_reader = channel.read();
    ///     let user_reader = channel_reader.recipient.read();
    ///
    ///     println!("The recipient is {}", user_reader.name);
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`private_channels`]: #structfield.private_channels
    #[inline]
    pub fn private_channel<C: Into<ChannelId>>(&self,
                                               channel_id: C)
                                               -> Option<Arc<RwLock<PrivateChannel>>> {
        self._private_channel(channel_id.into())
    }

    fn _private_channel(&self, channel_id: ChannelId) -> Option<Arc<RwLock<PrivateChannel>>> {
        self.private_channels.get(&channel_id).cloned()
    }

    /// Retrieves a [`Guild`]'s role by their Ids.
    ///
    /// **Note**: This will clone the entire role. Instead, retrieve the guild
    /// and retrieve from the guild's [`roles`] map to avoid this.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`roles`]: ../model/guild/struct.Guild.html#structfield.roles
    ///
    /// # Examples
    ///
    /// Retrieve a role from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::{Cache, CacheRwLock};
    /// # use parking_lot::RwLock;
    /// # use std::{error::Error, sync::Arc};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(role) = cache.read().role(7, 77) {
    ///     println!("Role with Id 77 is called {}", role.name);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn role<G, R>(&self, guild_id: G, role_id: R) -> Option<Role>
        where G: Into<GuildId>, R: Into<RoleId> {
        self._role(guild_id.into(), role_id.into())
    }

    fn _role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.guilds
            .get(&guild_id)
            .and_then(|g| g.read().roles.get(&role_id).cloned())
    }

    /// Returns an immutable reference to the settings.
    ///
    /// # Examples
    ///
    /// Printing the maximum number of messages in a channel to be cached:
    ///
    /// ```rust
    /// use serenity::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// println!("Max settings: {}", cache.settings().max_messages);
    /// ```
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Returns a mutable reference to the settings.
    ///
    /// # Examples
    ///
    /// Create a new cache and modify the settings afterwards:
    ///
    /// ```rust
    /// use serenity::cache::Cache;
    ///
    /// let mut cache = Cache::new();
    /// cache.settings_mut().max_messages(10);
    /// ```
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Retrieves a `User` from the cache's [`users`] map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`UserId`].
    ///
    /// [`UserId`]: ../model/id/struct.UserId.html
    /// [`users`]: #structfield.users
    ///
    /// # Examples
    ///
    /// Retrieve a user from the cache and print their name:
    ///
    /// ```rust,no_run
    /// # use serenity::client::Context;
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// #
    /// # #[command]
    /// # fn test(context: &mut Context) -> CommandResult {
    /// if let Some(user) = context.cache.read().user(7) {
    ///     println!("User with Id 7 is currently named {}", user.read().name);
    /// }
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn user<U: Into<UserId>>(&self, user_id: U) -> Option<Arc<RwLock<User>>> {
        self._user(user_id.into())
    }

    fn _user(&self, user_id: UserId) -> Option<Arc<RwLock<User>>> {
        self.users.get(&user_id).cloned()
    }

    #[inline]
    pub fn categories<C: Into<ChannelId>>(&self,
                                          channel_id: C)
                                          -> Option<Arc<RwLock<ChannelCategory>>> {
        self._categories(channel_id.into())
    }

    fn _categories(&self, channel_id: ChannelId) -> Option<Arc<RwLock<ChannelCategory>>> {
        self.categories.get(&channel_id).cloned()
    }

    /// Updates the cache with the update implementation for an event or other
    /// custom update implementation.
    ///
    /// Refer to the documentation for [`CacheUpdate`] for more information.
    ///
    /// # Examples
    ///
    /// Refer to the [`CacheUpdate` examples].
    ///
    /// [`CacheUpdate`]: trait.CacheUpdate.html
    /// [`CacheUpdate` examples]: trait.CacheUpdate.html#examples
    pub fn update<E: CacheUpdate>(&mut self, e: &mut E) -> Option<E::Output> {
        e.update(self)
    }

    pub(crate) fn update_user_entry(&mut self, user: &User) {
        match self.users.entry(user.id) {
            Entry::Vacant(e) => {
                e.insert(Arc::new(RwLock::new(user.clone())));
            },
            Entry::Occupied(mut e) => {
                e.get_mut().write().clone_from(user);
            },
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            channels: HashMap::default(),
            categories: HashMap::default(),
            groups: HashMap::with_capacity(128),
            guilds: HashMap::default(),
            messages: HashMap::default(),
            notes: HashMap::default(),
            presences: HashMap::default(),
            private_channels: HashMap::with_capacity(128),
            settings: Settings::default(),
            shard_count: 1,
            unavailable_guilds: HashSet::default(),
            user: CurrentUser::default(),
            users: HashMap::default(),
            message_queue: HashMap::default(),
            __nonexhaustive: (),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use serde_json::{Number, Value};
    use std::{
        collections::HashMap,
        sync::Arc,
    };
    use crate::{
        cache::{Cache, CacheUpdate, Settings},
        model::prelude::*,
        prelude::RwLock,
    };
    use crate::model::guild::PremiumTier::Tier2;

    #[test]
    fn test_cache_messages() {
        let mut settings = Settings::new();
        settings.max_messages(2);
        let mut cache = Cache::new_with_settings(settings);

        // Test inserting one message into a channel's message cache.
        let datetime = DateTime::parse_from_str(
            "1983 Apr 13 12:09:14.274 +0000",
            "%Y %b %d %H:%M:%S%.3f %z",
        ).unwrap();
        let mut event = MessageCreateEvent {
            message: Message {
                id: MessageId(3),
                attachments: vec![],
                author: User {
                    id: UserId(2),
                    avatar: None,
                    bot: false,
                    discriminator: 1,
                    name: "user 1".to_owned(),
                    _nonexhaustive: (),
                },
                channel_id: ChannelId(2),
                guild_id: Some(GuildId(1)),
                content: String::new(),
                edited_timestamp: None,
                embeds: vec![],
                kind: MessageType::Regular,
                member: None,
                mention_everyone: false,
                mention_roles: vec![],
                mention_channels: None,
                mentions: vec![],
                nonce: Value::Number(Number::from(1)),
                pinned: false,
                reactions: vec![],
                timestamp: datetime.clone(),
                tts: false,
                webhook_id: None,
                activity: None,
                application: None,
                message_reference: None,
                flags: None,
                _nonexhaustive: (),
            },
            _nonexhaustive: (),
        };
        // Check that the channel cache doesn't exist.
        assert!(!cache.messages.contains_key(&event.message.channel_id));
        // Add first message, none because message ID 2 doesn't already exist.
        assert!(event.update(&mut cache).is_none());
        // None, it only returns the oldest message if the cache was already full.
        assert!(event.update(&mut cache).is_none());
        // Assert there's only 1 message in the channel's message cache.
        assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 1);

        // Add a second message, assert that channel message cache length is 2.
        event.message.id = MessageId(4);
        assert!(event.update(&mut cache).is_none());
        assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 2);

        // Add a third message, the first should now be removed.
        event.message.id = MessageId(5);
        assert!(event.update(&mut cache).is_some());

        {
            let channel = cache.messages.get(&event.message.channel_id).unwrap();

            assert_eq!(channel.len(), 2);
            // Check that the first message is now removed.
            assert!(!channel.contains_key(&MessageId(3)));
        }

        let guild_channel = GuildChannel {
            id: event.message.channel_id,
            bitrate: None,
            category_id: None,
            guild_id: event.message.guild_id.unwrap(),
            kind: ChannelType::Text,
            last_message_id: None,
            last_pin_timestamp: None,
            name: String::new(),
            permission_overwrites: vec![],
            position: 0,
            topic: None,
            user_limit: None,
            nsfw: false,
            slow_mode_rate: Some(0),
            _nonexhaustive: (),
        };

        // Add a channel delete event to the cache, the cached messages for that
        // channel should now be gone.
        let mut delete = ChannelDeleteEvent {
            channel: Channel::Guild(Arc::new(RwLock::new(guild_channel.clone()))),
            _nonexhaustive: (),
        };
        assert!(cache.update(&mut delete).is_none());
        assert!(!cache.messages.contains_key(&delete.channel.id()));

        // Test deletion of a guild channel's message cache when a GuildDeleteEvent
        // is received.
        let mut guild_create = {
            let mut channels = HashMap::new();
            channels.insert(ChannelId(2), Arc::new(RwLock::new(guild_channel.clone())));

            GuildCreateEvent {
                guild: Guild {
                    id: GuildId(1),
                    afk_channel_id: None,
                    afk_timeout: 0,
                    application_id: None,
                    default_message_notifications: DefaultMessageNotificationLevel::All,
                    emojis: HashMap::new(),
                    explicit_content_filter: ExplicitContentFilter::None,
                    features: vec![],
                    icon: None,
                    joined_at: datetime,
                    large: false,
                    member_count: 0,
                    members: HashMap::new(),
                    mfa_level: MfaLevel::None,
                    name: String::new(),
                    owner_id: UserId(3),
                    presences: HashMap::new(),
                    region: String::new(),
                    roles: HashMap::new(),
                    splash: None,
                    system_channel_id: None,
                    verification_level: VerificationLevel::Low,
                    voice_states: HashMap::new(),
                    description: None,
                    premium_tier: PremiumTier::Tier0,
                    channels,
                    premium_subscription_count: 0,
                    banner: None,
                    vanity_url_code: Some("bruhmoment".to_string()),
                    preferred_locale: "en-US".to_string(),
                    _nonexhaustive: (),
                },
                _nonexhaustive: (),
            }
        };
        assert!(cache.update(&mut guild_create).is_none());
        assert!(cache.update(&mut event).is_none());

        let mut guild_delete = GuildDeleteEvent {
            guild: PartialGuild {
                id: GuildId(1),
                afk_channel_id: None,
                afk_timeout: 0,
                default_message_notifications: DefaultMessageNotificationLevel::All,
                embed_channel_id: None,
                embed_enabled: false,
                emojis: HashMap::new(),
                features: vec![],
                icon: None,
                mfa_level: MfaLevel::None,
                name: String::new(),
                owner_id: UserId(3),
                region: String::new(),
                roles: HashMap::new(),
                splash: None,
                verification_level: VerificationLevel::Low,
                description: None,
                premium_tier: Tier2,
                premium_subscription_count: 12,
                banner: None,
                vanity_url_code: Some("bruhmoment".to_string()),
                _nonexhaustive: (),
            },
            _nonexhaustive: (),
        };

        // The guild existed in the cache, so the cache's guild is returned by the
        // update.
        assert!(cache.update(&mut guild_delete).is_some());

        // Assert that the channel's message cache no longer exists.
        assert!(!cache.messages.contains_key(&ChannelId(2)));
    }
}

/// A neworphantype to allow implementing `AsRef<CacheRwLock>`
/// for the automatically dereferenced underlying type.
#[derive(Clone)]
pub struct CacheRwLock(Arc<RwLock<Cache>>);

impl From<Arc<RwLock<Cache>>> for CacheRwLock {
    fn from(cache: Arc<RwLock<Cache>>) -> Self {
        Self(cache)
    }
}

impl AsRef<CacheRwLock> for CacheRwLock {
    fn as_ref(&self) -> &CacheRwLock {
        &self
    }
}

impl Default for CacheRwLock {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(Cache::default())))
    }
}

impl Deref for CacheRwLock {
    type Target = Arc<RwLock<Cache>>;

    fn deref(&self) -> &Arc<RwLock<Cache>> {
        &self.0
    }
}
