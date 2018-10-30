use gateway::InterMessage;
use model::{
    channel::{Channel, Message},
    event::Event,
    guild::Member,
};
use std::sync::Arc;
use parking_lot::Mutex;
use super::{
    bridge::gateway::event::ClientEvent,
    event_handler::EventHandler,
    Context
};
use std::sync::mpsc::Sender;
use threadpool::ThreadPool;
use typemap::ShareMap;

#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "cache")]
use model::id::GuildId;
#[cfg(feature = "cache")]
use std::time::Duration;

#[cfg(feature = "cache")]
use super::CACHE;

#[cfg(feature = "cache")]
lazy_static! {
    pub static ref CACHE_TRY_WRITE_DURATION: Option<Duration> =
        CACHE.read().get_try_write_duration();
}

macro_rules! update {
    ($event:expr) => {
        {
            #[cfg(feature = "cache")]
            {
                match *CACHE_TRY_WRITE_DURATION {
                    Some(duration) => {
                        if let Some(mut lock) = CACHE.try_write_for(duration) {
                            lock.update(&mut $event)
                        } else {
                            warn!(
                                "[dispatch] Possible deadlock: couldn't unlock cache to update with event: {:?}",
                                $event,
                            );
                            None
                        }},
                    None => {
                        CACHE.write().update(&mut $event)
                    },
                }
            }
        }
    };
}

fn context(
    data: &Arc<Mutex<ShareMap>>,
    runner_tx: &Sender<InterMessage>,
    shard_id: u64,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id)
}

pub(crate) enum DispatchEvent {
    Client(ClientEvent),
    Model(Event),
}

#[cfg(feature = "framework")]
#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
pub(crate) fn dispatch<H: EventHandler + Send + Sync + 'static>(
    event: DispatchEvent,
    framework: &Arc<Mutex<Option<Box<Framework + Send>>>>,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    match event {
        DispatchEvent::Model(Event::MessageCreate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            dispatch_message(
                context.clone(),
                event.message.clone(),
                event_handler,
                threadpool,
            );

            if let Some(ref mut framework) = *framework.lock() {
                framework.dispatch(context, event.message, threadpool);
            }
        },
        other => handle_event(
            other,
            data,
            event_handler,
            runner_tx,
            threadpool,
            shard_id,
        ),
    }
}

#[cfg(not(feature = "framework"))]
#[allow(unused_mut)]
pub(crate) fn dispatch<H: EventHandler + Send + Sync + 'static>(
    event: DispatchEvent,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    match event {
        DispatchEvent::Model(Event::MessageCreate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            dispatch_message(context, event.message, event_handler, threadpool);
        },
        other => handle_event(
            other,
            data,
            event_handler,
            runner_tx,
            threadpool,
            shard_id,
        ),
    }
}

#[allow(unused_mut)]
fn dispatch_message<H>(
    context: Context,
    mut message: Message,
    event_handler: &Arc<H>,
    threadpool: &ThreadPool,
) where H: EventHandler + Send + Sync + 'static {
    #[cfg(feature = "model")]
    {
        message.transform_content();
    }

    let event_handler = Arc::clone(event_handler);

    threadpool.execute(move || {
        event_handler.message(context, message);
    });
}

#[allow(cyclomatic_complexity, unused_assignments, unused_mut)]
fn handle_event<H: EventHandler + Send + Sync + 'static>(
    event: DispatchEvent,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<InterMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    match event {
        DispatchEvent::Client(ClientEvent::ShardStageUpdate(event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.shard_stage_update(context, event);
            });
        }
        DispatchEvent::Model(Event::ChannelCreate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);

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
            }
        },
        DispatchEvent::Model(Event::ChannelDelete(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);

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
            }
        },
        DispatchEvent::Model(Event::ChannelPinsUpdate(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.channel_pins_update(context, event);
            });
        },
        DispatchEvent::Model(Event::ChannelRecipientAdd(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);

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
            update!(event);

            let context = context(data, runner_tx, shard_id);
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
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    let before = CACHE.read().channel(event.channel.id());

                    event_handler.channel_update(context, before, event.channel);
                } else {
                    event_handler.channel_update(context, event.channel);
                }}
            });
        },
        DispatchEvent::Model(Event::GuildBanAdd(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_addition(context, event.guild_id, event.user);
            });
        },
        DispatchEvent::Model(Event::GuildBanRemove(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_removal(context, event.guild_id, event.user);
            });
        },
        DispatchEvent::Model(Event::GuildCreate(mut event)) => {
            #[cfg(feature = "cache")]
            let _is_new = {
                let cache = CACHE.read();

                !cache.unavailable_guilds.contains(&event.guild.id)
            };

            update!(event);

            #[cfg(feature = "cache")]
            {
                let cache = CACHE.read();

                if cache.unavailable_guilds.is_empty() {
                    let context = context(data, runner_tx, shard_id);

                    let guild_amount = cache
                        .guilds
                        .iter()
                        .map(|(&id, _)| id)
                        .collect::<Vec<GuildId>>();
                    let event_handler = Arc::clone(event_handler);

                    threadpool.execute(move || {
                        event_handler.cached(context, guild_amount);
                    });
                }
            }

            let context = context(data, runner_tx, shard_id);
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
            let _full = update!(event);
            let context = context(data, runner_tx, shard_id);
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
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_emojis_update(context, event.guild_id, event.emojis);
            });
        },
        DispatchEvent::Model(Event::GuildIntegrationsUpdate(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_integrations_update(context, event.guild_id);
            });
        },
        DispatchEvent::Model(Event::GuildMemberAdd(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_member_addition(context, event.guild_id, event.member);
            });
        },
        DispatchEvent::Model(Event::GuildMemberRemove(mut event)) => {
            let _member = update!(event);
            let context = context(data, runner_tx, shard_id);
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
            let _before = update!(event);

            let _after: Option<Member> = feature_cache! {{
                CACHE.read().member(event.guild_id, event.user.id)
            } else {
                None
            }};

            let context = context(data, runner_tx, shard_id);
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
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_members_chunk(context, event.guild_id, event.members);
            });
        },
        DispatchEvent::Model(Event::GuildRoleCreate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_role_create(context, event.guild_id, event.role);
            });
        },
        DispatchEvent::Model(Event::GuildRoleDelete(mut event)) => {
            let _role = update!(event);
            let context = context(data, runner_tx, shard_id);
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
            let _before = update!(event);
            let context = context(data, runner_tx, shard_id);
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
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_unavailable(context, event.guild_id);
            });
        },
        DispatchEvent::Model(Event::GuildUpdate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    let before = CACHE.read()
                        .guilds
                        .get(&event.guild.id)
                        .cloned();

                    event_handler.guild_update(context, before, event.guild);
                } else {
                    event_handler.guild_update(context, event.guild);
                }}
            });
        },
        // Already handled by the framework check macro
        DispatchEvent::Model(Event::MessageCreate(_)) => {},
        DispatchEvent::Model(Event::MessageDeleteBulk(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete_bulk(context, event.channel_id, event.ids);
            });
        },
        DispatchEvent::Model(Event::MessageDelete(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete(context, event.channel_id, event.message_id);
            });
        },
        DispatchEvent::Model(Event::MessageUpdate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_update(context, event);
            });
        },
        DispatchEvent::Model(Event::PresencesReplace(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_replace(context, event.presences);
            });
        },
        DispatchEvent::Model(Event::PresenceUpdate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_update(context, event);
            });
        },
        DispatchEvent::Model(Event::ReactionAdd(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_add(context, event.reaction);
            });
        },
        DispatchEvent::Model(Event::ReactionRemove(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove(context, event.reaction);
            });
        },
        DispatchEvent::Model(Event::ReactionRemoveAll(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove_all(context, event.channel_id, event.message_id);
            });
        },
        DispatchEvent::Model(Event::Ready(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(&event_handler);

            threadpool.execute(move || {
                event_handler.ready(context, event.ready);
            });
        },
        DispatchEvent::Model(Event::Resumed(mut event)) => {
            let context = context(data, runner_tx, shard_id);

            event_handler.resume(context, event);
        },
        DispatchEvent::Model(Event::TypingStart(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.typing_start(context, event);
            });
        },
        DispatchEvent::Model(Event::Unknown(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.unknown(context, event.kind, event.value);
            });
        },
        DispatchEvent::Model(Event::UserUpdate(mut event)) => {
            let _before = update!(event);
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    event_handler.user_update(context, _before.unwrap(), event.current_user);
                } else {
                    event_handler.user_update(context, event.current_user);
                }}
            });
        },
        DispatchEvent::Model(Event::VoiceServerUpdate(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.voice_server_update(context, event);
            });
        },
        DispatchEvent::Model(Event::VoiceStateUpdate(mut event)) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.voice_state_update(context, event.guild_id, event.voice_state);
            });
        },
        DispatchEvent::Model(Event::WebhookUpdate(mut event)) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.webhook_update(context, event.guild_id, event.channel_id);
            });
        },
    }
}
