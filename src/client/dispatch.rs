use std::sync::Arc;

use tracing::debug;

#[cfg(feature = "gateway")]
use super::bridge::gateway::event::ClientEvent;
#[cfg(feature = "gateway")]
use super::event_handler::{EventHandler, RawEventHandler};
use super::{Context, FullEvent};
#[cfg(feature = "cache")]
use crate::cache::CacheUpdate;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::tokio::spawn_named;
use crate::model::channel::{Channel, ChannelType};
use crate::model::event::Event;
use crate::model::guild::Member;

#[cfg(feature = "cache")]
fn update_cache<E: CacheUpdate>(context: &Context, event: &mut E) -> Option<E::Output> {
    context.cache.update(event)
}

#[cfg(not(feature = "cache"))]
fn update_cache<E>(_: &Context, _: &mut E) -> Option<()> {
    None
}

pub(crate) async fn dispatch_model<'rec>(
    event: Event,
    context: Context,
    #[cfg(feature = "framework")] framework: Option<Arc<dyn Framework>>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
) {
    for raw_handler in raw_event_handlers {
        let (context, event) = (context.clone(), event.clone());
        tokio::spawn(async move { raw_handler.raw_event(context, event).await });
    }

    let full_event = update_cache_with_event(&context, event);
    if let Some(event) = full_event {
        for handler in event_handlers {
            let handler = handler.clone();
            let event = event.clone();
            let context = context.clone();

            spawn_named(event.snake_case_name(), async move {
                event.dispatch(context, &*handler).await;
            });
        }

        #[cfg(feature = "framework")]
        if let Some(framework) = framework {
            let framework = framework.clone();
            spawn_named("dispatch::framework::dispatch", async move {
                framework.dispatch(context, event).await;
            });
        }
    }
}

pub(crate) async fn dispatch_client<'rec>(
    event: ClientEvent,
    context: Context,
    event_handlers: Vec<Arc<dyn EventHandler>>,
) {
    match event {
        ClientEvent::ShardStageUpdate(event) => {
            for event_handler in event_handlers {
                let (context, event) = (context.clone(), event.clone());
                spawn_named("dispatch::event_handler::shard_stage_update", async move {
                    event_handler.shard_stage_update(context, event).await;
                });
            }
        },
    }
}

/// Updates the cache with the incoming event data and builds the full event data out of it.
///
/// Can return `None` if an event is unknown.
fn update_cache_with_event(ctx: &Context, event: Event) -> Option<FullEvent> {
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
            update_cache(ctx, &mut event);
            match event.channel {
                Channel::Guild(channel) => {
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
                Channel::Private(_) => unreachable!(
                    "Private channel create events are no longer sent to bots in the v8 gateway."
                ),
            }
        },
        Event::ChannelDelete(mut event) => {
            let cached_messages = if_cache!(update_cache(ctx, &mut event));

            match event.channel {
                Channel::Private(_) => unreachable!(
                    "Private channel create events are no longer sent to bots in the v8 gateway."
                ),
                Channel::Guild(channel) => {
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
            }
        },
        Event::ChannelPinsUpdate(event) => FullEvent::ChannelPinsUpdate {
            pin: event,
        },
        Event::ChannelUpdate(event) => {
            let old_channel = if_cache!(ctx.cache.channel(event.channel.id()));

            FullEvent::ChannelUpdate {
                old: old_channel,
                new: event.channel,
            }
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
            let is_new =
                if_cache!(Some(ctx.cache.unavailable_guilds.get(&event.guild.id).is_some()));

            update_cache(ctx, &mut event);

            FullEvent::GuildCreate {
                guild: event.guild,
                is_new,
            }
        },
        Event::GuildDelete(mut event) => {
            let full = if_cache!(update_cache(ctx, &mut event));

            FullEvent::GuildDelete {
                incomplete: event.guild,
                full,
            }
        },
        Event::GuildEmojisUpdate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::GuildEmojisUpdate {
                guild_id: event.guild_id,
                current_state: event.emojis,
            }
        },
        Event::GuildIntegrationsUpdate(event) => FullEvent::GuildIntegrationsUpdate {
            guild_id: event.guild_id,
        },
        Event::GuildMemberAdd(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::GuildMemberAddition {
                new_member: event.member,
            }
        },
        Event::GuildMemberRemove(mut event) => {
            let member = if_cache!(update_cache(ctx, &mut event));

            FullEvent::GuildMemberRemoval {
                guild_id: event.guild_id,
                user: event.user,
                member_data_if_available: member,
            }
        },
        Event::GuildMemberUpdate(mut event) => {
            let before = if_cache!(update_cache(ctx, &mut event));
            let after: Option<Member> = if_cache!(ctx.cache.member(event.guild_id, event.user.id));

            FullEvent::GuildMemberUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::GuildMembersChunk(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::GuildMembersChunk {
                chunk: event,
            }
        },
        Event::GuildRoleCreate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::GuildRoleCreate {
                new: event.role,
            }
        },
        Event::GuildRoleDelete(mut event) => {
            let role = if_cache!(update_cache(ctx, &mut event));

            FullEvent::GuildRoleDelete {
                guild_id: event.guild_id,
                removed_role_id: event.role_id,
                removed_role_data_if_available: role,
            }
        },
        Event::GuildRoleUpdate(mut event) => {
            let before = if_cache!(update_cache(ctx, &mut event));

            FullEvent::GuildRoleUpdate {
                old_data_if_available: before,
                new: event.role,
            }
        },
        Event::GuildStickersUpdate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::GuildStickersUpdate {
                guild_id: event.guild_id,
                current_state: event.stickers,
            }
        },
        Event::GuildUpdate(event) => {
            let before = if_cache!(ctx.cache.guild(event.guild.id).map(|g| g.clone()));

            FullEvent::GuildUpdate {
                old_data_if_available: before,
                new_but_incomplete: event.guild,
            }
        },
        Event::InviteCreate(event) => FullEvent::InviteCreate {
            data: event,
        },
        Event::InviteDelete(event) => FullEvent::InviteDelete {
            data: event,
        },
        Event::MessageCreate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::Message {
                new_message: event.message,
            }
        },
        Event::MessageDeleteBulk(event) => FullEvent::MessageDeleteBulk {
            channel_id: event.channel_id,
            multiple_deleted_messages_ids: event.ids,
            guild_id: event.guild_id,
        },
        Event::MessageDelete(event) => FullEvent::MessageDelete {
            channel_id: event.channel_id,
            deleted_message_id: event.message_id,
            guild_id: event.guild_id,
        },
        Event::MessageUpdate(mut event) => {
            let before = if_cache!(update_cache(ctx, &mut event));
            let after = if_cache!(ctx.cache.message(event.channel_id, event.id));

            FullEvent::MessageUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::PresencesReplace(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::PresenceReplace {
                presences: event.presences,
            }
        },
        Event::PresenceUpdate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::PresenceUpdate {
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
            update_cache(ctx, &mut event);

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
        Event::Unknown => {
            debug!("An unknown event was received");
            return None;
        },
        Event::UserUpdate(mut event) => {
            let before = if_cache!(update_cache(ctx, &mut event));

            FullEvent::UserUpdate {
                old_data: before,
                new: event.current_user,
            }
        },
        Event::VoiceServerUpdate(event) => FullEvent::VoiceServerUpdate {
            event,
        },
        Event::VoiceStateUpdate(mut event) => {
            let before = if_cache!(update_cache(ctx, &mut event));

            FullEvent::VoiceStateUpdate {
                old: before,
                new: event.voice_state,
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
            update_cache(ctx, &mut event);

            FullEvent::ThreadCreate {
                thread: event.thread,
            }
        },
        Event::ThreadUpdate(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::ThreadUpdate {
                thread: event.thread,
            }
        },
        Event::ThreadDelete(mut event) => {
            update_cache(ctx, &mut event);

            FullEvent::ThreadDelete {
                thread: event.thread,
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
    };

    Some(event)
}
