//! A cache containing data received from [`Shard`]s.
//!
//! Using the cache allows to avoid REST API requests via the [`http`] module where possible.
//! Issuing too many requests will lead to ratelimits.
//!
//! # Use by Models
//!
//! Most models of Discord objects, such as the [`Message`], [`GuildChannel`], or [`Emoji`], have
//! methods for interacting with that single instance. This feature is only compiled if the `model`
//! feature is enabled. An example of this is [`Guild::edit`], which performs a check to ensure that
//! the current user has the [Manage Guild] permission prior to actually performing the HTTP
//! request. The cache is involved due to the function's use of unlocking the cache and retrieving
//! the permissions of the current user. This is an inexpensive method of being able to access data
//! required by these sugary methods.
//!
//! # Do I need the Cache?
//!
//! If you're asking this, the answer is likely "definitely yes" or "definitely no"; any in-between
//! tends to be "yes". If you are low on RAM, and need to run on only a couple MB, then the answer
//! is "definitely no". If you do not care about RAM and want your bot to be able to access data
//! while needing to hit the REST API as little as possible, then the answer is "yes".
//!
//! [`Shard`]: crate::gateway::Shard
//! [`http`]: crate::http
//! [Manage Guild]: Permissions::MANAGE_GUILD

use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use std::num::NonZeroU16;
#[cfg(feature = "temp_cache")]
use std::sync::Arc;
#[cfg(feature = "temp_cache")]
use std::time::Duration;

use dashmap::mapref::one::{MappedRef, Ref};
use dashmap::DashMap;
#[cfg(feature = "temp_cache")]
use mini_moka::sync::Cache as MokaCache;
use parking_lot::RwLock;

pub use self::cache_update::CacheUpdate;
pub use self::settings::Settings;
use crate::model::prelude::*;

mod cache_update;
mod event;
mod settings;
pub(crate) mod wrappers;

#[cfg(feature = "temp_cache")]
pub(crate) use wrappers::MaybeOwnedArc;
use wrappers::{BuildHasher, MaybeMap, ReadOnlyMapRef};

struct NotSend;

enum CacheRefInner<'a, K, V, T> {
    #[cfg(feature = "temp_cache")]
    Arc(Arc<V>),
    DashRef(Ref<'a, K, V, BuildHasher>),
    DashMappedRef(MappedRef<'a, K, T, V, BuildHasher>),
    ReadGuard(parking_lot::RwLockReadGuard<'a, V>),
}

pub struct CacheRef<'a, K, V, T = ()> {
    inner: CacheRefInner<'a, K, V, T>,
    phantom: std::marker::PhantomData<*const NotSend>,
}

impl<'a, K, V, T> CacheRef<'a, K, V, T> {
    fn new(inner: CacheRefInner<'a, K, V, T>) -> Self {
        Self {
            inner,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "temp_cache")]
    fn from_arc(inner: MaybeOwnedArc<V>) -> Self {
        Self::new(CacheRefInner::Arc(inner.get_inner()))
    }

    fn from_ref(inner: Ref<'a, K, V, BuildHasher>) -> Self {
        Self::new(CacheRefInner::DashRef(inner))
    }

    fn from_mapped_ref(inner: MappedRef<'a, K, T, V, BuildHasher>) -> Self {
        Self::new(CacheRefInner::DashMappedRef(inner))
    }

    fn from_guard(inner: parking_lot::RwLockReadGuard<'a, V>) -> Self {
        Self::new(CacheRefInner::ReadGuard(inner))
    }
}

impl<K: Eq + Hash, V, T> std::ops::Deref for CacheRef<'_, K, V, T> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            #[cfg(feature = "temp_cache")]
            CacheRefInner::Arc(inner) => inner,
            CacheRefInner::DashRef(inner) => inner.value(),
            CacheRefInner::DashMappedRef(inner) => inner.value(),
            CacheRefInner::ReadGuard(inner) => inner,
        }
    }
}

type Never = std::convert::Infallible;
type MappedGuildRef<'a, T> = CacheRef<'a, GuildId, T, Guild>;

pub type MemberRef<'a> = MappedGuildRef<'a, Member>;
pub type GuildRoleRef<'a> = MappedGuildRef<'a, Role>;
pub type UserRef<'a> = CacheRef<'a, UserId, User, Never>;
pub type GuildRef<'a> = CacheRef<'a, GuildId, Guild, Never>;
pub type SettingsRef<'a> = CacheRef<'a, Never, Settings, Never>;
pub type GuildChannelRef<'a> = MappedGuildRef<'a, GuildChannel>;
pub type CurrentUserRef<'a> = CacheRef<'a, Never, CurrentUser, Never>;
pub type GuildRolesRef<'a> = MappedGuildRef<'a, HashMap<RoleId, Role>>;
pub type MessageRef<'a> = CacheRef<'a, ChannelId, Message, VecDeque<Message>>;
pub type ChannelMessagesRef<'a> = CacheRef<'a, ChannelId, VecDeque<Message>, Never>;
pub type GuildChannelsRef<'a> = MappedGuildRef<'a, HashMap<ChannelId, GuildChannel>>;

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug)]
pub(crate) struct CachedShardData {
    pub total: NonZeroU16,
    pub connected: HashSet<ShardId>,
    pub has_sent_shards_ready: bool,
}

/// A cache containing data received from [`Shard`]s.
///
/// Using the cache allows to avoid REST API requests via the [`http`] module where possible.
/// Issuing too many requests will lead to ratelimits.
///
/// This is the list of cached resources and the events that populate them:
/// - channels: [`ChannelCreateEvent`], [`ChannelUpdateEvent`], [`GuildCreateEvent`]
/// - guilds: [`GuildCreateEvent`]
/// - unavailable_guilds: [`ReadyEvent`], [`GuildDeleteEvent`]
/// - users: [`GuildMemberAddEvent`], [`GuildMemberRemoveEvent`], [`GuildMembersChunkEvent`],
///   [`PresenceUpdateEvent`], [`ReadyEvent`]
/// - presences: [`PresenceUpdateEvent`], [`ReadyEvent`]
/// - messages: [`MessageCreateEvent`]
///
/// The documentation of each event contains the required gateway intents.
///
/// [`Shard`]: crate::gateway::Shard
/// [`http`]: crate::http
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug)]
#[non_exhaustive]
pub struct Cache {
    // Temp cache:
    // ---
    /// Cache of channels that have been fetched via to_channel.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_channels: MokaCache<ChannelId, MaybeOwnedArc<GuildChannel>, BuildHasher>,
    /// Cache of private channels created via create_dm_channel.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_private_channels: MokaCache<UserId, MaybeOwnedArc<PrivateChannel>, BuildHasher>,
    /// Cache of messages that have been fetched via message.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_messages: MokaCache<MessageId, MaybeOwnedArc<Message>, BuildHasher>,
    /// Cache of users who have been fetched from `to_user`.
    ///
    /// The TTL for each value is configured in CacheSettings.
    #[cfg(feature = "temp_cache")]
    pub(crate) temp_users: MokaCache<UserId, MaybeOwnedArc<User>, BuildHasher>,

    // Channels cache:
    /// A map of channel ids to the guilds in which the channel data is stored.
    pub(crate) channels: MaybeMap<ChannelId, GuildId>,

    // Guilds cache:
    // ---
    /// A map of guilds with full data available. This includes data like [`Role`]s and [`Emoji`]s
    /// that are not available through the REST API.
    pub(crate) guilds: MaybeMap<GuildId, Guild>,
    /// A list of guilds which are "unavailable".
    ///
    /// Additionally, guilds are always unavailable for bot users when a Ready is received. Guilds
    /// are "sent in" over time through the receiving of [`Event::GuildCreate`]s.
    pub(crate) unavailable_guilds: MaybeMap<GuildId, ()>,

    // Messages cache:
    // ---
    pub(crate) messages: DashMap<ChannelId, VecDeque<Message>, BuildHasher>,

    // Miscellanous fixed-size data
    // ---
    /// Information about running shards
    pub(crate) shard_data: RwLock<CachedShardData>,
    /// The current user "logged in" and for which events are being received for.
    ///
    /// The current user contains information that a regular [`User`] does not, such as whether it
    /// is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    pub(crate) user: RwLock<CurrentUser>,
    /// The settings for the cache.
    settings: RwLock<Settings>,
}

impl Cache {
    /// Creates a new cache.
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
    #[cfg_attr(feature = "tracing_instrument", instrument)]
    pub fn new_with_settings(settings: Settings) -> Self {
        #[cfg(feature = "temp_cache")]
        fn temp_cache<K, V>(ttl: Duration) -> MokaCache<K, V, BuildHasher>
        where
            K: Hash + Eq + Send + Sync + 'static,
            V: Clone + Send + Sync + 'static,
        {
            MokaCache::builder().time_to_live(ttl).build_with_hasher(BuildHasher::default())
        }

        Self {
            #[cfg(feature = "temp_cache")]
            temp_private_channels: temp_cache(settings.time_to_live),
            #[cfg(feature = "temp_cache")]
            temp_channels: temp_cache(settings.time_to_live),
            #[cfg(feature = "temp_cache")]
            temp_messages: temp_cache(settings.time_to_live),
            #[cfg(feature = "temp_cache")]
            temp_users: temp_cache(settings.time_to_live),

            channels: MaybeMap(settings.cache_channels.then(DashMap::default)),

            guilds: MaybeMap(settings.cache_guilds.then(DashMap::default)),
            unavailable_guilds: MaybeMap(settings.cache_guilds.then(DashMap::default)),

            messages: DashMap::default(),

            shard_data: RwLock::new(CachedShardData {
                total: NonZeroU16::MIN,
                connected: HashSet::new(),
                has_sent_shards_ready: false,
            }),
            user: RwLock::new(CurrentUser::default()),
            settings: RwLock::new(settings),
        }
    }

    /// Fetches the number of [`Member`]s that have not had data received.
    ///
    /// The important detail to note here is that this is the number of _member_s that have not had
    /// data received. A single [`User`] may have multiple associated member objects that have not
    /// been received.
    ///
    /// This can be used in combination with [`Shard::chunk_guild`], and can be used to determine
    /// how many members have not yet been received.
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn cache_ready(&self, ctx: &Context, _: &Vec<GuildId>) {
    ///         println!("{} unknown members", ctx.cache.unknown_members());
    ///     }
    /// }
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

    /// Fetches a vector of all [`Guild`]s' Ids that are stored in the cache.
    ///
    /// Note that if you are utilizing multiple [`Shard`]s, then the guilds retrieved over all
    /// shards are included in this count -- not just the current [`Context`]'s shard, if accessing
    /// from one.
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
    ///     async fn ready(&self, context: &Context, _: &Ready) {
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
        let unavailable_guilds = self.unavailable_guilds();

        let unavailable_guild_ids = unavailable_guilds.iter().map(|i| *i.key());

        self.guilds.iter().map(|i| *i.key()).chain(unavailable_guild_ids).collect()
    }

    /// Retrieves a [`GuildChannel`] from the cache based on the given Id.
    #[deprecated = "Use Cache::guild and Guild::channels instead"]
    pub fn channel(&self, id: ChannelId) -> Option<GuildChannelRef<'_>> {
        let guild_id = *self.channels.get(&id)?;
        let guild_ref = self.guilds.get(&guild_id)?;
        let channel = guild_ref.try_map(|g| g.channels.get(&id)).ok();
        if let Some(channel) = channel {
            return Some(CacheRef::from_mapped_ref(channel));
        }

        #[cfg(feature = "temp_cache")]
        {
            if let Some(channel) = self.temp_channels.get(&id) {
                return Some(CacheRef::from_arc(channel));
            }
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
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # let cache: serenity::cache::Cache = todo!();
    /// if let Some(messages_in_channel) = cache.channel_messages(ChannelId::new(7)) {
    ///     let messages_by_user: Vec<_> =
    ///         messages_in_channel.iter().filter(|m| m.author.id == 8).collect();
    /// }
    /// ```
    pub fn channel_messages(&self, channel_id: ChannelId) -> Option<ChannelMessagesRef<'_>> {
        self.messages.get(&channel_id).map(CacheRef::from_ref)
    }

    /// Gets a reference to a guild from the cache based on the given `id`.
    ///
    /// # Examples
    ///
    /// Retrieve a guild from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::cache::Cache;
    /// # use serenity::model::id::GuildId;
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(guild) = cache.guild(GuildId::new(7)) {
    ///     println!("Guild name: {}", guild.name);
    /// };
    /// ```
    pub fn guild(&self, id: GuildId) -> Option<GuildRef<'_>> {
        self.guilds.get(&id).map(CacheRef::from_ref)
    }

    /// Returns the number of cached guilds.
    pub fn guild_count(&self) -> usize {
        self.guilds.len()
    }

    /// Retrieves a [`Guild`]'s member from the cache based on the guild's and user's given Ids.
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
    /// let roles_len = {
    ///     let channel = match cache.channel(message.channel_id) {
    ///         Some(channel) => channel,
    ///         None => {
    ///             if let Err(why) = message.channel_id.say(http, "Error finding channel data").await {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///             return;
    ///         },
    ///     };
    ///
    ///     cache.member(channel.guild_id, message.author.id).map(|m| m.roles.len())
    /// };
    ///
    /// let message_res = if let Some(roles_len) = roles_len {
    ///     let msg = format!("You have {} roles", roles_len);
    ///     message.channel_id.say(&http, &msg).await
    /// } else {
    ///     message.channel_id.say(&http, "Error finding member data").await
    /// };
    ///
    /// if let Err(why) = message_res {
    ///     println!("Error sending message: {:?}", why);
    /// }
    /// # }
    /// ```
    ///
    /// [`EventHandler::message`]: crate::client::EventHandler::message
    /// [`members`]: crate::model::guild::Guild::members
    #[deprecated = "Use Cache::guild and Guild::members instead"]
    pub fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<MemberRef<'_>> {
        self.member_(guild_id, user_id)
    }

    fn member_(&self, guild_id: GuildId, user_id: UserId) -> Option<MemberRef<'_>> {
        let member = self.guilds.get(&guild_id)?.try_map(|g| g.members.get(&user_id)).ok()?;
        Some(CacheRef::from_mapped_ref(member))
    }

    #[deprecated = "Use Cache::guild and Guild::roles instead"]
    pub fn guild_roles(&self, guild_id: GuildId) -> Option<GuildRolesRef<'_>> {
        self.guild_roles_(guild_id)
    }

    fn guild_roles_(&self, guild_id: GuildId) -> Option<GuildRolesRef<'_>> {
        let roles = self.guilds.get(&guild_id)?.map(|g| &g.roles);
        Some(CacheRef::from_mapped_ref(roles))
    }

    /// This method clones and returns all unavailable guilds.
    pub fn unavailable_guilds(&self) -> ReadOnlyMapRef<'_, GuildId, ()> {
        self.unavailable_guilds.as_read_only()
    }

    /// This method returns all channels from a guild of with the given `guild_id`.
    #[deprecated = "Use Cache::guild and Guild::channels instead"]
    pub fn guild_channels(&self, guild_id: GuildId) -> Option<GuildChannelsRef<'_>> {
        self.guild_channels_(guild_id)
    }

    fn guild_channels_(&self, guild_id: GuildId) -> Option<GuildChannelsRef<'_>> {
        let channels = self.guilds.get(&guild_id)?.map(|g| &g.channels);
        Some(CacheRef::from_mapped_ref(channels))
    }

    /// Returns the number of guild channels in the cache.
    pub fn guild_channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of shards.
    pub fn shard_count(&self) -> NonZeroU16 {
        self.shard_data.read().total
    }

    /// Retrieves a [`Channel`]'s message from the cache based on the channel's and message's given
    /// Ids.
    ///
    /// **Note**: This will clone the entire message.
    ///
    /// # Examples
    ///
    /// Retrieving the message object from a channel, in a [`EventHandler::message`] context:
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
    pub fn message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<MessageRef<'_>> {
        #[cfg(feature = "temp_cache")]
        if let Some(message) = self.temp_messages.get(&message_id) {
            return Some(CacheRef::from_arc(message));
        }

        let messages = self.messages.get(&channel_id)?;
        let message =
            messages.try_map(|messages| messages.iter().find(|m| m.id == message_id)).ok()?;
        Some(CacheRef::from_mapped_ref(message))
    }

    /// Retrieves a [`Guild`]'s role by their Ids.
    ///
    /// **Note**: This will clone the entire role. Instead, retrieve the guild and retrieve from
    /// the guild's [`roles`] map to avoid this.
    ///
    /// # Examples
    ///
    /// Retrieve a role from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::{GuildId, RoleId};
    /// # use serenity::cache::Cache;
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache is in scope, e.g. via `Context`
    /// if let Some(role) = cache.role(GuildId::new(7), RoleId::new(77)) {
    ///     println!("Role with Id 77 is called {}", role.name);
    /// };
    /// ```
    ///
    /// [`Guild`]: crate::model::guild::Guild
    /// [`roles`]: crate::model::guild::Guild::roles
    #[deprecated = "Use Cache::guild and Guild::roles instead"]
    pub fn role(&self, guild_id: GuildId, role_id: RoleId) -> Option<GuildRoleRef<'_>> {
        let role = self.guilds.get(&guild_id)?.try_map(|g| g.roles.get(&role_id)).ok()?;
        Some(CacheRef::from_mapped_ref(role))
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
    pub fn settings(&self) -> SettingsRef<'_> {
        CacheRef::from_guard(self.settings.read())
    }

    /// Sets the maximum amount of messages per channel to cache.
    ///
    /// By default, no messages will be cached.
    pub fn set_max_messages(&self, max: usize) {
        self.settings.write().max_messages = max;
    }

    /// This method provides a reference to the user used by the bot.
    pub fn current_user(&self) -> CurrentUserRef<'_> {
        CacheRef::from_guard(self.user.read())
    }

    /// Returns a channel category matching the given ID
    #[deprecated = "Use Cache::guild, Guild::channels, and GuildChannel::kind"]
    pub fn category(&self, channel_id: ChannelId) -> Option<GuildChannelRef<'_>> {
        #[allow(deprecated)]
        let channel = self.channel(channel_id)?;
        if channel.kind == ChannelType::Category {
            Some(channel)
        } else {
            None
        }
    }

    /// Returns the parent category of the given channel ID.
    #[deprecated = "Use Cache::guild, Guild::channels, and GuildChannel::parent_id"]
    pub fn channel_category_id(&self, channel_id: ChannelId) -> Option<ChannelId> {
        #[allow(deprecated)]
        self.channel(channel_id)?.parent_id
    }

    /// Clones all channel categories in the given guild and returns them.
    pub fn guild_categories(&self, guild_id: GuildId) -> Option<HashMap<ChannelId, GuildChannel>> {
        let guild = self.guilds.get(&guild_id)?;
        Some(
            guild
                .channels
                .iter()
                .filter(|(_id, channel)| channel.kind == ChannelType::Category)
                .map(|(id, channel)| (*id, channel.clone()))
                .collect(),
        )
    }

    /// Inserts new messages into the message cache for a channel manually.
    ///
    /// This will keep the ordering of the message cache consistent, even if the message iterator
    /// contains randomly ordered messages, and respects the [`Settings::max_messages`] setting.
    pub(crate) fn fill_message_cache(
        &self,
        channel_id: ChannelId,
        new_messages: impl Iterator<Item = Message>,
    ) {
        let max_messages = self.settings().max_messages;
        if max_messages == 0 {
            // Early exit for common case of message cache being disabled.
            return;
        }

        let mut channel_messages = self.messages.entry(channel_id).or_default();

        // Fill up the existing cache
        channel_messages.extend(new_messages.take(max_messages));
        // Make sure the cache stays sorted to messages
        channel_messages.make_contiguous().sort_unstable_by_key(|m| m.id);
        // Get rid of the overflow at the front of the queue.
        let truncate_end_index = channel_messages.len().saturating_sub(max_messages);
        channel_messages.drain(..truncate_end_index);
    }

    /// Updates the cache with the update implementation for an event or other custom update
    /// implementation.
    ///
    /// Refer to the documentation for [`CacheUpdate`] for more information.
    ///
    /// # Examples
    ///
    /// Refer to the [`CacheUpdate` examples].
    ///
    /// [`CacheUpdate` examples]: CacheUpdate#examples
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, e)))]
    pub fn update<E: CacheUpdate>(&self, e: &mut E) -> Option<E::Output> {
        e.update(self)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new_with_settings(Settings::default())
    }
}

#[cfg(test)]
mod test {

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
            assert!(!channel.iter().any(|m| m.id == MessageId::new(3)));
        }

        let channel = GuildChannel {
            id: event.message.channel_id,
            guild_id: event.message.guild_id.unwrap(),
            ..Default::default()
        };

        // Add a channel delete event to the cache, the cached messages for that channel should now
        // be gone.
        let mut delete = ChannelDeleteEvent {
            channel: channel.clone(),
        };
        assert!(cache.update(&mut delete).is_some());
        assert!(!cache.messages.contains_key(&delete.channel.id));

        // Test deletion of a guild channel's message cache when a GuildDeleteEvent is received.
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

        // The guild existed in the cache, so the cache's guild is returned by the update.
        assert!(cache.update(&mut guild_delete).is_some());

        // Assert that the channel's message cache no longer exists.
        assert!(!cache.messages.contains_key(&ChannelId::new(2)));
    }
}
