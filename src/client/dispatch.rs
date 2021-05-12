#[cfg(feature = "cache")]
use std::fmt;
use std::sync::Arc;

use futures::{
    channel::mpsc::UnboundedSender as Sender,
    future::{BoxFuture, FutureExt},
};
use tokio::sync::RwLock;
use tracing::instrument;
use typemap_rev::TypeMap;

use super::Context;
#[cfg(feature = "gateway")]
use super::{
    bridge::gateway::event::ClientEvent,
    event_handler::{EventHandler, RawEventHandler},
};
#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheUpdate};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::InterMessage;
use crate::http::Http;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;
use crate::model::{
    channel::{Channel, Message},
    event::Event,
    guild::Member,
};
use crate::CacheAndHttp;

#[inline]
#[cfg(feature = "cache")]
async fn update<E: CacheUpdate + fmt::Debug>(
    cache_and_http: &Arc<CacheAndHttp>,
    event: &mut E,
) -> Option<E::Output> {
    cache_and_http.cache.update(event).await
}

#[inline]
#[cfg(not(feature = "cache"))]
async fn update<E>(_cache_and_http: &Arc<CacheAndHttp>, _event: &mut E) -> Option<()> {
    None
}

#[cfg(feature = "cache")]
fn context(
    data: &Arc<RwLock<TypeMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    http: &Arc<Http>,
    cache: &Arc<Cache>,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id, Arc::clone(http), Arc::clone(cache))
}

#[cfg(not(feature = "cache"))]
fn context(
    data: &Arc<RwLock<TypeMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    http: &Arc<Http>,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id, Arc::clone(http))
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
    async fn update(&mut self, cache_and_http: &Arc<CacheAndHttp>) {
        match self {
            Self::Model(Event::ChannelCreate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::ChannelDelete(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::ChannelUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildCreate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildDelete(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildEmojisUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildMemberAdd(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildMemberRemove(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildMemberUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildMembersChunk(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildRoleCreate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildRoleDelete(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildRoleUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::GuildUnavailable(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            // Already handled by the framework check macro
            Self::Model(Event::MessageCreate(_)) => {},
            Self::Model(Event::MessageUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::PresencesReplace(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::PresenceUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::Ready(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::UserUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            Self::Model(Event::VoiceStateUpdate(ref mut event)) => {
                update(cache_and_http, event).await;
            },
            _ => (),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch<'rec>(
    // #[allow(unused_variables)]
    mut event: DispatchEvent,
    #[cfg(feature = "framework")] framework: &'rec Arc<Box<dyn Framework + Send + Sync>>,
    data: &'rec Arc<RwLock<TypeMap>>,
    event_handler: &'rec Option<Arc<dyn EventHandler>>,
    raw_event_handler: &'rec Option<Arc<dyn RawEventHandler>>,
    runner_tx: &'rec Sender<InterMessage>,
    shard_id: u64,
    cache_and_http: Arc<CacheAndHttp>,
) -> BoxFuture<'rec, ()> {
    async move {
        match (event_handler, raw_event_handler) {
            (None, None) => {
                event.update(&cache_and_http).await;

                if let DispatchEvent::Model(Event::MessageCreate(event)) = event {
                    #[cfg(feature = "framework")]
                    {
                        #[cfg(not(feature = "cache"))]
                        let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                        #[cfg(feature = "cache")]
                        let context = context(
                            data,
                            runner_tx,
                            shard_id,
                            &cache_and_http.http,
                            &cache_and_http.cache,
                        );

                        let framework = Arc::clone(&framework);

                        tokio::spawn(async move {
                            framework.dispatch(context, event.message).await;
                        });
                    }
                }
            },
            (Some(ref h), None) => match event {
                DispatchEvent::Model(Event::MessageCreate(mut event)) => {
                    update(&cache_and_http, &mut event).await;

                    #[cfg(not(feature = "cache"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                    #[cfg(feature = "cache")]
                    let context = context(
                        data,
                        runner_tx,
                        shard_id,
                        &cache_and_http.http,
                        &cache_and_http.cache,
                    );

                    #[cfg(not(feature = "framework"))]
                    {
                        // Avoid cloning if there will be no framework dispatch.
                        dispatch_message(context, event.message, h).await;
                    }

                    #[cfg(feature = "framework")]
                    {
                        dispatch_message(context.clone(), event.message.clone(), h).await;

                        let framework = Arc::clone(&framework);

                        tokio::spawn(async move {
                            framework.dispatch(context, event.message).await;
                        });
                    }
                },
                other => {
                    handle_event(other, data, h, runner_tx, shard_id, cache_and_http).await;
                },
            },
            (None, Some(ref rh)) => {
                event.update(&cache_and_http).await;

                if let DispatchEvent::Model(event) = event {
                    let event_handler = Arc::clone(rh);

                    #[cfg(not(feature = "cache"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                    #[cfg(feature = "cache")]
                    let context = context(
                        data,
                        runner_tx,
                        shard_id,
                        &cache_and_http.http,
                        &cache_and_http.cache,
                    );

                    #[cfg(not(feature = "framework"))]
                    {
                        // No clone needed, as there will be no framework dispatch.
                        event_handler.raw_event(context, event).await;
                    }

                    #[cfg(feature = "framework")]
                    {
                        if let Event::MessageCreate(ref msg_event) = event {
                            // Must clone in order to dispatch the framework too.
                            let message = msg_event.message.clone();
                            event_handler.raw_event(context.clone(), event).await;

                            let framework = Arc::clone(&framework);

                            tokio::spawn(async move {
                                framework.dispatch(context, message).await;
                            });
                        } else {
                            // Avoid cloning if there will be no framework dispatch.
                            event_handler.raw_event(context, event).await;
                        }
                    }
                }
            },
            // We call this function again, passing `None` for each event handler
            // and passing no framework, as we dispatch once we are done right here.
            (Some(ref handler), Some(ref raw_handler)) => {
                #[cfg(not(feature = "cache"))]
                let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                #[cfg(feature = "cache")]
                let context =
                    context(data, runner_tx, shard_id, &cache_and_http.http, &cache_and_http.cache);

                if let DispatchEvent::Model(ref event) = event {
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

                            let framework = Arc::clone(&framework);
                            let message = event.message;
                            tokio::spawn(async move {
                                framework.dispatch(context, message).await;
                            });
                        }
                    },
                    other => {
                        handle_event(other, data, handler, runner_tx, shard_id, cache_and_http)
                            .await
                    },
                }
            },
        }
    }
    .boxed()
}

async fn dispatch_message(
    context: Context,
    mut message: Message,
    event_handler: &Arc<dyn EventHandler>,
) {
    #[cfg(feature = "model")]
    {
        message.transform_content();
    }

    let event_handler = Arc::clone(event_handler);

    tokio::spawn(async move {
        event_handler.message(context, message).await;
    });
}
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::too_many_arguments)]
#[instrument(skip(event, data, event_handler, cache_and_http))]
async fn handle_event(
    event: DispatchEvent,
    data: &Arc<RwLock<TypeMap>>,
    event_handler: &Arc<dyn EventHandler>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    cache_and_http: Arc<CacheAndHttp>,
) {
    #[cfg(not(feature = "cache"))]
    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
    #[cfg(feature = "cache")]
    let context = context(data, runner_tx, shard_id, &cache_and_http.http, &cache_and_http.cache);

    match event {
        DispatchEvent::Client(ClientEvent::ShardStageUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.shard_stage_update(context, event).await;
            });
        },
        DispatchEvent::Model(Event::ChannelCreate(mut event)) => {
            update(&cache_and_http, &mut event).await;
            match event.channel {
                Channel::Guild(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    tokio::spawn(async move {
                        event_handler.channel_create(context, &channel).await;
                    });
                },
                Channel::Category(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    tokio::spawn(async move {
                        event_handler.category_create(context, &channel).await;
                    });
                },
                // Private channel create events are no longer sent to bots in the v8 gateway.
                _ => {},
            }
        },
        DispatchEvent::Model(Event::ChannelDelete(mut event)) => {
            update(&cache_and_http, &mut event).await;

            match event.channel {
                Channel::Private(_) => {},
                Channel::Guild(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    tokio::spawn(async move {
                        event_handler.channel_delete(context, &channel).await;
                    });
                },
                Channel::Category(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    tokio::spawn(async move {
                        event_handler.category_delete(context, &channel).await;
                    });
                },
            }
        },
        DispatchEvent::Model(Event::ChannelPinsUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.channel_pins_update(context, event).await;
            });
        },
        DispatchEvent::Model(Event::ChannelUpdate(mut event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    let old_channel = cache_and_http.cache.as_ref().channel(event.channel.id()).await;
                    update(&cache_and_http, &mut event).await;

                    event_handler.channel_update(context, old_channel, event.channel).await;
                } else {
                    update(&cache_and_http, &mut event).await;

                    event_handler.channel_update(context, event.channel).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildBanAdd(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_ban_addition(context, event.guild_id, event.user).await;
            });
        },
        DispatchEvent::Model(Event::GuildBanRemove(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_ban_removal(context, event.guild_id, event.user).await;
            });
        },
        DispatchEvent::Model(Event::GuildCreate(mut event)) => {
            #[cfg(feature = "cache")]
            let _is_new =
                { !cache_and_http.cache.unavailable_guilds.read().await.contains(&event.guild.id) };

            update(&cache_and_http, &mut event).await;

            #[cfg(feature = "cache")]
            {
                let context = context.clone();

                if cache_and_http.cache.unavailable_guilds.read().await.is_empty() {
                    let guild_amount = cache_and_http
                        .cache
                        .guilds
                        .read()
                        .await
                        .iter()
                        .map(|(&id, _)| id)
                        .collect::<Vec<GuildId>>();
                    let event_handler = Arc::clone(event_handler);

                    tokio::spawn(async move {
                        event_handler.cache_ready(context, guild_amount).await;
                    });
                }
            }

            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.guild_create(context, event.guild, _is_new).await;
                } else {
                    event_handler.guild_create(context, event.guild).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildDelete(mut event)) => {
            let _full = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.guild_delete(context, event.guild, _full).await;
                } else {
                    event_handler.guild_delete(context, event.guild).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildEmojisUpdate(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_emojis_update(context, event.guild_id, event.emojis).await;
            });
        },
        DispatchEvent::Model(Event::GuildIntegrationsUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_integrations_update(context, event.guild_id).await;
            });
        },
        DispatchEvent::Model(Event::GuildMemberAdd(mut event)) => {
            update(&cache_and_http, &mut event).await;

            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_member_addition(context, event.guild_id, event.member).await;
            });
        },
        DispatchEvent::Model(Event::GuildMemberRemove(mut event)) => {
            let _member = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.guild_member_removal(context, event.guild_id, event.user, _member).await;
                } else {
                    event_handler.guild_member_removal(context, event.guild_id, event.user).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildMemberUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event).await;
            let _after: Option<Member> = feature_cache! {{
                cache_and_http.cache.member(event.guild_id, event.user.id).await
            } else {
                None
            }};

            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    if let Some(after) = _after {
                        event_handler.guild_member_update(context, _before, after).await;
                    }
                } else {
                    event_handler.guild_member_update(context, event).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildMembersChunk(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_members_chunk(context, event).await;
            });
        },
        DispatchEvent::Model(Event::GuildRoleCreate(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_role_create(context, event.guild_id, event.role).await;
            });
        },
        DispatchEvent::Model(Event::GuildRoleDelete(mut event)) => {
            let _role = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.guild_role_delete(context, event.guild_id, event.role_id, _role).await;
                } else {
                    event_handler.guild_role_delete(context, event.guild_id, event.role_id).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildRoleUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.guild_role_update(context, event.guild_id, _before, event.role).await;
                } else {
                    event_handler.guild_role_update(context, event.guild_id, event.role).await;
                }}
            });
        },
        DispatchEvent::Model(Event::GuildUnavailable(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.guild_unavailable(context, event.guild_id).await;
            });
        },
        DispatchEvent::Model(Event::GuildUpdate(mut event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    let before = cache_and_http.cache
                        .guild(&event.guild.id)
                        .await;

                    update(&cache_and_http, &mut event).await;

                    event_handler.guild_update(context, before, event.guild).await;
                } else {
                    update(&cache_and_http, &mut event).await;

                    event_handler.guild_update(context, event.guild).await;
                }}
            });
        },
        DispatchEvent::Model(Event::InviteCreate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.invite_create(context, event).await;
            });
        },
        DispatchEvent::Model(Event::InviteDelete(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.invite_delete(context, event).await;
            });
        },
        // Already handled by the framework check macro
        DispatchEvent::Model(Event::MessageCreate(_)) => {},
        DispatchEvent::Model(Event::MessageDeleteBulk(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler
                    .message_delete_bulk(context, event.channel_id, event.ids, event.guild_id)
                    .await;
            });
        },
        DispatchEvent::Model(Event::MessageDelete(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler
                    .message_delete(context, event.channel_id, event.message_id, event.guild_id)
                    .await;
            });
        },
        DispatchEvent::Model(Event::MessageUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    let _after = cache_and_http.cache.message(event.channel_id, event.id).await;
                    event_handler.message_update(context, _before, _after, event).await;
                } else {
                    event_handler.message_update(context, event).await;
                }}
            });
        },
        DispatchEvent::Model(Event::PresencesReplace(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.presence_replace(context, event.presences).await;
            });
        },
        DispatchEvent::Model(Event::PresenceUpdate(mut event)) => {
            update(&cache_and_http, &mut event).await;

            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.presence_update(context, event).await;
            });
        },
        DispatchEvent::Model(Event::ReactionAdd(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.reaction_add(context, event.reaction).await;
            });
        },
        DispatchEvent::Model(Event::ReactionRemove(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.reaction_remove(context, event.reaction).await;
            });
        },
        DispatchEvent::Model(Event::ReactionRemoveAll(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler
                    .reaction_remove_all(context, event.channel_id, event.message_id)
                    .await;
            });
        },
        DispatchEvent::Model(Event::Ready(mut event)) => {
            update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(&event_handler);

            tokio::spawn(async move {
                event_handler.ready(context, event.ready).await;
            });
        },
        DispatchEvent::Model(Event::Resumed(event)) => {
            let event_handler = Arc::clone(&event_handler);

            tokio::spawn(async move {
                event_handler.resume(context, event).await;
            });
        },
        DispatchEvent::Model(Event::TypingStart(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.typing_start(context, event).await;
            });
        },
        DispatchEvent::Model(Event::Unknown(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.unknown(context, event.kind, event.value).await;
            });
        },
        DispatchEvent::Model(Event::UserUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.user_update(context, _before.expect("missing old user"), event.current_user).await;
                } else {
                    event_handler.user_update(context, event.current_user).await;
                }}
            });
        },
        DispatchEvent::Model(Event::VoiceServerUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.voice_server_update(context, event).await;
            });
        },
        DispatchEvent::Model(Event::VoiceStateUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event).await;
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                feature_cache! {{
                    event_handler.voice_state_update(context, event.guild_id, _before, event.voice_state).await;
                } else {
                    event_handler.voice_state_update(context, event.guild_id, event.voice_state).await;
                }}
            });
        },
        DispatchEvent::Model(Event::WebhookUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.webhook_update(context, event.guild_id, event.channel_id).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::InteractionCreate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.interaction_create(context, event.interaction).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::IntegrationCreate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.integration_create(context, event.integration).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::IntegrationUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.integration_update(context, event.integration).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::IntegrationDelete(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler
                    .integration_delete(context, event.id, event.guild_id, event.application_id)
                    .await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::ApplicationCommandCreate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.application_command_create(context, event.application_command).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::ApplicationCommandUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.application_command_update(context, event.application_command).await;
            });
        },
        #[cfg(feature = "unstable_discord_api")]
        DispatchEvent::Model(Event::ApplicationCommandDelete(event)) => {
            let event_handler = Arc::clone(event_handler);

            tokio::spawn(async move {
                event_handler.application_command_delete(context, event.application_command).await;
            });
        },
    }
}
