use std::sync::Arc;

use super::event_handler::{EventHandler, RawEventHandler};
use super::{Context, FullEvent};
#[cfg(feature = "cache")]
use crate::cache::CacheUpdate;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::tokio::spawn_named;
use crate::model::channel::ChannelType;
use crate::model::event::Event;

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
    ($cache:expr, $event:ident) => {
        $event.update($cache)
    };
}

#[cfg(not(feature = "cache"))]
macro_rules! update_cache {
    ($cache:expr, $event:ident) => {};
}

/// Calls the user's event handlers and the framework handler.
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

    let extra_event = get_virtual_event(&context, &event);
    let full_event = update_cache_with_event(&context, event);

    spawn_named("dispatch::user", async move {
        #[cfg(feature = "framework")]
        tokio::join!(
            dispatch_framework(&context, framework, &full_event, extra_event.as_ref()),
            dispatch_event_handler(&context, event_handler, &full_event, extra_event.as_ref())
        );

        #[cfg(not(feature = "framework"))]
        dispatch_event_handler(&context, event_handler, &full_event, extra_event.as_ref()).await;
    });
}

#[cfg(feature = "framework")]
async fn dispatch_framework(
    context: &Context,
    framework: Option<Arc<dyn Framework>>,
    full_event: &FullEvent,
    extra_event: Option<&FullEvent>,
) {
    if let Some(framework) = framework {
        if let Some(extra_event) = extra_event {
            framework.dispatch(context, extra_event).await;
        }

        framework.dispatch(context, full_event).await;
    }
}

async fn dispatch_event_handler(
    context: &Context,
    event_handler: Option<Arc<dyn EventHandler>>,
    full_event: &FullEvent,
    extra_event: Option<&FullEvent>,
) {
    if let Some(handler) = event_handler {
        if let Some(extra_event) = extra_event {
            handler.dispatch(context, extra_event).await;
        }

        handler.dispatch(context, full_event).await;
    }
}

/// Updates the cache with the incoming event data and builds the full event data out of it.
fn update_cache_with_event(ctx: &Context, event: Event) -> FullEvent {
    match event {
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
        Event::ChannelCreate(event) => {
            update_cache!(&ctx.cache, event);

            let channel = event.channel;
            if channel.base.kind == ChannelType::Category {
                FullEvent::CategoryCreate {
                    category: channel,
                }
            } else {
                FullEvent::ChannelCreate {
                    channel,
                }
            }
        },
        Event::ChannelDelete(event) => {
            let cached_messages = if_cache!(update_cache!(&ctx.cache, event));

            let channel = event.channel;
            if channel.base.kind == ChannelType::Category {
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
        Event::ChannelUpdate(event) => {
            let old_channel = if_cache!(update_cache!(&ctx.cache, event));

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
        Event::GuildCreate(event) => {
            let is_new =
                if_cache!(Some(!&ctx.cache.unavailable_guilds().contains(&event.guild.id)));

            FullEvent::GuildCreate {
                guild: event.guild,
                is_new,
            }
        },
        Event::GuildDelete(event) => {
            let full = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::GuildDelete {
                incomplete: event.guild,
                full,
            }
        },
        Event::GuildEmojisUpdate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildEmojisUpdate {
                guild_id: event.guild_id,
                current_state: event.emojis,
            }
        },
        Event::GuildIntegrationsUpdate(event) => FullEvent::GuildIntegrationsUpdate {
            guild_id: event.guild_id,
        },
        Event::GuildMemberAdd(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildMemberAddition {
                new_member: event.member,
            }
        },
        Event::GuildMemberRemove(event) => {
            let member = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::GuildMemberRemoval {
                guild_id: event.guild_id,
                user: event.user,
                member_data_if_available: member,
            }
        },
        Event::GuildMemberUpdate(event) => {
            let before = if_cache!(update_cache!(&ctx.cache, event));
            let after = if_cache!(
                ctx.cache
                    .guild(event.guild_id)
                    .and_then(|g| g.members.get(&event.user.id).cloned())
            );

            FullEvent::GuildMemberUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::GuildMembersChunk(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildMembersChunk {
                chunk: event,
            }
        },
        Event::GuildRoleCreate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildRoleCreate {
                new: event.role,
            }
        },
        Event::GuildRoleDelete(event) => {
            let role = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::GuildRoleDelete {
                guild_id: event.guild_id,
                removed_role_id: event.role_id,
                removed_role_data_if_available: role,
            }
        },
        Event::GuildRoleUpdate(event) => {
            let before = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::GuildRoleUpdate {
                old_data_if_available: before,
                new: event.role,
            }
        },
        Event::GuildStickersUpdate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildStickersUpdate {
                guild_id: event.guild_id,
                current_state: event.stickers,
            }
        },
        Event::GuildUpdate(event) => {
            let before = if_cache!(ctx.cache.guild(event.guild.id).map(|g| g.clone()));

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
        Event::MessageCreate(event) => {
            update_cache!(&ctx.cache, event);

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
        Event::MessageUpdate(event) => {
            let before = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::MessageUpdate {
                old_if_available: before,
                event,
            }
        },
        Event::PresenceUpdate(event) => {
            let old_data = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::PresenceUpdate {
                old_data,
                new_data: event.presence,
            }
        },
        Event::ReactionAdd(event) => {
            let old_message_if_available = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ReactionAdd {
                add_reaction: event.reaction,
                old_message_if_available,
            }
        },
        Event::ReactionRemove(event) => {
            let old_message_if_available = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ReactionRemove {
                removed_reaction: event.reaction,
                old_message_if_available,
            }
        },
        Event::ReactionRemoveAll(event) => {
            let old_message_if_available = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ReactionRemoveAll {
                guild_id: event.guild_id,
                channel_id: event.channel_id,
                removed_from_message_id: event.message_id,
                old_message_if_available,
            }
        },
        Event::ReactionRemoveEmoji(event) => {
            let old_message_if_available = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ReactionRemoveEmoji {
                removed_reactions: event.reaction,
                old_message_if_available,
            }
        },
        Event::Ready(event) => FullEvent::Ready {
            data_about_bot: event.ready,
        },
        Event::Resumed(event) => FullEvent::Resume {
            event,
        },
        Event::SoundboardSounds(event) => FullEvent::SoundboardSounds {
            event,
        },
        Event::SoundboardSoundCreate(event) => FullEvent::SoundboardSoundCreate {
            event,
        },
        Event::SoundboardSoundUpdate(event) => FullEvent::SoundboardSoundUpdate {
            event,
        },
        Event::SoundboardSoundsUpdate(event) => FullEvent::SoundboardSoundsUpdate {
            event,
        },
        Event::SoundboardSoundDelete(event) => FullEvent::SoundboardSoundDelete {
            event,
        },
        Event::TypingStart(event) => FullEvent::TypingStart {
            event,
        },
        Event::UserUpdate(event) => {
            let before = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::UserUpdate {
                old_data: before,
                new: event.current_user,
            }
        },
        Event::VoiceServerUpdate(event) => FullEvent::VoiceServerUpdate {
            event,
        },
        Event::VoiceStateUpdate(event) => {
            let before = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::VoiceStateUpdate {
                old: before,
                new: event.voice_state,
            }
        },
        Event::VoiceChannelStatusUpdate(event) => {
            let old = if_cache!(event.update(&ctx.cache).map(Into::into));

            FullEvent::VoiceChannelStatusUpdate {
                old,
                status: event.status.map(Into::into),
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
        Event::ThreadCreate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::ThreadCreate {
                thread: event.thread,
                newly_created: event.newly_created,
            }
        },
        Event::ThreadUpdate(event) => {
            let old = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ThreadUpdate {
                old,
                new: event.thread,
            }
        },
        Event::ThreadDelete(event) => {
            let full_thread_data = if_cache!(update_cache!(&ctx.cache, event));

            FullEvent::ThreadDelete {
                thread: event.thread,
                full_thread_data,
            }
        },
        Event::ThreadListSync(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::ThreadListSync {
                thread_list_sync: event,
            }
        },
        Event::ThreadMemberUpdate(event) => FullEvent::ThreadMemberUpdate {
            thread_member: event.member,
        },
        Event::ThreadMembersUpdate(event) => FullEvent::ThreadMembersUpdate {
            thread_members_update: event,
        },
        Event::GuildScheduledEventCreate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildScheduledEventCreate {
                event: event.event,
            }
        },
        Event::GuildScheduledEventUpdate(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildScheduledEventUpdate {
                event: event.event,
            }
        },
        Event::GuildScheduledEventDelete(event) => {
            update_cache!(&ctx.cache, event);

            FullEvent::GuildScheduledEventDelete {
                event: event.event,
            }
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
    }
}

fn get_virtual_event(ctx: &Context, event: &Event) -> Option<FullEvent> {
    match event {
        Event::GuildCreate(event) =>
        {
            #[cfg(feature = "cache")]
            if let Some(guilds) = update_cache!(&ctx.cache, event) {
                return Some(FullEvent::CacheReady {
                    guilds,
                });
            }
        },
        Event::Ready(event) =>
        {
            #[cfg(feature = "cache")]
            if let Some(total_shards) = update_cache!(&ctx.cache, event) {
                return Some(FullEvent::ShardsReady {
                    total_shards,
                });
            }
        },
        _ => {},
    }
    None
}
