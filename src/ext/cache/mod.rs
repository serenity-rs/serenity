//! A cache of events received over a [`Shard`], where storing at least some
//! data from the event is possible.
//!
//! This acts as a hot cache, to avoid making requests over the REST API through
//! the [`http`] module where possible. All fields are public, and do not have
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
//! [`http`] module.
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
//! [`Shard`]: ../../client/gateway/struct.Shard.html
//! [`client::CACHE`]: ../../client/struct.CACHE.html
//! [`http`]: ../../client/http/index.html

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::default::Default;
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
#[derive(Debug, Clone)]
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
    pub calls: HashMap<ChannelId, Call>,
    /// A map of the groups that the current user is in.
    ///
    /// For bot users this will always be empty, except for in [special cases].
    ///
    /// [special cases]: index.html#special-cases-in-the-cache
    pub groups: HashMap<ChannelId, Group>,
    /// Settings specific to a guild.
    ///
    /// This will always be empty for bot users.
    pub guild_settings: HashMap<Option<GuildId>, UserGuildSettings>,
    /// A map of guilds with full data available. This includes data like
    /// [`Role`]s and [`Emoji`]s that are not available through the REST API.
    ///
    /// [`Emoji`]: ../../model/struct.Emoji.html
    /// [`Role`]: ../../model/struct.Role.html
    pub guilds: HashMap<GuildId, Guild>,
    /// A map of notes that a user has made for individual users.
    ///
    /// An empty note is equivilant to having no note, and creating an empty
    /// note is equivilant to deleting a note.
    ///
    /// This will always be empty for bot users.
    pub notes: HashMap<UserId, String>,
    /// A map of users' presences. This is updated in real-time. Note that
    /// status updates are often "eaten" by the gateway, and this should not
    /// be treated as being entirely 100% accurate.
    pub presences: HashMap<UserId, Presence>,
    /// A map of direct message channels that the current user has open with
    /// other users.
    pub private_channels: HashMap<ChannelId, PrivateChannel>,
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
    pub unavailable_guilds: Vec<GuildId>,
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
}

impl Cache {
    /// Calculates the number of [`Member`]s that have not had data received.
    ///
    /// The important detail to note here is that this is the number of
    /// _member_s that have not had data downloaded. A single [`User`] may have
    /// multiple associated member objects that have not been received.
    ///
    /// This can be used in combination with [`Shard::sync_guilds`], and can be
    /// used to determine how many members have not yet been downloaded.
    ///
    /// [`Member`]: ../../model/struct.Member.html
    /// [`Shard::sync_guilds`]: ../../client/gateway/struct.Shard.html#method.sync_guilds
    /// [`User`]: ../../model/struct.User.html
    pub fn unknown_members(&self) -> u64 {
        let mut total = 0;

        for guild in self.guilds.values() {
            let members = guild.members.len() as u64;

            if guild.member_count > members {
                total += guild.member_count - members;
            }
        }

        total
    }

    /// Calculates a vector of all [`PrivateChannel`] and [`Group`] Ids that are
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

    /// Calculates a vector of all [`Guild`]s' Ids that are stored in the cache.
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
            .map(|g| g.id)
            .chain(self.unavailable_guilds.iter().cloned())
            .collect()
    }

    #[doc(hidden)]
    pub fn __download_members(&mut self) -> Vec<GuildId> {
        self.guilds
            .values_mut()
            .filter(|guild| guild.large)
            .map(|ref mut guild| {
                guild.members.clear();

                guild.id
            })
            .collect::<Vec<GuildId>>()
    }

    /// Retrieves a reference to a [`Call`] from the cache based on the
    /// associated [`Group`]'s channel Id.
    ///
    /// [`Call`]: ../../model/struct.Call.html
    /// [`Group`]: ../../model/struct.Group.html
    pub fn get_call<C: Into<ChannelId>>(&self, group_id: C) -> Option<&Call> {
        self.calls.get(&group_id.into())
    }

    /// Retrieves a [`Channel`] from the cache based on the given Id.
    ///
    /// This will search the [`groups`] map, the [`private_channels`] map, and
    /// then the map of [`guilds`] to find the channel.
    ///
    /// [`Channel`]: ../../model/enum.Channel.html
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`groups`]: #structfield.groups
    /// [`private_channels`]: #structfield.private_channels
    /// [`guilds`]: #structfield.guilds
    pub fn get_channel<C: Into<ChannelId>>(&self, id: C) -> Option<ChannelRef> {
        let id = id.into();

        if let Some(private_channel) = self.private_channels.get(&id) {
            return Some(ChannelRef::Private(private_channel));
        }

        if let Some(group) = self.groups.get(&id) {
            return Some(ChannelRef::Group(group));
        }

        for guild in self.guilds.values() {
            for channel in guild.channels.values() {
                if channel.id == id {
                    return Some(ChannelRef::Guild(channel));
                }
            }
        }

        None
    }

    /// Retrieves a reference to a guild from the cache based on the given Id.
    pub fn get_guild<G: Into<GuildId>>(&self, id: G) -> Option<&Guild> {
        self.guilds.get(&id.into())
    }

    /// Retrieves a reference to a [`Guild`]'s channel. Unlike [`get_channel`],
    /// this will only search guilds for the given channel.
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
    /// [`Client::on_message`]: ../../client/struct.Client.html#method.on_message
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`get_channel`]: #method.get_channel
    pub fn get_guild_channel<C: Into<ChannelId>>(&self, id: C) -> Option<&GuildChannel> {
        let id = id.into();

        for guild in self.guilds.values() {
            if let Some(channel) = guild.channels.get(&id) {
                return Some(channel);
            }
        }

        None
    }

    /// Retrieves a reference to a [`Group`] from the cache based on the given
    /// associated channel Id.
    ///
    /// [`Group`]: ../../model/struct.Group.html
    pub fn get_group<C: Into<ChannelId>>(&self, id: C) -> Option<&Group> {
        self.groups.get(&id.into())
    }

    /// Retrieves a reference to a [`Guild`]'s member from the cache based on
    /// the guild's and user's given Ids.
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
    pub fn get_member<G, U>(&self, guild_id: G, user_id: U) -> Option<&Member>
        where G: Into<GuildId>, U: Into<UserId> {
        self.guilds
            .get(&guild_id.into())
            .map(|guild| {
                guild.members.get(&user_id.into())
            }).and_then(|x| match x {
                Some(x) => Some(x),
                None => None,
            })
    }

    /// Retrieves a reference to a [`Guild`]'s role by their Ids.
    ///
    /// [`Guild`]: ../../model/struct.Guild.html
    pub fn get_role<G, R>(&self, guild_id: G, role_id: R) -> Option<&Role>
        where G: Into<GuildId>, R: Into<RoleId> {
        if let Some(guild) = self.guilds.get(&guild_id.into()) {
            guild.roles.get(&role_id.into())
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn update_with_call_create(&mut self, event: &CallCreateEvent) {
        match self.calls.entry(event.call.channel_id) {
            Entry::Vacant(e) => {
                e.insert(event.call.clone());
            },
            Entry::Occupied(mut e) => {
                e.get_mut().clone_from(&event.call);
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_call_delete(&mut self, event: &CallDeleteEvent)
        -> Option<Call> {
        self.calls.remove(&event.channel_id)
    }

    #[doc(hidden)]
    pub fn update_with_call_update(&mut self, event: &CallUpdateEvent, old: bool)
        -> Option<Call> {
        let item = if old {
            self.calls.get(&event.channel_id).cloned()
        } else {
            None
        };

        self.calls
            .get_mut(&event.channel_id)
            .map(|call| {
                call.region.clone_from(&event.region);
                call.ringing.clone_from(&event.ringing);
            });

        item
    }

    #[doc(hidden)]
    pub fn update_with_channel_create(&mut self, event: &ChannelCreateEvent)
        -> Option<Channel> {
        match event.channel {
            Channel::Group(ref group) => {
                let ch = self.groups.insert(group.channel_id, group.clone());

                ch.map(Channel::Group)
            },
            Channel::Private(ref channel) => {
                let ch = self.private_channels.insert(channel.id, channel.clone());

                ch.map(Channel::Private)
            },
            Channel::Guild(ref channel) => {
                let ch = self.guilds
                    .get_mut(&channel.guild_id)
                    .map(|guild| {
                        guild.channels.insert(channel.id, channel.clone())
                    });

                match ch {
                    Some(Some(ch)) => Some(Channel::Guild(ch)),
                    _ => None,
                }
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_delete(&mut self, event: &ChannelDeleteEvent)
        -> Option<Channel> {
        match event.channel {
            Channel::Group(ref group) => {
                self.groups.remove(&group.channel_id).map(Channel::Group)
            },
            Channel::Private(ref channel) => {
                self.private_channels.remove(&channel.id)
                    .map(Channel::Private)
            },
            Channel::Guild(ref channel) => {
                let ch = self.guilds
                    .get_mut(&channel.guild_id)
                    .map(|guild| guild.channels.remove(&channel.id));

                match ch {
                    Some(Some(ch)) => Some(Channel::Guild(ch)),
                    _ => None,
                }
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_pins_update(&mut self,
                                           event: &ChannelPinsUpdateEvent) {
        if let Some(channel) = self.private_channels.get_mut(&event.channel_id) {
            channel.last_pin_timestamp = event.last_pin_timestamp.clone();

            return;
        }

        if let Some(group) = self.groups.get_mut(&event.channel_id) {
            group.last_pin_timestamp = event.last_pin_timestamp.clone();

            return;
        }

        // Guild searching is last because it is expensive
        // in comparison to private channel and group searching.
        for guild in self.guilds.values_mut() {
            for channel in guild.channels.values_mut() {
                if channel.id == event.channel_id {
                    channel.last_pin_timestamp = event.last_pin_timestamp.clone();

                    return;
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn update_with_channel_recipient_add(&mut self,
                                             event: &ChannelRecipientAddEvent) {
        self.groups
            .get_mut(&event.channel_id)
            .map(|group| group.recipients.insert(event.user.id,
                                                 event.user.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_channel_recipient_remove(&mut self,
                                                event: &ChannelRecipientRemoveEvent) {
        self.groups
            .get_mut(&event.channel_id)
            .map(|group| group.recipients.remove(&event.user.id));
    }

    #[doc(hidden)]
    pub fn update_with_channel_update(&mut self, event: &ChannelUpdateEvent) {
        match event.channel {
            Channel::Group(ref group) => {
                match self.groups.entry(group.channel_id) {
                    Entry::Vacant(e) => {
                        e.insert(group.clone());
                    },
                    Entry::Occupied(mut e) => {
                        let dest = e.get_mut();

                        if group.recipients.is_empty() {
                            let recipients = mem::replace(&mut dest.recipients,
                                                          HashMap::new());

                            dest.clone_from(group);

                            dest.recipients = recipients;
                        } else {
                            dest.clone_from(group);
                        }
                    },
                }
            },
            Channel::Guild(ref channel) => {
                self.guilds
                    .get_mut(&channel.guild_id)
                    .map(|guild| guild.channels
                        .insert(channel.id, channel.clone()));
            },
            Channel::Private(ref channel) => {
                self.private_channels
                    .get_mut(&channel.id)
                    .map(|private| private.clone_from(channel));
            },
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_create(&mut self, event: &GuildCreateEvent) {
        self.unavailable_guilds.retain(|guild_id| *guild_id != event.guild.id);

        self.guilds.insert(event.guild.id, event.guild.clone());
    }

    #[doc(hidden)]
    pub fn update_with_guild_delete(&mut self, event: &GuildDeleteEvent)
        -> Option<Guild> {
        if !self.unavailable_guilds.contains(&event.guild.id) {
            self.unavailable_guilds.push(event.guild.id);
        }

        self.guilds.remove(&event.guild.id)
    }

    #[doc(hidden)]
    pub fn update_with_guild_emojis_update(&mut self,
                                           event: &GuildEmojisUpdateEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.emojis.extend(event.emojis.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_add(&mut self,
                                        event: &GuildMemberAddEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| {
                guild.member_count += 1;
                guild.members.insert(event.member.user.id,
                                     event.member.clone());
            });
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_remove(&mut self,
                                           event: &GuildMemberRemoveEvent)
                                           -> Option<Member> {
        let member = self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| {
                guild.member_count -= 1;
                guild.members.remove(&event.user.id)
            });

        match member {
            Some(Some(member)) => Some(member),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_member_update(&mut self,
                                           event: &GuildMemberUpdateEvent)
                                           -> Option<Member> {
        if let Some(guild) = self.guilds.get_mut(&event.guild_id) {
            let mut found = false;

            let item = if let Some(member) = guild.members.get_mut(&event.user.id) {
                let item = Some(member.clone());

                member.nick.clone_from(&event.nick);
                member.roles.clone_from(&event.roles);
                member.user.clone_from(&event.user);

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
                    user: event.user.clone(),
                });
            }

            item
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_members_chunk(&mut self,
                                           event: &GuildMembersChunkEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.members.extend(event.members.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_create(&mut self,
                                         event: &GuildRoleCreateEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.roles.insert(event.role.id, event.role.clone()));
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_delete(&mut self,
                                         event: &GuildRoleDeleteEvent)
                                         -> Option<Role> {
        let role = self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.roles.remove(&event.role_id));

        match role {
            Some(Some(x)) => Some(x),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_role_update(&mut self,
                                         event: &GuildRoleUpdateEvent)
                                         -> Option<Role> {
        let item = self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| guild.roles
                .get_mut(&event.role.id)
                .map(|role| mem::replace(role, event.role.clone())));

        match item {
            Some(Some(x)) => Some(x),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_sync(&mut self, event: &GuildSyncEvent) {
        self.guilds
            .get_mut(&event.guild_id)
            .map(|guild| {
                guild.large = event.large;
                guild.members.clone_from(&event.members);
                guild.presences.clone_from(&event.presences);
            });
    }

    #[doc(hidden)]
    pub fn update_with_guild_unavailable(&mut self,
                                         event: &GuildUnavailableEvent) {
        if !self.unavailable_guilds.contains(&event.guild_id) {
            self.unavailable_guilds.push(event.guild_id);
        }
    }

    #[doc(hidden)]
    pub fn update_with_guild_update(&mut self, event: &GuildUpdateEvent) {
        self.guilds
            .get_mut(&event.guild.id)
            .map(|guild| {
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
        self.presences.clone_from(&{
            let mut p = HashMap::default();

            for presence in &event.presences {
                p.insert(presence.user_id, presence.clone());
            }

            p
        });
    }

    #[doc(hidden)]
    pub fn update_with_presence_update(&mut self, event: &PresenceUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            if let Some(guild) = self.guilds.get_mut(&guild_id) {
                // If the user was modified, update the member list
                if let Some(user) = event.presence.user.as_ref() {
                    guild.members
                        .get_mut(&user.id)
                        .map(|member| member.user.clone_from(user));
                }

                update_presence(&mut guild.presences, &event.presence);
            }
        }
    }

    #[doc(hidden)]
    pub fn update_with_ready(&mut self, ready: &ReadyEvent) {
        let ready = ready.ready.clone();

        for guild in ready.guilds {
            match guild {
                PossibleGuild::Offline(guild_id) => {
                    self.unavailable_guilds.push(guild_id);
                }
                PossibleGuild::Online(guild) => {
                    self.guilds.insert(guild.id, guild);
                },
            }
        }

        self.unavailable_guilds.sort();
        self.unavailable_guilds.dedup();

        // The private channels sent in the READY contains both the actual
        // private channels, and the groups.
        for (channel_id, channel) in ready.private_channels {
            match channel {
                Channel::Group(group) => {
                    self.groups.insert(channel_id, group);
                },
                Channel::Private(channel) => {
                    self.private_channels.insert(channel_id, channel);
                },
                Channel::Guild(_) => {},
            }
        }

        for guild in ready.user_guild_settings.unwrap_or_default() {
            self.guild_settings.insert(guild.guild_id, guild);
        }

        for (user_id, presence) in ready.presences {
            self.presences.insert(user_id, presence);
        }

        for (user_id, relationship) in ready.relationships {
            self.relationships.insert(user_id, relationship);
        }

        self.notes.extend(ready.notes);

        self.settings = ready.user_settings;
        self.user = ready.user;
    }

    #[doc(hidden)]
    pub fn update_with_relationship_add(&mut self, event: &RelationshipAddEvent) {
        self.relationships.insert(event.relationship.id,
                                  event.relationship.clone());
    }

    #[doc(hidden)]
    pub fn update_with_relationship_remove(&mut self,
                                           event: &RelationshipRemoveEvent) {
        self.relationships.remove(&event.user_id);
    }

    #[doc(hidden)]
    pub fn update_with_user_guild_settings_update(&mut self,
                                                  event: &UserGuildSettingsUpdateEvent)
                                                  -> Option<UserGuildSettings> {
        self.guild_settings
            .get_mut(&event.settings.guild_id)
            .map(|guild_setting| mem::replace(guild_setting, event.settings.clone()))
    }

    #[doc(hidden)]
    pub fn update_with_user_note_update(&mut self,
                                        event: &UserNoteUpdateEvent)
                                        -> Option<String> {
        if event.note.is_empty() {
            self.notes.remove(&event.user_id)
        } else {
            self.notes.insert(event.user_id, event.note.clone())
        }
    }

    #[doc(hidden)]
    pub fn update_with_user_settings_update(&mut self,
                                            event: &UserSettingsUpdateEvent,
                                            old: bool)
                                            -> Option<UserSettings> {
        let item = if old {
            self.settings.clone()
        } else {
            None
        };

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
    pub fn update_with_user_update(&mut self, event: &UserUpdateEvent)
        -> CurrentUser {
        mem::replace(&mut self.user, event.current_user.clone())
    }

    #[doc(hidden)]
    pub fn update_with_voice_state_update(&mut self,
                                          event: &VoiceStateUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            if let Some(guild) = self.guilds.get_mut(&guild_id) {
                if event.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    {
                        let finding = guild.voice_states
                            .get_mut(&event.voice_state.user_id);

                        if let Some(srv_state) = finding {
                            srv_state.clone_from(&event.voice_state);

                            return;
                        }
                    }

                    guild.voice_states.insert(event.voice_state.user_id,
                                              event.voice_state.clone());
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
                {
                    let finding = call.voice_states
                        .get_mut(&event.voice_state.user_id);

                    if let Some(group_state) = finding {
                        group_state.clone_from(&event.voice_state);

                        return;
                    }
                }

                call.voice_states.insert(event.voice_state.user_id,
                                         event.voice_state.clone());
            }
        } else {
            // delete this user from any group call containing them
            for call in self.calls.values_mut() {
                call.voice_states.remove(&event.voice_state.user_id);
            }
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            calls: HashMap::default(),
            groups: HashMap::default(),
            guild_settings: HashMap::default(),
            guilds: HashMap::default(),
            notes: HashMap::default(),
            presences: HashMap::default(),
            private_channels: HashMap::default(),
            relationships: HashMap::default(),
            settings: None,
            unavailable_guilds: Vec::default(),
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
            }
        }
    }
}

fn update_presence(presences: &mut HashMap<UserId, Presence>,
                   presence: &Presence) {
    if presence.status == OnlineStatus::Offline {
        // Remove the user from the presence list
        presences.remove(&presence.user_id);
    } else {
        // Update or add to the presence list
        if let Some(ref mut guild_presence) = presences.get(&presence.user_id) {
            if presence.user.is_none() {
                guild_presence.clone_from(&presence);
            }

            return;
        }
        presences.insert(presence.user_id, presence.clone());
    }
}

/// A reference to a private channel, guild's channel, or group.
pub enum ChannelRef<'a> {
    /// A group's channel
    Group(&'a Group),
    /// A guild channel and its guild
    Guild(&'a GuildChannel),
    /// A private channel
    Private(&'a PrivateChannel),
}

impl<'a> ChannelRef<'a> {
    /// Clones the inner value of the variant.
    pub fn clone_inner(&self) -> Channel {
        match *self {
            ChannelRef::Group(group) => Channel::Group(group.clone()),
            ChannelRef::Guild(channel) => Channel::Guild(channel.clone()),
            ChannelRef::Private(private) => Channel::Private(private.clone()),
        }
    }
}

fn opt_modify<T: Clone>(dest: &mut T, src: &Option<T>) {
    if let Some(val) = src.as_ref() {
        dest.clone_from(val);
    }
}
