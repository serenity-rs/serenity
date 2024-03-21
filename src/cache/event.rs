use std::collections::{HashSet, VecDeque};
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
    ReadyEvent,
    ThreadCreateEvent,
    ThreadDeleteEvent,
    ThreadUpdateEvent,
    UserUpdateEvent,
    VoiceChannelStatusUpdateEvent,
    VoiceStateUpdateEvent,
};
use crate::model::gateway::ShardInfo;
use crate::model::guild::{Guild, GuildMemberFlags, Member, MemberGeneratedFlags, Role};
use crate::model::id::ShardId;
use crate::model::user::{CurrentUser, OnlineStatus};
use crate::model::voice::VoiceState;

impl CacheUpdate for ChannelCreateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        let old_channel = cache
            .guilds
            .get_mut(&self.channel.guild_id)
            .and_then(|mut g| g.channels.insert(self.channel.clone()));

        old_channel
    }
}

impl CacheUpdate for ChannelDeleteEvent {
    type Output = VecDeque<Message>;

    fn update(&mut self, cache: &Cache) -> Option<VecDeque<Message>> {
        let (channel_id, guild_id) = (self.channel.id, self.channel.guild_id);

        cache.guilds.get_mut(&guild_id).map(|mut g| g.channels.remove(&channel_id));

        // Remove the cached messages for the channel.
        cache.messages.remove(&channel_id).map(|(_, messages)| messages)
    }
}

impl CacheUpdate for ChannelUpdateEvent {
    type Output = GuildChannel;

    fn update(&mut self, cache: &Cache) -> Option<GuildChannel> {
        cache
            .guilds
            .get_mut(&self.channel.guild_id)
            .and_then(|mut g| g.channels.insert(self.channel.clone()))
    }
}

impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(guild_id) = self.guild_id {
            if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
                if let Some(mut channel) = guild.channels.get_mut(&self.channel_id) {
                    channel.last_pin_timestamp = self.last_pin_timestamp;
                }
            }
        }

        None
    }
}

impl CacheUpdate for GuildCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.unavailable_guilds.remove(&self.guild.id);
        let guild = self.guild.clone();

        cache.guilds.insert(self.guild.id, guild);

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
                for channel in &guild.1.channels {
                    // Remove the channel's cached messages.
                    cache.messages.remove(&channel.id);
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
        if let Some(mut guild) = cache.guilds.get_mut(&self.member.guild_id) {
            guild.member_count += 1;
            guild.members.insert(self.member.clone());
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
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            let item = if let Some(mut member) = guild.members.get_mut(&self.user.id) {
                let item = Some(member.clone());

                member.joined_at.clone_from(&Some(self.joined_at));
                member.nick.clone_from(&self.nick);
                member.roles.clone_from(&self.roles);
                member.user.clone_from(&self.user);
                member.premium_since.clone_from(&self.premium_since);
                member.avatar.clone_from(&self.avatar);
                member.communication_disabled_until.clone_from(&self.communication_disabled_until);
                member.unusual_dm_activity_until.clone_from(&self.unusual_dm_activity_until);
                member.set_pending(self.pending());
                member.set_deaf(self.deaf());
                member.set_mute(self.mute());

                item
            } else {
                None
            };

            if item.is_none() {
                let mut new_member = Member {
                    __generated_flags: MemberGeneratedFlags::empty(),
                    guild_id: self.guild_id,
                    joined_at: Some(self.joined_at),
                    nick: self.nick.clone(),
                    roles: self.roles.clone(),
                    user: self.user.clone(),
                    premium_since: self.premium_since,
                    permissions: None,
                    avatar: self.avatar,
                    communication_disabled_until: self.communication_disabled_until,
                    flags: GuildMemberFlags::default(),
                    unusual_dm_activity_until: self.unusual_dm_activity_until,
                };

                new_member.set_pending(self.pending());
                new_member.set_deaf(self.deaf());
                new_member.set_mute(self.mute());

                guild.members.insert(new_member);
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
        if let Some(mut g) = cache.guilds.get_mut(&self.guild_id) {
            g.members.extend(self.members.clone());
        }

        None
    }
}

impl CacheUpdate for GuildRoleCreateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        cache.guilds.get_mut(&self.role.guild_id).map(|mut g| g.roles.insert(self.role.clone()));
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
            if let Some(mut role) = guild.roles.get_mut(&self.role.id) {
                return Some(std::mem::replace(&mut *role, self.role.clone()));
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
            guild.set_widget_enabled(self.guild.widget_enabled());
        }

        None
    }
}

impl CacheUpdate for MessageCreateEvent {
    /// The oldest message, if the channel's message cache was already full.
    type Output = Message;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        // Update the relevant channel object with the new latest message if this message is newer
        let guild = self.message.guild_id.and_then(|g_id| cache.guilds.get_mut(&g_id));

        if let Some(mut guild) = guild {
            let mut found_channel = false;
            if let Some(mut channel) = guild.channels.get_mut(&self.message.channel_id) {
                update_channel_last_message_id(&self.message, &mut channel, cache);
                found_channel = true;
            }

            // found_channel is to avoid limitations of the NLL borrow checker.
            if !found_channel {
                // This may be a thread.
                let thread =
                    guild.threads.iter_mut().find(|thread| thread.id == self.message.channel_id);
                if let Some(thread) = thread {
                    update_channel_last_message_id(&self.message, thread, cache);
                }
            }
        }

        // Add the new message to the cache and remove the oldest cached message.
        let max = cache.settings().max_messages;

        if max == 0 {
            return None;
        }

        let mut messages = cache.messages.entry(self.message.channel_id).or_default();

        let mut removed_msg = None;
        if messages.len() == max {
            removed_msg = messages.pop_front();
        }

        if !messages.iter().any(|m| m.id == self.message.id) {
            messages.push_back(self.message.clone());
        }

        removed_msg
    }
}

fn update_channel_last_message_id(message: &Message, channel: &mut GuildChannel, cache: &Cache) {
    if let Some(last_message_id) = channel.last_message_id {
        let most_recent_timestamp = cache.message(channel.id, last_message_id).map(|m| m.timestamp);
        if let Some(most_recent_timestamp) = most_recent_timestamp {
            if message.timestamp > most_recent_timestamp {
                channel.last_message_id = Some(message.id);
            }
        } else {
            channel.last_message_id = Some(message.id);
        }
    } else {
        channel.last_message_id = Some(message.id);
    }
}

impl CacheUpdate for MessageUpdateEvent {
    type Output = Message;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        for message in cache.messages.get_mut(&self.channel_id)?.iter_mut() {
            if message.id == self.id {
                let old_message = message.clone();
                self.apply_to_message(message);
                return Some(old_message);
            }
        }

        None
    }
}

impl CacheUpdate for PresenceUpdateEvent {
    type Output = ();

    fn update(&mut self, cache: &Cache) -> Option<()> {
        if let Some(guild_id) = self.presence.guild_id {
            if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
                // If the member went offline, remove them from the presence list.
                if self.presence.status == OnlineStatus::Offline {
                    guild.presences.remove(&self.presence.user.id);
                } else {
                    guild.presences.insert(self.presence.clone());
                }

                // Create a partial member instance out of the presence update data.
                if let Some(user) = self.presence.user.to_user() {
                    if !guild.members.contains_key(&self.presence.user.id) {
                        guild.members.insert(Member {
                            guild_id,
                            joined_at: None,
                            nick: None,
                            user,
                            roles: FixedArray::default(),
                            premium_since: None,
                            permissions: None,
                            avatar: None,
                            communication_disabled_until: None,
                            flags: GuildMemberFlags::default(),
                            unusual_dm_activity_until: None,
                            __generated_flags: MemberGeneratedFlags::empty(),
                        });
                    }
                }
            }
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
            if crate::utils::shard_id(*guild, shard_data.total) == shard_data.id.0
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
                Some(std::mem::replace(&mut g.threads[i as u32], self.thread.clone()))
            } else {
                // This is a rare enough occurence to realloc.
                let mut threads = std::mem::take(&mut g.threads).into_vec();
                threads.push(self.thread.clone());

                g.threads = FixedArray::try_from(threads.into_boxed_slice())
                    .expect("A guild should not have 4 billion threads");

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
                Some(std::mem::replace(&mut g.threads[i as u32], self.thread.clone()))
            } else {
                // This is a rare enough occurence to realloc.
                let mut threads = std::mem::take(&mut g.threads).into_vec();
                threads.push(self.thread.clone());

                g.threads = FixedArray::try_from(threads.into_boxed_slice())
                    .expect("A guild should not have 4 billion threads");

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

                g.threads = FixedArray::try_from(threads.into_boxed_slice())
                    .expect("A guild should not have 4 billion threads");

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
                    guild.members.insert(member.clone());
                }

                if self.voice_state.channel_id.is_some() {
                    // Update or add to the voice state list
                    let old_state = guild.voice_states.remove(&self.voice_state.user_id);
                    guild.voice_states.insert(self.voice_state.clone());
                    old_state
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
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let mut channel = guild.channels.get_mut(&self.id)?;

        let old = channel.status.clone();
        channel.status.clone_from(&self.status);
        old
    }
}
