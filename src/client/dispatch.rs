#[cfg(feature = "cache")]
use std::fmt;
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender as Sender;
use futures::future::{BoxFuture, FutureExt};
use tokio::sync::RwLock;
use tracing::{debug, instrument};
use typemap_rev::TypeMap;

#[cfg(feature = "gateway")]
use super::bridge::gateway::event::ClientEvent;
#[cfg(feature = "gateway")]
use super::event_handler::{EventHandler, RawEventHandler};
use super::Context;
#[cfg(feature = "cache")]
use crate::cache::CacheUpdate;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::InterMessage;
use crate::internal::tokio::spawn_named;
use crate::model::channel::{Channel, ChannelType, Message};
use crate::model::event::Event;
use crate::model::guild::Member;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;
use crate::CacheAndHttp;

#[inline]
#[cfg(feature = "cache")]
fn update<E: CacheUpdate + fmt::Debug>(
    cache_and_http: &CacheAndHttp,
    event: &mut E,
) -> Option<E::Output> {
    cache_and_http.cache.update(event)
}

#[inline]
#[cfg(not(feature = "cache"))]
fn update<E>(_cache_and_http: &CacheAndHttp, _event: &mut E) -> Option<()> {
    None
}

fn context(
    data: &Arc<RwLock<TypeMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u32,
    cache_and_http: &CacheAndHttp,
) -> Context {
    Context::new(
        Arc::clone(data),
        runner_tx.clone(),
        shard_id,
        Arc::clone(&cache_and_http.http),
        #[cfg(feature = "cache")]
        Arc::clone(&cache_and_http.cache),
    )
}

// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub(crate) enum DispatchEvent {
    Client(ClientEvent),
    Model(Event),
}

impl DispatchEvent {
    #[instrument(skip(self, cache_and_http))]
    fn update(&mut self, cache_and_http: &CacheAndHttp) {
        match self {
            Self::Model(Event::ChannelCreate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::ChannelDelete(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::ChannelUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildCreate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildDelete(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildEmojisUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildMemberAdd(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildMemberRemove(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildMemberUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildMembersChunk(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildRoleCreate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildRoleDelete(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildRoleUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildStickersUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::GuildUpdate(ref mut event)) => {
                update(cache_and_http, event);
            },
            // Already handled by the framework check macro
            Self::Model(Event::MessageCreate(_)) => {},
            Self::Model(Event::MessageUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::PresencesReplace(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::PresenceUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::Ready(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::UserUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::VoiceStateUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::ThreadCreate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::ThreadUpdate(event)) => {
                update(cache_and_http, event);
            },
            Self::Model(Event::ThreadDelete(event)) => {
                update(cache_and_http, event);
            },
            _ => (),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch<'rec>(
    // #[allow(unused_variables)]
    mut event: DispatchEvent,
    #[cfg(feature = "framework")] framework: &'rec Option<Arc<dyn Framework + Send + Sync>>,
    data: &'rec Arc<RwLock<TypeMap>>,
    event_handler: &'rec Option<Arc<dyn EventHandler>>,
    raw_event_handler: &'rec Option<Arc<dyn RawEventHandler>>,
    runner_tx: &'rec Sender<InterMessage>,
    shard_id: u32,
    cache_and_http: &'rec CacheAndHttp,
) -> BoxFuture<'rec, ()> {
    async move {
        match (event_handler, raw_event_handler) {
            (None, None) => {
                event.update(cache_and_http);

                #[cfg(feature = "framework")]
                if let DispatchEvent::Model(Event::MessageCreate(event)) = event {
                    let context = context(data, runner_tx, shard_id, cache_and_http);

                    if let Some(framework) = framework {
                        let framework = Arc::clone(framework);

                        spawn_named("dispatch::framework::message", async move {
                            framework.dispatch(context, event.message).await;
                        });
                    }
                }
            },
            (Some(h), None) => match event {
                DispatchEvent::Model(Event::MessageCreate(mut event)) => {
                    update(cache_and_http, &mut event);

                    let context = context(data, runner_tx, shard_id, cache_and_http);

                    #[cfg(not(feature = "framework"))]
                    {
                        // Avoid cloning if there will be no framework dispatch.
                        dispatch_message(context, event.message, h).await;
                    }

                    #[cfg(feature = "framework")]
                    {
                        dispatch_message(context.clone(), event.message.clone(), h).await;

                        if let Some(framework) = framework {
                            let framework = Arc::clone(framework);

                            spawn_named("dispatch::framework::message", async move {
                                framework.dispatch(context, event.message).await;
                            });
                        }
                    }
                },
                other => {
                    handle_event(other, data, h, runner_tx, shard_id, cache_and_http).await;
                },
            },
            (None, Some(rh)) => {
                event.update(cache_and_http);

                if let DispatchEvent::Model(event) = event {
                    let event_handler = Arc::clone(rh);

                    let context = context(data, runner_tx, shard_id, cache_and_http);

                    #[cfg(not(feature = "framework"))]
                    {
                        // No clone needed, as there will be no framework dispatch.
                        event_handler.raw_event(context, event).await;
                    }

                    #[cfg(feature = "framework")]
                    {
                        if let Event::MessageCreate(msg_event) = &event {
                            // Must clone in order to dispatch the framework too.
                            let message = msg_event.message.clone();
                            event_handler.raw_event(context.clone(), event).await;

                            if let Some(framework) = framework {
                                let framework = Arc::clone(framework);

                                spawn_named("dispatch::framework::message", async move {
                                    framework.dispatch(context, message).await;
                                });
                            }
                        } else {
                            // Avoid cloning if there will be no framework dispatch.
                            event_handler.raw_event(context, event).await;
                        }
                    }
                }
            },
            // We call this function again, passing `None` for each event handler
            // and passing no framework, as we dispatch once we are done right here.
            (Some(handler), Some(raw_handler)) => {
                let context = context(data, runner_tx, shard_id, cache_and_http);

                if let DispatchEvent::Model(event) = &event {
                    raw_handler.raw_event(context.clone(), event.clone()).await;
                }

                match event {
                    DispatchEvent::Model(Event::MessageCreate(event)) => {
                        #[cfg(not(feature = "framework"))]
                        {
                            // Avoid cloning if there will be no framework dispatch.
                            dispatch_message(context, event.message, handler).await;
                        }

                        #[cfg(feature = "framework")]
                        {
                            dispatch_message(context.clone(), event.message.clone(), handler).await;

                            if let Some(framework) = framework {
                                let framework = Arc::clone(framework);

                                spawn_named("dispatch::framework::message", async move {
                                    framework.dispatch(context, event.message).await;
                                });
                            }
                        }
                    },
                    other => {
                        handle_event(other, data, handler, runner_tx, shard_id, cache_and_http)
                            .await;
                    },
                }
            },
        }
    }
    .boxed()
}

async fn dispatch_message(
    context: Context,
    message: Message,
    event_handler: &Arc<dyn EventHandler>,
) {
    let event_handler = Arc::clone(event_handler);

    spawn_named("dispatch::event_handler::message", async move {
        event_handler.message(context, message).await;
    });
}
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::too_many_arguments)]
#[cfg_attr(feature = "cache", allow(clippy::used_underscore_binding))]
#[instrument(skip(event, data, event_handler, cache_and_http))]
async fn handle_event(
    event: DispatchEvent,
    data: &Arc<RwLock<TypeMap>>,
    event_handler: &Arc<dyn EventHandler>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u32,
    cache_and_http: &CacheAndHttp,
) {
    let context = context(data, runner_tx, shard_id, cache_and_http);

    let event_handler = Arc::clone(event_handler);
    let cache_and_http = cache_and_http.clone();
    #[cfg(feature = "cache")]
    let cache = cache_and_http.cache.clone();

    // Handle ClientEvent or return back Event
    let model_event = match event {
        DispatchEvent::Model(event) => event,
        DispatchEvent::Client(event) => {
            return match event {
                ClientEvent::ShardStageUpdate(event) => {
                    spawn_named("dispatch::event_handler::shard_stage_update", async move {
                        event_handler.shard_stage_update(context, event).await;
                    });
                },
            }
        },
    };

    // Handle Event, this is done to prevent indenting twice (once to destructure DispatchEvent, then to destructure Event)
    #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
    match model_event {
        Event::ApplicationCommandPermissionsUpdate(event) => {
            spawn_named(
                "dispatch::event_handler::application_command_permissions_update",
                async move {
                    event_handler
                        .application_command_permissions_update(context, event.permission)
                        .await;
                },
            );
        },
        Event::AutoModerationRuleCreate(event) => {
            spawn_named("dispatch::event_handler::auto_moderation_rule_create", async move {
                event_handler.auto_moderation_rule_create(context, event.rule).await;
            });
        },
        Event::AutoModerationRuleUpdate(event) => {
            spawn_named("dispatch::event_handler::auto_moderation_rule_update", async move {
                event_handler.auto_moderation_rule_update(context, event.rule).await;
            });
        },
        Event::AutoModerationRuleDelete(event) => {
            spawn_named("dispatch::event_handler::auto_moderation_rule_delete", async move {
                event_handler.auto_moderation_rule_delete(context, event.rule).await;
            });
        },
        Event::AutoModerationActionExecution(event) => {
            spawn_named("dispatch::event_handler::auto_moderation_action_execution", async move {
                event_handler.auto_moderation_action_execution(context, event.execution).await;
            });
        },
        Event::ChannelCreate(mut event) => {
            update(&cache_and_http, &mut event);
            match event.channel {
                Channel::Guild(channel) => {
                    if channel.kind == ChannelType::Category {
                        spawn_named("dispatch::event_handler::category_create", async move {
                            event_handler.category_create(context, &channel).await;
                        });
                    } else {
                        spawn_named("dispatch::event_handler::channel_create", async move {
                            event_handler.channel_create(context, &channel).await;
                        });
                    }
                },
                // Private channel create events are no longer sent to bots in the v8 gateway.
                Channel::Private(_) => {},
            }
        },
        Event::ChannelDelete(mut event) => {
            update(&cache_and_http, &mut event);

            match event.channel {
                Channel::Private(_) => {},
                Channel::Guild(channel) => {
                    if channel.kind == ChannelType::Category {
                        spawn_named("dispatch::event_handler::category_delete", async move {
                            event_handler.category_delete(context, &channel).await;
                        });
                    } else {
                        spawn_named("dispatch::event_handler::channel_delete", async move {
                            event_handler.channel_delete(context, &channel).await;
                        });
                    }
                },
            }
        },
        Event::ChannelPinsUpdate(event) => {
            spawn_named("dispatch::event_handler::channel_pins_update", async move {
                event_handler.channel_pins_update(context, event).await;
            });
        },
        Event::ChannelUpdate(mut event) => {
            spawn_named("dispatch::event_handler::channel_update", async move {
                let old_channel = if_cache!(cache.channel(event.channel.id()));
                update(&cache_and_http, &mut event);

                event_handler.channel_update(context, old_channel, event.channel).await;
            });
        },
        Event::GuildBanAdd(event) => {
            spawn_named("dispatch::event_handler::guild_ban_addition", async move {
                event_handler.guild_ban_addition(context, event.guild_id, event.user).await;
            });
        },
        Event::GuildBanRemove(event) => {
            spawn_named("dispatch::event_handler::guild_ban_removal", async move {
                event_handler.guild_ban_removal(context, event.guild_id, event.user).await;
            });
        },
        Event::GuildCreate(mut event) => {
            let _is_new = if_cache!(Some(cache.unavailable_guilds.contains(&event.guild.id)));

            update(&cache_and_http, &mut event);

            #[cfg(feature = "cache")]
            {
                let context = context.clone();

                if cache_and_http.cache.unavailable_guilds.is_empty() {
                    let guild_amount = cache_and_http
                        .cache
                        .guilds
                        .iter()
                        .map(|i| *i.key())
                        .collect::<Vec<GuildId>>();
                    let event_handler = Arc::clone(&event_handler);

                    spawn_named("dispatch::event_handler::cache_ready", async move {
                        event_handler.cache_ready(context, guild_amount).await;
                    });
                }
            }

            spawn_named("dispatch::event_handler::guild_create", async move {
                event_handler.guild_create(context, event.guild, _is_new).await;
            });
        },
        Event::GuildDelete(mut event) => {
            let _full = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::guild_delete", async move {
                event_handler.guild_delete(context, event.guild, _full).await;
            });
        },
        Event::GuildEmojisUpdate(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::guild_emojis_update", async move {
                event_handler.guild_emojis_update(context, event.guild_id, event.emojis).await;
            });
        },
        Event::GuildIntegrationsUpdate(event) => {
            spawn_named("dispatch::event_handler::guild_integrations_update", async move {
                event_handler.guild_integrations_update(context, event.guild_id).await;
            });
        },
        Event::GuildMemberAdd(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::guild_member_addition", async move {
                event_handler.guild_member_addition(context, event.member).await;
            });
        },
        Event::GuildMemberRemove(mut event) => {
            let _member = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::guild_member_removal", async move {
                event_handler
                    .guild_member_removal(context, event.guild_id, event.user, _member)
                    .await;
            });
        },
        Event::GuildMemberUpdate(mut event) => {
            let _before = if_cache!(update(&cache_and_http, &mut event));
            let _after: Option<Member> = if_cache!(cache.member(event.guild_id, event.user.id));

            spawn_named("dispatch::event_handler::guild_member_update", async move {
                event_handler.guild_member_update(context, _before, _after, event).await;
            });
        },
        Event::GuildMembersChunk(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::guild_members_chunk", async move {
                event_handler.guild_members_chunk(context, event).await;
            });
        },
        Event::GuildRoleCreate(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::guild_role_create", async move {
                event_handler.guild_role_create(context, event.role).await;
            });
        },
        Event::GuildRoleDelete(mut event) => {
            let _role = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::guild_role_delete", async move {
                event_handler
                    .guild_role_delete(context, event.guild_id, event.role_id, _role)
                    .await;
            });
        },
        Event::GuildRoleUpdate(mut event) => {
            let _before = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::guild_role_update", async move {
                event_handler.guild_role_update(context, _before, event.role).await;
            });
        },
        Event::GuildStickersUpdate(mut event) => {
            update(&cache_and_http, &mut event);

            tokio::spawn(async move {
                event_handler.guild_stickers_update(context, event.guild_id, event.stickers).await;
            });
        },
        Event::GuildUpdate(mut event) => {
            spawn_named("dispatch::event_handler::guild_update", async move {
                let before = if_cache!(cache.guild(event.guild.id).map(|g| g.clone()));

                update(&cache_and_http, &mut event);

                event_handler.guild_update(context, before, event.guild).await;
            });
        },
        Event::InviteCreate(event) => {
            spawn_named("dispatch::event_handler::invite_create", async move {
                event_handler.invite_create(context, event).await;
            });
        },
        Event::InviteDelete(event) => {
            spawn_named("dispatch::event_handler::invite_delete", async move {
                event_handler.invite_delete(context, event).await;
            });
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(event) => {
            spawn_named("dispatch::event_handler::message_delete_bulk", async move {
                event_handler
                    .message_delete_bulk(context, event.channel_id, event.ids, event.guild_id)
                    .await;
            });
        },
        Event::MessageDelete(event) => {
            spawn_named("dispatch::event_handler::message_delete", async move {
                event_handler
                    .message_delete(context, event.channel_id, event.message_id, event.guild_id)
                    .await;
            });
        },
        Event::MessageUpdate(mut event) => {
            let _before = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::message_update", async move {
                let _after = if_cache!(cache.message(event.channel_id, event.id));
                event_handler.message_update(context, _before, _after, event).await;
            });
        },
        Event::PresencesReplace(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::presence_replace", async move {
                event_handler.presence_replace(context, event.presences).await;
            });
        },
        Event::PresenceUpdate(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::presence_update", async move {
                event_handler.presence_update(context, event.presence).await;
            });
        },
        Event::ReactionAdd(event) => {
            spawn_named("dispatch::event_handler::reaction_add", async move {
                event_handler.reaction_add(context, event.reaction).await;
            });
        },
        Event::ReactionRemove(event) => {
            spawn_named("dispatch::event_handler::reaction_remove", async move {
                event_handler.reaction_remove(context, event.reaction).await;
            });
        },
        Event::ReactionRemoveAll(event) => {
            spawn_named("dispatch::event_handler::remove_all", async move {
                event_handler
                    .reaction_remove_all(context, event.channel_id, event.message_id)
                    .await;
            });
        },
        Event::Ready(mut event) => {
            update(&cache_and_http, &mut event);
            spawn_named("dispatch::event_handler::ready", async move {
                event_handler.ready(context, event.ready).await;
            });
        },
        Event::Resumed(event) => {
            spawn_named("dispatch::event_handler::resume", async move {
                event_handler.resume(context, event).await;
            });
        },
        Event::TypingStart(event) => {
            spawn_named("dispatch::event_handler::typing_start", async move {
                event_handler.typing_start(context, event).await;
            });
        },
        Event::Unknown => debug!("An unknown event was received"),
        Event::UserUpdate(mut event) => {
            let _before = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::user_update", async move {
                event_handler.user_update(context, _before, event.current_user).await;
            });
        },
        Event::VoiceServerUpdate(event) => {
            spawn_named("dispatch::event_handler::voice_server_update", async move {
                event_handler.voice_server_update(context, event).await;
            });
        },
        Event::VoiceStateUpdate(mut event) => {
            let _before = if_cache!(update(&cache_and_http, &mut event));

            spawn_named("dispatch::event_handler::voice_state_update", async move {
                event_handler.voice_state_update(context, _before, event.voice_state).await;
            });
        },
        Event::WebhookUpdate(event) => {
            spawn_named("dispatch::event_handler::webhook_update", async move {
                event_handler.webhook_update(context, event.guild_id, event.channel_id).await;
            });
        },
        Event::InteractionCreate(event) => {
            spawn_named("dispatch::event_handler::interaction_create", async move {
                event_handler.interaction_create(context, event.interaction).await;
            });
        },
        Event::IntegrationCreate(event) => {
            spawn_named("dispatch::event_handler::integration_create", async move {
                event_handler.integration_create(context, event.integration).await;
            });
        },
        Event::IntegrationUpdate(event) => {
            spawn_named("dispatch::event_handler::integration_update", async move {
                event_handler.integration_update(context, event.integration).await;
            });
        },
        Event::IntegrationDelete(event) => {
            spawn_named("dispatch::event_handler::integration_delete", async move {
                event_handler
                    .integration_delete(context, event.id, event.guild_id, event.application_id)
                    .await;
            });
        },
        Event::StageInstanceCreate(event) => {
            spawn_named("dispatch::event_handler::stage_instance_create", async move {
                event_handler.stage_instance_create(context, event.stage_instance).await;
            });
        },
        Event::StageInstanceUpdate(event) => {
            spawn_named("dispatch::event_handler::stage_instance_update", async move {
                event_handler.stage_instance_update(context, event.stage_instance).await;
            });
        },
        Event::StageInstanceDelete(event) => {
            spawn_named("dispatch::event_handler::stage_instance_delete", async move {
                event_handler.stage_instance_delete(context, event.stage_instance).await;
            });
        },
        Event::ThreadCreate(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::thread_create", async move {
                event_handler.thread_create(context, event.thread).await;
            });
        },
        Event::ThreadUpdate(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::thread_update", async move {
                event_handler.thread_update(context, event.thread).await;
            });
        },
        Event::ThreadDelete(mut event) => {
            update(&cache_and_http, &mut event);

            spawn_named("dispatch::event_handler::thread_delete", async move {
                event_handler.thread_delete(context, event.thread).await;
            });
        },
        Event::ThreadListSync(event) => {
            spawn_named("dispatch::event_handler::thread_list_sync", async move {
                event_handler.thread_list_sync(context, event).await;
            });
        },
        Event::ThreadMemberUpdate(event) => {
            spawn_named("dispatch::event_handler::thread_member_update", async move {
                event_handler.thread_member_update(context, event.member).await;
            });
        },
        Event::ThreadMembersUpdate(event) => {
            spawn_named("dispatch::event_handler::thread_members_update", async move {
                event_handler.thread_members_update(context, event).await;
            });
        },
        Event::GuildScheduledEventCreate(event) => {
            spawn_named("dispatch::event_handler::guild_scheduled_event_create", async move {
                event_handler.guild_scheduled_event_create(context, event.event).await;
            });
        },
        Event::GuildScheduledEventUpdate(event) => {
            spawn_named("dispatch::event_handler::guild_scheduled_event_update", async move {
                event_handler.guild_scheduled_event_update(context, event.event).await;
            });
        },
        Event::GuildScheduledEventDelete(event) => {
            spawn_named("dispatch::event_handler::guild_scheduled_event_delete", async move {
                event_handler.guild_scheduled_event_delete(context, event.event).await;
            });
        },
        Event::GuildScheduledEventUserAdd(event) => {
            spawn_named("dispatch::event_handler::guild_scheduled_event_user_add", async move {
                event_handler.guild_scheduled_event_user_add(context, event).await;
            });
        },
        Event::GuildScheduledEventUserRemove(event) => {
            spawn_named("dispatch::event_handler::guild_scheduled_event_user_remove", async move {
                event_handler.guild_scheduled_event_user_remove(context, event).await;
            });
        },
    }
}
