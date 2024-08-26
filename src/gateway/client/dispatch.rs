use std::sync::Arc;

use super::event_handler::{EventHandler, RawEventHandler};
use super::{Context, FullEvent};
#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheUpdate};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::prelude::*;
use crate::model::channel::ChannelType;
use crate::model::event::Event;
use crate::model::guild::Member;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;

#[cfg(feature = "cache")]
macro_rules! if_cache {
    ($e:expr) => {
        $e
    };
}

#[cfg(not(feature = "cache"))]
macro_rules! if_cache {
    ($e:expr) => {
        None
    };
}

#[cfg(feature = "cache")]
macro_rules! update_cache {
    ($cache:ident, $event:ident) => {
        $event.update($cache)
    };
}

#[cfg(not(feature = "cache"))]
macro_rules! update_cache {
    ($cache:ident, $event:ident) => {};
}

/// Calls the user's event handlers and the framework handler.
///
/// This MUST be called from a different task to the recv_event loop, to allow for
/// intra-shard concurrency between the shard loop and event handler.
pub(crate) async fn dispatch_model(
    event: Event,
    context: Context,
    #[cfg(feature = "framework")] framework: Option<Arc<dyn Framework>>,
    event_handler: Option<Arc<dyn EventHandler>>,
    raw_event_handler: Option<Arc<dyn RawEventHandler>>,
) {
    if let Some(raw_handler) = raw_event_handler {
        raw_handler.raw_event(context.clone(), &event).await;
    }

    let (full_event, extra_event) = update_cache_with_event(
        #[cfg(feature = "cache")]
        &context.cache,
        event,
    );

    #[cfg(feature = "framework")]
    if let Some(framework) = framework {
        if let Some(extra_event) = &extra_event {
            framework.dispatch(&context, extra_event).await;
        }

        framework.dispatch(&context, &full_event).await;
    }

    if let Some(handler) = event_handler {
        if let Some(extra_event) = extra_event {
            extra_event.dispatch(context.clone(), &*handler).await;
        }

        full_event.dispatch(context, &*handler).await;
    }
}

/// Updates the cache with the incoming event data and builds the full event data out of it.
///
/// Can return a secondary [`FullEvent`] for "virtual" events like [`FullEvent::CacheReady`] or
/// [`FullEvent::ShardsReady`]. Secondary events are traditionally dispatched first.
///
/// Can return `None` if an event is unknown.
#[cfg_attr(not(feature = "cache"), allow(unused_mut))]
fn update_cache_with_event(
    #[cfg(feature = "cache")] cache: &Cache,
    event: Event,
) -> (FullEvent, Option<FullEvent>) {
    let mut extra_event = None;
    let event = match event {
        Event::CommandPermissionsUpdate(event) => FullEvent::CommandPermissionsUpdate {
            permission: event.permission,
        },
        Event::AutoModRuleCreate(event) => FullEvent::AutoModRuleCreate {
            rule: event.rule,
        },
        Event::AutoModRuleUpdate(event) => FullEvent::AutoModRuleUpdate {
            rule: event.rule,
        },
        Event::AutoModRuleDelete(event) => FullEvent::AutoModRuleDelete {
            rule: event.rule,
        },
        Event::AutoModActionExecution(event) => FullEvent::AutoModActionExecution {
            execution: event.execution,
        },
        Event::ChannelCreate(mut event) => {
            update_cache!(cache, event);

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryCreate {
                    category: channel,
                }
            } else {
                FullEvent::ChannelCreate {
                    channel,
                }
            }
        },
        Event::ChannelDelete(mut event) => {
            let cached_messages = if_cache!(event.update(cache));

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryDelete {
                    category: channel,
                }
            } else {
                FullEvent::ChannelDelete {
                    channel,
                    messages: cached_messages,
                }
            }
        },
        Event::ChannelPinsUpdate(event) => FullEvent::ChannelPinsUpdate {
            pin: event,
        },
        Event::ChannelUpdate(mut event) => {
            let old_channel = if_cache!(event.update(cache));

            FullEvent::ChannelUpdate {
                old: old_channel,
                new: event.channel,
            }
        },
        Event::GuildAuditLogEntryCreate(event) => FullEvent::GuildAuditLogEntryCreate {
            entry: event.entry,
            guild_id: event.guild_id,
        },
        Event::GuildBanAdd(event) => FullEvent::GuildBanAddition {
            guild_id: event.guild_id,
            banned_user: event.user,
        },
        Event::GuildBanRemove(event) => FullEvent::GuildBanRemoval {
            guild_id: event.guild_id,
            unbanned_user: event.user,
        },
        Event::GuildCreate(mut event) => {
            let is_new = if_cache!(Some(!cache.unavailable_guilds.contains(&event.guild.id)));

            update_cache!(cache, event);

            #[cfg(feature = "cache")]
            {
                if cache.unavailable_guilds.len() == 0 {
                    cache.unavailable_guilds.shrink_to_fit();

                    let guild_amount =
                        cache.guilds.iter().map(|i| *i.key()).collect::<Vec<GuildId>>();

                    extra_event = Some(FullEvent::CacheReady {
                        guilds: guild_amount,
                    });
                }
            }

            FullEvent::GuildCreate {
                guild: event.guild,
                is_new,
            }
        },
        Event::GuildDelete(mut event) => {
            let full = if_cache!(event.update(cache));

            FullEvent::GuildDelete {
                incomplete: event.guild,
                full,
            }
        },
        Event::GuildEmojisUpdate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildEmojisUpdate {
                guild_id: event.guild_id,
                current_state: event.emojis,
            }
        },
        Event::GuildIntegrationsUpdate(event) => FullEvent::GuildIntegrationsUpdate {
            guild_id: event.guild_id,
        },
        Event::GuildMemberAdd(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildMemberAddition {
                new_member: event.member,
            }
        },
        Event::GuildMemberRemove(mut event) => {
            let member = if_cache!(event.update(cache));

            FullEvent::GuildMemberRemoval {
                guild_id: event.guild_id,
                user: event.user,
                member_data_if_available: member,
            }
        },
        Event::GuildMemberUpdate(mut event) => {
            let before = if_cache!(event.update(cache));
            let after: Option<Member> = if_cache!({
                let guild = cache.guild(event.guild_id);
                guild.and_then(|g| g.members.get(&event.user.id).cloned())
            });

            FullEvent::GuildMemberUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::GuildMembersChunk(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildMembersChunk {
                chunk: event,
            }
        },
        Event::GuildRoleCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildRoleCreate {
                new: event.role,
            }
        },
        Event::GuildRoleDelete(mut event) => {
            let role = if_cache!(event.update(cache));

            FullEvent::GuildRoleDelete {
                guild_id: event.guild_id,
                removed_role_id: event.role_id,
                removed_role_data_if_available: role,
            }
        },
        Event::GuildRoleUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::GuildRoleUpdate {
                old_data_if_available: before,
                new: event.role,
            }
        },
        Event::GuildStickersUpdate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildStickersUpdate {
                guild_id: event.guild_id,
                current_state: event.stickers,
            }
        },
        Event::GuildUpdate(event) => {
            let before = if_cache!(cache.guild(event.guild.id).map(|g| g.clone()));

            FullEvent::GuildUpdate {
                old_data_if_available: before,
                new_data: event.guild,
            }
        },
        Event::InviteCreate(event) => FullEvent::InviteCreate {
            data: event,
        },
        Event::InviteDelete(event) => FullEvent::InviteDelete {
            data: event,
        },
        Event::MessageCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::Message {
                new_message: event.message,
            }
        },
        Event::MessageDeleteBulk(event) => FullEvent::MessageDeleteBulk {
            channel_id: event.channel_id,
            multiple_deleted_messages_ids: event.ids.into_vec(),
            guild_id: event.guild_id,
        },
        Event::MessageDelete(event) => FullEvent::MessageDelete {
            channel_id: event.channel_id,
            deleted_message_id: event.message_id,
            guild_id: event.guild_id,
        },
        Event::MessageUpdate(mut event) => {
            let before = if_cache!(event.update(cache));
            let after = if_cache!(cache.message(event.channel_id, event.id).map(|m| m.clone()));

            FullEvent::MessageUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::PresenceUpdate(mut event) => {
            let old_data = if_cache!(event.update(cache));

            FullEvent::PresenceUpdate {
                old_data,
                new_data: event.presence,
            }
        },
        Event::ReactionAdd(event) => FullEvent::ReactionAdd {
            add_reaction: event.reaction,
        },
        Event::ReactionRemove(event) => FullEvent::ReactionRemove {
            removed_reaction: event.reaction,
        },
        Event::ReactionRemoveAll(event) => FullEvent::ReactionRemoveAll {
            channel_id: event.channel_id,
            removed_from_message_id: event.message_id,
        },
        Event::ReactionRemoveEmoji(event) => FullEvent::ReactionRemoveEmoji {
            removed_reactions: event.reaction,
        },
        Event::Ready(mut event) => {
            update_cache!(cache, event);

            #[cfg(feature = "cache")]
            {
                let mut shards = cache.shard_data.write();
                if shards.connected.len() == shards.total.get() as usize
                    && !shards.has_sent_shards_ready
                {
                    shards.has_sent_shards_ready = true;
                    extra_event = Some(FullEvent::ShardsReady {
                        total_shards: shards.total,
                    });
                }
            }

            FullEvent::Ready {
                data_about_bot: event.ready,
            }
        },
        Event::Resumed(event) => FullEvent::Resume {
            event,
        },
        Event::TypingStart(event) => FullEvent::TypingStart {
            event,
        },
        Event::UserUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::UserUpdate {
                old_data: before,
                new: event.current_user,
            }
        },
        Event::VoiceServerUpdate(event) => FullEvent::VoiceServerUpdate {
            event,
        },
        Event::VoiceStateUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::VoiceStateUpdate {
                old: before,
                new: event.voice_state,
            }
        },
        Event::VoiceChannelStatusUpdate(mut event) => {
            let old = if_cache!(event.update(cache).map(FixedString::into_string));

            FullEvent::VoiceChannelStatusUpdate {
                old,
                status: event.status.map(FixedString::into_string),
                id: event.id,
                guild_id: event.guild_id,
            }
        },

        Event::WebhookUpdate(event) => FullEvent::WebhookUpdate {
            guild_id: event.guild_id,
            belongs_to_channel_id: event.channel_id,
        },
        Event::InteractionCreate(event) => FullEvent::InteractionCreate {
            interaction: event.interaction,
        },
        Event::IntegrationCreate(event) => FullEvent::IntegrationCreate {
            integration: event.integration,
        },
        Event::IntegrationUpdate(event) => FullEvent::IntegrationUpdate {
            integration: event.integration,
        },
        Event::IntegrationDelete(event) => FullEvent::IntegrationDelete {
            integration_id: event.id,
            guild_id: event.guild_id,
            application_id: event.application_id,
        },
        Event::StageInstanceCreate(event) => FullEvent::StageInstanceCreate {
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceUpdate(event) => FullEvent::StageInstanceUpdate {
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceDelete(event) => FullEvent::StageInstanceDelete {
            stage_instance: event.stage_instance,
        },
        Event::ThreadCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::ThreadCreate {
                thread: event.thread,
            }
        },
        Event::ThreadUpdate(mut event) => {
            let old = if_cache!(event.update(cache));

            FullEvent::ThreadUpdate {
                old,
                new: event.thread,
            }
        },
        Event::ThreadDelete(mut event) => {
            let full_thread_data = if_cache!(event.update(cache));

            FullEvent::ThreadDelete {
                thread: event.thread,
                full_thread_data,
            }
        },
        Event::ThreadListSync(event) => FullEvent::ThreadListSync {
            thread_list_sync: event,
        },
        Event::ThreadMemberUpdate(event) => FullEvent::ThreadMemberUpdate {
            thread_member: event.member,
        },
        Event::ThreadMembersUpdate(event) => FullEvent::ThreadMembersUpdate {
            thread_members_update: event,
        },
        Event::GuildScheduledEventCreate(event) => FullEvent::GuildScheduledEventCreate {
            event: event.event,
        },
        Event::GuildScheduledEventUpdate(event) => FullEvent::GuildScheduledEventUpdate {
            event: event.event,
        },
        Event::GuildScheduledEventDelete(event) => FullEvent::GuildScheduledEventDelete {
            event: event.event,
        },
        Event::GuildScheduledEventUserAdd(event) => FullEvent::GuildScheduledEventUserAdd {
            subscribed: event,
        },
        Event::GuildScheduledEventUserRemove(event) => FullEvent::GuildScheduledEventUserRemove {
            unsubscribed: event,
        },
        Event::EntitlementCreate(event) => FullEvent::EntitlementCreate {
            entitlement: event.entitlement,
        },
        Event::EntitlementUpdate(event) => FullEvent::EntitlementUpdate {
            entitlement: event.entitlement,
        },
        Event::EntitlementDelete(event) => FullEvent::EntitlementDelete {
            entitlement: event.entitlement,
        },
        Event::MessagePollVoteAdd(event) => FullEvent::MessagePollVoteAdd {
            event,
        },
        Event::MessagePollVoteRemove(event) => FullEvent::MessagePollVoteRemove {
            event,
        },
    };

    (event, extra_event)
}
