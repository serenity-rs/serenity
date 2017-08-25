//! A cache of events received over a `Shard`, where storing at least some
//! data from the event is possible.
//!
//! This acts as a cache, to avoid making requests over the REST API through
//! the [`http`] module where possible. All fields are public, and do not have
//! getters, to allow you more flexibility with the stored data. However, this
//! allows data to be "corrupted", and _may or may not_ cause misfunctions
//! within the library. Mutate data at your own discretion.
//!
//! A "globally available" instance of the Cache is available at
//! [`client::CACHE`]. This is the instance that is updated by the library,
//! meaning you should _not_ need to maintain updating it yourself in any case.
//!
//! # Use by Models
//!
//! Most models of Discord objects, such as the [`Message`], [`PublicChannel`],
//! or [`Emoji`], have methods for interacting with that single instance. This
//! feature is only compiled if the `methods` feature is enabled. An example of
//! this is [`LiveGuild::edit`], which performs a check to ensure that the
//! current user is the owner of the guild, prior to actually performing the
//! HTTP request. The cache is involved due to the function's use of unlocking
//! the cache and retrieving the Id of the current user, and comparing it to
//! the Id of the user that owns the guild. This is an inexpensive method of
//! being able to access data required by these sugary methods.
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
//! [`Emoji`]: ../model/struct.Emoji.html
//! [`Group`]: ../model/struct.Group.html
//! [`LiveGuild`]: ../model/struct.LiveGuild.html
//! [`LiveGuild::edit`]: ../model/struct.LiveGuild.html#method.edit
//! [`Message`]: ../model/struct.Message.html
//! [`PublicChannel`]: ../model/struct.PublicChannel.html
//! [`Role`]: ../model/struct.Role.html
//! [`CACHE`]: ../struct.CACHE.html
//! [`http`]: ../http/index.html
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::sync::{Arc, RwLock};
use model::*;

mod cache_events_impl;

pub(crate) use self::cache_events_impl::*;

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
    /// [`Guild`]: ../model/struct.Guild.html
    pub channels: HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
    /// A map of the groups that the current user is in.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [special cases]: index.html#special-cases-in-the-cache
    pub groups: HashMap<ChannelId, Arc<RwLock<Group>>>,
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    ///
    /// [`Emoji`]: ../model/struct.Emoji.html
    /// [`Role`]: ../model/struct.Role.html
    pub guilds: HashMap<GuildId, Arc<RwLock<Guild>>>,
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
    /// [`Event::GuildCreate`]: ../model/enum.Event.html#variant.GuildCreate
    /// [`Event::GuildUnavailable`]: ../model/enum.Event.html#variant.GuildUnavailable
    pub unavailable_guilds: HashSet<GuildId>,
    /// The current user "logged in" and for which events are being received
    /// for.
    ///
    /// The current user contains information that a regular [`User`] does not,
    /// such as whether it is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    ///
    /// [`CurrentUser`]: ../model/struct.CurrentUser.html
    /// [`User`]: ../model/struct.User.html
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
    /// - [`GuildSync`][`GuildSyncEvent`]
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
    /// [`GuildSyncEvent`]: ../model/event/struct.GuildSyncEvent.html
    /// [`PresenceUpdateEvent`]: ../model/event/struct.PresenceUpdateEvent.html
    /// [`ReadyEvent`]: ../model/event/struct.ReadyEvent.html
    pub users: HashMap<UserId, Arc<RwLock<User>>>,
}

impl Cache {
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
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// use serenity::client::CACHE;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, ctx: Context, _: Ready) {
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
    ///         println!("{} unknown members", CACHE.read().unwrap().unknown_members());
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Member`]: ../model/struct.Member.html
    /// [`Shard::chunk_guilds`]: ../gateway/struct.Shard.html#method.chunk_guilds
    /// [`User`]: ../model/struct.User.html
    pub fn unknown_members(&self) -> u64 {
        let mut total = 0;

        for guild in self.guilds.values() {
            let guild = guild.read().unwrap();

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
    /// use serenity::client::CACHE;
    ///
    /// let amount = CACHE.read().unwrap().all_private_channels().len();
    ///
    /// println!("There are {} private channels", amount);
    /// ```
    ///
    /// [`Group`]: ../model/struct.Group.html
    /// [`PrivateChannel`]: ../model/struct.PrivateChannel.html
    pub fn all_private_channels(&self) -> Vec<ChannelId> {
        self.groups
            .keys()
            .cloned()
            .chain(self.private_channels.keys().cloned())
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
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// use serenity::client::CACHE;
    ///
    /// struct Handler;
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, _: Context, _: Ready) {
    ///         println!("Guilds in the Cache: {:?}", CACHE.read().unwrap().all_guilds());
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler);
    /// ```
    ///
    /// [`Context`]: ../client/struct.Context.html
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`Shard`]: ../gateway/struct.Shard.html
    pub fn all_guilds(&self) -> Vec<GuildId> {
        self.guilds
            .keys()
            .cloned()
            .chain(self.unavailable_guilds.iter().cloned())
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
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Group`]: ../model/struct.Group.html
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`channels`]: #structfield.channels
    /// [`group`]: #method.group
    /// [`guild_channel`]: #method.guild_channel
    /// [`private_channel`]: #method.private_channel
    /// [`groups`]: #structfield.groups
    /// [`private_channels`]: #structfield.private_channels
    pub fn channel<C: Into<ChannelId>>(&self, id: C) -> Option<Channel> {
        let id = id.into();

        if let Some(channel) = self.channels.get(&id) {
            return Some(Channel::Guild(channel.clone()));
        }

        if let Some(private_channel) = self.private_channels.get(&id) {
            return Some(Channel::Private(private_channel.clone()));
        }

        if let Some(group) = self.groups.get(&id) {
            return Some(Channel::Group(group.clone()));
        }

        None
    }

    /// Retrieves a guild from the cache based on the given Id.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`GuildId`].
    ///
    /// [`GuildId`]: ../model/struct.GuildId.html
    ///
    /// # Examples
    ///
    /// Retrieve a guild from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::CACHE;
    ///
    /// let cache = CACHE.read()?;
    ///
    /// if let Some(guild) = cache.guild(7) {
    ///     println!("Guild name: {}", guild.read().unwrap().name);
    /// }
    /// #   Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn guild<G: Into<GuildId>>(&self, id: G) -> Option<Arc<RwLock<Guild>>> {
        self.guilds.get(&id.into()).cloned()
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
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// use serenity::client::CACHE;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, message: Message) {
    ///         let cache = CACHE.read().unwrap();
    ///
    ///         let channel = match cache.guild_channel(message.channel_id) {
    ///             Some(channel) => channel,
    ///             None => {
    /// if let Err(why) = message.channel_id.say("Could not find guild's
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
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`Client::on_message`]: ../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`channel`]: #method.channel
    #[inline]
    pub fn guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<GuildChannel>>> {
        self.channels.get(&id.into()).cloned()
    }

    /// Retrieves a reference to a [`Group`] from the cache based on the given
    /// associated channel Id.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`Group`]: ../model/struct.Group.html
    ///
    /// # Examples
    ///
    /// Retrieve a group from the cache and print its owner's id:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::CACHE;
    ///
    /// let cache = CACHE.read()?;
    ///
    /// if let Some(group) = cache.group(7) {
    ///     println!("Owner Id: {}", group.read().unwrap().owner_id);
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn group<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<Group>>> {
        self.groups.get(&id.into()).cloned()
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
    /// use serenity::CACHE;
    ///
    /// let cache = CACHE.read().unwrap();
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
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`members`]: ../model/struct.Guild.html#structfield.members
    pub fn member<G, U>(&self, guild_id: G, user_id: U) -> Option<Member>
        where G: Into<GuildId>, U: Into<UserId> {
        self.guilds.get(&guild_id.into()).and_then(|guild| {
            guild.read().unwrap().members.get(&user_id.into()).cloned()
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
    /// Retrieve a private channel from the cache and send a message:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::CACHE;
    ///
    /// let cache = CACHE.read()?;
    ///
    /// if let Some(channel) = cache.private_channel(7) {
    ///     channel.read().unwrap().say("Hello there!");
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn private_channel<C: Into<ChannelId>>(&self,
                                               channel_id: C)
                                               -> Option<Arc<RwLock<PrivateChannel>>> {
        self.private_channels.get(&channel_id.into()).cloned()
    }

    /// Retrieves a [`Guild`]'s role by their Ids.
    ///
    /// **Note**: This will clone the entire role. Instead, retrieve the guild
    /// and retrieve from the guild's [`roles`] map to avoid this.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`roles`]: ../model/struct.Guild.html#structfield.roles
    ///
    /// # Examples
    ///
    /// Retrieve a role from the cache and print its name:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::CACHE;
    ///
    /// let cache = CACHE.read()?;
    ///
    /// if let Some(role) = cache.role(7, 77) {
    ///     println!("Role with Id 77 is called {}", role.name);
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #   try_main().unwrap();
    /// # }
    /// ```
    pub fn role<G, R>(&self, guild_id: G, role_id: R) -> Option<Role>
        where G: Into<GuildId>, R: Into<RoleId> {
        self.guilds.get(&guild_id.into()).and_then(|g| {
            g.read().unwrap().roles.get(&role_id.into()).cloned()
        })
    }

    /// Retrieves a `User` from the cache's [`users`] map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`UserId`].
    ///
    /// [`UserId`]: ../model/struct.UserId.html
    /// [`users`]: #structfield.users
    ///
    /// # Examples
    ///
    /// Retrieve a user from the cache and print their name:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// use serenity::client::CACHE;
    ///
    /// let cache = CACHE.read()?;
    ///
    /// if let Some(user) = cache.user(7) {
    ///     println!("User with Id 7 is currently named {}", user.read().unwrap().name);
    /// }
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn user<U: Into<UserId>>(&self, user_id: U) -> Option<Arc<RwLock<User>>> {
        self.users.get(&user_id.into()).cloned()
    }

    fn update_user_entry(&mut self, user: &User) {
        match self.users.entry(user.id) {
            Entry::Vacant(e) => {
                e.insert(Arc::new(RwLock::new(user.clone())));
            },
            Entry::Occupied(mut e) => {
                e.get_mut().write().unwrap().clone_from(user);
            },
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            channels: HashMap::default(),
            groups: HashMap::with_capacity(128),
            guilds: HashMap::default(),
            notes: HashMap::default(),
            presences: HashMap::default(),
            private_channels: HashMap::with_capacity(128),
            shard_count: 1,
            unavailable_guilds: HashSet::default(),
            user: CurrentUser {
                avatar: None,
                bot: false,
                discriminator: 0,
                email: None,
                id: UserId(0),
                mfa_enabled: false,
                name: String::default(),
                verified: false,
            },
            users: HashMap::default(),
        }
    }
}
