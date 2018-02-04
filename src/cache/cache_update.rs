use model::prelude::*;
use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::rc::Rc;
use std::mem;
use super::Cache;

pub(crate) trait CacheUpdate {
    type Output;

    fn update(&mut self, &mut Cache) -> Option<Self::Output>;
}

impl CacheUpdate for ChannelCreateEvent {
    type Output = Channel;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        match self.channel {
            Channel::Group(ref group) => {
                let group = Rc::clone(group);

                let channel_id = {
                    let mut writer = group.borrow_mut();

                    for (recipient_id, recipient) in &mut writer.recipients {
                        cache.update_user_entry(&recipient.borrow());

                        *recipient = Rc::clone(&cache.users[recipient_id]);
                    }

                    writer.channel_id
                };

                let old = cache.groups.insert(channel_id, group);

                old.map(Channel::Group)
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = {
                    let channel = channel.borrow();

                    (channel.guild_id, channel.id)
                };

                cache.channels.insert(channel_id, Rc::clone(channel));

                cache
                    .guilds
                    .get_mut(&guild_id)
                    .and_then(|guild| {
                        let mut guild = guild.borrow_mut();

                        guild.channels.insert(channel_id, Rc::clone(channel))
                    })
                    .map(Channel::Guild)
            },
            Channel::Private(ref channel) => {
                let channel_id = channel.borrow().id;

                if let Some(channel) = cache.private_channels.get(&channel_id) {
                    return Some(Channel::Private(Rc::clone(channel)));
                }

                let channel = Rc::clone(channel);

                let id = {
                    let mut writer = channel.borrow_mut();

                    let user_id = {
                        let user = writer.recipient.borrow();

                        cache.update_user_entry(&*user);

                        user.id
                    };

                    writer.recipient = Rc::clone(&cache.users[&user_id]);

                    writer.id
                };

                let old = cache.private_channels.insert(id, Rc::clone(&channel));
                old.map(Channel::Private)
            },
            Channel::Category(ref category) => cache
                .categories
                .insert(category.borrow().id, Rc::clone(category))
                .map(Channel::Category),
        }
    }
}

impl CacheUpdate for ChannelDeleteEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = {
                    let channel = channel.borrow();

                    (channel.guild_id, channel.id)
                };

                cache.channels.remove(&channel_id);

                if let Some(guild) = cache.guilds.get(&guild_id) {
                    let mut guild = guild.borrow_mut();

                    guild.channels.remove(&channel_id);
                }
            },
            Channel::Category(ref category) => {
                let channel_id = category.borrow().id;

                cache.categories.remove(&channel_id);
            },
            // We ignore these two due to the fact that the delete event for dms/groups
            // will _not_ fire anymore.
            Channel::Private(_) | Channel::Group(_) => unreachable!(),
        };

        None
    }
}

impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(channel) = cache.channels.get(&self.channel_id) {
            channel.borrow_mut().last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        if let Some(channel) = cache.private_channels.get_mut(&self.channel_id) {
            channel.borrow_mut().last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        if let Some(group) = cache.groups.get_mut(&self.channel_id) {
            group.borrow_mut().last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        None
    }
}

impl CacheUpdate for ChannelRecipientAddEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.update_user_entry(&self.user);
        let user = Rc::clone(&cache.users[&self.user.id]);

        cache.groups.get_mut(&self.channel_id).map(|group| {
            group.borrow_mut().recipients.insert(self.user.id, user);
        });

        None
    }
}

impl CacheUpdate for ChannelRecipientRemoveEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(group) = cache.groups.get_mut(&self.channel_id) {
            group.borrow_mut().recipients.remove(&self.user.id);
        }

        None
    }
}

impl CacheUpdate for ChannelUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        match self.channel {
            Channel::Group(ref group) => {
                let (ch_id, no_recipients) = {
                    let group = group.borrow();

                    (group.channel_id, group.recipients.is_empty())
                };

                match cache.groups.entry(ch_id) {
                    Entry::Vacant(e) => {
                        e.insert(Rc::clone(group));
                    },
                    Entry::Occupied(mut e) => {
                        let mut dest = e.get_mut().borrow_mut();

                        if no_recipients {
                            let recipients = mem::replace(&mut dest.recipients, HashMap::new());

                            dest.clone_from(&group.borrow());

                            dest.recipients = recipients;
                        } else {
                            dest.clone_from(&group.borrow());
                        }
                    },
                }
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = {
                    let channel = channel.borrow();

                    (channel.guild_id, channel.id)
                };

                cache.channels.insert(channel_id, Rc::clone(channel));
                cache.guilds.get_mut(&guild_id).map(|guild| {
                    let mut guild = guild.borrow_mut();

                    guild.channels.insert(channel_id, Rc::clone(channel))
                });
            },
            Channel::Private(ref channel) => {
                cache
                    .private_channels
                    .get_mut(&channel.borrow().id)
                    .map(|private| private.clone_from(channel));
            },
            Channel::Category(ref category) => {
                cache
                    .categories
                    .get_mut(&category.borrow().id)
                    .map(|c| c.clone_from(category));
            },
        }

        None
    }
}

impl CacheUpdate for GuildCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.unavailable_guilds.remove(&self.guild.id);

        let mut guild = self.guild.clone();

        for (user_id, member) in &mut guild.members {
            let mut member = member.borrow_mut();

            cache.update_user_entry(&member.user.borrow_mut());

            let user = Rc::clone(&cache.users[user_id]);
            member.user = Rc::clone(&user);
        }

        cache.channels.extend(guild.channels.clone());
        cache
            .guilds
            .insert(self.guild.id, Rc::new(RefCell::new(guild)));

        None
    }
}

impl CacheUpdate for GuildDeleteEvent {
    type Output = Rc<RefCell<Guild>>;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        // Remove channel entries for the guild if the guild is found.
        cache.guilds.remove(&self.guild.id).map(|guild| {
            for channel_id in guild.borrow().channels.keys() {
                cache.channels.remove(channel_id);
            }

            guild
        })
    }
}

impl CacheUpdate for GuildEmojisUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(guild) = cache.guilds.get_mut(&self.guild_id) {
            let mut guild = guild.borrow_mut();

            guild.emojis.clone_from(&self.emojis);
        }

        None
    }
}

impl CacheUpdate for GuildMemberAddEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let user_id = self.member.user.borrow().id;
        cache.update_user_entry(&self.member.user.borrow());

        // Always safe due to being inserted above.
        self.member.user = Rc::clone(&cache.users[&user_id]);

        if let Some(guild) = cache.guilds.get_mut(&self.guild_id) {
            let mut guild = guild.borrow_mut();

            guild.member_count += 1;
            guild.members.insert(user_id, Rc::new(RefCell::new(self.member.clone())));
        }

        None
    }
}

impl CacheUpdate for GuildMemberRemoveEvent {
    type Output = Rc<RefCell<Member>>;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            guild.try_borrow_mut().ok().and_then(|mut guild| {
                guild.member_count -= 1;
                guild.members.remove(&self.user.id)
            })
        })
    }
}

impl CacheUpdate for GuildMemberUpdateEvent {
    type Output = Member;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.update_user_entry(&self.user);

        if let Some(guild) = cache.guilds.get_mut(&self.guild_id) {
            let mut guild = guild.borrow_mut();

            let mut found = false;

            let item = {
                let member = guild.members.get(&self.user.id).and_then(|member| {
                    member.try_borrow_mut().ok()
                });
                if let Some(mut member) = member {
                    let item = Some(member.clone());

                    member.nick.clone_from(&self.nick);
                    member.roles.clone_from(&self.roles);
                    member.user.borrow_mut().clone_from(&self.user);

                    found = true;

                    item
                } else {
                    None
                }
            };

            if !found {
                guild.members.insert(
                    self.user.id,
                    Rc::new(RefCell::new(Member {
                        client: None,
                        deaf: false,
                        guild_id: self.guild_id,
                        joined_at: None,
                        mute: false,
                        nick: self.nick.clone(),
                        roles: self.roles.clone(),
                        user: Rc::new(RefCell::new(self.user.clone())),
                    })),
                );
            }

            item
        } else {
            None
        }
    }
}

impl CacheUpdate for GuildMembersChunkEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        for member in self.members.values() {
            cache.update_user_entry(&member.user.borrow());
        }

        if let Some(guild) = cache.guilds.get(&self.guild_id) {
            let mut guild = guild.borrow_mut();

            guild.members.extend({
                let mut m: HashMap<UserId, Rc<RefCell<Member>>> = HashMap::new();

                for (id, member) in &self.members {
                    m.insert(*id, Rc::new(RefCell::new(member.clone())));
                }

                m
            });
        }

        None
    }
}

impl CacheUpdate for GuildRoleCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let guild = cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            guild.try_borrow_mut().ok()
        });

        if let Some(mut guild) = guild {
            guild
                .roles
                .insert(self.role.id, Rc::new(RefCell::new(self.role.clone())));
        }

        None
    }
}

impl CacheUpdate for GuildRoleDeleteEvent {
    type Output = Rc<RefCell<Role>>;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache
            .guilds
            .get_mut(&self.guild_id)
            .and_then(|guild| {
                let mut guild = guild.borrow_mut();

                guild.roles.remove(&self.role_id)
            })
    }
}

impl CacheUpdate for GuildRoleUpdateEvent {
    type Output = Role;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|guild| {
            let mut guild = guild.borrow_mut();

            guild.roles.get_mut(&self.role.id).map(|role| {
                mem::replace(&mut *role.borrow_mut(), self.role.clone())
            })
        })
    }
}

impl CacheUpdate for GuildUnavailableEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.unavailable_guilds.insert(self.guild_id);
        cache.guilds.remove(&self.guild_id);

        None
    }
}

impl CacheUpdate for GuildUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(guild) = cache.guilds.get(&self.guild.id) {
            let mut guild = guild.try_borrow_mut().ok()?;

            guild.afk_timeout = self.guild.afk_timeout;
            guild.afk_channel_id.clone_from(&self.guild.afk_channel_id);
            guild.icon.clone_from(&self.guild.icon);
            guild.name.clone_from(&self.guild.name);
            guild.owner_id.clone_from(&self.guild.owner_id);
            guild.region.clone_from(&self.guild.region);
            guild.roles.clone_from(&self.guild.roles);
            guild.verification_level = self.guild.verification_level;
        }

        None
    }
}

impl CacheUpdate for PresenceUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let user_id = self.presence.user_id;

        if let Some(user) = self.presence.user.as_mut() {
            cache.update_user_entry(&user.borrow());
            *user = Rc::clone(&cache.users[&user_id]);
        }

        if let Some(guild_id) = self.guild_id {
            let guild = cache.guilds.get(&guild_id).and_then(|guild| {
                guild.try_borrow_mut().ok()
            });

            if let Some(mut guild) = guild {
                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user_id);
                } else {
                    guild
                        .presences
                        .insert(self.presence.user_id, Rc::new(RefCell::new(self.presence.clone())));
                }

                // Create a partial member instance out of the presence update
                // data. This includes everything but `deaf`, `mute`, and
                // `joined_at`.
                if !guild.members.contains_key(&self.presence.user_id) {
                    if let Some(user) = self.presence.user.as_ref() {
                        let roles = self.roles.clone().unwrap_or_default();

                        guild.members.insert(self.presence.user_id, Rc::new(RefCell::new(Member {
                            client: None,
                            deaf: false,
                            guild_id: guild_id,
                            joined_at: None,
                            mute: false,
                            nick: self.presence.nick.clone(),
                            user: Rc::clone(&user),
                            roles,
                        })));
                    }
                }
            }
        } else if self.presence.status == OnlineStatus::Offline {
            cache.presences.remove(&self.presence.user_id);
        } else {
            cache
                .presences
                .insert(self.presence.user_id, Rc::new(RefCell::new(self.presence.clone())));
        }

        None
    }
}

impl CacheUpdate for PresencesReplaceEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        cache.presences.extend({
            let mut p: HashMap<UserId, Rc<RefCell<Presence>>> = HashMap::default();

            for presence in &self.presences {
                p.insert(presence.user_id, Rc::new(RefCell::new(presence.clone())));
            }

            p
        });

        None
    }
}

impl CacheUpdate for ReadyEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        let mut ready = self.ready.clone();

        for guild in ready.guilds {
            match guild {
                GuildStatus::Offline(unavailable) => {
                    cache.guilds.remove(&unavailable.id);
                    cache.unavailable_guilds.insert(unavailable.id);
                },
                GuildStatus::OnlineGuild(guild) => {
                    cache.unavailable_guilds.remove(&guild.id);
                    cache.guilds.insert(guild.id, Rc::new(RefCell::new(guild)));
                },
                GuildStatus::OnlinePartialGuild(_) => {},
            }
        }

        // `ready.private_channels` will always be empty, and possibly be removed in the future.
        // So don't handle it at all.

        for (user_id, presence) in &mut ready.presences {
            let mut presence = presence.borrow_mut();

            if let Some(ref user) = presence.user {
                cache.update_user_entry(&user.borrow());
            }

            presence.user = cache.users.get(user_id).cloned();
        }

        cache.presences.extend(ready.presences);
        cache.shard_count = ready.shard.map_or(1, |s| s[1]);
        cache.user = ready.user;

        None
    }
}

impl CacheUpdate for UserUpdateEvent {
    type Output = CurrentUser;

    fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
        Some(mem::replace(&mut cache.user, self.current_user.clone()))
    }
}

impl CacheUpdate for VoiceStateUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &mut Cache) -> Option<()> {
        if let Some(guild_id) = self.guild_id {
            if let Some(guild) = cache.guilds.get_mut(&guild_id) {
                let mut guild = guild.borrow_mut();

                if self.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    {
                        let finding = guild.voice_states.get_mut(&self.voice_state.user_id);

                        if let Some(srv_state) = finding {
                            srv_state.clone_from(&self.voice_state);

                            return None;
                        }
                    }

                    guild
                        .voice_states
                        .insert(self.voice_state.user_id, self.voice_state.clone());
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&self.voice_state.user_id);
                }
            }

            return None;
        }

        None
    }
}
