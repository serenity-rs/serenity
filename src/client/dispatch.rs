use std::sync::Arc;

use tracing::debug;

#[cfg(feature = "gateway")]
use super::event_handler::{EventHandler, RawEventHandler};
use super::{Context, FullEvent};
#[cfg(feature = "cache")]
use crate::cache::CacheUpdate;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::tokio::spawn_named;
use crate::model::channel::ChannelType;
use crate::model::event::Event;
use crate::model::guild::Member;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;

#[cfg(feature = "cache")]
fn update_cache<E: CacheUpdate>(context: &Context, event: &mut E) -> Option<E::Output> {
    context.cache.update(event)
}

#[cfg(not(feature = "cache"))]
fn update_cache<E>(_: &Context, _: &mut E) -> Option<()> {
    None
}

pub(crate) fn dispatch_model(
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

    let full_events = update_cache_with_event(context, event);
    if let Some(events) = full_events {
        let iter = std::iter::once(events.0).chain(events.1);
        for handler in event_handlers {
            for event in iter.clone() {
                let handler = Arc::clone(&handler);
                spawn_named(
                    event.snake_case_name(),
                    async move { event.dispatch(&*handler).await },
                );
            }
        }

        #[cfg(feature = "framework")]
        if let Some(framework) = framework {
            for event in iter {
                let framework = Arc::clone(&framework);
                spawn_named("dispatch::framework::dispatch", async move {
                    framework.dispatch(event).await;
                });
            }
        }
    }
}

/// Updates the cache with the incoming event data and builds the full event data out of it.
///
/// Can return a secondary [`FullEvent`] for "virtual" events like [`FullEvent::CacheReady`] or
/// [`FullEvent::ShardsReady`]. Secondary events are traditionally dispatched first.
///
/// Can return `None` if an event is unknown.
#[cfg_attr(not(feature = "cache"), allow(unused_mut))]
fn update_cache_with_event(ctx: Context, event: Event) -> Option<(FullEvent, Option<FullEvent>)> {
    let mut extra_event = None;
    let event = match event {
        Event::CommandPermissionsUpdate(event) => FullEvent::CommandPermissionsUpdate {
            ctx,
            permission: event.permission,
        },
        Event::AutoModRuleCreate(event) => FullEvent::AutoModRuleCreate {
            ctx,
            rule: event.rule,
        },
        Event::AutoModRuleUpdate(event) => FullEvent::AutoModRuleUpdate {
            ctx,
            rule: event.rule,
        },
        Event::AutoModRuleDelete(event) => FullEvent::AutoModRuleDelete {
            ctx,
            rule: event.rule,
        },
        Event::AutoModActionExecution(event) => FullEvent::AutoModActionExecution {
            ctx,
            execution: event.execution,
        },
        Event::ChannelCreate(mut event) => {
            update_cache(&ctx, &mut event);

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryCreate {
                    ctx,
                    category: channel,
                }
            } else {
                FullEvent::ChannelCreate {
                    ctx,
                    channel,
                }
            }
        },
        Event::ChannelDelete(mut event) => {
            let cached_messages = if_cache!(update_cache(&ctx, &mut event));

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryDelete {
                    ctx,
                    category: channel,
                }
            } else {
                FullEvent::ChannelDelete {
                    ctx,
                    channel,
                    messages: cached_messages,
                }
            }
        },
        Event::ChannelPinsUpdate(event) => FullEvent::ChannelPinsUpdate {
            ctx,
            pin: event,
        },
        Event::ChannelUpdate(mut event) => {
            let old_channel = if_cache!(event.update(&ctx.cache));

            FullEvent::ChannelUpdate {
                ctx,
                old: old_channel,
                new: event.channel,
            }
        },
        Event::GuildAuditLogEntryCreate(event) => FullEvent::GuildAuditLogEntryCreate {
            ctx,
            entry: event.entry,
            guild_id: event.guild_id,
        },
        Event::GuildBanAdd(event) => FullEvent::GuildBanAddition {
            ctx,
            guild_id: event.guild_id,
            banned_user: event.user,
        },
        Event::GuildBanRemove(event) => FullEvent::GuildBanRemoval {
            ctx,
            guild_id: event.guild_id,
            unbanned_user: event.user,
        },
        Event::GuildCreate(mut event) => {
            let is_new = if_cache!(Some(!ctx.cache.unavailable_guilds.contains(&event.guild.id)));

            update_cache(&ctx, &mut event);

            #[cfg(feature = "cache")]
            {
                let context = ctx.clone();

                if context.cache.unavailable_guilds.len() == 0 {
                    context.cache.unavailable_guilds.shrink_to_fit();

                    let guild_amount =
                        context.cache.guilds.iter().map(|i| *i.key()).collect::<Vec<GuildId>>();

                    extra_event = Some(FullEvent::CacheReady {
                        ctx: context,
                        guilds: guild_amount,
                    });
                }
            }

            FullEvent::GuildCreate {
                ctx,
                guild: event.guild,
                is_new,
            }
        },
        Event::GuildDelete(mut event) => {
            let full = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::GuildDelete {
                ctx,
                incomplete: event.guild,
                full,
            }
        },
        Event::GuildEmojisUpdate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::GuildEmojisUpdate {
                ctx,
                guild_id: event.guild_id,
                current_state: event.emojis,
            }
        },
        Event::GuildIntegrationsUpdate(event) => FullEvent::GuildIntegrationsUpdate {
            ctx,
            guild_id: event.guild_id,
        },
        Event::GuildMemberAdd(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::GuildMemberAddition {
                ctx,
                new_member: event.member,
            }
        },
        Event::GuildMemberRemove(mut event) => {
            let member = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::GuildMemberRemoval {
                ctx,
                guild_id: event.guild_id,
                user: event.user,
                member_data_if_available: member,
            }
        },
        Event::GuildMemberUpdate(mut event) => {
            let before = if_cache!(update_cache(&ctx, &mut event));
            let after: Option<Member> =
                if_cache!(ctx.cache.member(event.guild_id, event.user.id).map(|m| m.clone()));

            FullEvent::GuildMemberUpdate {
                ctx,
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::GuildMembersChunk(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::GuildMembersChunk {
                ctx,
                chunk: event,
            }
        },
        Event::GuildRoleCreate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::GuildRoleCreate {
                ctx,
                new: event.role,
            }
        },
        Event::GuildRoleDelete(mut event) => {
            let role = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::GuildRoleDelete {
                ctx,
                guild_id: event.guild_id,
                removed_role_id: event.role_id,
                removed_role_data_if_available: role,
            }
        },
        Event::GuildRoleUpdate(mut event) => {
            let before = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::GuildRoleUpdate {
                ctx,
                old_data_if_available: before,
                new: event.role,
            }
        },
        Event::GuildStickersUpdate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::GuildStickersUpdate {
                ctx,
                guild_id: event.guild_id,
                current_state: event.stickers,
            }
        },
        Event::GuildUpdate(event) => {
            let before = if_cache!(ctx.cache.guild(event.guild.id).map(|g| g.clone()));

            FullEvent::GuildUpdate {
                ctx,
                old_data_if_available: before,
                new_data: event.guild,
            }
        },
        Event::InviteCreate(event) => FullEvent::InviteCreate {
            ctx,
            data: event,
        },
        Event::InviteDelete(event) => FullEvent::InviteDelete {
            ctx,
            data: event,
        },
        Event::MessageCreate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::Message {
                ctx,
                new_message: event.message,
            }
        },
        Event::MessageDeleteBulk(event) => FullEvent::MessageDeleteBulk {
            ctx,
            channel_id: event.channel_id,
            multiple_deleted_messages_ids: event.ids,
            guild_id: event.guild_id,
        },
        Event::MessageDelete(event) => FullEvent::MessageDelete {
            ctx,
            channel_id: event.channel_id,
            deleted_message_id: event.message_id,
            guild_id: event.guild_id,
        },
        Event::MessageUpdate(mut event) => {
            let before = if_cache!(update_cache(&ctx, &mut event));
            let after = if_cache!(ctx.cache.message(event.channel_id, event.id).map(|m| m.clone()));

            FullEvent::MessageUpdate {
                ctx,
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::PresencesReplace(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::PresenceReplace {
                ctx,
                presences: event.presences,
            }
        },
        Event::PresenceUpdate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::PresenceUpdate {
                ctx,
                new_data: event.presence,
            }
        },
        Event::ReactionAdd(event) => FullEvent::ReactionAdd {
            ctx,
            add_reaction: event.reaction,
        },
        Event::ReactionRemove(event) => FullEvent::ReactionRemove {
            ctx,
            removed_reaction: event.reaction,
        },
        Event::ReactionRemoveAll(event) => FullEvent::ReactionRemoveAll {
            ctx,
            channel_id: event.channel_id,
            removed_from_message_id: event.message_id,
        },
        Event::ReactionRemoveEmoji(event) => FullEvent::ReactionRemoveEmoji {
            ctx,
            removed_reactions: event.reaction,
        },
        Event::Ready(mut event) => {
            update_cache(&ctx, &mut event);

            #[cfg(feature = "cache")]
            {
                let mut shards = ctx.cache.shard_data.write();
                if shards.connected.len() as u32 == shards.total && !shards.has_sent_shards_ready {
                    shards.has_sent_shards_ready = true;
                    let total = shards.total;
                    drop(shards);

                    extra_event = Some(FullEvent::ShardsReady {
                        ctx: ctx.clone(),
                        total_shards: total,
                    });
                }
            }

            FullEvent::Ready {
                ctx,
                data_about_bot: event.ready,
            }
        },
        Event::Resumed(event) => FullEvent::Resume {
            ctx,
            event,
        },
        Event::TypingStart(event) => FullEvent::TypingStart {
            ctx,
            event,
        },
        Event::Unknown(event) => {
            debug!("An unknown event was received: {event:?}");
            return None;
        },
        Event::UserUpdate(mut event) => {
            let before = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::UserUpdate {
                ctx,
                old_data: before,
                new: event.current_user,
            }
        },
        Event::VoiceServerUpdate(event) => FullEvent::VoiceServerUpdate {
            ctx,
            event,
        },
        Event::VoiceStateUpdate(mut event) => {
            let before = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::VoiceStateUpdate {
                ctx,
                old: before,
                new: event.voice_state,
            }
        },
        Event::VoiceChannelStatusUpdate(mut event) => {
            let old = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::VoiceChannelStatusUpdate {
                ctx,
                old,
                status: event.status,
                id: event.id,
                guild_id: event.guild_id,
            }
        },

        Event::WebhookUpdate(event) => FullEvent::WebhookUpdate {
            ctx,
            guild_id: event.guild_id,
            belongs_to_channel_id: event.channel_id,
        },
        Event::InteractionCreate(event) => FullEvent::InteractionCreate {
            ctx,
            interaction: event.interaction,
        },
        Event::IntegrationCreate(event) => FullEvent::IntegrationCreate {
            ctx,
            integration: event.integration,
        },
        Event::IntegrationUpdate(event) => FullEvent::IntegrationUpdate {
            ctx,
            integration: event.integration,
        },
        Event::IntegrationDelete(event) => FullEvent::IntegrationDelete {
            ctx,
            integration_id: event.id,
            guild_id: event.guild_id,
            application_id: event.application_id,
        },
        Event::StageInstanceCreate(event) => FullEvent::StageInstanceCreate {
            ctx,
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceUpdate(event) => FullEvent::StageInstanceUpdate {
            ctx,
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceDelete(event) => FullEvent::StageInstanceDelete {
            ctx,
            stage_instance: event.stage_instance,
        },
        Event::ThreadCreate(mut event) => {
            update_cache(&ctx, &mut event);

            FullEvent::ThreadCreate {
                ctx,
                thread: event.thread,
            }
        },
        Event::ThreadUpdate(mut event) => {
            let old = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::ThreadUpdate {
                ctx,
                old,
                new: event.thread,
            }
        },
        Event::ThreadDelete(mut event) => {
            let full_thread_data = if_cache!(update_cache(&ctx, &mut event));

            FullEvent::ThreadDelete {
                ctx,
                thread: event.thread,
                full_thread_data,
            }
        },
        Event::ThreadListSync(event) => FullEvent::ThreadListSync {
            ctx,
            thread_list_sync: event,
        },
        Event::ThreadMemberUpdate(event) => FullEvent::ThreadMemberUpdate {
            ctx,
            thread_member: event.member,
        },
        Event::ThreadMembersUpdate(event) => FullEvent::ThreadMembersUpdate {
            ctx,
            thread_members_update: event,
        },
        Event::GuildScheduledEventCreate(event) => FullEvent::GuildScheduledEventCreate {
            ctx,
            event: event.event,
        },
        Event::GuildScheduledEventUpdate(event) => FullEvent::GuildScheduledEventUpdate {
            ctx,
            event: event.event,
        },
        Event::GuildScheduledEventDelete(event) => FullEvent::GuildScheduledEventDelete {
            ctx,
            event: event.event,
        },
        Event::GuildScheduledEventUserAdd(event) => FullEvent::GuildScheduledEventUserAdd {
            ctx,
            subscribed: event,
        },
        Event::GuildScheduledEventUserRemove(event) => FullEvent::GuildScheduledEventUserRemove {
            ctx,
            unsubscribed: event,
        },
    };

    Some((event, extra_event))
}
