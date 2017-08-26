use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::default::Default;
use std::sync::{Arc, RwLock};
use std::mem;
use model::*;
use model::event::*;
use internal::RwLockExt;

pub(crate) trait CacheEventsImpl {
    fn update_with_channel_create(&mut self, event: &ChannelCreateEvent) -> Option<Channel>;

    fn update_with_channel_delete(&mut self, event: &ChannelDeleteEvent);

    fn update_with_channel_pins_update(&mut self, event: &ChannelPinsUpdateEvent);

    fn update_with_channel_recipient_add(&mut self, event: &mut ChannelRecipientAddEvent);

    fn update_with_channel_recipient_remove(&mut self, event: &ChannelRecipientRemoveEvent);

    fn update_with_channel_update(&mut self, event: &ChannelUpdateEvent);

    fn update_with_guild_create(&mut self, event: &GuildCreateEvent);

    fn update_with_guild_delete(&mut self, event: &GuildDeleteEvent) -> Option<Arc<RwLock<Guild>>>;

    fn update_with_guild_emojis_update(&mut self, event: &GuildEmojisUpdateEvent);

    fn update_with_guild_member_add(&mut self, event: &mut GuildMemberAddEvent);

    fn update_with_guild_member_remove(&mut self, event: &GuildMemberRemoveEvent)
                                       -> Option<Member>;

    fn update_with_guild_member_update(&mut self, event: &GuildMemberUpdateEvent)
                                       -> Option<Member>;

    fn update_with_guild_members_chunk(&mut self, event: &GuildMembersChunkEvent);

    fn update_with_guild_role_create(&mut self, event: &GuildRoleCreateEvent);

    fn update_with_guild_role_delete(&mut self, event: &GuildRoleDeleteEvent) -> Option<Role>;

    fn update_with_guild_role_update(&mut self, event: &GuildRoleUpdateEvent) -> Option<Role>;

    fn update_with_guild_unavailable(&mut self, event: &GuildUnavailableEvent);

    fn update_with_guild_update(&mut self, event: &GuildUpdateEvent);

    fn update_with_presences_replace(&mut self, event: &PresencesReplaceEvent);

    fn update_with_presence_update(&mut self, event: &mut PresenceUpdateEvent);

    fn update_with_ready(&mut self, event: &ReadyEvent);

    fn update_with_user_update(&mut self, event: &UserUpdateEvent) -> CurrentUser;

    fn update_with_voice_state_update(&mut self, event: &VoiceStateUpdateEvent);
}

impl CacheEventsImpl for super::Cache {
    fn update_with_channel_create(&mut self, event: &ChannelCreateEvent) -> Option<Channel> {
        match event.channel {
            Channel::Group(ref group) => {
                let group = group.clone();

                let channel_id = group.with_mut(|writer| {
                    for (recipient_id, recipient) in &mut writer.recipients {
                        self.update_user_entry(&recipient.read().unwrap());

                        *recipient = self.users[recipient_id].clone();
                    }

                    writer.channel_id
                });

                let ch = self.groups.insert(channel_id, group);

                ch.map(Channel::Group)
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

                self.channels.insert(channel_id, channel.clone());

                self.guilds
                    .get_mut(&guild_id)
                    .and_then(|guild| {
                        guild.with_mut(|guild| {
                            guild.channels.insert(channel_id, channel.clone())
                        })
                    })
                    .map(Channel::Guild)
            },
            Channel::Private(ref channel) => {
                if let Some(channel) = self.private_channels.get(&channel.with(|c| c.id)) {
                    return Some(Channel::Private((*channel).clone()));
                }

                let channel = channel.clone();

                let id = channel.with_mut(|writer| {
                    let user_id = writer.recipient.with_mut(|user| {
                        self.update_user_entry(&user);

                        user.id
                    });

                    writer.recipient = self.users[&user_id].clone();
                    writer.id
                });

                let ch = self.private_channels.insert(id, channel.clone());
                ch.map(Channel::Private)
            },
        }
    }

    fn update_with_channel_delete(&mut self, event: &ChannelDeleteEvent) {
        let channel = match event.channel {
            Channel::Guild(ref channel) => channel,
            // We ignore these two due to the fact that the delete event for dms/groups
            // will _not_ fire
            // anymore.
            Channel::Private(_) |
            Channel::Group(_) => unreachable!(),
        };

        let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

        self.channels.remove(&channel_id);

        self.guilds.get_mut(&guild_id).and_then(|guild| {
            guild.with_mut(|g| g.channels.remove(&channel_id))
        });
    }

    #[allow(dead_code)]
    fn update_with_channel_pins_update(&mut self, event: &ChannelPinsUpdateEvent) {
        if let Some(channel) = self.channels.get(&event.channel_id) {
            channel.with_mut(|c| {
                c.last_pin_timestamp = event.last_pin_timestamp;
            });

            return;
        }

        if let Some(channel) = self.private_channels.get_mut(&event.channel_id) {
            channel.with_mut(|c| {
                c.last_pin_timestamp = event.last_pin_timestamp;
            });

            return;
        }

        if let Some(group) = self.groups.get_mut(&event.channel_id) {
            group.with_mut(|c| {
                c.last_pin_timestamp = event.last_pin_timestamp;
            });

            return;
        }
    }

    fn update_with_channel_recipient_add(&mut self, event: &mut ChannelRecipientAddEvent) {
        self.update_user_entry(&event.user);
        let user = self.users[&event.user.id].clone();

        self.groups.get_mut(&event.channel_id).map(|group| {
            group.write().unwrap().recipients.insert(
                event.user.id,
                user,
            );
        });
    }

    fn update_with_channel_recipient_remove(&mut self, event: &ChannelRecipientRemoveEvent) {
        self.groups.get_mut(&event.channel_id).map(|group| {
            group.with_mut(|g| g.recipients.remove(&event.user.id))
        });
    }

    fn update_with_channel_update(&mut self, event: &ChannelUpdateEvent) {
        match event.channel {
            Channel::Group(ref group) => {
                let (ch_id, no_recipients) =
                    group.with(|g| (g.channel_id, g.recipients.is_empty()));

                match self.groups.entry(ch_id) {
                    Entry::Vacant(e) => {
                        e.insert(group.clone());
                    },
                    Entry::Occupied(mut e) => {
                        let mut dest = e.get_mut().write().unwrap();

                        if no_recipients {
                            let recipients = mem::replace(&mut dest.recipients, HashMap::new());

                            dest.clone_from(&group.read().unwrap());

                            dest.recipients = recipients;
                        } else {
                            dest.clone_from(&group.read().unwrap());
                        }
                    },
                }
            },
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = channel.with(|channel| (channel.guild_id, channel.id));

                self.channels.insert(channel_id, channel.clone());
                self.guilds.get_mut(&guild_id).map(|guild| {
                    guild.with_mut(
                        |g| g.channels.insert(channel_id, channel.clone()),
                    )
                });
            },
            Channel::Private(ref channel) => {
                self.private_channels
                    .get_mut(&channel.read().unwrap().id)
                    .map(|private| private.clone_from(channel));
            },
        }
    }

    fn update_with_guild_create(&mut self, event: &GuildCreateEvent) {
        self.unavailable_guilds.remove(&event.guild.id);

        let mut guild = event.guild.clone();

        for (user_id, member) in &mut guild.members {
            self.update_user_entry(&member.user.read().unwrap());
            let user = self.users[user_id].clone();

            member.user = user.clone();
        }

        self.channels.extend(guild.channels.clone());
        self.guilds.insert(
            event.guild.id,
            Arc::new(RwLock::new(guild)),
        );
    }

    fn update_with_guild_delete(&mut self, event: &GuildDeleteEvent) -> Option<Arc<RwLock<Guild>>> {
        // Remove channel entries for the guild if the guild is found.
        self.guilds.remove(&event.guild.id).map(|guild| {
            for channel_id in guild.write().unwrap().channels.keys() {
                self.channels.remove(channel_id);
            }

            guild
        })
    }

    fn update_with_guild_emojis_update(&mut self, event: &GuildEmojisUpdateEvent) {
        self.guilds.get_mut(&event.guild_id).map(|guild| {
            guild.with_mut(|g| g.emojis.extend(event.emojis.clone()))
        });
    }

    fn update_with_guild_member_add(&mut self, event: &mut GuildMemberAddEvent) {
        let user_id = event.member.user.with(|u| u.id);
        self.update_user_entry(&event.member.user.read().unwrap());

        // Always safe due to being inserted above.
        event.member.user = self.users[&user_id].clone();

        self.guilds.get_mut(&event.guild_id).map(|guild| {
            guild.with_mut(|guild| {
                guild.member_count += 1;
                guild.members.insert(user_id, event.member.clone());
            })
        });
    }

    fn update_with_guild_member_remove(&mut self,
                                       event: &GuildMemberRemoveEvent)
                                       -> Option<Member> {
        self.guilds.get_mut(&event.guild_id).and_then(|guild| {
            guild.with_mut(|guild| {
                guild.member_count -= 1;
                guild.members.remove(&event.user.id)
            })
        })
    }

    fn update_with_guild_member_update(&mut self,
                                       event: &GuildMemberUpdateEvent)
                                       -> Option<Member> {
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
                guild.members.insert(
                    event.user.id,
                    Member {
                        deaf: false,
                        guild_id: event.guild_id,
                        joined_at: None,
                        mute: false,
                        nick: event.nick.clone(),
                        roles: event.roles.clone(),
                        user: Arc::new(RwLock::new(event.user.clone())),
                    },
                );
            }

            item
        } else {
            None
        }
    }

    fn update_with_guild_members_chunk(&mut self, event: &GuildMembersChunkEvent) {
        for member in event.members.values() {
            self.update_user_entry(&member.user.read().unwrap());
        }

        self.guilds.get_mut(&event.guild_id).map(|guild| {
            guild.with_mut(|g| g.members.extend(event.members.clone()))
        });
    }

    fn update_with_guild_role_create(&mut self, event: &GuildRoleCreateEvent) {
        self.guilds.get_mut(&event.guild_id).map(|guild| {
            guild.write().unwrap().roles.insert(
                event.role.id,
                event.role.clone(),
            )
        });
    }

    fn update_with_guild_role_delete(&mut self, event: &GuildRoleDeleteEvent) -> Option<Role> {
        self.guilds.get_mut(&event.guild_id).and_then(|guild| {
            guild.with_mut(|g| g.roles.remove(&event.role_id))
        })
    }

    fn update_with_guild_role_update(&mut self, event: &GuildRoleUpdateEvent) -> Option<Role> {
        self.guilds.get_mut(&event.guild_id).and_then(|guild| {
            guild.with_mut(|g| {
                g.roles.get_mut(&event.role.id).map(|role| {
                    mem::replace(role, event.role.clone())
                })
            })
        })
    }

    fn update_with_guild_unavailable(&mut self, event: &GuildUnavailableEvent) {
        self.unavailable_guilds.insert(event.guild_id);
        self.guilds.remove(&event.guild_id);
    }

    fn update_with_guild_update(&mut self, event: &GuildUpdateEvent) {
        self.guilds.get_mut(&event.guild.id).map(|guild| {
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

    fn update_with_presences_replace(&mut self, event: &PresencesReplaceEvent) {
        self.presences.extend({
            let mut p: HashMap<UserId, Presence> = HashMap::default();

            for presence in &event.presences {
                p.insert(presence.user_id, presence.clone());
            }

            p
        });
    }

    fn update_with_presence_update(&mut self, event: &mut PresenceUpdateEvent) {
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
                    guild.presences.insert(
                        event.presence.user_id,
                        event.presence.clone(),
                    );
                }
            }
        } else if event.presence.status == OnlineStatus::Offline {
            self.presences.remove(&event.presence.user_id);
        } else {
            self.presences.insert(
                event.presence.user_id,
                event.presence.clone(),
            );
        }
    }

    fn update_with_ready(&mut self, event: &ReadyEvent) {
        let mut ready = event.ready.clone();

        for guild in ready.guilds {
            match guild {
                GuildStatus::Offline(unavailable) => {
                    self.guilds.remove(&unavailable.id);
                    self.unavailable_guilds.insert(unavailable.id);
                },
                GuildStatus::OnlineGuild(guild) => {
                    self.unavailable_guilds.remove(&guild.id);
                    self.guilds.insert(guild.id, Arc::new(RwLock::new(guild)));
                },
                GuildStatus::OnlinePartialGuild(_) => {},
            }
        }

        // `ready.private_channels` will always be empty, and possibly be removed in the future.
        // So don't handle it at all.

        for (user_id, presence) in &mut ready.presences {
            if let Some(ref user) = presence.user {
                self.update_user_entry(&user.read().unwrap());
            }

            presence.user = self.users.get(user_id).cloned();
        }

        self.presences.extend(ready.presences);
        self.shard_count = ready.shard.map_or(1, |s| s[1]);
        self.user = ready.user;
    }

    fn update_with_user_update(&mut self, event: &UserUpdateEvent) -> CurrentUser {
        mem::replace(&mut self.user, event.current_user.clone())
    }

    fn update_with_voice_state_update(&mut self, event: &VoiceStateUpdateEvent) {
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

                    guild.voice_states.insert(
                        event.voice_state.user_id,
                        event.voice_state.clone(),
                    );
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&event.voice_state.user_id);
                }
            }

            return;
        }
    }
}
