use model::event::Event;
use model::channel::{Channel, Message};
use std::sync::Arc;
use parking_lot::Mutex;
use super::bridge::gateway::ShardClientMessage;
use super::event_handler::EventHandler;
use super::Context;
use std::sync::mpsc::Sender;
use threadpool::ThreadPool;
use typemap::ShareMap;

#[cfg(feature = "cache")]
use chrono::{Timelike, Utc};
#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "cache")]
use model::id::GuildId;
#[cfg(feature = "cache")]
use std::{thread, time};

#[cfg(feature = "cache")]
use super::CACHE;

macro_rules! update {
    ($event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().update(&mut $event)
            }
        }
    };
}

#[cfg(feature = "cache")]
macro_rules! now {
    () => (Utc::now().time().second() * 1000)
}

fn context(
    data: &Arc<Mutex<ShareMap>>,
    runner_tx: &Sender<ShardClientMessage>,
    shard_id: u64,
) -> Context {
    Context::new(Arc::clone(data), runner_tx.clone(), shard_id)
}

#[cfg(feature = "framework")]
#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
pub fn dispatch<H: EventHandler + Send + Sync + 'static>(
    event: Event,
    framework: &Arc<Mutex<Option<Box<Framework + Send>>>>,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<ShardClientMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    match event {
        Event::MessageCreate(event) => {
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
pub fn dispatch<H: EventHandler + Send + Sync + 'static>(
    event: Event,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<ShardClientMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    match event {
        Event::MessageCreate(event) => {
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
    event: Event,
    data: &Arc<Mutex<ShareMap>>,
    event_handler: &Arc<H>,
    runner_tx: &Sender<ShardClientMessage>,
    threadpool: &ThreadPool,
    shard_id: u64,
) {
    #[cfg(feature = "cache")]
    let mut last_guild_create_time = now!();

    #[cfg(feature = "cache")]
    let wait_for_guilds = move || -> ::Result<()> {
        let unavailable_guilds = CACHE.read().unavailable_guilds.len();

        while unavailable_guilds != 0 && (now!() < last_guild_create_time + 2000) {
            thread::sleep(time::Duration::from_millis(500));
        }

        Ok(())
    };

    match event {
        Event::ChannelCreate(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);

            // This different channel_create dispatching is only due to the fact that
            // each time the bot receives a dm, this event is also fired.
            // So in short, only exists to reduce unnecessary clutter.
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
        Event::ChannelDelete(mut event) => {
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
        Event::ChannelPinsUpdate(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.channel_pins_update(context, event);
            });
        },
        Event::ChannelRecipientAdd(mut event) => {
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
        Event::ChannelRecipientRemove(mut event) => {
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
        Event::ChannelUpdate(mut event) => {
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
        Event::GuildBanAdd(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_addition(context, event.guild_id, event.user);
            });
        },
        Event::GuildBanRemove(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_ban_removal(context, event.guild_id, event.user);
            });
        },
        Event::GuildCreate(mut event) => {
            #[cfg(feature = "cache")]
            let _is_new = {
                let cache = CACHE.read();

                !cache.unavailable_guilds.contains(&event.guild.id)
            };

            update!(event);

            #[cfg(feature = "cache")]
            {
                last_guild_create_time = now!();

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
        Event::GuildDelete(mut event) => {
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
        Event::GuildEmojisUpdate(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_emojis_update(context, event.guild_id, event.emojis);
            });
        },
        Event::GuildIntegrationsUpdate(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_integrations_update(context, event.guild_id);
            });
        },
        Event::GuildMemberAdd(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_member_addition(context, event.guild_id, event.member);
            });
        },
        Event::GuildMemberRemove(mut event) => {
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
        Event::GuildMemberUpdate(mut event) => {
            let _before = update!(event);
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                feature_cache! {{
                    // This is safe to unwrap, as the update would have created
                    // the member if it did not exist. So, there is be _no_ way
                    // that this could fail under any circumstance.
                    let after = CACHE.read()
                        .member(event.guild_id, event.user.id)
                        .unwrap()
                        .clone();

                    event_handler.guild_member_update(context, _before, after);
                } else {
                    event_handler.guild_member_update(context, event);
                }}
            });
        },
        Event::GuildMembersChunk(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_members_chunk(context, event.guild_id, event.members);
            });
        },
        Event::GuildRoleCreate(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_role_create(context, event.guild_id, event.role);
            });
        },
        Event::GuildRoleDelete(mut event) => {
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
        Event::GuildRoleUpdate(mut event) => {
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
        Event::GuildUnavailable(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.guild_unavailable(context, event.guild_id);
            });
        },
        Event::GuildUpdate(mut event) => {
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
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete_bulk(context, event.channel_id, event.ids);
            });
        },
        Event::MessageDelete(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_delete(context, event.channel_id, event.message_id);
            });
        },
        Event::MessageUpdate(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.message_update(context, event);
            });
        },
        Event::PresencesReplace(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_replace(context, event.presences);
            });
        },
        Event::PresenceUpdate(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.presence_update(context, event);
            });
        },
        Event::ReactionAdd(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_add(context, event.reaction);
            });
        },
        Event::ReactionRemove(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove(context, event.reaction);
            });
        },
        Event::ReactionRemoveAll(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.reaction_remove_all(context, event.channel_id, event.message_id);
            });
        },
        Event::Ready(mut event) => {
            update!(event);

            let event_handler = Arc::clone(event_handler);

            feature_cache! {{
                last_guild_create_time = now!();

                let _ = wait_for_guilds()
                    .map(move |_| {
                        let context = context(data, runner_tx, shard_id);
                        let event_handler = Arc::clone(&event_handler);

                        threadpool.execute(move || {
                            event_handler.ready(context, event.ready);
                        });
                    });
            } else {
                let context = context(data, runner_tx, shard_id);
                let event_handler = Arc::clone(event_handler);

                threadpool.execute(move || {
                    event_handler.ready(context, event.ready);
                });
            }}
        },
        Event::Resumed(mut event) => {
            let context = context(data, runner_tx, shard_id);

            event_handler.resume(context, event);
        },
        Event::TypingStart(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.typing_start(context, event);
            });
        },
        Event::Unknown(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.unknown(context, event.kind, event.value);
            });
        },
        Event::UserUpdate(mut event) => {
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
        Event::VoiceServerUpdate(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.voice_server_update(context, event);
            });
        },
        Event::VoiceStateUpdate(mut event) => {
            update!(event);

            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.voice_state_update(context, event.guild_id, event.voice_state);
            });
        },
        Event::WebhookUpdate(mut event) => {
            let context = context(data, runner_tx, shard_id);
            let event_handler = Arc::clone(event_handler);

            threadpool.execute(move || {
                event_handler.webhook_update(context, event.guild_id, event.channel_id);
            });
        },
    }
}
