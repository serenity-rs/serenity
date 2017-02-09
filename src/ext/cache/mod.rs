//! A cache of events received over a `Shard`, where storing at least some
//! data from the event is possible.
//!
//! This acts as a cache, to avoid making requests over the REST API through
//! the [`rest`] module where possible. All fields are public, and do not have
//! getters, to allow you more flexibility with the stored data. However, this
//! allows data to be "corrupted", and _may or may not_ cause misfunctions
//! within the library. Mutate data at your own discretion.
//!
//! A "globally available" instance of the Cache is available at
//! [`client::CACHE`]. This is the instance that is updated by the library,
//! meaning you should _not_ need to maintain updating it yourself in any case.
//!
//! # Use by the Context
//!
//! The [`Context`] will automatically attempt to pull from the cache for you.
//! For example, the [`Context::get_channel`] method will attempt to find the
//! channel in the cache. If it can not find it, it will perform a request
//! through the REST API, and then insert a clone of the channel - if found -
//! into the Cache, giving you the original.
//!
//! This allows you to save a step, by only needing to perform the
//! [`Context::get_channel`] call and not need to first search through the cache
//! - and if not found - _then_ perform an HTTP request through the Context or
//! [`rest`] module.
//!
//! Additionally, note that some information received through events can _not_
//! be retrieved through the REST API. This is information such as [`Role`]s in
//! [`LiveGuild`]s.
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
//! # Special cases in the Cache
//!
//! Some items in the cache, notably [`Call`]s and [`Group`]s, will "always be
//! empty". The exception to this rule, is for:
//!
//! 1. Bots which used to be userbots prior to the conversion made available by
//! Discord when the official Bot API was introduced;
//! 2. For groups and calls:
//! 2a. Bots that have friends from before the conversion that have not been
//! removed, as those users can still add the bots to groups;
//! 2b. Bots that have the "Create Group" endpoint whitelisted specifically for
//! them.
//!
//! [`Call`]: ../../model/struct.Call.html
//! [`Context`]: ../../client/struct.Context.html
//! [`Context::get_channel`]: ../../client/struct.Context.html#method.get_channel
//! [`Emoji`]: ../../model/struct.Emoji.html
//! [`Group`]: ../../model/struct.Group.html
//! [`LiveGuild`]: ../../model/struct.LiveGuild.html
//! [`LiveGuild::edit`]: ../../model/struct.LiveGuild.html#method.edit
//! [`Message`]: ../../model/struct.Message.html
//! [`PublicChannel`]: ../../model/struct.PublicChannel.html
//! [`Role`]: ../../model/struct.Role.html
//! [`client::CACHE`]: ../../client/struct.CACHE.html
//! [`rest`]: ../../client/rest/index.html

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::sync::{Arc, RwLock};
use std::mem;
use ::model::*;
use ::model::event::*;

/// A cache of all events received over a [`Connection`], where storing at least
/// some data from the event is possible.
///
/// This acts as a cache, to avoid making requests over the REST API through the
/// [`rest`] module where possible. All fields are public, and do not have
/// getters, to allow you more flexibility with the stored data. However, this
/// allows data to be "corrupted", and _may or may not_ cause misfunctions
/// within the library. Mutate data at your own discretion.
///
/// # Use by the Context
///
/// The [`Context`] will automatically attempt to pull from the cache for you.
/// For example, the [`Context::get_channel`] method will attempt to find the
/// channel in the cache. If it can not find it, it will perform a request
/// through the REST API, and then insert a clone of the channel - if found -
/// into the Cache.
///
/// This allows you to only need to perform the `Context::get_channel` call,
/// and not need to first search through the cache - and if not found - _then_
/// perform an HTTP request through the Context or `rest` module.
///
/// Additionally, note that some information received through events can _not_
/// be retrieved through the REST API. This is information such as [`Role`]s in
/// [`Guild`]s.
///
/// [`Connection`]: ../../client/struct.Connection.html
/// [`Context`]: ../../client/struct.Context.html
/// [`Context::get_channel`]: ../../client/struct.Context.html#method.get_channel
/// [`Guild`]: ../../model/struct.Guild.html
/// [`Role`]: ../../model/struct.Role.html
/// [`rest`]: ../../client/rest/index.html
#[derive(Clone, Debug)]
pub struct Cache {
    /// A map of the currently active calls that the current user knows about,
    /// where the key is the Id of the [`PrivateChannel`] or [`Group`] hosting
    /// the call.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [`Group`]: ../../model/struct.Group.html
    /// [`PrivateChannel`]: ../../model/struct.PrivateChannel.html
    /// [special cases]: index.html#special-cases-in-the-cache
    pub calls: HashMap<ChannelId, Arc<RwLock<Call>>>,
    /// A map of channels in [`Guild`]s that the current user has received data
    /// for.
    ///
    /// When a [`Event::GuildDelete`] or [`Event::GuildUnavailable`] is
    /// received and processed by the cache, the relevant channels are also
    /// removed from this map.
    ///
    /// [`Event::GuildDelete`]: ../../model/event/struct.GuildDeleteEvent.html
    /// [`Event::GuildUnavailable`]: ../../model/event/struct.GuildUnavailableEvent.html
    /// [`Guild`]: ../../model/struct.Guild.html
    pub channels: HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
    /// A map of the groups that the current user is in.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [special cases]: index.html#special-cases-in-the-cache
    pub groups: HashMap<ChannelId, Arc<RwLock<Group>>>,
    /// Settings specific to a guild.
    ///
    /// This will always be empty for bot users.
    pub guild_settings: HashMap<Option<GuildId>, UserGuildSettings>,
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    ///
    /// [`Emoji`]: ../../model/struct.Emoji.html
    /// [`Role`]: ../../model/struct.Role.html
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
    /// A map of relationships that the current user has with other users.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [special cases]: index.html#special-cases-in-the-cache
    pub relationships: HashMap<UserId, Relationship>,
    /// Account-specific settings for a user account.
    pub settings: Option<UserSettings>,
    /// A list of guilds which are "unavailable". Refer to the documentation for
    /// [`Event::GuildUnavailable`] for more information on when this can occur.
    ///
    /// Additionally, guilds are always unavailable for bot users when a Ready
    /// is received. Guilds are "sent in" over time through the receiving of
    /// [`Event::GuildCreate`]s.
    ///
    /// [`Event::GuildCreate`]: ../../model/enum.Event.html#variant.GuildCreate
    /// [`Event::GuildUnavailable`]: ../../model/enum.Event.html#variant.GuildUnavailable
    pub unavailable_guilds: HashSet<GuildId>,
    /// The current user "logged in" and for which events are being received
    /// for.
    ///
    /// The current user contains information that a regular [`User`] does not,
    /// such as whether it is a bot, whether the user is verified, etc.
    ///
    /// Refer to the documentation for [`CurrentUser`] for more information.
    ///
    /// [`CurrentUser`]: ../../model/struct.CurrentUser.html
    /// [`User`]: ../../model/struct.User.html
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
    /// - [`RelationshipAdd`][`RelationshipAddEvent`]
    ///
    /// Note, however, that users are _not_ removed from the map on removal
    /// events such as [`GuildMemberRemove`][`GuildMemberRemoveEvent`], as other
    /// structs such as members or recipients may still exist.
    ///
    /// [`ChannelRecipientAddEvent`]: ../../model/event/struct.ChannelRecipientAddEvent.html
    /// [`GuildMemberAddEvent`]: ../../model/event/struct.GuildMemberAddEvent.html
    /// [`GuildMemberRemoveEvent`]: ../../model/event/struct.GuildMemberRemoveEvent.html
    /// [`GuildMemberUpdateEvent`]: ../../model/event/struct.GuildMemberUpdateEvent.html
    /// [`GuildMembersChunkEvent`]: ../../model/event/struct.GuildMembersChunkEvent.html
    /// [`GuildSyncEvent`]: ../../model/event/struct.GuildSyncEvent.html
    /// [`PresenceUpdateEvent`]: ../../model/event/struct.PresenceUpdateEvent.html
    /// [`Ready`]: ../../model/event/struct.ReadyEvent.html
    /// [`RelationshipAdd`][`RelationshipAddEvent`]
    pub users: HashMap<UserId, Arc<RwLock<User>>>,
}

impl Cache {
    /// Fetches the number of [`Member`]s that have not had data received.
    ///
    /// The important detail to note here is that this is the number of
    /// _member_s that have not had data downloaded. A single [`User`] may have
    /// multiple associated member objects that have not been received.
    ///
    /// This can be used in combination with [`Shard::chunk_guilds`], and can be
    /// used to determine how many members have not yet been downloaded.
    ///
    /// [`Member`]: ../../model/struct.Member.html
    /// [`User`]: ../../model/struct.User.html
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
    /// [`Group`]: ../../model/struct.Group.html
    /// [`PrivateChannel`]: ../../model/struct.PrivateChannel.html
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
    /// [`Context`]: ../../client/struct.Context.html
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`Shard`]: ../../client/gateway/struct.Shard.html
    pub fn all_guilds(&self) -> Vec<GuildId> {
        self.guilds
            .values()
            .map(|g| g.read().unwrap().id)
            .chain(self.unavailable_guilds.iter().cloned())
            .collect()
    }

    /// Retrieves a reference to a [`Call`] from the cache based on the
    /// associated [`Group`]'s channel Id.
    ///
    /// [`Call`]: ../../model/struct.Call.html
    /// [`Group`]: ../../model/struct.Group.html
    #[inline]
    pub fn get_call<C: Into<ChannelId>>(&self, group_id: C) -> Option<Arc<RwLock<Call>>> {
        self.calls.get(&group_id.into()).cloned()
    }

    /// Retrieves a [`Channel`] from the cache based on the given Id.
    ///
    /// This will search the [`channels`] map, the [`private_channels`] map, and
    /// then the map of [`groups`] to find the channel.
    ///
    /// If you know what type of channel you're looking for, you should instead
    /// manually retrieve from one of the respective maps or methods:
    ///
    /// - [`GuildChannel`]: [`get_guild_channel`] or [`channels`]
    /// - [`PrivateChannel`]: [`get_private_channel`] or [`private_channels`]
    /// - [`Group`]: [`get_group`] or [`groups`]
    ///
    /// [`Channel`]: ../../model/enum.Channel.html
    /// [`Group`]: ../../model/struct.Group.html
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`channels`]: #structfield.channels
    /// [`get_group`]: #method.get_group
    /// [`get_guild_channel`]: #method.get_guild_channel
    /// [`get_private_channel`]: #method.get_private_channel
    /// [`groups`]: #structfield.groups
    /// [`private_channels`]: #structfield.private_channels
    pub fn get_channel<C: Into<ChannelId>>(&self, id: C) -> Option<Channel> {
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
    /// [`GuildId`]: ../../model/struct.GuildId.html
    #[inline]
    pub fn get_guild<G: Into<GuildId>>(&self, id: G) -> Option<Arc<RwLock<Guild>>> {
        self.guilds.get(&id.into()).cloned()
    }

    /// Retrieves a reference to a [`Guild`]'s channel. Unlike [`get_channel`],
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
    /// ```rust,ignore
    /// use serenity::cache::CACHE;
    ///
    /// let cache = CACHE.read().unwrap();
    ///
    /// let channel = match cache.get_guild_channel(message.channel_id) {
    ///     Some(channel) => channel,
    ///     None => {
    ///         if let Err(why) = context.say("Could not find guild's channel data") {
    ///             println!("Error sending message: {:?}", why);
    ///         }
    ///
    ///         return;
    ///     },
    /// };
    /// ```
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    /// [`Client::on_message`]: ../../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`get_channel`]: #method.get_channel
    #[inline]
    pub fn get_guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<GuildChannel>>> {
        self.channels.get(&id.into()).cloned()
    }

    /// Retrieves a reference to a [`Group`] from the cache based on the given
    /// associated channel Id.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    /// [`Group`]: ../../model/struct.Group.html
    #[inline]
    pub fn get_group<C: Into<ChannelId>>(&self, id: C) -> Option<Arc<RwLock<Group>>> {
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
    /// use serenity::client::CACHE;
    ///
    /// // assuming you are in a context
    ///
    /// let cache = CACHE.read().unwrap();
    /// let member = {
    ///     let channel = match cache.get_guild_channel(message.channel_id) {
    ///         Some(channel) => channel,
    ///         None => {
    ///             if let Err(why) = context.say("Error finding channel data") {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///         },
    ///     };
    ///
    ///     match cache.get_member(channel.guild_id, message.author.id) {
    ///         Some(member) => member,
    ///         None => {
    ///             if let Err(why) = context.say("Error finding member data") {
    ///                 println!("Error sending message: {:?}", why);
    ///             }
    ///         },
    ///     }
    /// };
    ///
    /// let msg = format!("You have {} roles", member.roles.len());
    ///
    /// if let Err(why) = context.say(&msg) {
    ///     println!("Error sending message: {:?}", why);
    /// }
    /// ```
    ///
    /// [`Client::on_message`]: ../../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`members`]: ../../model/struct.Guild.html#structfield.members
    pub fn get_member<G, U>(&self, guild_id: G, user_id: U) -> Option<Member>
        where G: Into<GuildId>, U: Into<UserId> {
        self.guilds
            .get(&guild_id.into())
            .and_then(|guild| guild.write().unwrap().members.get(&user_id.into()).cloned())
    }

    /// Retrieves a [`PrivateChannel`] from the cache's [`private_channels`]
    /// map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`ChannelId`].
    #[inline]
    pub fn get_private_channel<C: Into<ChannelId>>(&self, channel_id: C) -> Option<Arc<RwLock<PrivateChannel>>> {
        self.private_channels.get(&channel_id.into()).cloned()
    }

    /// Retrieves a [`Guild`]'s role by their Ids.
    ///
    /// **Note**: This will clone the entire role. Instead, retrieve the guild
    /// and retrieve from the guild's [`roles`] map to avoid this.
    ///
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`roles`]: ../../model/struct.Guild.html#structfield.roles
    pub fn get_role<G, R>(&self, guild_id: G, role_id: R) -> Option<Role>
        where G: Into<GuildId>, R: Into<RoleId>
    {
        self.guilds
            .get(&guild_id.into())
            .and_then(|g| g.read().unwrap().roles.get(&role_id.into()).cloned())
    }

    /// Retrieves a `User` from the cache's [`users`] map, if it exists.
    ///
    /// The only advantage of this method is that you can pass in anything that
    /// is indirectly a [`UserId`].
    ///
    /// [`UserId`]: ../../model/struct.UserId.html
    /// [`users`]: #structfield.users
    #[inline]
    pub fn get_user<U: Into<UserId>>(&self, user_id: U) -> Option<Arc<RwLock<User>>> {
        self.users.get(&user_id.into()).cloned()
    }

    #[doc(hidden)]
    pub fn update_with_call_create(&mut self, event: &CallCreateEvent) {
        match self.calls.entry(event.call.channel_id) {
            Entry::Vacant(e) => {
                e.insert(Arc::new(RwLock::new(event.call.clone())));
            }
            Entry::Occupied(mut e) => {
                *e.get_mut() = Arc::new(RwLock::new(event.call.clone()));
            }
        }
    }

    #[doc(hidden)]
    pub fn update_with_call_delete(&mut self, event: &CallDeleteEvent) -> Option<Arc<RwLock<Call>>> {
        self.calls.remove(&event.channel_id)
    }

    #[doc(hidden)]
    pub fn update_with_call_update(&mut self, event: &CallUpdateEvent, old: bool) -> Option<Arc<RwLock<Call>>> {
        let item = if old {
            self.calls.get(&event.channel_id).cloned()
        } else {
            None
        };

        self.calls
            .get_mut(&event.channel_id)
            .map(|call| {
                let mut call = call.write().unwrap();

                call.region.clone_from(&event.region);
                call.ringing.clone_from(&event.ringing);
            });

        item
    }

    #[doc(hidden)]
    pub fn update_with_channel_create(&mut self, event: &ChannelCreateEvent) -> Option<Channel> {
        match event.channel {
            Channel::Group(ref group) => {
                let group = group.clone();

                let channel_id = {
                    let writer = group.write().unwrap();

                    for (recipient_id, recipient) in &mut group.write().unwrap().recipients {
                        self.update_user_entry(&recipient.read().unwrap());

                        *recipient = self.users[recipient_id].clone();
                    }

                    writer.channel_id
                };

                let ch = self.groups.insert(channel_id, group);

                ch.map(Channel::Group)
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = {
                    let channel = channel.read().unwrap();

                    (channel.guild_id, channel.id)
                };

                self.channels.insert(channel_id, channel.clone());

                self.guilds
                    .get_mut(&guild_id)
                    .and_then(|guild| {
                        guild.write().unwrap().channels.insert(channel_id, channel.clone())
                    })
            },
            Channel::Private(ref channel) => {
                let channel = channel.clone();

                let mut channel_writer = channel.write().unwrap();

                let user_id = {
                    let user_reader = channel_writer.recipient.read().unwrap();

                    self.update_user_entry(&user_reader);

                    user_reader.id
                };

                channel_writer.recipient = self.users[&user_id].clone();

                let ch = self.private_channels.insert(channel.read().unwrap().id, channel.clone());
                ch.map(Channel::Private)
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_delete(&mut self, event: &ChannelDeleteEvent) -> Option<Channel> {
        match event.channel {
            Channel::Group(ref group) => {
                self.groups.remove(&group.read().unwrap().channel_id).map(Channel::Group)
            },
            Channel::Private(ref channel) => {
                self.private_channels.remove(&channel.read().unwrap().id).map(Channel::Private)
            },
            Channel::Guild(ref channel) => {
                let (channel_id, guild_id) = {
                    let channel = channel.read().unwrap();

                    (channel.id, channel.guild_id)
                };

                self.channels.remove(&channel_id);

                self.guilds
                    .get_mut(&guild_id)
                    .and_then(|guild| guild.write().unwrap().channels.remove(&channel_id))
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_pins_update(&mut self, event: &ChannelPinsUpdateEvent) {
        if let Some(channel) = self.channels.get(&event.channel_id) {
            channel.write().unwrap().last_pin_timestamp = event.last_pin_timestamp.clone();

            return;
        }

        if let Some(channel) = self.private_channels.get_mut(&event.channel_id) {
            channel.write().unwrap().last_pin_timestamp = event.last_pin_timestamp.clone();

            return;
        }

        if let Some(group) = self.groups.get_mut(&event.channel_id) {
            group.write().unwrap().last_pin_timestamp = event.last_pin_timestamp.clone();

            return;
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_recipient_add(&mut self, event: &mut ChannelRecipientAddEvent) {
        self.update_user_entry(&event.user);
        let user = self.users[&event.user.id].clone();

        self.groups
            .get_mut(&event.channel_id)
            .map(|group| {
                group.write()
                    .unwrap()
                    .recipients
                    .insert(event.user.id, user);
            });
    }

    #[doc(hidden)]
    pub fn update_with_channel_recipient_remove(&mut self, event: &ChannelRecipientRemoveEvent) {
        self.groups
            .get_mut(&event.channel_id)
            .map(|group| group.write().unwrap().recipients.remove(&event.user.id));
    }

    #[doc(hidden)]
    pub fn update_with_channel_update(&mut self, event: &ChannelUpdateEvent) {
        match event.channel {
            Channel::Group(ref group) => {
                let (ch_id, no_recipients) = {
                    let group = group.read().unwrap();

                    (group.channel_id, group.recipients.is_empty())
                };

                match self.groups.entry(ch_id) {
                    Entry::Vacant(e) => {
                        e.insert(group.clone());
                    }
                    Entry::Occupied(mut e) => {
                        let mut dest = e.get_mut().write().unwrap();

                        if no_recipients {
                            let recipients = mem::replace(&mut dest.recipients, HashMap::new());

                            dest.clone_from(&group.read().unwrap());

                            dest.recipients = recipients;
                        } else {
                            dest.clone_from(&group.read().unwrap());
                        }
                    }
                }
            },
            Channel::Guild(ref channel) => {
                let (channel_id, guild_id) = {
                    let channel = channel.read().unwrap();

                    (channel.id, channel.guild_id)
                };

                self.channels.insert(channel_id, channel.clone());
                self.guilds
                    .get_mut(&guild_id)
                    .map(|guild| {
                        guild.write()
                            .unwrap()
                            .channels
                            .insert(channel_id, channel.clone())
                    });
            },
            Channel::Private(ref channel) => {
                self.private_channels
                    .get_mut(&channel.read().unwrap().id)
                    .map(|private| private.clone_from(channel));
            }
        },
    }

    #[doc(hidden)]
    pub fn update_with_guild_create(&mut self, event: &GuildCreateEvent) {
        self.unavailable_guilds.remove(&event.guild.id);

        let mut guild = event.guild.clone();

        for (user_id, member) in &mut guild.members {
            self.update_user_entry(&member.user.read().unwrap());
            let user = self.users[user_id].clone();

            member.user = user.clone();
        }

        self.channels.extend(guild.channels.clone());
        self.guilds.insert(event.guild.id, Arc::new(RwLock::new(guild)));
    }

    #[doc(hidden)]
    pub fn update_with_guild_delete(&mut self, event: &GuildDeleteEvent) -> Option<Arc<RwLock<Guild>>> {
        // Remove channel entries for the guild if the guild is found.
        self.guilds.remove(&event.guild.id).map(|guild| {
            for channel_id in guild.read().unwrap().channels.keys() {
                self.channels.remove(channel_id);
            }

            guild
        })
    }

    #[doc(hidden)]
    pub fn update_with_guild_emojis_update(&mut self, event: &GuildEmojisUpdateEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.write().unwrap().emojis.extend(event.emojis.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_add(&mut self, event: &mut GuildMemberAddEvent) {
        let user_id = event.member.user.read().unwrap().id;
        self.update_user_entry(&event.member.user.read().unwrap());

        // Always safe due to being inserted above.
        event.member.user = self.users[&user_id].clone();

        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| {
                let mut guild = guild.write().unwrap();

                guild.member_count += 1;
                guild.members.insert(user_id, event.member.clone());
            });
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_remove(&mut self, event: &GuildMemberRemoveEvent) -> Option<Member> {
        self.guilds
            .get_mut(&event.guild_id)
            .and_then(|guild| {
                let mut guild = guild.write().unwrap();

                guild.member_count -= 1;
                guild.members.remove(&event.user.id)
            })
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_update(&mut self, event: &GuildMemberUpdateEvent) -> Option<Member> {
        self.update_user_entry(&event.user);

        if let Some(guild) = self.guilds.get_mut(&event.guild_id) {
            let mut guild = guild.write().unwrap();

            let mut found = false;

            let item = if let Some(member) = guild.members.get_mut(&event.user.id) {
                let item = Some(member.clone());

                member.nick.clone_from(&event.nick);
                member.roles.clone_from(&event.roles);
                member.user.write().unwrap().clone_from(&event.user);

                found = true;

                item
            } else {
                None
            };

            if !found {
                guild.members.insert(event.user.id, Member {
                    deaf: false,
                    joined_at: String::default(),
                    mute: false,
                    nick: event.nick.clone(),
                    roles: event.roles.clone(),
                    user: Arc::new(RwLock::new(event.user.clone())),
                });
            }

            item
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_members_chunk(&mut self, event: &GuildMembersChunkEvent) {
        for member in event.members.values() {
            self.update_user_entry(&member.user.read().unwrap());
        }

        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.write().unwrap().members.extend(event.members.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_create(&mut self, event: &GuildRoleCreateEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.write().unwrap().roles.insert(event.role.id, event.role.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_delete(&mut self, event: &GuildRoleDeleteEvent) -> Option<Role> {
        self.guilds
            .get_mut(&event.guild_id)
            .and_then(|guild| guild.write().unwrap().roles.remove(&event.role_id))
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_update(&mut self, event: &GuildRoleUpdateEvent) -> Option<Role> {
        self.guilds
            .get_mut(&event.guild_id)
            .and_then(|guild| {
                guild.write()
                    .unwrap()
                    .roles
                    .get_mut(&event.role.id)
                    .map(|role| mem::replace(role, event.role.clone()))
            })
    }

    #[doc(hidden)]
    pub fn update_with_guild_sync(&mut self, event: &GuildSyncEvent) {
        for member in event.members.values() {
            self.update_user_entry(&member.user.read().unwrap());
        }

        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| {
                let mut guild = guild.write().unwrap();

                guild.large = event.large;
                guild.members.clone_from(&event.members);
                guild.presences.clone_from(&event.presences);
            });
    }

    #[doc(hidden)]
    pub fn update_with_guild_unavailable(&mut self, event: &GuildUnavailableEvent) {
        self.unavailable_guilds.insert(event.guild_id);
        self.guilds.remove(&event.guild_id);
    }

    #[doc(hidden)]
    pub fn update_with_guild_update(&mut self, event: &GuildUpdateEvent) {
        self.guilds
            .get_mut(&event.guild.id)
            .map(|guild| {
                let mut guild = guild.write().unwrap();

                guild.afk_timeout = event.guild.afk_timeout;
                guild.afk_channel_id.clone_from(&event.guild.afk_channel_id);
                guild.icon.clone_from(&event.guild.icon);
                guild.name.clone_from(&event.guild.name);
                guild.owner_id.clone_from(&event.guild.owner_id);
                guild.region.clone_from(&event.guild.region);
                guild.roles.clone_from(&event.guild.roles);
                guild.verification_level = event.guild.verification_level;
            });
    }

    #[doc(hidden)]
    pub fn update_with_presences_replace(&mut self, event: &PresencesReplaceEvent) {
        self.presences.extend({
            let mut p: HashMap<UserId, Presence> = HashMap::default();

            for presence in &event.presences {
                p.insert(presence.user_id, presence.clone());
            }

            p
        });
    }

    #[doc(hidden)]
    pub fn update_with_presence_update(&mut self, event: &mut PresenceUpdateEvent) {
        let user_id = event.presence.user_id;

        if let Some(user) = event.presence.user.as_mut() {
            self.update_user_entry(&user.read().unwrap());
            *user = self.users[&user_id].clone();
        }

        if let Some(guild_id) = event.guild_id {
            if let Some(guild) = self.guilds.get_mut(&guild_id) {
                let mut guild = guild.write().unwrap();

                // If the member went offline, remove them from the presence list.
                if event.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&event.presence.user_id);
                } else {
                    guild.presences.insert(event.presence.user_id, event.presence.clone());
                }
            }
        } else if event.presence.status != OnlineStatus::Offline {
            self.presences.insert(event.presence.user_id, event.presence.clone());
        } else {
            self.presences.remove(&event.presence.user_id);
        }
    }

    #[doc(hidden)]
    pub fn update_with_ready(&mut self, event: &ReadyEvent) {
        let mut ready = event.ready.clone();

        for guild in ready.guilds {
            match guild {
                PossibleGuild::Offline(guild_id) => {
                    self.unavailable_guilds.insert(guild_id);
                    self.guilds.remove(&guild_id);
                },
                PossibleGuild::Online(guild) => {
                    self.channels.extend(guild.channels.clone());
                    self.guilds.insert(guild.id, Arc::new(RwLock::new(guild)));
                },
            }
        }

        // The private channels sent in the READY contains both the actual
        // private channels and the groups.
        for (channel_id, channel) in ready.private_channels {
            match channel {
                Channel::Group(group) => {
                    self.groups.insert(channel_id, group);
                }
                Channel::Private(channel) => {
                    self.private_channels.insert(channel_id, channel);
                }
                Channel::Guild(guild) => warn!("Got a guild in DMs: {:?}", guild),
            }
        }

        if let Some(user_guild_settings) = ready.user_guild_settings {
            for guild in user_guild_settings {
                self.guild_settings.insert(guild.guild_id, guild);
            }
        }

        for (user_id, presence) in &mut ready.presences {
            if let Some(ref user) = presence.user {
                self.update_user_entry(&user.read().unwrap());
            }

            presence.user = self.users.get(user_id).cloned();
        }

        self.presences.extend(ready.presences);
        self.relationships.extend(ready.relationships);
        self.notes.extend(ready.notes);
        self.settings = ready.user_settings;
        self.user = ready.user;
    }

    #[doc(hidden)]
    pub fn update_with_relationship_add(&mut self, event: &RelationshipAddEvent) {
        self.update_user_entry(&event.relationship.user);

        self.relationships.insert(event.relationship.id, event.relationship.clone());
    }

    #[doc(hidden)]
    pub fn update_with_relationship_remove(&mut self, event: &RelationshipRemoveEvent) {
        self.relationships.remove(&event.user_id);
    }

    #[doc(hidden)]
    pub fn update_with_user_guild_settings_update(&mut self, event: &UserGuildSettingsUpdateEvent) -> Option<UserGuildSettings> {
        self.guild_settings
            .get_mut(&event.settings.guild_id)
            .map(|guild_setting| mem::replace(guild_setting, event.settings.clone()))
    }

    #[doc(hidden)]
    pub fn update_with_user_note_update(&mut self, event: &UserNoteUpdateEvent) -> Option<String> {
        if event.note.is_empty() {
            self.notes.remove(&event.user_id)
        } else {
            self.notes.insert(event.user_id, event.note.clone())
        }
    }

    #[doc(hidden)]
    pub fn update_with_user_settings_update(&mut self, event: &UserSettingsUpdateEvent, old: bool) -> Option<UserSettings> {
        let item = if old { self.settings.clone() } else { None };

        self.settings
            .as_mut()
            .map(|settings| {
                opt_modify(&mut settings.enable_tts_command, &event.enable_tts_command);
                opt_modify(&mut settings.inline_attachment_media, &event.inline_attachment_media);
                opt_modify(&mut settings.inline_embed_media, &event.inline_embed_media);
                opt_modify(&mut settings.locale, &event.locale);
                opt_modify(&mut settings.message_display_compact, &event.message_display_compact);
                opt_modify(&mut settings.render_embeds, &event.render_embeds);
                opt_modify(&mut settings.show_current_game, &event.show_current_game);
                opt_modify(&mut settings.theme, &event.theme);
                opt_modify(&mut settings.convert_emoticons, &event.convert_emoticons);
                opt_modify(&mut settings.friend_source_flags, &event.friend_source_flags);
            });

        item
    }

    #[doc(hidden)]
    pub fn update_with_user_update(&mut self, event: &UserUpdateEvent) -> CurrentUser {
        mem::replace(&mut self.user, event.current_user.clone())
    }

    #[doc(hidden)]
    pub fn update_with_voice_state_update(&mut self, event: &VoiceStateUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            if let Some(guild) = self.guilds.get_mut(&guild_id) {
                let mut guild = guild.write().unwrap();

                if event.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    {
                        let finding = guild.voice_states.get_mut(&event.voice_state.user_id);

                        if let Some(srv_state) = finding {
                            srv_state.clone_from(&event.voice_state);

                            return;
                        }
                    }

                    guild.voice_states.insert(event.voice_state.user_id, event.voice_state.clone());
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&event.voice_state.user_id);
                }
            }

            return;
        }

        if let Some(channel) = event.voice_state.channel_id {
            // channel id available, insert voice state
            if let Some(call) = self.calls.get_mut(&channel) {
                let mut call = call.write().unwrap();

                {
                    let finding = call.voice_states
                        .get_mut(&event.voice_state.user_id);

                    if let Some(group_state) = finding {
                        group_state.clone_from(&event.voice_state);

                        return;
                    }
                }

                call.voice_states.insert(event.voice_state.user_id, event.voice_state.clone());
            }
        } else {
            // delete this user from any group call containing them
            for call in self.calls.values_mut() {
                call.write().unwrap().voice_states.remove(&event.voice_state.user_id);
            }
        }
    }

    // Adds or updates a user entry in the [`users`] map with a received user.
    //
    // [`users`]: #structfield.users
    fn update_user_entry(&mut self, user: &User) {
        match self.users.entry(user.id) {
            Entry::Vacant(e) => {
                e.insert(Arc::new(RwLock::new(user.clone())));
            }
            Entry::Occupied(mut e) => {
                e.get_mut().write().unwrap().clone_from(user);
            }
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            calls: HashMap::default(),
            channels: HashMap::default(),
            groups: HashMap::default(),
            guild_settings: HashMap::default(),
            guilds: HashMap::default(),
            notes: HashMap::default(),
            presences: HashMap::default(),
            private_channels: HashMap::default(),
            relationships: HashMap::default(),
            settings: None,
            unavailable_guilds: HashSet::default(),
            user: CurrentUser {
                avatar: None,
                bot: false,
                discriminator: 0,
                email: None,
                id: UserId(0),
                mfa_enabled: false,
                mobile: None,
                name: String::default(),
                verified: false,
            },
            users: HashMap::default(),
        }
    }
}

fn opt_modify<T: Clone>(dest: &mut T, src: &Option<T>) {
    if let Some(val) = src.as_ref() {
        dest.clone_from(val);
    }
}
