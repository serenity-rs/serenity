use std::collections::HashSet;

use super::{Cache, CacheUpdate};
use crate::model::channel::{Channel, GuildChannel, Message};
use crate::model::event::{
    ChannelCreateEvent,
    ChannelDeleteEvent,
    ChannelPinsUpdateEvent,
    ChannelUpdateEvent,
    GuildCreateEvent,
    GuildDeleteEvent,
    GuildEmojisUpdateEvent,
    GuildMemberAddEvent,
    GuildMemberRemoveEvent,
    GuildMemberUpdateEvent,
    GuildMembersChunkEvent,
    GuildRoleCreateEvent,
    GuildRoleDeleteEvent,
    GuildRoleUpdateEvent,
    GuildStickersUpdateEvent,
    GuildUnavailableEvent,
    GuildUpdateEvent,
    MessageCreateEvent,
    MessageUpdateEvent,
    PresenceUpdateEvent,
    PresencesReplaceEvent,
    ReadyEvent,
    ThreadCreateEvent,
    ThreadDeleteEvent,
    ThreadUpdateEvent,
    UserUpdateEvent,
    VoiceStateUpdateEvent,
};
use crate::model::guild::{Guild, Member, Role};
use crate::model::user::{CurrentUser, OnlineStatus};
use crate::model::voice::VoiceState;

impl CacheUpdate for ChannelCreateEvent {
    type Output = Channel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                let old_channel = cache
                    .guilds
                    .get_mut(&guild_id)
                    .and_then(|mut g| g.channels.insert(channel_id, self.channel.clone()));

                cache.channels.insert(channel_id, channel.clone());

                old_channel
            },
            Channel::Private(ref mut channel) => {
                if let Some(channel) = cache.private_channels.get(&channel.id) {
                    return Some(Channel::Private(channel.clone()));
                }

                let id = {
                    let user_id = {
                        cache.update_user_entry(&channel.recipient);

                        channel.recipient.id
                    };

                    if let Some(u) = cache.users.get(&user_id) {
                        channel.recipient = u.clone();
                    }

                    channel.id
                };

                cache.private_channels.insert(id, channel.clone()).map(Channel::Private)
            },
            Channel::Category(ref category) => {
                let (guild_id, channel_id) = (category.guild_id, category.id);

                let old_channel = cache
                    .guilds
                    .get_mut(&guild_id)
                    .and_then(|mut g| g.channels.insert(channel_id, self.channel.clone()));

                cache.categories.insert(channel_id, category.clone());

                old_channel
            },
        }
    }
}

impl CacheUpdate for ChannelDeleteEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                cache.channels.remove(&channel_id);

                cache.guilds.get_mut(&guild_id).map(|mut g| g.channels.remove(&channel_id));
            },
            Channel::Category(ref category) => {
                let (guild_id, channel_id) = (category.guild_id, category.id);

                cache.categories.remove(&channel_id);

                cache.guilds.get_mut(&guild_id).map(|mut g| g.channels.remove(&channel_id));
            },
            Channel::Private(ref channel) => {
                let id = { channel.id };

                cache.private_channels.remove(&id);
            },
        };

        // Remove the cached messages for the channel.
        cache.messages.remove(&self.channel.id());

        None
    }
}

impl CacheUpdate for ChannelUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        match self.channel {
            Channel::Guild(ref channel) => {
                let (guild_id, channel_id) = (channel.guild_id, channel.id);

                cache.channels.insert(channel_id, channel.clone());

                cache
                    .guilds
                    .get_mut(&guild_id)
                    .map(|mut g| g.channels.insert(channel_id, self.channel.clone()));
            },
            Channel::Private(ref channel) => {
                if let Some(mut c) = cache.private_channels.get_mut(&channel.id) {
                    c.clone_from(channel);
                }
            },
            Channel::Category(ref category) => {
                let (guild_id, channel_id) = (category.guild_id, category.id);

                cache.categories.insert(channel_id, category.clone());

                cache
                    .guilds
                    .get_mut(&guild_id)
                    .map(|mut g| g.channels.insert(channel_id, self.channel.clone()));
            },
        }

        None
    }
}

impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut channel) = cache.channels.get_mut(&self.channel_id) {
            channel.last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        if let Some(mut channel) = cache.private_channels.get_mut(&self.channel_id) {
            channel.last_pin_timestamp = self.last_pin_timestamp;

            return None;
        }

        None
    }
}

impl CacheUpdate for GuildCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.unavailable_guilds.remove(&self.guild.id);
        let mut guild = self.guild.clone();

        for (user_id, member) in &mut guild.members {
            cache.update_user_entry(&member.user);
            if let Some(u) = cache.user(user_id) {
                member.user = u;
            }
        }

        for pair in guild.channels.clone() {
            if let Channel::Guild(channel) = pair.1 {
                cache.channels.insert(pair.0, channel);
            }
        }

        for pair in guild.channels.clone() {
            if let Channel::Category(category) = pair.1 {
                cache.categories.insert(pair.0, category);
            }
        }

        cache.guilds.insert(self.guild.id, guild);

        None
    }
}

impl CacheUpdate for GuildDeleteEvent {
    type Output = Guild;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        match cache.guilds.remove(&self.guild.id) {
            Some(guild) => {
                for (channel_id, channel) in &guild.1.channels {
                    match channel {
                        Channel::Guild(_) => {
                            // Remove the channel from the cache.
                            cache.channels.remove(channel_id);

                            // Remove the channel's cached messages.
                            cache.messages.remove(channel_id);
                        },
                        Channel::Category(_) => {
                            // Remove the category from the cache
                            cache.categories.remove(channel_id);
                        },
                        _ => {},
                    }
                }

                Some(guild.1)
            },
            None => None,
        }
    }
}

impl CacheUpdate for GuildEmojisUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.emojis.clone_from(&self.emojis);
        }

        None
    }
}

impl CacheUpdate for GuildMemberAddEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        let user_id = self.member.user.id;
        cache.update_user_entry(&self.member.user);
        if let Some(u) = cache.user(user_id) {
            self.member.user = u;
        }

        if let Some(mut guild) = cache.guilds.get_mut(&self.member.guild_id) {
            guild.member_count += 1;
            guild.members.insert(user_id, self.member.clone());
        }

        None
    }
}

impl CacheUpdate for GuildMemberRemoveEvent {
    type Output = Member;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.member_count -= 1;
            return guild.members.remove(&self.user.id);
        }

        None
    }
}

impl CacheUpdate for GuildMemberUpdateEvent {
    type Output = Member;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        cache.update_user_entry(&self.user);

        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            let item = if let Some(member) = guild.members.get_mut(&self.user.id) {
                let item = Some(member.clone());

                member.joined_at.clone_from(&Some(self.joined_at));
                member.nick.clone_from(&self.nick);
                member.roles.clone_from(&self.roles);
                member.user.clone_from(&self.user);
                member.pending.clone_from(&self.pending);
                member.premium_since.clone_from(&self.premium_since);
                member.deaf.clone_from(&self.deaf);
                member.mute.clone_from(&self.mute);
                member.avatar.clone_from(&self.avatar);
                member.communication_disabled_until.clone_from(&self.communication_disabled_until);

                item
            } else {
                None
            };

            if item.is_none() {
                guild.members.insert(self.user.id, Member {
                    deaf: false,
                    guild_id: self.guild_id,
                    joined_at: Some(self.joined_at),
                    mute: false,
                    nick: self.nick.clone(),
                    roles: self.roles.clone(),
                    user: self.user.clone(),
                    pending: self.pending,
                    premium_since: self.premium_since,
                    permissions: None,
                    avatar: self.avatar.clone(),
                    communication_disabled_until: self.communication_disabled_until,
                });
            }

            item
        } else {
            None
        }
    }
}

impl CacheUpdate for GuildMembersChunkEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        for member in self.members.values() {
            cache.update_user_entry(&member.user);
        }

        if let Some(mut g) = cache.guilds.get_mut(&self.guild_id) {
            g.members.extend(self.members.clone());
        }

        None
    }
}

impl CacheUpdate for GuildRoleCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        cache
            .guilds
            .get_mut(&self.role.guild_id)
            .map(|mut g| g.roles.insert(self.role.id, self.role.clone()));

        None
    }
}

impl CacheUpdate for GuildRoleDeleteEvent {
    type Output = Role;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|mut g| g.roles.remove(&self.role_id))
    }
}

impl CacheUpdate for GuildRoleUpdateEvent {
    type Output = Role;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.role.guild_id) {
            if let Some(role) = guild.roles.get_mut(&self.role.id) {
                return Some(std::mem::replace(role, self.role.clone()));
            }
        }

        None
    }
}

impl CacheUpdate for GuildStickersUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.stickers.clone_from(&self.stickers);
        }

        None
    }
}

impl CacheUpdate for GuildUnavailableEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.unavailable_guilds.insert(self.guild_id);
        cache.guilds.remove(&self.guild_id);

        None
    }
}

impl CacheUpdate for GuildUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild.id) {
            guild.afk_channel_id.clone_from(&self.guild.afk_channel_id);
            guild.afk_timeout = self.guild.afk_timeout;
            guild.banner.clone_from(&self.guild.banner);
            guild.discovery_splash.clone_from(&self.guild.discovery_splash);
            guild.features.clone_from(&self.guild.features);
            guild.icon.clone_from(&self.guild.icon);
            guild.name.clone_from(&self.guild.name);
            guild.owner_id.clone_from(&self.guild.owner_id);
            guild.roles.clone_from(&self.guild.roles);
            guild.splash.clone_from(&self.guild.splash);
            guild.vanity_url_code.clone_from(&self.guild.vanity_url_code);
            guild.welcome_screen.clone_from(&self.guild.welcome_screen);
            guild.default_message_notifications = self.guild.default_message_notifications;
            guild.max_members = self.guild.max_members;
            guild.max_presences = self.guild.max_presences;
            guild.max_video_channel_users = self.guild.max_video_channel_users;
            guild.mfa_level = self.guild.mfa_level;
            guild.nsfw_level = self.guild.nsfw_level;
            guild.premium_subscription_count = self.guild.premium_subscription_count;
            guild.premium_tier = self.guild.premium_tier;
            guild.public_updates_channel_id = self.guild.public_updates_channel_id;
            guild.rules_channel_id = self.guild.rules_channel_id;
            guild.system_channel_flags = self.guild.system_channel_flags;
            guild.system_channel_id = self.guild.system_channel_id;
            guild.verification_level = self.guild.verification_level;
            guild.widget_channel_id = self.guild.widget_channel_id;
            guild.widget_enabled = self.guild.widget_enabled;
        }

        None
    }
}

impl CacheUpdate for MessageCreateEvent {
    /// The oldest message, if the channel's message cache was already full.
    type Output = Message;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let max = cache.settings().max_messages;

        if max == 0 {
            return None;
        }

        let messages =
            cache.messages.entry(self.message.channel_id).or_insert_with(Default::default);
        let mut queue =
            cache.message_queue.entry(self.message.channel_id).or_insert_with(Default::default);

        let mut removed_msg = None;

        if messages.len() == max {
            if let Some(id) = queue.pop_front() {
                removed_msg = messages.remove(&id);
            }
        }

        queue.push_back(self.message.id);
        messages.insert(self.message.id, self.message.clone());

        removed_msg.map(|i| i.1)
    }
}

impl CacheUpdate for MessageUpdateEvent {
    type Output = Message;

    #[rustfmt::skip]
    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        // Destructure, so we get an `unused` warning when we forget to process one of the fields
        // in this method
        #[allow(deprecated)] // yes rust, exhaustive means exhaustive, even the deprecated ones
        let Self {
            id, channel_id, content, edited_timestamp, tts, mention_everyone, mentions,
            mention_roles, mention_channels, attachments, embeds, reactions, pinned, flags,
            components, sticker_items,

            author: _, timestamp: _,  nonce: _, kind: _, stickers: _,  guild_id: _,
        } = &self;

        let messages = cache.messages.get_mut(channel_id)?;
        let mut message = messages.get_mut(id)?;
        let old_message = message.clone();

        if let Some(x) = attachments { message.attachments = x.clone() }
        if let Some(x) = content { message.content = x.clone() }
        if let Some(x) = edited_timestamp { message.edited_timestamp = Some(*x) }
        if let Some(x) = mentions { message.mentions = x.clone() }
        if let Some(x) = mention_everyone { message.mention_everyone = *x }
        if let Some(x) = mention_roles { message.mention_roles = x.clone() }
        if let Some(x) = mention_channels { message.mention_channels = x.clone() }
        if let Some(x) = pinned { message.pinned = *x }
        if let Some(x) = flags { message.flags = Some(*x) }
        if let Some(x) = tts { message.tts = *x }
        if let Some(x) = embeds { message.embeds = x.clone() }
        if let Some(x) = reactions { message.reactions = x.clone() }
        if let Some(x) = components { message.components = x.clone() }
        if let Some(x) = sticker_items { message.sticker_items = x.clone() }

        Some(old_message)
    }
}

impl CacheUpdate for PresenceUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(user) = self.presence.user.to_user() {
            cache.update_user_entry(&user);
        }

        if let Some(user) = cache.user(self.presence.user.id) {
            self.presence.user.update_with_user(user);
        }

        if let Some(guild_id) = self.presence.guild_id {
            if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user.id);
                } else {
                    guild.presences.insert(self.presence.user.id, self.presence.clone());
                }

                // Create a partial member instance out of the presence update
                // data.
                if let Some(user) = self.presence.user.to_user() {
                    guild.members.entry(self.presence.user.id).or_insert_with(|| Member {
                        deaf: false,
                        guild_id,
                        joined_at: None,
                        mute: false,
                        nick: None,
                        user,
                        roles: vec![],
                        pending: false,
                        premium_since: None,
                        permissions: None,
                        avatar: None,
                        communication_disabled_until: None,
                    });
                }
            }
        } else if self.presence.status == OnlineStatus::Offline {
            cache.presences.remove(&self.presence.user.id);
        } else {
            cache.presences.insert(self.presence.user.id, self.presence.clone());
        }

        None
    }
}

impl CacheUpdate for PresencesReplaceEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        for presence in &self.presences {
            cache.presences.insert(presence.user.id, presence.clone());
        }

        None
    }
}

impl CacheUpdate for ReadyEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        let mut ready = self.ready.clone();

        for unavailable in ready.guilds {
            cache.guilds.remove(&unavailable.id);
            cache.unavailable_guilds.insert(unavailable.id);
        }

        // We may be removed from some guilds between disconnect and ready, so we should handle that.
        let mut guilds_to_remove = vec![];
        let ready_guilds_hashset =
            self.ready.guilds.iter().map(|status| status.id).collect::<HashSet<_>>();
        let shard_data = self.ready.shard.unwrap_or([1, 1]);
        for guild_entry in cache.guilds.iter() {
            let guild = guild_entry.key();
            // Only handle data for our shard.
            if crate::utils::shard_id(guild.0, shard_data[1]) == shard_data[0]
                && !ready_guilds_hashset.contains(guild)
            {
                guilds_to_remove.push(*guild);
            }
        }
        if !guilds_to_remove.is_empty() {
            for guild in guilds_to_remove {
                cache.guilds.remove(&guild);
            }
        }

        // `ready.private_channels` will always be empty, and possibly be removed in the future.
        // So don't handle it at all.

        for (user_id, presence) in &mut ready.presences {
            if let Some(user) = presence.user.to_user() {
                cache.update_user_entry(&user);
            }
            if let Some(user) = cache.user(user_id) {
                presence.user.update_with_user(user);
            }

            cache.presences.insert(*user_id, presence.clone());
        }

        *cache.shard_count.write() = ready.shard.map_or(1, |s| s[1]);
        *cache.user.write() = ready.user;

        None
    }
}

impl CacheUpdate for ThreadCreateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let (guild_id, thread_id) = (self.thread.guild_id, self.thread.id);

        cache.guilds.get_mut(&guild_id).and_then(|mut g| {
            if let Some(i) = g.threads.iter().position(|e| e.id == thread_id) {
                Some(std::mem::replace(&mut g.threads[i], self.thread.clone()))
            } else {
                g.threads.push(self.thread.clone());
                None
            }
        })
    }
}

impl CacheUpdate for ThreadUpdateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let (guild_id, thread_id) = (self.thread.guild_id, self.thread.id);

        cache.guilds.get_mut(&guild_id).and_then(|mut g| {
            if let Some(i) = g.threads.iter().position(|e| e.id == thread_id) {
                Some(std::mem::replace(&mut g.threads[i], self.thread.clone()))
            } else {
                g.threads.push(self.thread.clone());
                None
            }
        })
    }
}

impl CacheUpdate for ThreadDeleteEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let (guild_id, thread_id) = (self.thread.guild_id, self.thread.id);

        cache.guilds.get_mut(&guild_id).and_then(|mut g| {
            g.threads.iter().position(|e| e.id == thread_id).map(|i| g.threads.remove(i))
        })
    }
}

impl CacheUpdate for UserUpdateEvent {
    type Output = CurrentUser;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let mut user = cache.user.write();
        Some(std::mem::replace(&mut user, self.current_user.clone()))
    }
}

impl CacheUpdate for VoiceStateUpdateEvent {
    type Output = VoiceState;

    fn update(&mut self, cache: &Cache) -> Option<VoiceState> {
        if let Some(guild_id) = self.voice_state.guild_id {
            if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
                if let Some(member) = &self.voice_state.member {
                    guild.members.insert(member.user.id, member.clone());
                }

                if self.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    guild.voice_states.insert(self.voice_state.user_id, self.voice_state.clone())
                } else {
                    // Remove the user from the voice state list
                    guild.voice_states.remove(&self.voice_state.user_id)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
