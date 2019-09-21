use crate::gateway::InterMessage;
use crate::model::{
    channel::{Channel, Message},
    event::Event,
    guild::Member,
};
use std::{sync::{Arc, mpsc::Sender}};
use parking_lot::{Mutex, RwLock};
use super::{
    bridge::gateway::event::ClientEvent,
    event_handler::{EventHandler, RawEventHandler},
    Context
};
use threadpool::ThreadPool;
use typemap::ShareMap;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(any(feature = "cache", feature = "http"))]
use crate::CacheAndHttp;
#[cfg(feature = "cache")]
use crate::cache::CacheUpdate;
#[cfg(feature = "cache")]
use std::fmt;
#[cfg(feature = "cache")]
use log::warn;

#[inline]
#[cfg(feature = "cache")]
fn update<E: CacheUpdate + fmt::Debug>(cache_and_http: &Arc<CacheAndHttp>, event: &mut E) -> Option<E::Output> {
    if let Some(millis_timeout) = cache_and_http.update_cache_timeout {

        if let Some(mut lock) = cache_and_http.cache.try_write_for(millis_timeout) {
            lock.update(event)
        } else {
            warn!("[dispatch] Possible deadlock: Couldn't unlock cache to update with event: {:?}", event);

            None
        }
    } else {
        cache_and_http.cache.write().update(event)
    }
}

#[inline]
#[cfg(not(feature = "cache"))]
fn update<E>(_cache_and_http: &Arc<CacheAndHttp>, _event: &mut E) -> Option<()> {
    None
}

#[cfg(all(feature = "cache", feature = "http"))]
fn context(
    data: &Arc<RwLock<ShareMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    cache: &Arc<RwLock<Cache>>,
    http: &Arc<Http>,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id, cache.clone(), Arc::clone(http))
}

#[cfg(all(feature = "cache", not(feature = "http")))]
fn context(
    data: &Arc<RwLock<ShareMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    cache: &Arc<RwLock<Cache>>,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id, cache.clone())
}

#[cfg(all(not(feature = "cache"), feature = "http"))]
fn context(
    data: &Arc<RwLock<ShareMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
    http: &Arc<Http>,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id, http.clone())
}

#[cfg(not(any(feature = "cache", feature = "http")))]
fn context(
    data: &Arc<RwLock<ShareMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id)
}

// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::large_enum_variant)]
pub(crate) enum DispatchEvent {
    Client(ClientEvent),
    Model(Event),
    #[doc(hidden)]
    __Nonexhaustive,
}

#[cfg(feature = "framework")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch(
    event: DispatchEvent,
    framework: &Arc<Mutex<Option<Box<dyn Framework + Send>>>>,
    data: &Arc<RwLock<ShareMap>>,
    event_handler: &Option<Arc<dyn EventHandler>>,
    raw_event_handler: &Option<Arc<dyn RawEventHandler>>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
    cache_and_http: Arc<CacheAndHttp>,
) {
    match (event_handler, raw_event_handler) {
        (None, None) => {}, // Do nothing
        (Some(ref h), None) => {
            match event {
                DispatchEvent::Model(Event::MessageCreate(mut event)) => {
                    update(&cache_and_http, &mut event);

                    #[cfg(not(any(feature = "cache", feature = "http")))]
                    let context = context(data, runner_tx, shard_id);
                    #[cfg(all(feature = "cache", not(feature = "http")))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache);
                    #[cfg(all(not(feature = "cache"), feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                    #[cfg(all(feature = "cache", feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache, &cache_and_http.http);

                    dispatch_message(
                        context.clone(),
                        event.message.clone(),
                        h,
                        threadpool,
                    );
                    if let Some(ref mut framework) = *framework.lock() {
                        framework.dispatch(context, event.message, threadpool);
                    }
                },
                other => {
                    handle_event(
                        other,
                        data,
                        h,
                        runner_tx,
                        threadpool,
                        shard_id,
                        cache_and_http,
                    );
                }
            }
        },
        (None, Some(ref rh)) => {
            if let DispatchEvent::Model(e) = event {
                #[cfg(not(any(feature = "cache", feature = "http")))]
                let context = context(data, runner_tx, shard_id);
                #[cfg(all(feature = "cache", not(feature = "http")))]
                let context = context(data, runner_tx, shard_id, &cache_and_http.cache);
                #[cfg(all(not(feature = "cache"), feature = "http"))]
                let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                #[cfg(all(feature = "cache", feature = "http"))]
                let context = context(data, runner_tx, shard_id, &cache_and_http.cache, &cache_and_http.http);

                let event_handler = Arc::clone(rh);
                threadpool.execute(move || {
                    event_handler.raw_event(context, e);
                });
            }
        },
        (Some(_), Some(_)) => {
            if let DispatchEvent::Model(ref e) = event {
                    dispatch(DispatchEvent::Model(e.clone()),
                             framework,
                             data,
                             &None,
                             raw_event_handler,
                             runner_tx,
                             threadpool,
                             shard_id,
                             Arc::clone(&cache_and_http))
            }
            dispatch(event,
                     framework,
                     data,
                     event_handler,
                     &None,
                     runner_tx,
                     threadpool,
                     shard_id,
                     cache_and_http);
        }
    };
}

#[cfg(not(feature = "framework"))]
pub(crate) fn dispatch(
    event: DispatchEvent,
    data: &Arc<RwLock<ShareMap>>,
    event_handler: &Option<Arc<dyn EventHandler>>,
    raw_event_handler: &Option<Arc<dyn RawEventHandler>>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
    cache_and_http: Arc<CacheAndHttp>,
) {
    match (event_handler, raw_event_handler) {
        (None, None) => {}, // Do nothing
        (Some(ref h), None) => {
            match event {
                DispatchEvent::Model(Event::MessageCreate(mut event)) => {
                    update(&cache_and_http, &mut event);

                    #[cfg(not(any(feature = "cache", feature = "http")))]
                    let context = context(data, runner_tx, shard_id);
                    #[cfg(all(feature = "cache", not(feature = "http")))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache);
                    #[cfg(all(not(feature = "cache"), feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                    #[cfg(all(feature = "cache", feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache, &cache_and_http.http);

                    dispatch_message(
                        context.clone(),
                        event.message.clone(),
                        h,
                        threadpool,
                    );
                },
                other => {
                    handle_event(
                        other,
                        data,
                        h,
                        runner_tx,
                        threadpool,
                        shard_id,
                        cache_and_http,
                    );
                }
            }
        },
        (None, Some(ref rh)) => {
            match event {
                DispatchEvent::Model(e) => {
                    #[cfg(not(any(feature = "cache", feature = "http")))]
                    let context = context(data, runner_tx, shard_id);
                    #[cfg(all(feature = "cache", not(feature = "http")))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache);
                    #[cfg(all(not(feature = "cache"), feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
                    #[cfg(all(feature = "cache", feature = "http"))]
                    let context = context(data, runner_tx, shard_id, &cache_and_http.cache, &cache_and_http.http);

                    let event_handler = Arc::clone(rh);
                    threadpool.execute(move || {
                        event_handler.raw_event(context, e);
                    });
                },
                _ => {}
            }
        },
        (Some(ref h), Some(ref rh)) => {
            match event {
                DispatchEvent::Model(ref e) =>
                    dispatch(DispatchEvent::Model(e.clone()),
                             data,
                             &None,
                             raw_event_handler,
                             runner_tx,
                             threadpool,
                             shard_id,
                             Arc::clone(&cache_and_http)),
                _ => {}
            }
            dispatch(event,
                     data,
                     event_handler,
                     &None,
                     runner_tx,
                     threadpool,
                     shard_id,
                     cache_and_http);
        }
    };
}

fn dispatch_message(
    context: Context,
    mut message: Message,
    event_handler: &Arc<dyn EventHandler>,
    threadpool: &ThreadPool,
) {
    #[cfg(feature = "model")]
    {
        message.transform_content();
    }

    let event_handler = Arc::clone(event_handler);

    threadpool.execute(move || {
        event_handler.message(context, message);
    });
}
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::too_many_arguments)]
fn handle_event(
    event: DispatchEvent,
    data: &Arc<RwLock<ShareMap>>,
    event_handler: &Arc<dyn EventHandler>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
    cache_and_http: Arc<CacheAndHttp>,
) {
    #[cfg(not(any(feature = "cache", feature = "http")))]
    let context = context(data, runner_tx, shard_id);
    #[cfg(all(feature = "cache", not(feature = "http")))]
    let context = context(data, runner_tx, shard_id, &cache_and_http.cache);
    #[cfg(all(not(feature = "cache"), feature = "http"))]
    let context = context(data, runner_tx, shard_id, &cache_and_http.http);
    #[cfg(all(feature = "cache", feature = "http"))]
    let context = context(data, runner_tx, shard_id, &cache_and_http.cache, &cache_and_http.http);

    match event {
        DispatchEvent::Client(ClientEvent::ShardStageUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.shard_stage_update(context, event);
            });
        }
        DispatchEvent::Model(Event::ChannelCreate(mut event)) => {
            update(&cache_and_http, &mut event);
            // Discord sends both a MessageCreate and a ChannelCreate upon a new message in a private channel.
            // This could potentially be annoying to handle when otherwise wanting to normally take care of a new channel.
            // So therefore, private channels are dispatched to their own handler code.
            match event.channel {
                Channel::Private(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.private_channel_create(context, channel);
                    });
                },
                Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.channel_create(context, channel);
                    });
                },
                Channel::Category(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.category_create(context, channel);
                    });
                },
                Channel::__Nonexhaustive => unreachable!(),
            }
        },
        DispatchEvent::Model(Event::ChannelDelete(mut event)) => {
            update(&cache_and_http, &mut event);

            match event.channel {
                Channel::Private(_) | Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.channel_delete(context, channel);
                    });
                },
                Channel::Category(channel) => {
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.category_delete(context, channel);
                    });
                },
                Channel::__Nonexhaustive => unreachable!(),
            }
        },
        DispatchEvent::Model(Event::ChannelPinsUpdate(event)) => {

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.channel_pins_update(context, event);
            });
        },
        DispatchEvent::Model(Event::ChannelRecipientAdd(mut event)) => {
            update(&cache_and_http, &mut event);

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.channel_recipient_addition(
                    context,
                    event.channel_id,
                    event.user,
                );
            });
        },
        DispatchEvent::Model(Event::ChannelRecipientRemove(mut event)) => {
            update(&cache_and_http, &mut event);

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.channel_recipient_removal(
                    context,
                    event.channel_id,
                    event.user,
                );
            });
        },
        DispatchEvent::Model(Event::ChannelUpdate(mut event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    let before = cache_and_http.cache.as_ref().read().channel(event.channel.id());
                    update(&cache_and_http, &mut event);

                    event_handler.channel_update(context, before, event.channel);
                } else {
                    update(&cache_and_http, &mut event);

                    event_handler.channel_update(context, event.channel);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildBanAdd(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_addition(context, event.guild_id, event.user);
            });
        },
        DispatchEvent::Model(Event::GuildBanRemove(event)) => {

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_removal(context, event.guild_id, event.user);
            });
        },
        DispatchEvent::Model(Event::GuildCreate(mut event)) => {
            #[cfg(feature = "cache")]
            let _is_new = {
                let cache = cache_and_http.cache.as_ref().read();

                !cache.unavailable_guilds.contains(&event.guild.id)
            };

            update(&cache_and_http, &mut event);

            #[cfg(feature = "cache")]
            {
                let locked_cache = cache_and_http.cache.as_ref().read();
                let context = context.clone();

                if locked_cache.unavailable_guilds.is_empty() {
                    let guild_amount = locked_cache
                        .guilds
                        .iter()
                        .map(|(&id, _)| id)
                        .collect::<Vec<GuildId>>();
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.cache_ready(context, guild_amount);
                    });
                }
            }

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.guild_create(context, event.guild, _is_new);
                } else {
                    event_handler.guild_create(context, event.guild);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildDelete(mut event)) => {
            let _full = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.guild_delete(context, event.guild, _full);
                } else {
                    event_handler.guild_delete(context, event.guild);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildEmojisUpdate(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_emojis_update(context, event.guild_id, event.emojis);
            });
        },
        DispatchEvent::Model(Event::GuildIntegrationsUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_integrations_update(context, event.guild_id);
            });
        },
        DispatchEvent::Model(Event::GuildMemberAdd(mut event)) => {
            update(&cache_and_http, &mut event);

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_member_addition(context, event.guild_id, event.member);
            });
        },
        DispatchEvent::Model(Event::GuildMemberRemove(mut event)) => {
            let _member = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.guild_member_removal(context, event.guild_id, event.user, _member);
                } else {
                    event_handler.guild_member_removal(context, event.guild_id, event.user);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildMemberUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event);
            let _after: Option<Member> = feature_cache! {{
                cache_and_http.cache.as_ref().read().member(event.guild_id, event.user.id)
            } else {
                None
            }};

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    if let Some(after) = _after {
                        event_handler.guild_member_update(context, _before, after);
                    }
                } else {
                    event_handler.guild_member_update(context, event);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildMembersChunk(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_members_chunk(context, event.guild_id, event.members);
            });
        },
        DispatchEvent::Model(Event::GuildRoleCreate(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_role_create(context, event.guild_id, event.role);
            });
        },
        DispatchEvent::Model(Event::GuildRoleDelete(mut event)) => {
            let _role = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.guild_role_delete(context, event.guild_id, event.role_id, _role);
                } else {
                    event_handler.guild_role_delete(context, event.guild_id, event.role_id);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildRoleUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.guild_role_update(context, event.guild_id, _before, event.role);
                } else {
                    event_handler.guild_role_update(context, event.guild_id, event.role);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildUnavailable(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_unavailable(context, event.guild_id);
            });
        },
        DispatchEvent::Model(Event::GuildUpdate(mut event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    let before = cache_and_http.cache.as_ref().read()
                        .guilds
                        .get(&event.guild.id)
                        .cloned();
                    update(&cache_and_http, &mut event);

                    event_handler.guild_update(context, before, event.guild);
                } else {
                    update(&cache_and_http, &mut event);

                    event_handler.guild_update(context, event.guild);
                }}
            });
        },
        // Already handled by the framework check macro
        DispatchEvent::Model(Event::MessageCreate(_)) => {},
        DispatchEvent::Model(Event::MessageDeleteBulk(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete_bulk(context, event.channel_id, event.ids);
            });
        },
        DispatchEvent::Model(Event::MessageDelete(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete(context, event.channel_id, event.message_id);
            });
        },
        DispatchEvent::Model(Event::MessageUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    let _after = cache_and_http.cache.as_ref().read().message(event.channel_id, event.id);
                    event_handler.message_update(context, _before, _after, event);
                } else {
                    event_handler.message_update(context, event);
                }}
            });
        },
        DispatchEvent::Model(Event::PresencesReplace(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_replace(context, event.presences);
            });
        },
        DispatchEvent::Model(Event::PresenceUpdate(mut event)) => {
            update(&cache_and_http, &mut event);

            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_update(context, event);
            });
        },
        DispatchEvent::Model(Event::ReactionAdd(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_add(context, event.reaction);
            });
        },
        DispatchEvent::Model(Event::ReactionRemove(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove(context, event.reaction);
            });
        },
        DispatchEvent::Model(Event::ReactionRemoveAll(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove_all(context, event.channel_id, event.message_id);
            });
        },
        DispatchEvent::Model(Event::Ready(mut event)) => {
            update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(&event_handler);

            threadpool.execute(move || {
                event_handler.ready(context, event.ready);
            });
        },
        DispatchEvent::Model(Event::Resumed(event)) => {
            event_handler.resume(context, event);
        },
        DispatchEvent::Model(Event::TypingStart(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.typing_start(context, event);
            });
        },
        DispatchEvent::Model(Event::Unknown(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.unknown(context, event.kind, event.value);
            });
        },
        DispatchEvent::Model(Event::UserUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.user_update(context, _before.unwrap(), event.current_user);
                } else {
                    event_handler.user_update(context, event.current_user);
                }}
            });
        },
        DispatchEvent::Model(Event::VoiceServerUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.voice_server_update(context, event);
            });
        },
        DispatchEvent::Model(Event::VoiceStateUpdate(mut event)) => {
            let _before = update(&cache_and_http, &mut event);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.voice_state_update(context, event.guild_id, _before, event.voice_state);
                } else {
                    event_handler.voice_state_update(context, event.guild_id, event.voice_state);
                }}
            });
        },
        DispatchEvent::Model(Event::WebhookUpdate(event)) => {
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.webhook_update(context, event.guild_id, event.channel_id);
            });
        },
        DispatchEvent::Model(Event::__Nonexhaustive) => unreachable!(),
        DispatchEvent::__Nonexhaustive => unreachable!(),
    }
}
