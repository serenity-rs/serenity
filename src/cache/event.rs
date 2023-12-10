use std::collections::HashSet;
use std::num::NonZeroU16;

use super::{Cache, CacheUpdate};
use crate::internal::prelude::*;
use crate::model::channel::{GuildChannel, Message};
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
    VoiceChannelStatusUpdateEvent,
    VoiceStateUpdateEvent,
};
use crate::model::gateway::ShardInfo;
use crate::model::guild::{Guild, GuildMemberFlags, Member, Role};
use crate::model::id::ShardId;
use crate::model::user::{CurrentUser, OnlineStatus};
use crate::model::voice::VoiceState;

impl CacheUpdate for ChannelCreateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let old_channel = cache
            .guilds
            .get_mut(&self.channel.guild_id)
            .and_then(|mut g| g.channels.insert(self.channel.id, self.channel.clone()));

        cache.channels.insert(self.channel.id, self.channel.guild_id);
        old_channel
    }
}

impl CacheUpdate for ChannelDeleteEvent {
    type Output = Vec<Message>;

    fn update(&mut self, cache: &Cache) -> Option<Vec<Message>> {
        let (channel_id, guild_id) = (self.channel.id, self.channel.guild_id);

        cache.channels.remove(&channel_id);
        cache.guilds.get_mut(&guild_id).map(|mut g| g.channels.remove(&channel_id));

        // Remove the cached messages for the channel.
        cache.messages.remove(&channel_id).map(|(_, messages)| messages.into_values().collect())
    }
}

impl CacheUpdate for ChannelUpdateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<GuildChannel> {
        cache.channels.insert(self.channel.id, self.channel.guild_id);

        cache
            .guilds
            .get_mut(&self.channel.guild_id)
            .and_then(|mut g| g.channels.insert(self.channel.id, self.channel.clone()))
    }
}

impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut channel) = cache.channel_mut(self.channel_id) {
            channel.last_pin_timestamp = self.last_pin_timestamp;
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
                member.user = u.clone();
            }
        }

        cache.guilds.insert(self.guild.id, guild);
        for channel_id in self.guild.channels.keys() {
            cache.channels.insert(*channel_id, self.guild.id);
        }

        None
    }
}

impl CacheUpdate for GuildDeleteEvent {
    type Output = Guild;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if self.guild.unavailable {
            cache.unavailable_guilds.insert(self.guild.id, ());
            cache.guilds.remove(&self.guild.id);

            return None;
        }

        match cache.guilds.remove(&self.guild.id) {
            Some(guild) => {
                for channel_id in guild.1.channels.keys() {
                    // Remove the channel from the cache.
                    cache.channels.remove(channel_id);

                    // Remove the channel's cached messages.
                    cache.messages.remove(channel_id);
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
            self.member.user = u.clone();
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
                    avatar: self.avatar,
                    communication_disabled_until: self.communication_disabled_until,
                    flags: GuildMemberFlags::default(),
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

impl CacheUpdate for GuildUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild.id) {
            guild.afk_metadata.clone_from(&self.guild.afk_metadata);
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

        let mut messages = cache.messages.entry(self.message.channel_id).or_default();
        let mut queue = cache.message_queue.entry(self.message.channel_id).or_default();

        let mut removed_msg = None;

        if messages.len() == max {
            if let Some(id) = queue.pop_front() {
                removed_msg = messages.remove(&id);
            }
        }

        queue.push_back(self.message.id);
        messages.insert(self.message.id, self.message.clone());

        removed_msg
    }
}

impl CacheUpdate for MessageUpdateEvent {
    type Output = Message;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let mut messages = cache.messages.get_mut(&self.channel_id)?;
        let message = messages.get_mut(&self.id)?;
        let old_message = message.clone();

        self.apply_to_message(message);

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
            self.presence.user.update_with_user(&user);
        }

        if let Some(guild_id) = self.presence.guild_id {
            if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user.id);
                } else {
                    guild.presences.insert(self.presence.user.id, self.presence.clone());
                }

                // Create a partial member instance out of the presence update data.
                if let Some(user) = self.presence.user.to_user() {
                    guild.members.entry(self.presence.user.id).or_insert_with(|| Member {
                        deaf: false,
                        guild_id,
                        joined_at: None,
                        mute: false,
                        nick: None,
                        user,
                        roles: FixedArray::default(),
                        pending: false,
                        premium_since: None,
                        permissions: None,
                        avatar: None,
                        communication_disabled_until: None,
                        flags: GuildMemberFlags::default(),
                    });
                }
            }
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
        for unavailable in &self.ready.guilds {
            cache.guilds.remove(&unavailable.id);
            cache.unavailable_guilds.insert(unavailable.id, ());
        }

        // We may be removed from some guilds between disconnect and ready, so handle that.
        let mut guilds_to_remove = vec![];
        let ready_guilds_hashset =
            self.ready.guilds.iter().map(|status| status.id).collect::<HashSet<_>>();
        let shard_data =
            self.ready.shard.unwrap_or_else(|| ShardInfo::new(ShardId(1), NonZeroU16::MIN));

        for guild_entry in cache.guilds.iter() {
            let guild = guild_entry.key();
            // Only handle data for our shard.
            if crate::utils::shard_id(*guild, shard_data.total.get()) == shard_data.id.0
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

        {
            let mut cached_shard_data = cache.shard_data.write();
            cached_shard_data.total = shard_data.total;
            cached_shard_data.connected.insert(shard_data.id);
        }
        cache.user.write().clone_from(&self.ready.user);

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
                // This is a rare enough occurence to realloc.
                let mut threads = std::mem::take(&mut g.threads).into_vec();
                threads.push(self.thread.clone());
                g.threads = threads.into();

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
                // This is a rare enough occurence to realloc.
                let mut threads = std::mem::take(&mut g.threads).into_vec();
                threads.push(self.thread.clone());
                g.threads = threads.into();

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
            g.threads.iter().position(|e| e.id == thread_id).map(|i| {
                let mut threads = std::mem::take(&mut g.threads).into_vec();
                let thread = threads.remove(i);
                g.threads = threads.into();

                thread
            })
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

impl CacheUpdate for VoiceChannelStatusUpdateEvent {
    type Output = FixedString<u16>;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut channel) = cache.channel_mut(self.id) {
            let old = channel.status.clone();
            channel.status = self.status.clone();
            // Discord updates topic but doesn't fire ChannelUpdate.
            channel.topic = self.status.clone();
            old
        } else {
            None
        }
    }
}
