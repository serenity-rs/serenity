use std::collections::VecDeque;
use std::num::NonZeroU16;

use extract_map::entry::Entry;

use super::{Cache, CacheUpdate};
use crate::model::prelude::*;

impl CacheUpdate for ChannelCreateEvent {
    type Output = GuildChannel;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.channel.base.guild_id).and_then(|mut g| {
            if self.channel.flags.contains(ChannelFlags::CHANNEL_OBFUSCATED) {
                g.obfuscated_channels.insert((&self.channel).into()).map(Into::into)
            } else {
                g.channels.insert(self.channel.clone())
            }
        })
    }
}

impl CacheUpdate for ChannelDeleteEvent {
    type Output = VecDeque<Message>;

    fn update(&self, cache: &Cache) -> Option<VecDeque<Message>> {
        let (channel_id, guild_id) = (self.channel.id, self.channel.base.guild_id);

        if let Some(mut guild) = cache.guilds.get_mut(&guild_id)
            && guild.channels.remove(&channel_id).is_none()
        {
            guild.obfuscated_channels.remove(&channel_id);
        }

        // Remove the cached messages for the channel.
        cache.messages.remove(&channel_id.widen()).map(|(_, messages)| messages)
    }
}

impl CacheUpdate for ChannelUpdateEvent {
    type Output = GuildChannel;

    fn update(&self, cache: &Cache) -> Option<GuildChannel> {
        cache.guilds.get_mut(&self.channel.base.guild_id).and_then(|mut g| {
            if self.channel.flags.contains(ChannelFlags::CHANNEL_OBFUSCATED) {
                g.obfuscated_channels
                    .insert((&self.channel).into())
                    .map(Into::into)
                    .or_else(|| g.channels.remove(&self.channel.id))
            } else {
                g.channels
                    .insert(self.channel.clone())
                    .or_else(|| g.obfuscated_channels.remove(&self.channel.id).map(Into::into))
            }
        })
    }
}

impl CacheUpdate for ChannelInfoEvent {
    type Output = Vec<ChannelInfoChannel>;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut old: Vec<ChannelInfoChannel> = Vec::new();
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;

        for channel_info_channel in &self.channels {
            let mut channel = guild.channels.get_mut(&channel_info_channel.id)?;
            let old_status = std::mem::take(&mut channel.status);
            let old_voice_start_time = channel.voice_start_time;
            channel.status.clone_from(&channel_info_channel.status);
            channel.voice_start_time.clone_from(&channel_info_channel.voice_start_time);
            old.push(ChannelInfoChannel {
                id: channel_info_channel.id,
                status: old_status,
                voice_start_time: old_voice_start_time,
            });
        }

        if old.is_empty() { None } else { Some(old) }
    }
}

impl CacheUpdate for ChannelPinsUpdateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(guild_id) = self.guild_id
            && let Some(mut guild) = cache.guilds.get_mut(&guild_id)
        {
            let (channel_id, thread_id) = self.channel_id.split();
            if let Some(mut channel) = guild.channels.get_mut(&channel_id) {
                channel.base.last_pin_timestamp = self.last_pin_timestamp;
                return None;
            }

            if let Some(mut thread) = guild.threads.get_mut(&thread_id) {
                thread.base.last_pin_timestamp = self.last_pin_timestamp;
            }
        }

        None
    }
}

impl CacheUpdate for GuildCreateEvent {
    type Output = Vec<GuildId>;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache.unavailable_guilds.remove(&self.guild.id);
        let guild = self.guild.clone();

        cache.guilds.insert(self.guild.id, guild);

        if cache.unavailable_guilds.len() == 0 {
            cache.unavailable_guilds.shrink_to_fit();
            Some(cache.guilds.iter().map(|i| *i.key()).collect())
        } else {
            None
        }
    }
}

impl CacheUpdate for GuildDeleteEvent {
    type Output = Guild;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if self.guild.unavailable {
            cache.unavailable_guilds.insert(self.guild.id, ());
            cache.guilds.remove(&self.guild.id);

            return None;
        }

        match cache.guilds.remove(&self.guild.id) {
            Some(guild) => {
                for channel in &guild.1.channels {
                    // Remove the channel's cached messages.
                    cache.messages.remove(&channel.id.widen());
                }

                Some(guild.1)
            },
            None => None,
        }
    }
}

impl CacheUpdate for GuildEmojisUpdateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.emojis.clone_from(&self.emojis);
        }

        None
    }
}

impl CacheUpdate for GuildMemberAddEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.member.guild_id) {
            guild.member_count = MemberCount::new(guild.member_count.get() + 1)
                .expect("member count should not overflow");
            guild.members.insert(self.member.clone());
        }

        None
    }
}

impl CacheUpdate for GuildMemberRemoveEvent {
    type Output = Member;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.member_count = MemberCount::new(guild.member_count.get() - 1)
                .expect("member count should not underflow");
            return guild.members.remove(&self.user.id);
        }

        None
    }
}

impl CacheUpdate for GuildMemberUpdateEvent {
    type Output = Member;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let old_member = guild.members.get_mut(&self.user.id).and_then(|mut member| {
            let old_member = Some(member.clone());

            member.joined_at.clone_from(&self.joined_at);
            member.nick.clone_from(&self.nick);
            member.roles.clone_from(&self.roles);
            member.user.clone_from(&self.user);
            member.premium_since.clone_from(&self.premium_since);
            member.avatar.clone_from(&self.avatar);
            member.banner.clone_from(&self.banner);
            member.communication_disabled_until.clone_from(&self.communication_disabled_until);
            member.unusual_dm_activity_until.clone_from(&self.unusual_dm_activity_until);
            member.collectibles.clone_from(&self.collectibles);
            member.set_pending(self.pending());
            member.set_deaf(self.deaf());
            member.set_mute(self.mute());

            old_member
        });

        if self.user.id == cache.current_user().id
            && let Some(old_member) = &old_member
        {
            let mut to_obfuscate: Vec<ChannelId> = Vec::new();
            if let Some(new_member) = guild.members.get(&self.user.id)
                && old_member.roles.iter().any(|role| !new_member.roles.contains(role))
            {
                for channel in &guild.channels {
                    if !guild.user_permissions_in(channel, new_member).view_channel() {
                        to_obfuscate.push(channel.id);
                    }
                }
            }
            for id in to_obfuscate {
                if let Some(channel) = guild.channels.remove(&id) {
                    guild.obfuscated_channels.insert(channel.into());
                }
            }
        }

        if old_member.is_none() {
            let mut new_member = Member {
                __generated_flags: MemberGeneratedFlags::empty(),
                guild_id: self.guild_id,
                joined_at: self.joined_at,
                nick: self.nick.clone(),
                roles: self.roles.clone(),
                user: self.user.clone(),
                premium_since: self.premium_since,
                permissions: None,
                avatar: self.avatar,
                banner: self.banner,
                communication_disabled_until: self.communication_disabled_until,
                flags: self.flags.unwrap_or_default(),
                unusual_dm_activity_until: self.unusual_dm_activity_until,
                avatar_decoration_data: self.avatar_decoration_data,
                collectibles: self.collectibles.clone(),
            };

            new_member.set_pending(self.pending());
            new_member.set_deaf(self.deaf());
            new_member.set_mute(self.mute());

            guild.members.insert(new_member);
        }

        old_member
    }
}

impl CacheUpdate for GuildMembersChunkEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut g) = cache.guilds.get_mut(&self.guild_id) {
            g.members.extend(self.members.clone());
        }

        None
    }
}

impl CacheUpdate for GuildRoleCreateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.role.guild_id).map(|mut g| g.roles.insert(self.role.clone()));
        None
    }
}

impl CacheUpdate for GuildRoleDeleteEvent {
    type Output = Role;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache.guilds.get_mut(&self.guild_id).and_then(|mut g| g.roles.remove(&self.role_id))
    }
}

impl CacheUpdate for GuildRoleUpdateEvent {
    type Output = Role;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.role.guild_id)?;
        let old_role = guild
            .roles
            .get_mut(&self.role.id)
            .map(|mut role| std::mem::replace(&mut *role, self.role.clone()));

        if let Some(old_role) = &old_role
            && let Some(member) = guild.members.get(&cache.current_user().id)
            && member.roles.contains(&self.role.id)
            && (old_role.permissions.view_channel() && !self.role.permissions.view_channel()
                || old_role.permissions.administrator() && !self.role.permissions.administrator())
        {
            let mut to_obfuscate: Vec<ChannelId> = Vec::new();
            for channel in &guild.channels {
                if !guild.user_permissions_in(channel, member).view_channel() {
                    to_obfuscate.push(channel.id);
                }
            }
            for id in to_obfuscate {
                if let Some(channel) = guild.channels.remove(&id) {
                    guild.obfuscated_channels.insert(channel.into());
                }
            }
        }

        old_role
    }
}

impl CacheUpdate for GuildStickersUpdateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild_id) {
            guild.stickers.clone_from(&self.stickers);
        }

        None
    }
}

impl CacheUpdate for GuildUpdateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        if let Some(mut guild) = cache.guilds.get_mut(&self.guild.id) {
            guild.afk_metadata.clone_from(&self.guild.afk_metadata);
            guild.banner.clone_from(&self.guild.banner);
            guild.description.clone_from(&self.guild.description);
            guild.discovery_splash.clone_from(&self.guild.discovery_splash);
            guild.emojis.clone_from(&self.guild.emojis);
            guild.features.clone_from(&self.guild.features);
            guild.icon.clone_from(&self.guild.icon);
            guild.icon_hash.clone_from(&self.guild.icon_hash);
            guild.name.clone_from(&self.guild.name);
            guild.owner_id.clone_from(&self.guild.owner_id);
            guild.preferred_locale.clone_from(&self.guild.preferred_locale);
            guild.roles.clone_from(&self.guild.roles);
            guild.splash.clone_from(&self.guild.splash);
            guild.stickers.clone_from(&self.guild.stickers);
            guild.vanity_url_code.clone_from(&self.guild.vanity_url_code);
            guild.welcome_screen.clone_from(&self.guild.welcome_screen);
            guild.application_id = self.guild.application_id;
            guild.approximate_member_count = self.guild.approximate_member_count;
            guild.approximate_presence_count = self.guild.approximate_presence_count;
            guild.default_message_notifications = self.guild.default_message_notifications;
            guild.explicit_content_filter = self.guild.explicit_content_filter;
            guild.max_members = self.guild.max_members;
            guild.max_presences = self.guild.max_presences;
            guild.max_video_channel_users = self.guild.max_video_channel_users;
            guild.max_stage_video_channel_users = self.guild.max_stage_video_channel_users;
            guild.mfa_level = self.guild.mfa_level;
            guild.nsfw_level = self.guild.nsfw_level;
            guild.set_premium_progress_bar_enabled(self.guild.premium_progress_bar_enabled());
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

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        // Update the relevant channel object with the new latest message if this message is newer
        let guild = self.message.guild_id.and_then(|g_id| cache.guilds.get_mut(&g_id));

        if let Some(mut guild) = guild {
            let shared_id = self.message.channel_id;
            let (channel_id, thread_id) = shared_id.split();
            if let Some(mut channel) = guild.channels.get_mut(&channel_id) {
                update_channel_last_message_id(&self.message, &mut channel.base, shared_id, cache);
            }

            if let Some(mut thread) = guild.threads.get_mut(&thread_id) {
                update_channel_last_message_id(&self.message, &mut thread.base, shared_id, cache);
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

fn update_channel_last_message_id(
    message: &Message,
    channel: &mut BaseGuildChannel,
    channel_id: GenericChannelId,
    cache: &Cache,
) {
    if let Some(last_message_id) = channel.last_message_id {
        let most_recent_timestamp = cache.message(channel_id, last_message_id).map(|m| m.timestamp);
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

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        for message in cache.messages.get_mut(&self.message.channel_id)?.iter_mut() {
            if message.id == self.message.id {
                let old_message = message.clone();
                message.clone_from(&self.message);
                return Some(old_message);
            }
        }

        None
    }
}

impl CacheUpdate for PresenceUpdateEvent {
    type Output = Presence;

    fn update(&self, cache: &Cache) -> Option<Presence> {
        if let Some(guild_id) = self.presence.guild_id
            && let Some(mut guild) = cache.guilds.get_mut(&guild_id)
        {
            let old = guild.presences.get(&self.presence.user.id).cloned();

            // If the member went offline, remove them from the presence list.
            if self.presence.status == OnlineStatus::Offline {
                guild.presences.remove(&self.presence.user.id);
            } else {
                guild.presences.insert(self.presence.clone());
            }

            // Create a partial member instance out of the presence update data.
            if let Some(user) = self.presence.user.to_user()
                && !guild.members.contains_key(&self.presence.user.id)
            {
                guild.members.insert(Member {
                    guild_id,
                    joined_at: None,
                    nick: None,
                    user,
                    roles: FixedArray::default(),
                    premium_since: None,
                    permissions: None,
                    avatar: None,
                    banner: None,
                    communication_disabled_until: None,
                    flags: GuildMemberFlags::default(),
                    unusual_dm_activity_until: None,
                    avatar_decoration_data: None,
                    collectibles: Collectibles {
                        nameplate: None,
                    },
                    __generated_flags: MemberGeneratedFlags::empty(),
                });
            }

            return old;
        }

        None
    }
}

impl CacheUpdate for ReadyEvent {
    type Output = NonZeroU16;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        for unavailable in &self.ready.guilds {
            cache.guilds.remove(&unavailable.id);
            cache.unavailable_guilds.insert(unavailable.id, ());
        }

        let shard_info = self.ready.shard.unwrap_or_default();

        cache.user.write().clone_from(&self.ready.user);

        let mut shards = cache.shard_data.write();
        shards.total = shard_info.total;
        shards.connected.insert(shard_info.id);

        if shards.connected.len() == shards.total.get() as usize && !shards.has_sent_shards_ready {
            shards.has_sent_shards_ready = true;
            Some(shards.total)
        } else {
            None
        }
    }
}

impl CacheUpdate for ThreadCreateEvent {
    type Output = GuildThread;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache
            .guilds
            .get_mut(&self.thread.base.guild_id)
            .and_then(|mut g| g.threads.insert(self.thread.clone()))
    }
}

impl CacheUpdate for ThreadUpdateEvent {
    type Output = GuildThread;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache
            .guilds
            .get_mut(&self.thread.base.guild_id)
            .and_then(|mut g| g.threads.insert(self.thread.clone()))
    }
}

impl CacheUpdate for ThreadDeleteEvent {
    type Output = GuildThread;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        cache
            .guilds
            .get_mut(&self.thread.guild_id)
            .and_then(|mut g| g.threads.remove(&self.thread.id))
    }
}

impl CacheUpdate for ThreadListSyncEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let Some(channel_ids) = &self.channel_ids else {
            // channel_ids is none, this is a full sync, easy path
            guild.threads.clone_from(&self.threads);
            return None;
        };

        // Add new threads and update existing threads.
        for new_thread in &self.threads {
            match guild.threads.entry(&new_thread.id) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().clone_from(new_thread);
                },
                Entry::Vacant(entry) => {
                    entry.insert(new_thread.clone());
                },
            }
        }

        // Remove threads which are not provided in the sync and are in the provided channels.
        let mut removed_threads = Vec::new();
        for &channel_id in channel_ids {
            for thread in &guild.threads {
                if thread.parent_id != channel_id {
                    continue;
                }

                if !self.threads.contains_key(&thread.id) {
                    removed_threads.push(thread.id);
                }
            }
        }

        for to_remove in removed_threads {
            guild.threads.remove(&to_remove);
        }

        None
    }
}

impl CacheUpdate for UserUpdateEvent {
    type Output = CurrentUser;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut user = cache.user.write();
        Some(std::mem::replace(&mut user, self.current_user.clone()))
    }
}

impl CacheUpdate for VoiceStateUpdateEvent {
    type Output = VoiceState;

    fn update(&self, cache: &Cache) -> Option<VoiceState> {
        let guild_id = self.voice_state.guild_id?;
        let mut guild = cache.guilds.get_mut(&guild_id)?;

        if let Some(member) = &self.voice_state.member {
            guild.members.insert(member.clone());
        }

        let old_state = guild.voice_states.remove(&self.voice_state.user_id);
        if self.voice_state.channel_id.is_some() {
            guild.voice_states.insert(self.voice_state.clone());
        }

        if self.voice_state.user_id == cache.current_user().id
            && let Some(old_state) = &old_state
            && let Some(channel_id) = &old_state.channel_id
            && let Some(channel) = guild.channels.get(channel_id)
            && let Some(member) = guild.members.get(&self.voice_state.user_id)
            && !guild.user_permissions_in(channel, member).view_channel()
            && let Some(removed_channel) = guild.channels.remove(channel_id)
        {
            guild.obfuscated_channels.insert(removed_channel.into());
        }

        old_state
    }
}

impl CacheUpdate for VoiceChannelStartTimeUpdateEvent {
    type Output = i64;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let mut channel = guild.channels.get_mut(&self.id)?;

        let old = channel.voice_start_time;
        channel.voice_start_time.clone_from(&self.voice_start_time);
        old
    }
}

impl CacheUpdate for VoiceChannelStatusUpdateEvent {
    type Output = FixedString<u16>;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let mut channel = guild.channels.get_mut(&self.id)?;

        let old = if channel.status.as_ref().is_some_and(FixedString::is_empty) {
            None
        } else {
            channel.status.clone()
        };
        channel.status.clone_from(&self.status);
        old
    }
}

fn update_guild_event(cache: &Cache, event: &ScheduledEvent) -> Option<ScheduledEvent> {
    let mut guild = cache.guilds.get_mut(&event.guild_id)?;
    guild.scheduled_events.insert(event.clone())
}

impl CacheUpdate for GuildScheduledEventCreateEvent {
    type Output = std::convert::Infallible;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        update_guild_event(cache, &self.event);
        None
    }
}

impl CacheUpdate for GuildScheduledEventUpdateEvent {
    type Output = ScheduledEvent;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        update_guild_event(cache, &self.event)
    }
}

impl CacheUpdate for GuildScheduledEventDeleteEvent {
    type Output = ScheduledEvent;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut guild = cache.guilds.get_mut(&self.event.guild_id)?;
        guild.scheduled_events.remove(&self.event.id)
    }
}

impl CacheUpdate for ReactionAddEvent {
    type Output = Message;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let reaction = &self.reaction;
        let mut messages = cache.messages.get_mut(&reaction.channel_id)?;

        for message in messages.iter_mut() {
            if message.id != reaction.message_id {
                continue;
            }

            let prev = message.clone();

            if let Some(existing) =
                message.reactions.iter_mut().find(|r| r.reaction_type == reaction.emoji)
            {
                existing.count += 1;
                if reaction.burst {
                    existing.count_details.burst += 1;
                } else {
                    existing.count_details.normal += 1;
                }
                return Some(prev);
            }

            let me = self.reaction.user_id == Some(cache.current_user().id);
            let new_reaction = MessageReaction {
                me,
                burst_colours: reaction.burst_colours.clone().unwrap_or_default(),
                count: 1,
                count_details: CountDetails {
                    burst: u64::from(reaction.burst),
                    normal: u64::from(!reaction.burst),
                },
                me_burst: if me { reaction.burst } else { false },
                reaction_type: reaction.emoji.clone(),
            };

            message.reactions.push(new_reaction);

            return Some(prev);
        }

        None
    }
}

impl CacheUpdate for ReactionRemoveEvent {
    type Output = Message;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let reaction = &self.reaction;
        let mut messages = cache.messages.get_mut(&reaction.channel_id)?;

        for message in messages.iter_mut() {
            if message.id != reaction.message_id {
                continue;
            }

            let old_message = message.clone();

            let index = message.reactions.iter().position(|r| r.reaction_type == reaction.emoji)?;

            let existing_reaction = &mut message.reactions[index];

            existing_reaction.count -= 1;
            if reaction.burst {
                existing_reaction.count_details.burst -= 1;
            } else {
                existing_reaction.count_details.normal -= 1;
            }

            if existing_reaction.count == 0 {
                message.reactions.remove(index);
            }

            return Some(old_message);
        }

        None
    }
}

impl CacheUpdate for ReactionRemoveEmojiEvent {
    type Output = Message;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let reaction = &self.reaction;
        let mut messages = cache.messages.get_mut(&reaction.channel_id)?;

        for message in messages.iter_mut() {
            if message.id != reaction.message_id {
                continue;
            }

            let old_message = message.clone();

            let index = message.reactions.iter().position(|r| r.reaction_type == reaction.emoji)?;

            message.reactions.remove(index);

            return Some(old_message);
        }

        None
    }
}

impl CacheUpdate for ReactionRemoveAllEvent {
    type Output = Message;

    fn update(&self, cache: &Cache) -> Option<Self::Output> {
        let mut messages = cache.messages.get_mut(&self.channel_id)?;

        for message in messages.iter_mut() {
            if message.id != self.message_id {
                continue;
            }

            let old_message = message.clone();

            message.reactions.clear();

            return Some(old_message);
        }

        None
    }
}
