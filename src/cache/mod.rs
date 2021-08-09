//! A cache containing data received from [`Shard`]s.
//!
//! Using the cache allows to avoid REST API requests via the [`http`] module
//! where possible. Issuing too many requests will lead to ratelimits.
//!
//! Following a policy to never hand out locks, the cache will clone all values
//! when calling its methods.
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
//! [`Shard`]: crate::gateway::Shard
//! [`http`]: crate::http

use std::collections::{hash_map::RandomState, HashMap, VecDeque};
use std::default::Default;
use std::hash::BuildHasher;
use std::str::FromStr;

use dashmap::{
    iter::Iter,
    mapref::{entry::Entry, multiple::RefMulti},
    DashMap,
    DashSet,
};
use parking_lot::RwLock;
// use tokio::sync::RwLock;
use tracing::instrument;

use crate::model::prelude::*;
mod cache_update;
mod settings;

pub use self::cache_update::CacheUpdate;
pub use self::settings::Settings;

type MessageCache = DashMap<ChannelId, DashMap<MessageId, Message>>;

pub trait FromStrAndCache: Sized {
    type Err;

    #[allow(clippy::missing_errors_doc)]
    fn from_str<CRL>(cache: CRL, s: &str) -> Result<Self, Self::Err>
    where
        CRL: AsRef<Cache> + Send + Sync;
}

pub trait StrExt: Sized {
    #[allow(clippy::missing_errors_doc)]
    fn parse_cached<CRL, F: FromStrAndCache>(&self, cache: CRL) -> Result<F, F::Err>
    where
        CRL: AsRef<Cache> + Send + Sync;
}

impl StrExt for &str {
    #[allow(clippy::missing_errors_doc)]
    fn parse_cached<CRL, F: FromStrAndCache>(&self, cache: CRL) -> Result<F, F::Err>
    where
        CRL: AsRef<Cache> + Send + Sync,
    {
        F::from_str(&cache, self)
    }
}

impl<F: FromStr> FromStrAndCache for F {
    type Err = F::Err;

    #[allow(clippy::missing_errors_doc)]
    fn from_str<CRL>(_cache: CRL, s: &str) -> Result<Self, Self::Err>
    where
        CRL: AsRef<Cache> + Send + Sync,
    {
        s.parse::<F>()
    }
}

/// Iterator given to the selector closure in [`Cache::channel_messages_field`].
// Wrapper around a specific iterator type to allow swapping out iterators on cache design changes
//
// Clone impl waiting on this https://github.com/xacrimon/dashmap/pull/152
//#[derive(Clone)]
pub struct MessageIterator<'a, S: BuildHasher + Clone>(
    Iter<'a, MessageId, Message, S, DashMap<MessageId, Message, S>>,
);

impl<'a, S: 'a + BuildHasher + Clone> Iterator for MessageIterator<'a, S> {
    // type Item = &'a Message;
    type Item = RefMulti<'a, MessageId, Message, S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// A cache containing data received from [`Shard`]s.
///
/// Using the cache allows to avoid REST API requests via the [`http`] module
/// where possible. Issuing too many requests will lead to ratelimits.
///
/// The cache will clone all values when calling its methods.
///
/// [`Shard`]: crate::gateway::Shard
/// [`http`]: crate::http
#[derive(Debug)]
#[non_exhaustive]
pub struct Cache {
    /// A map of channels in [`Guild`]s that the current user has received data
    /// for.
    ///
    /// When a [`Event::GuildDelete`] or [`Event::GuildUnavailable`] is
    /// received and processed by the cache, the relevant channels are also
    /// removed from this map.
    pub(crate) channels: DashMap<ChannelId, GuildChannel>,
    /// A map of channel categories.
    pub(crate) categories: DashMap<ChannelId, ChannelCategory>,
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    pub(crate) guilds: DashMap<GuildId, Guild>,
    pub(crate) messages: MessageCache,
    /// A map of users' presences. This is updated in real-time. Note that
    /// status updates are often "eaten" by the gateway, and this should not
    /// be treated as being entirely 100% accurate.
    pub(crate) presences: DashMap<UserId, Presence>,
    /// A map of direct message channels that the current user has open with
    /// other users.
    pub(crate) private_channels: DashMap<ChannelId, PrivateChannel>,
    /// The total number of shards being used by the bot.
    pub(crate) shard_count: RwLock<u64>,
    /// A list of guilds which are "unavailable". Refer to the documentation for
    /// [`Event::GuildUnavailable`] for more information on when this can occur.
    ///
    /// Additionally, guilds are always unavailable for bot users when a Ready
    /// is received. Guilds are "sent in" over time through the receiving of
    /// [`Event::GuildCreate`]s.
    pub(crate) unavailable_guilds: DashSet<GuildId>,
    /// The current user "logged in" and for which events are being received
    /// for.
    ///
    /// The current user contains information that a regular [`User`] does not,
    /// such as whether it is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    pub(crate) user: RwLock<CurrentUser>,
    /// A map of users that the current user sees.
    ///
    /// Users are added to - and updated from - this map via the following
    /// received events:
    ///
    /// - [`GuildMemberAdd`][`GuildMemberAddEvent`]
    /// - [`GuildMemberRemove`][`GuildMemberRemoveEvent`]
    /// - [`GuildMembersChunk`][`GuildMembersChunkEvent`]
    /// - [`PresenceUpdate`][`PresenceUpdateEvent`]
    /// - [`Ready`][`ReadyEvent`]
    ///
    /// Note, however, that users are _not_ removed from the map on removal
    /// events such as [`GuildMemberRemove`][`GuildMemberRemoveEvent`], as other
    /// structs such as members or recipients may still exist.
    pub(crate) users: DashMap<UserId, User>,
    /// Queue of message IDs for each channel.
    ///
    /// This is simply a vecdeque so we can keep track of the order of messages
    /// inserted into the cache. When a maximum number of messages are in a
    /// channel's cache, we can pop the front and remove that ID from the cache.
    pub(crate) message_queue: DashMap<ChannelId, VecDeque<MessageId>>,
    /// The settings for the cache.
    settings: RwLock<Settings>,
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
    #[instrument]
    pub fn new_with_settings(settings: Settings) -> Self {
        Self {
            settings: RwLock::new(settings),
            ..Default::default()
        }
    }

    /// Fetches the number of [`Member`]s that have not had data received.
    ///
    /// The important detail to note here is that this is the number of
    /// _member_s that have not had data received. A single [`User`] may have
    /// multiple associated member objects that have not been received.
    ///
    /// This can be used in combination with [`Shard::chunk_guild`], and can be
    /// used to determine how many members have not yet been received.
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, ctx: Context, _: Ready) {
    ///         // Wait some time for guilds to be received.
    ///         //
    ///         // You should keep track of this in a better fashion by tracking how
    ///         // many guilds each `ready` has, and incrementing a counter on
    ///         // GUILD_CREATEs. Once the number is equal, print the number of
    ///         // unknown members.
    ///         //
    ///         // For demonstrative purposes we're just sleeping the thread for 5
    ///         // seconds.
    ///         tokio::time::sleep(Duration::from_secs(5)).await;
    ///
    ///         println!("{} unknown members", ctx.cache.unknown_members());
    ///     }
    /// }
    ///
    /// let mut client = Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Shard::chunk_guild`]: crate::gateway::Shard::chunk_guild
    pub fn unknown_members(&self) -> u64 {
        let mut total = 0;

        for guild_entry in self.guilds.iter() {
            let guild = guild_entry.value();

            let members = guild.members.len() as u64;

            if guild.member_count > members {
                total += guild.member_count - members;
            }
        }

        total
    }

    /// Fetches a vector of all [`PrivateChannel`] Ids that are
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
    /// # use serenity::cache::Cache;
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn run() {
    /// # let cache = Cache::default();
    /// let amount = cache.private_channels().len();
    ///
    /// println!("There are {} private channels", amount);
    /// # }
    /// ```
    pub fn private_channels(&self) -> DashMap<ChannelId, PrivateChannel> {
        self.private_channels.clone()
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
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, context: Context, _: Ready) {
    ///         let guilds = context.cache.guilds().len();
    ///
    ///         println!("Guilds in the Cache: {}", guilds);
    ///     }
    /// }
    /// ```
    ///
    /// [`Context`]: crate::client::Context
    /// [`Shard`]: crate::gateway::Shard
    pub fn guilds(&self) -> Vec<GuildId> {
        let chain = self.unavailable_guilds.clone().into_iter();
        self.guilds.iter().map(|i| *i.key()).chain(chain).collect()
    }

    /// Retrieves a [`Channel`] from the cache based on the given Id.
    ///
    /// This will search the [`Self::channels`] map, then the [`Self::private_channels`] map.
    ///
    /// If you know what type of channel you're looking for, you should instead
    /// manually retrieve from one of the respective maps or methods:
    ///
    /// - [`GuildChannel`]: [`Self::guild_channel`] or [`Self::channels`]
    /// - [`PrivateChannel`]: [`Self::private_channel`] or [`Self::private_channels`]
    #[inline]
    pub fn channel<C: Into<ChannelId>>(&self, id: C) -> Option<Channel> {
        self._channel(id.into())
    }

    fn _channel(&self, id: ChannelId) -> Option<Channel> {
        if let Some(channel) = self.channels.get(&id) {
            let channel = channel.clone();
            return Some(Channel::Guild(channel));
        }

        if let Some(private_channel) = self.private_channels.get(&id) {
            return Some(Channel::Private(private_channel.clone()));
        }

        None
    }

    /// This method allows to extract specific data from the cached messages of a channel by
    /// providing a `selector` closure picking what you want to extract from the messages
    /// iterator of a given channel.
    ///
    /// ```rust,no_run
    /// # let cache: serenity::cache::Cache = todo!();
    /// // Find all messages by user ID 8 in channel ID 7
    /// let messages_by_user = cache.channel_messages_field(7, |msgs| {
    ///     msgs.filter_map(|m| if m.author.id == 8 { Some(m.clone()) } else { None } ).collect::<Vec<_>>()
    /// });
    /// ```
    pub fn channel_messages_field<T>(
        &self,
        channel_id: impl Into<ChannelId>,
        selector: impl FnOnce(MessageIterator<'_, RandomState>) -> T,
    ) -> Option<T> {
        let msg = self.messages.get(&channel_id.into())?;
        let message_iter = MessageIterator(msg.iter());

        Some(selector(message_iter))
    }

    /// Clones an entire guild from the cache based on the given `id`.
    ///
    /// In order to clone only a field of the guild, use [`Self::guild_field`].
    ///
    ///
    /// # Examples
    ///
    /// Retrieve a guild from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(guild) = cache.guild(7) {
    ///     println!("Guild name: {}", guild.name);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn guild<G: Into<GuildId>>(&self, id: G) -> Option<Guild> {
        self._guild(id.into())
    }

    fn _guild(&self, id: GuildId) -> Option<Guild> {
        self.guilds.get(&id).map(|i| i.clone())
    }

    /// This method allows to select a field of the guild instead of
    /// the entire guild by providing a `field_selector`-closure picking what
    /// you want to clone.
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // We clone only the `len()` returned `usize` instead of the entire guild or the channels.
    /// if let Some(channel_len) = cache.guild_field(7, |guild| guild.channels.len()) {
    ///     println!("Guild channels count: {}", channel_len);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn guild_field<Ret, Fun>(&self, id: impl Into<GuildId>, field_selector: Fun) -> Option<Ret>
    where
        Fun: FnOnce(&Guild) -> Ret,
    {
        self._guild_field(id.into(), field_selector)
    }

    fn _guild_field<Ret, Fun>(&self, id: GuildId, field_accessor: Fun) -> Option<Ret>
    where
        Fun: FnOnce(&Guild) -> Ret,
    {
        let guild = self.guilds.get(&id)?;

        Some(field_accessor(&*guild))
    }

    /// Returns the number of cached guilds.
    pub fn guild_count(&self) -> usize {
        self.guilds.len()
    }

    /// Retrieves a reference to a [`Guild`]'s channel. Unlike [`Self::channel`],
    /// this will only search guilds for the given channel.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// # Examples
    ///
    /// Getting a guild's channel via the Id of the message received through a
    /// [`EventHandler::message`] event dispatch:
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// #
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, message: Message) {
    ///
    ///         let channel = match context.cache.guild_channel(message.channel_id) {
    ///             Some(channel) => channel,
    ///             None => {
    ///                 let result = message.channel_id.say(&context, "Could not find guild's channel data").await;
    ///                 if let Err(why) = result {
    ///                     println!("Error sending message: {:?}", why);
    ///                 }
    ///
    ///                 return;
    ///             },
    ///         };
    ///     }
    /// }
    ///
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EventHandler::message`]: crate::client::EventHandler::message
    #[inline]
    pub fn guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<GuildChannel> {
        self._guild_channel(id.into())
    }

    fn _guild_channel(&self, id: ChannelId) -> Option<GuildChannel> {
        self.channels.get(&id).map(|i| i.clone())
    }

    /// This method allows to only clone a field of the guild channel instead of
    /// the entire guild by providing a `field_selector`-closure picking what
    /// you want to clone.
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // We clone only the `name` instead of the entire channel.
    /// if let Some(channel_name) = cache.guild_channel_field(7, |channel| channel.name.clone()) {
    ///     println!("Guild channel name: {}", channel_name);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn guild_channel_field<Ret, Fun>(
        &self,
        id: impl Into<ChannelId>,
        field_selector: Fun,
    ) -> Option<Ret>
    where
        Fun: FnOnce(&GuildChannel) -> Ret,
    {
        self._guild_channel_field(id.into(), field_selector)
    }

    fn _guild_channel_field<Ret, Fun>(&self, id: ChannelId, field_selector: Fun) -> Option<Ret>
    where
        Fun: FnOnce(&GuildChannel) -> Ret,
    {
        let channel = self.channels.get(&id)?;

        Some(field_selector(&*channel))
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
    /// [`EventHandler::message`] context:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::{ChannelId, MessageId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::new_with_token("DISCORD_TOKEN"));
    /// # let message = ChannelId(0).message(&http, MessageId(1)).await.unwrap();
    /// # let cache = Cache::default();
    /// #
    /// let member = {
    ///     let channel = match cache.guild_channel(message.channel_id) {
    ///         Some(channel) => channel,
    ///         None => {
    ///             if let Err(why) = message.channel_id.say(http, "Error finding channel data").await {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///             return;
    ///         },
    ///     };
    ///
    ///     match cache.member(channel.guild_id, message.author.id) {
    ///         Some(member) => member,
    ///         None => {
    ///             if let Err(why) = message.channel_id.say(&http, "Error finding member data").await {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///             return;
    ///         },
    ///     }
    /// };
    ///
    /// let msg = format!("You have {} roles", member.roles.len());
    ///
    /// if let Err(why) = message.channel_id.say(&http, &msg).await {
    ///     println!("Error sending message: {:?}", why);
    /// }
    /// # }
    /// ```
    ///
    /// [`EventHandler::message`]: crate::client::EventHandler::message
    /// [`members`]: crate::model::guild::Guild::members
    #[inline]
    pub fn member<G, U>(&self, guild_id: G, user_id: U) -> Option<Member>
    where
        G: Into<GuildId>,
        U: Into<UserId>,
    {
        self._member(guild_id.into(), user_id.into())
    }

    fn _member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        match self.guilds.get(&guild_id) {
            Some(guild) => guild.members.get(&user_id).cloned(),
            None => None,
        }
    }

    /// This method allows to only clone a field of a member instead of
    /// the entire member by providing a `field_selector`-closure picking what
    /// you want to clone.
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // We clone only the `name` instead of the entire channel.
    /// if let Some(Some(nick)) = cache.member_field(7, 8, |member| member.nick.clone()) {
    ///     println!("Member's nick: {}", nick);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn member_field<Ret, Fun>(
        &self,
        guild_id: impl Into<GuildId>,
        user_id: impl Into<UserId>,
        field_selector: Fun,
    ) -> Option<Ret>
    where
        Fun: FnOnce(&Member) -> Ret,
    {
        self._member_field(guild_id.into(), user_id.into(), field_selector)
    }

    fn _member_field<Ret, Fun>(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        field_selector: Fun,
    ) -> Option<Ret>
    where
        Fun: FnOnce(&Member) -> Ret,
    {
        let guild = self.guilds.get(&guild_id)?;
        let member = guild.members.get(&user_id)?;

        Some(field_selector(member))
    }

    #[inline]
    pub fn guild_roles(&self, guild_id: impl Into<GuildId>) -> Option<HashMap<RoleId, Role>> {
        self._guild_roles(guild_id.into())
    }

    fn _guild_roles(&self, guild_id: GuildId) -> Option<HashMap<RoleId, Role>> {
        self.guilds.get(&guild_id).map(|g| g.roles.clone())
    }

    /// This method clones and returns all unavailable guilds.
    #[inline]
    pub fn unavailable_guilds(&self) -> DashSet<GuildId> {
        self.unavailable_guilds.clone()
    }

    /// This method returns all channels from a guild of with the given `guild_id`.
    #[inline]
    pub fn guild_channels(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> Option<DashMap<ChannelId, GuildChannel>> {
        self._guild_channels(guild_id.into())
    }

    fn _guild_channels(&self, guild_id: GuildId) -> Option<DashMap<ChannelId, GuildChannel>> {
        self.guilds.get(&guild_id).map(|g| {
            g.channels
                .iter()
                .filter_map(|c| match c.1 {
                    Channel::Guild(channel) => Some((channel.id, channel.clone())),
                    _ => None,
                })
                .collect()
        })
    }

    /// Returns the number of guild channels in the cache.
    pub fn guild_channel_count(&self) -> usize {
        self.channels.len()
    }

    /// This method returns all categories from a guild of with the given `guild_id`.
    #[inline]
    pub fn guild_categories(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> Option<DashMap<ChannelId, ChannelCategory>> {
        self._guild_categories(guild_id.into())
    }

    fn _guild_categories(&self, guild_id: GuildId) -> Option<DashMap<ChannelId, ChannelCategory>> {
        self.guilds.get(&guild_id).map(|g| {
            g.channels
                .iter()
                .filter_map(|c| match c.1 {
                    Channel::Category(category) => Some((category.id, category.clone())),
                    _ => None,
                })
                .collect()
        })
    }

    /// Returns the number of shards.
    #[inline]
    pub fn shard_count(&self) -> u64 {
        *self.shard_count.read()
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
    /// # use serenity::cache::Cache;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::{ChannelId, MessageId};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Arc::new(Http::new_with_token("DISCORD_TOKEN"));
    /// # let message = ChannelId(0).message(&http, MessageId(1)).await?;
    /// # let cache = Cache::default();
    /// #
    /// match cache.message(message.channel_id, message.id) {
    ///     Some(m) => assert_eq!(message.content, m.content),
    ///     None => println!("No message found in cache."),
    /// };
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EventHandler::message`]: crate::client::EventHandler::message
    #[inline]
    pub fn message<C, M>(&self, channel_id: C, message_id: M) -> Option<Message>
    where
        C: Into<ChannelId>,
        M: Into<MessageId>,
    {
        self._message(channel_id.into(), message_id.into())
    }

    fn _message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        self.messages
            .get(&channel_id)
            .and_then(|messages| messages.get(&message_id).map(|i| i.clone()))
    }

    /// Retrieves a [`PrivateChannel`] from the cache's [`Self::private_channels`]
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
    /// # use serenity::cache::Cache;
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #   let cache = Cache::default();
    /// // assuming the cache has been unlocked
    ///
    /// if let Some(channel) = cache.private_channel(7) {
    ///     println!("The recipient is {}", channel.recipient);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn private_channel(&self, channel_id: impl Into<ChannelId>) -> Option<PrivateChannel> {
        self._private_channel(channel_id.into())
    }

    fn _private_channel(&self, channel_id: ChannelId) -> Option<PrivateChannel> {
        self.private_channels.get(&channel_id).map(|i| i.clone())
    }

    /// Retrieves a [`Guild`]'s role by their Ids.
    ///
    /// **Note**: This will clone the entire role. Instead, retrieve the guild
    /// and retrieve from the guild's [`roles`] map to avoid this.
    ///
    /// [`Guild`]: crate::model::guild::Guild
    /// [`roles`]: crate::model::guild::Guild::roles
    ///
    /// # Examples
    ///
    /// Retrieve a role from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(role) = cache.role(7, 77) {
    ///     println!("Role with Id 77 is called {}", role.name);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn role<G, R>(&self, guild_id: G, role_id: R) -> Option<Role>
    where
        G: Into<GuildId>,
        R: Into<RoleId>,
    {
        self._role(guild_id.into(), role_id.into())
    }

    fn _role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.guilds.get(&guild_id).and_then(|g| g.roles.get(&role_id).cloned())
    }

    /// Returns the settings.
    ///
    /// # Examples
    ///
    /// Printing the maximum number of messages in a channel to be cached:
    ///
    /// ```rust
    /// use serenity::cache::Cache;
    ///
    /// # fn test() {
    /// let mut cache = Cache::new();
    /// println!("Max settings: {}", cache.settings().max_messages);
    /// # }
    /// ```
    pub fn settings(&self) -> Settings {
        self.settings.read().clone()
    }

    /// Sets the maximum amount of messages per channel to cache.
    ///
    /// By default, no messages will be cached.
    pub fn set_max_messages(&self, max: usize) {
        self.settings.write().max_messages = max;
    }

    /// Retrieves a [`User`] from the cache's [`Self::users`] map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`UserId`].
    ///
    /// [`UserId`]: crate::model::id::UserId
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
    /// # async fn test(context: &Context) -> CommandResult {
    /// if let Some(user) = context.cache.user(7) {
    ///     println!("User with Id 7 is currently named {}", user.name);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn user<U: Into<UserId>>(&self, user_id: U) -> Option<User> {
        self._user(user_id.into())
    }

    fn _user(&self, user_id: UserId) -> Option<User> {
        self.users.get(&user_id).map(|i| i.clone())
    }

    /// Clones all users and returns them.
    #[inline]
    pub fn users(&self) -> DashMap<UserId, User> {
        self.users.clone()
    }

    /// Returns the amount of cached users.
    #[inline]
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Clones a category matching the `channel_id` and returns it.
    #[inline]
    pub fn category<C: Into<ChannelId>>(&self, channel_id: C) -> Option<ChannelCategory> {
        self._category(channel_id.into())
    }

    fn _category(&self, channel_id: ChannelId) -> Option<ChannelCategory> {
        self.categories.get(&channel_id).map(|i| i.clone())
    }

    /// Clones all categories and returns them.
    #[inline]
    pub fn categories(&self) -> DashMap<ChannelId, ChannelCategory> {
        self.categories.clone()
    }

    /// Returns the amount of cached categories.
    #[inline]
    pub fn category_count(&self) -> usize {
        self.categories.len()
    }

    /// Returns the optional category ID of a channel.
    #[inline]
    pub fn channel_category_id(&self, channel_id: ChannelId) -> Option<ChannelId> {
        self.categories.get(&channel_id).map(|category| category.id)
    }

    /// This method clones and returns the user used by the bot.
    #[inline]
    pub fn current_user(&self) -> CurrentUser {
        self.user.read().clone()
    }

    /// This method returns the bot's ID.
    #[inline]
    pub fn current_user_id(&self) -> UserId {
        self.user.read().id
    }

    /// This method allows to only clone a field of the current user instead of
    /// the entire user by providing a `field_selector`-closure picking what
    /// you want to clone.
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// #
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cache = Cache::default();
    /// // We clone only the `name` instead of the entire channel.
    /// let id = cache.current_user_field(|user| user.id);
    /// println!("Current user's ID: {}", id);
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn current_user_field<Ret: Clone, Fun>(&self, field_selector: Fun) -> Ret
    where
        Fun: FnOnce(&CurrentUser) -> Ret,
    {
        let user = self.user.read();

        field_selector(&user)
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
    /// [`CacheUpdate`]: CacheUpdate
    /// [`CacheUpdate` examples]: CacheUpdate#examples
    #[instrument(skip(self, e))]
    pub fn update<E: CacheUpdate>(&self, e: &mut E) -> Option<E::Output> {
        e.update(self)
    }

    pub(crate) fn update_user_entry(&self, user: &User) {
        match self.users.entry(user.id) {
            Entry::Vacant(e) => {
                e.insert(user.clone());
            },
            Entry::Occupied(mut e) => {
                e.get_mut().clone_from(user);
            },
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            channels: DashMap::default(),
            categories: DashMap::default(),
            guilds: DashMap::default(),
            messages: DashMap::default(),
            presences: DashMap::default(),
            private_channels: DashMap::with_capacity(128),
            settings: RwLock::new(Settings::default()),
            shard_count: RwLock::new(1),
            unavailable_guilds: DashSet::default(),
            user: RwLock::new(CurrentUser::default()),
            users: DashMap::default(),
            message_queue: DashMap::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};

    use crate::json::from_number;
    use crate::{
        cache::{Cache, CacheUpdate, Settings},
        model::prelude::*,
    };

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cache_messages() {
        let mut settings = Settings::new();
        settings.max_messages(2);
        let cache = Cache::new_with_settings(settings);

        // Test inserting one message into a channel's message cache.
        let datetime =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .with_timezone(&Utc);
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
                    public_flags: None,
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
                mention_channels: vec![],
                mentions: vec![],
                nonce: from_number(1),
                pinned: false,
                reactions: vec![],
                timestamp: datetime,
                tts: false,
                webhook_id: None,
                activity: None,
                application: None,
                message_reference: None,
                flags: None,
                sticker_items: vec![],
                referenced_message: None,
                #[cfg(feature = "unstable_discord_api")]
                interaction: None,
                #[cfg(feature = "unstable_discord_api")]
                components: vec![],
            },
        };

        // Check that the channel cache doesn't exist.
        assert!(!cache.messages.contains_key(&event.message.channel_id));
        // Add first message, none because message ID 2 doesn't already exist.
        assert!(event.update(&cache).is_none());
        // None, it only returns the oldest message if the cache was already full.
        assert!(event.update(&cache).is_none());
        // Assert there's only 1 message in the channel's message cache.
        assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 1);

        // Add a second message, assert that channel message cache length is 2.
        event.message.id = MessageId(4);
        assert!(event.update(&cache).is_none());
        assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 2);

        // Add a third message, the first should now be removed.
        event.message.id = MessageId(5);
        assert!(event.update(&cache).is_some());

        {
            let channel = cache.messages.get(&event.message.channel_id).unwrap();

            assert_eq!(channel.len(), 2);
            // Check that the first message is now removed.
            assert!(!channel.contains_key(&MessageId(3)));
        }

        let channel = Channel::Guild(GuildChannel {
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
            rtc_region: None,
            video_quality_mode: None,
            message_count: None,
            member_count: None,
            thread_metadata: None,
            member: None,
            default_auto_archive_duration: None,
        });

        // Add a channel delete event to the cache, the cached messages for that
        // channel should now be gone.
        let mut delete = ChannelDeleteEvent {
            channel: channel.clone(),
        };
        assert!(cache.update(&mut delete).is_none());
        assert!(!cache.messages.contains_key(&delete.channel.id()));

        // Test deletion of a guild channel's message cache when a GuildDeleteEvent
        // is received.
        let mut guild_create = {
            let mut channels = HashMap::new();
            channels.insert(ChannelId(2), channel);

            #[allow(deprecated)]
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
                    discovery_splash: None,
                    system_channel_id: None,
                    system_channel_flags: Default::default(),
                    rules_channel_id: None,
                    public_updates_channel_id: None,
                    verification_level: VerificationLevel::Low,
                    voice_states: HashMap::new(),
                    description: None,
                    premium_tier: PremiumTier::Tier0,
                    channels,
                    premium_subscription_count: 0,
                    banner: None,
                    vanity_url_code: Some("bruhmoment".to_string()),
                    preferred_locale: "en-US".to_string(),
                    welcome_screen: None,
                    approximate_member_count: None,
                    approximate_presence_count: None,
                    nsfw: false,
                    nsfw_level: NsfwLevel::Default,
                    max_video_channel_users: None,
                    max_presences: None,
                    max_members: None,
                    widget_enabled: Some(false),
                    widget_channel_id: None,
                    stage_instances: vec![],
                    threads: vec![],
                    stickers: HashMap::new(),
                },
            }
        };
        assert!(cache.update(&mut guild_create).is_none());
        assert!(cache.update(&mut event).is_none());

        let mut guild_delete = GuildDeleteEvent {
            guild: GuildUnavailable {
                id: GuildId(1),
                unavailable: false,
            },
        };

        // The guild existed in the cache, so the cache's guild is returned by the
        // update.
        assert!(cache.update(&mut guild_delete).is_some());

        // Assert that the channel's message cache no longer exists.
        assert!(!cache.messages.contains_key(&ChannelId(2)));
    }
}
