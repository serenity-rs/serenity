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

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::str::FromStr;
#[cfg(feature = "temp_cache")]
use std::sync::Arc;
#[cfg(feature = "temp_cache")]
use std::time::Duration;

use dashmap::mapref::entry::Entry;
use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
#[cfg(feature = "temp_cache")]
use moka::dash::Cache as DashCache;
use parking_lot::RwLock;
use tracing::instrument;

pub use self::cache_update::CacheUpdate;
pub use self::settings::Settings;
use crate::model::prelude::*;

mod cache_update;
mod event;
mod settings;

type MessageCache = DashMap<ChannelId, HashMap<MessageId, Message>, FxBuildHasher>;

struct NotSend;

enum CacheRefInner<'a, K, V> {
    #[cfg(feature = "temp_cache")]
    Arc(Arc<V>),
    DashRef(Ref<'a, K, V, FxBuildHasher>),
    ReadGuard(parking_lot::RwLockReadGuard<'a, V>),
}

pub struct CacheRef<'a, K, V> {
    inner: CacheRefInner<'a, K, V>,
    phantom: std::marker::PhantomData<*const NotSend>,
}

impl<'a, K, V> CacheRef<'a, K, V> {
    fn new(inner: CacheRefInner<'a, K, V>) -> Self {
        Self {
            inner,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "temp_cache")]
    fn from_arc(inner: Arc<V>) -> Self {
        Self::new(CacheRefInner::Arc(inner))
    }

    fn from_ref(inner: Ref<'a, K, V, FxBuildHasher>) -> Self {
        Self::new(CacheRefInner::DashRef(inner))
    }

    fn from_guard(inner: parking_lot::RwLockReadGuard<'a, V>) -> Self {
        Self::new(CacheRefInner::ReadGuard(inner))
    }
}

impl<K: Eq + Hash, V> std::ops::Deref for CacheRef<'_, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            #[cfg(feature = "temp_cache")]
            CacheRefInner::Arc(inner) => inner,
            CacheRefInner::DashRef(inner) => inner.value(),
            CacheRefInner::ReadGuard(inner) => inner,
        }
    }
}

pub type UserRef<'a> = CacheRef<'a, UserId, User>;
pub type GuildRef<'a> = CacheRef<'a, GuildId, Guild>;
pub type CurrentUserRef<'a> = CacheRef<'a, (), CurrentUser>;
pub type GuildChannelRef<'a> = CacheRef<'a, ChannelId, GuildChannel>;
pub type PrivateChannelRef<'a> = CacheRef<'a, ChannelId, PrivateChannel>;
pub type ChannelMessagesRef<'a> = CacheRef<'a, ChannelId, HashMap<MessageId, Message>>;

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

#[derive(Debug)]
pub(crate) struct CachedShardData {
    pub total: u32,
    pub connected: HashSet<u32>,
    pub has_sent_shards_ready: bool,
}

#[derive(Debug)]
pub(crate) struct MaybeMap<K: Eq + Hash, V>(Option<DashMap<K, V, FxBuildHasher>>);
impl<K: Eq + Hash, V> MaybeMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V, FxBuildHasher>> {
        Option::iter(&self.0).flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V, FxBuildHasher>> {
        self.0.as_ref()?.get(k)
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<'_, K, V, FxBuildHasher>> {
        self.0.as_ref()?.get_mut(k)
    }

    pub fn insert(&self, k: K, v: V) -> Option<V> {
        self.0.as_ref()?.insert(k, v)
    }

    pub fn remove(&self, k: &K) -> Option<(K, V)> {
        self.0.as_ref()?.remove(k)
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |map| map.len())
    }
}

/// A cache containing data received from [`Shard`]s.
///
/// Using the cache allows to avoid REST API requests via the [`http`] module
/// where possible. Issuing too many requests will lead to ratelimits.
///
/// This is the list of cached resources and the events that populate them:
/// - channels: [`ChannelCreateEvent`], [`ChannelUpdateEvent`], [`GuildCreateEvent`]
/// - private_channels: [`ChannelCreateEvent`]
/// - guilds: [`GuildCreateEvent`]
/// - unavailable_guilds: [`ReadyEvent`], [`GuildDeleteEvent`]
/// - users: [`GuildMemberAddEvent`], [`GuildMemberRemoveEvent`], [`GuildMembersChunkEvent`], [`PresenceUpdateEvent`], [`ReadyEvent`]
/// - presences: [`PresenceUpdateEvent`], [`ReadyEvent`]
/// - messages: [`MessageCreateEvent`]
///
/// The documentation of each event contains the required gateway intents.
///
/// [`Shard`]: crate::gateway::Shard
/// [`http`]: crate::http
#[derive(Debug)]
#[non_exhaustive]
pub struct Cache {
    // Temp cache:
    // ---
    /// Cache of channels that have been fetched via to_channel.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_channels: DashCache<ChannelId, GuildChannel, FxBuildHasher>,
    /// Cache of users who have been fetched from `to_user`.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_users: DashCache<UserId, Arc<User>, FxBuildHasher>,

    // Channels cache:
    // ---
    /// A map of channels in [`Guild`]s that the current user has received data
    /// for.
    ///
    /// When a [`Event::GuildDelete`] is received and processed by the cache,
    /// the relevant channels are also removed from this map.
    pub(crate) channels: MaybeMap<ChannelId, GuildChannel>,
    /// A map of direct message channels that the current user has open with
    /// other users.
    pub(crate) private_channels: MaybeMap<ChannelId, PrivateChannel>,

    // Guilds cache:
    // ---
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    pub(crate) guilds: MaybeMap<GuildId, Guild>,
    /// A list of guilds which are "unavailable".
    ///
    /// Additionally, guilds are always unavailable for bot users when a Ready
    /// is received. Guilds are "sent in" over time through the receiving of
    /// [`Event::GuildCreate`]s.
    pub(crate) unavailable_guilds: MaybeMap<GuildId, ()>,

    // Users cache:
    // ---
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
    pub(crate) users: MaybeMap<UserId, User>,
    /// A map of users' presences. This is updated in real-time. Note that
    /// status updates are often "eaten" by the gateway, and this should not
    /// be treated as being entirely 100% accurate.
    pub(crate) presences: MaybeMap<UserId, Presence>,

    // Messages cache:
    // ---
    pub(crate) messages: MessageCache,
    /// Queue of message IDs for each channel.
    ///
    /// This is simply a vecdeque so we can keep track of the order of messages
    /// inserted into the cache. When a maximum number of messages are in a
    /// channel's cache, we can pop the front and remove that ID from the cache.
    pub(crate) message_queue: DashMap<ChannelId, VecDeque<MessageId>, FxBuildHasher>,

    // Miscellanous fixed-size data
    // ---
    /// Information about running shards
    pub(crate) shard_data: RwLock<CachedShardData>,
    /// The current user "logged in" and for which events are being received
    /// for.
    ///
    /// The current user contains information that a regular [`User`] does not,
    /// such as whether it is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    pub(crate) user: RwLock<CurrentUser>,
    /// The settings for the cache.
    settings: RwLock<Settings>,
}

impl Cache {
    /// Creates a new cache.
    #[inline]
    #[must_use]
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
    /// let mut settings = Settings::default();
    /// settings.max_messages = 10;
    ///
    /// let cache = Cache::new_with_settings(settings);
    /// ```
    #[instrument]
    pub fn new_with_settings(settings: Settings) -> Self {
        #[cfg(feature = "temp_cache")]
        fn temp_cache<K, V>(ttl: Duration) -> DashCache<K, V, FxBuildHasher>
        where
            K: Hash + Eq + Send + Sync + 'static,
            V: Clone + Send + Sync + 'static,
        {
            DashCache::builder().time_to_live(ttl).build_with_hasher(FxBuildHasher::default())
        }

        Self {
            #[cfg(feature = "temp_cache")]
            temp_channels: temp_cache(settings.time_to_live),
            #[cfg(feature = "temp_cache")]
            temp_users: temp_cache(settings.time_to_live),

            channels: MaybeMap(settings.cache_channels.then(DashMap::default)),
            private_channels: MaybeMap(settings.cache_channels.then(DashMap::default)),

            guilds: MaybeMap(settings.cache_guilds.then(DashMap::default)),
            unavailable_guilds: MaybeMap(settings.cache_guilds.then(DashMap::default)),

            users: MaybeMap(settings.cache_users.then(DashMap::default)),
            presences: MaybeMap(settings.cache_users.then(DashMap::default)),

            messages: DashMap::default(),
            message_queue: DashMap::default(),

            shard_data: RwLock::new(CachedShardData {
                total: 1,
                connected: HashSet::new(),
                has_sent_shards_ready: false,
            }),
            user: RwLock::new(CurrentUser::default()),
            settings: RwLock::new(settings),
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
    ///     async fn cache_ready(&self, ctx: Context, _: Vec<GuildId>) {
    ///         println!("{} unknown members", ctx.cache.unknown_members());
    ///     }
    /// }
    ///
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
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
    /// #
    /// # let cache = Cache::default();
    /// let amount = cache.private_channels().len();
    ///
    /// println!("There are {} private channels", amount);
    /// ```
    pub fn private_channels(
        &self,
    ) -> DashMap<ChannelId, PrivateChannel, impl BuildHasher + Send + Sync + Clone> {
        self.private_channels.0.clone().unwrap_or_default()
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
        let chain = self.unavailable_guilds().into_iter().map(|(k, _)| k);
        self.guilds.iter().map(|i| *i.key()).chain(chain).collect()
    }

    /// Retrieves a [`Channel`] from the cache based on the given Id.
    ///
    /// This will search the `channels` map, then the [`Self::private_channels`] map.
    ///
    /// If you know what type of channel you're looking for, you should instead
    /// manually retrieve from one of the respective methods:
    ///
    /// - [`GuildChannel`]: [`Self::guild_channel`] or [`Self::guild_channels`]
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

        #[cfg(feature = "temp_cache")]
        {
            if let Some(channel) = self.temp_channels.get(&id) {
                return Some(Channel::Guild(channel));
            }
        }

        if let Some(private_channel) = self.private_channels.get(&id) {
            return Some(Channel::Private(private_channel.clone()));
        }

        None
    }

    /// Get a reference to the cached messages for a channel based on the given `Id`.
    ///
    /// # Examples
    ///
    /// Find all messages by user ID 8 in channel ID 7:
    ///
    /// ```rust,no_run
    /// # let cache: serenity::cache::Cache = todo!();
    /// let messages_in_channel = cache.channel_messages(7);
    /// let messages_by_user = messages_in_channel
    ///     .as_ref()
    ///     .map(|msgs| msgs.values().filter(|m| m.author.id == 8).collect::<Vec<_>>());
    /// ```
    pub fn channel_messages(
        &self,
        channel_id: impl Into<ChannelId>,
    ) -> Option<ChannelMessagesRef<'_>> {
        self.messages.get(&channel_id.into()).map(CacheRef::from_ref)
    }

    /// Gets a reference to a guild from the cache based on the given `id`.
    ///
    /// # Examples
    ///
    /// Retrieve a guild from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(guild) = cache.guild(7) {
    ///     println!("Guild name: {}", guild.name);
    /// };
    /// ```
    #[inline]
    pub fn guild<G: Into<GuildId>>(&self, id: G) -> Option<GuildRef<'_>> {
        self._guild(id.into())
    }

    fn _guild(&self, id: GuildId) -> Option<GuildRef<'_>> {
        self.guilds.get(&id).map(CacheRef::from_ref)
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
    ///         let channel_opt = context.cache.guild_channel(message.channel_id).as_deref().cloned();
    ///         let channel = match channel_opt {
    ///             Some(channel) => channel,
    ///             None => {
    ///                 let result = message
    ///                     .channel_id
    ///                     .say(&context, "Could not find guild's channel data")
    ///                     .await;
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
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EventHandler::message`]: crate::client::EventHandler::message
    #[inline]
    pub fn guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<GuildChannelRef<'_>> {
        self._guild_channel(id.into())
    }

    fn _guild_channel(&self, id: ChannelId) -> Option<GuildChannelRef<'_>> {
        self.channels.get(&id).map(CacheRef::from_ref)
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
    /// # use serenity::model::channel::Message;
    /// #
    /// # async fn run(http: Http, cache: Cache, message: Message) {
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
    pub fn unavailable_guilds(
        &self,
    ) -> DashMap<GuildId, (), impl BuildHasher + Send + Sync + Clone> {
        self.unavailable_guilds.0.clone().unwrap_or_default()
    }

    /// This method returns all channels from a guild of with the given `guild_id`.
    #[inline]
    pub fn guild_channels(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> Option<DashMap<ChannelId, GuildChannel, FxBuildHasher>> {
        self._guild_channels(guild_id.into())
    }

    fn _guild_channels(
        &self,
        guild_id: GuildId,
    ) -> Option<DashMap<ChannelId, GuildChannel, FxBuildHasher>> {
        self.guilds.get(&guild_id).map(|g| g.channels.clone().into_iter().collect())
    }

    /// Returns the number of guild channels in the cache.
    pub fn guild_channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of shards.
    #[inline]
    pub fn shard_count(&self) -> u32 {
        self.shard_data.read().total
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
    /// # use serenity::model::channel::Message;
    /// #
    /// # fn run(cache: Cache, message: Message) {
    /// #
    /// match cache.message(message.channel_id, message.id) {
    ///     Some(m) => assert_eq!(message.content, m.content),
    ///     None => println!("No message found in cache."),
    /// };
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
        self.messages.get(&channel_id).and_then(|messages| messages.get(&message_id).cloned())
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
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache has been unlocked
    ///
    /// if let Some(channel) = cache.private_channel(7) {
    ///     println!("The recipient is {}", channel.recipient);
    /// };
    /// ```
    #[inline]
    pub fn private_channel(
        &self,
        channel_id: impl Into<ChannelId>,
    ) -> Option<PrivateChannelRef<'_>> {
        self._private_channel(channel_id.into())
    }

    fn _private_channel(&self, channel_id: ChannelId) -> Option<PrivateChannelRef<'_>> {
        self.private_channels.get(&channel_id).map(CacheRef::from_ref)
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
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(role) = cache.role(7, 77) {
    ///     println!("Role with Id 77 is called {}", role.name);
    /// }
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
    pub fn user<U: Into<UserId>>(&self, user_id: U) -> Option<UserRef<'_>> {
        self._user(user_id.into())
    }

    #[cfg(feature = "temp_cache")]
    fn _user(&self, user_id: UserId) -> Option<UserRef<'_>> {
        if let Some(user) = self.users.get(&user_id) {
            Some(CacheRef::from_ref(user))
        } else {
            self.temp_users.get(&user_id).map(CacheRef::from_arc)
        }
    }

    #[cfg(not(feature = "temp_cache"))]
    fn _user(&self, user_id: UserId) -> Option<UserRef<'_>> {
        self.users.get(&user_id).map(CacheRef::from_ref)
    }

    /// Clones all users and returns them.
    #[inline]
    pub fn users(&self) -> DashMap<UserId, User, impl BuildHasher + Send + Sync + Clone> {
        self.users.0.clone().unwrap_or_default()
    }

    /// Returns the amount of cached users.
    #[inline]
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// This method provides a reference to the user used by the bot.
    #[inline]
    pub fn current_user(&self) -> CurrentUserRef<'_> {
        CacheRef::from_guard(self.user.read())
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
    /// [`CacheUpdate` examples]: CacheUpdate#examples
    #[instrument(skip(self, e))]
    pub fn update<E: CacheUpdate>(&self, e: &mut E) -> Option<E::Output> {
        e.update(self)
    }

    pub(crate) fn update_user_entry(&self, user: &User) {
        if let Some(users) = &self.users.0 {
            match users.entry(user.id) {
                Entry::Vacant(e) => {
                    e.insert(user.clone());
                },
                Entry::Occupied(mut e) => {
                    e.get_mut().clone_from(user);
                },
            }
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new_with_settings(Settings::default())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::cache::{Cache, CacheUpdate, Settings};
    use crate::model::prelude::*;

    #[test]
    fn test_cache_messages() {
        let settings = Settings {
            max_messages: 2,
            ..Default::default()
        };
        let cache = Cache::new_with_settings(settings);

        // Test inserting one message into a channel's message cache.
        let mut event = MessageCreateEvent {
            message: Message {
                id: MessageId::new(3),
                guild_id: Some(GuildId::new(1)),
                ..Default::default()
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
        event.message.id = MessageId::new(4);
        assert!(event.update(&cache).is_none());
        assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 2);

        // Add a third message, the first should now be removed.
        event.message.id = MessageId::new(5);
        assert!(event.update(&cache).is_some());

        {
            let channel = cache.messages.get(&event.message.channel_id).unwrap();

            assert_eq!(channel.len(), 2);
            // Check that the first message is now removed.
            assert!(!channel.contains_key(&MessageId::new(3)));
        }

        let channel = GuildChannel {
            id: event.message.channel_id,
            guild_id: event.message.guild_id.unwrap(),
            ..Default::default()
        };

        // Add a channel delete event to the cache, the cached messages for that
        // channel should now be gone.
        let mut delete = ChannelDeleteEvent {
            channel: Channel::Guild(channel.clone()),
        };
        assert!(cache.update(&mut delete).is_some());
        assert!(!cache.messages.contains_key(&delete.channel.id()));

        // Test deletion of a guild channel's message cache when a GuildDeleteEvent
        // is received.
        let mut guild_create = GuildCreateEvent {
            guild: Guild {
                id: GuildId::new(1),
                channels: HashMap::from([(ChannelId::new(2), channel)]),
                ..Default::default()
            },
        };
        assert!(cache.update(&mut guild_create).is_none());
        assert!(cache.update(&mut event).is_none());

        let mut guild_delete = GuildDeleteEvent {
            guild: UnavailableGuild {
                id: GuildId::new(1),
                unavailable: false,
            },
        };

        // The guild existed in the cache, so the cache's guild is returned by the
        // update.
        assert!(cache.update(&mut guild_delete).is_some());

        // Assert that the channel's message cache no longer exists.
        assert!(!cache.messages.contains_key(&ChannelId::new(2)));
    }
}
