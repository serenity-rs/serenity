use std::sync::{self, Arc};
use parking_lot::Mutex;
use std::thread;
use std::time;
use super::event_handler::EventHandler;
use super::Context;
use typemap::ShareMap;
use gateway::Shard;
use model::event::Event;
use model::{Channel, GuildId, Message};
use chrono::{Timelike, Utc};
use tokio_core::reactor::Handle;

#[cfg(feature = "framework")]
use framework::Framework;

#[cfg(feature = "cache")]
use super::CACHE;

macro_rules! update {
    ($event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().update(&mut $event)
            }
        }
    };
}

macro_rules! now {
    () => (Utc::now().time().second() * 1000)
}

fn context(conn: &Arc<Mutex<Shard>>, data: &Arc<Mutex<ShareMap>>, handle: &Handle) -> Context {
    Context::new(conn.clone(), data.clone(), handle.clone())
}

#[cfg(feature = "standard_framework")]
macro_rules! helper {
    ($enabled:block else $disabled:block) => { $enabled }
}

#[cfg(not(feature = "standard_framework"))]
macro_rules! helper {
    ($enabled:block else $disabled:block) => { $disabled }
}

#[cfg(feature = "framework")]
pub fn dispatch<H: EventHandler + 'static>(event: Event,
                                           conn: &Arc<Mutex<Shard>>,
                                           framework: &Arc<sync::Mutex<Option<Box<Framework>>>>,
                                           data: &Arc<Mutex<ShareMap>>,
                                           event_handler: &Arc<H>,
                                           tokio_handle: &Handle) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data, tokio_handle);
            dispatch_message(
                context.clone(),
                event.message.clone(),
                event_handler,
                tokio_handle,
            );

            if let Some(ref mut framework) = *framework.lock().unwrap() {
                helper! {{
                if framework.initialized() {
                framework.dispatch(context, event.message, tokio_handle);
                }
                } else {
                framework.dispatch(context, event.message, tokio_handle);
                }}
            }
        },
        other => handle_event(other, conn, data, event_handler, tokio_handle),
    }
}

#[cfg(not(feature = "framework"))]
pub fn dispatch<H: EventHandler + 'static>(event: Event,
                                           conn: &Arc<Mutex<Shard>>,
                                           data: &Arc<Mutex<ShareMap>>,
                                           event_handler: &Arc<H>,
                                           tokio_handle: &Handle) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data, tokio_handle);
            dispatch_message(context, event.message, event_handler, tokio_handle);
        },
        other => handle_event(other, conn, data, event_handler, tokio_handle),
    }
}

#[allow(unused_mut)]
fn dispatch_message<H: EventHandler + 'static>(context: Context,
                                               mut message: Message,
                                               event_handler: &Arc<H>,
                                               tokio_handle: &Handle) {
    let h = event_handler.clone();
    tokio_handle.spawn_fn(move || {
        #[cfg(feature = "model")]
        {
            message.transform_content();
        }

        h.on_message(context, message);

        Ok(())
    });
}

#[allow(cyclomatic_complexity, unused_assignments, unused_mut)]
fn handle_event<H: EventHandler + 'static>(event: Event,
                                           conn: &Arc<Mutex<Shard>>,
                                           data: &Arc<Mutex<ShareMap>>,
                                           event_handler: &Arc<H>,
                                           tokio_handle: &Handle) {
    #[cfg(feature="cache")]
    let mut last_guild_create_time = now!();

    #[cfg(feature="cache")]
    let wait_for_guilds = move || -> ::Result<()> {
        let unavailable_guilds = CACHE.read().unwrap().unavailable_guilds.len();

        while unavailable_guilds != 0 && (now!() < last_guild_create_time + 2000) {
            thread::sleep(time::Duration::from_millis(500));
        }

        Ok(())
    };

    match event {
        Event::ChannelCreate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            // This different channel_create dispacthing is only due to the fact that
            // each time the bot receives a dm, this event is also fired.
            // So in short, only exists to reduce unnecessary clutter.
            let h = event_handler.clone();
            match event.channel {
                Channel::Private(channel) => {
                    tokio_handle.spawn_fn(move || {
                        h.on_private_channel_create(context, channel);
                        Ok(())
                    });
                },
                Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    tokio_handle.spawn_fn(move || {
                        h.on_channel_create(context, channel);
                        Ok(())
                    });
                },
                Channel::Category(channel) => {
                    tokio_handle.spawn_fn(move || {
                        h.on_category_create(context, channel);
                        Ok(())
                    });
                },
            }
        },
        Event::ChannelDelete(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            match event.channel {
                Channel::Private(_) |
                Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    let h = event_handler.clone();
                    tokio_handle.spawn_fn(move || {
                        h.on_channel_delete(context, channel);
                        Ok(())
                    });
                },
                Channel::Category(channel) => {
                    let h = event_handler.clone();
                    tokio_handle.spawn_fn(move || {
                        h.on_category_delete(context, channel);
                        Ok(())
                    });
                },
            }
        },
        Event::ChannelPinsUpdate(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_channel_pins_update(context, event);
                Ok(())
            });
        },
        Event::ChannelRecipientAdd(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_channel_recipient_addition(
                    context,
                    event.channel_id,
                    event.user,
                );
                Ok(())
            });
        },
        Event::ChannelRecipientRemove(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_channel_recipient_removal(
                    context,
                    event.channel_id,
                    event.user,
                );
                Ok(())
            });
        },
        Event::ChannelUpdate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                let before = CACHE.read().unwrap().channel(event.channel.id());
                tokio_handle.spawn_fn(move || {
                    h.on_channel_update(context, before, event.channel);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_channel_update(context, event.channel);
                    Ok(())
                });
            }}
        },
        Event::GuildBanAdd(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_ban_addition(context, event.guild_id, event.user);
                Ok(())
            });
        },
        Event::GuildBanRemove(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_ban_removal(context, event.guild_id, event.user);
                Ok(())
            });
        },
        Event::GuildCreate(mut event) => {
            #[cfg(feature="cache")]
            let _is_new = {
                let cache = CACHE.read().unwrap();

                !cache.unavailable_guilds.contains(&event.guild.id)
            };

            update!(event);

            #[cfg(feature = "cache")]
            {
                last_guild_create_time = now!();

                let cache = CACHE.read().unwrap();

                if cache.unavailable_guilds.is_empty() {
                    let h = event_handler.clone();

                    let context = context(conn, data, tokio_handle);

                    let guild_amount = cache
                        .guilds
                        .iter()
                        .map(|(&id, _)| id)
                        .collect::<Vec<GuildId>>();

                    tokio_handle.spawn_fn(move || {
                        h.on_cached(context, guild_amount);
                        Ok(())
                    });
                }
            }

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_guild_create(context, event.guild, _is_new);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_guild_create(context, event.guild);
                    Ok(())
                });
            }}
        },
        Event::GuildDelete(mut event) => {
            let _full = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_guild_delete(context, event.guild, _full);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_guild_delete(context, event.guild);
                    Ok(())
                });
            }}
        },
        Event::GuildEmojisUpdate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_emojis_update(
                    context,
                    event.guild_id,
                    event.emojis,
                );
                Ok(())
            });
        },
        Event::GuildIntegrationsUpdate(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_integrations_update(context, event.guild_id);
                Ok(())
            });
        },
        Event::GuildMemberAdd(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_member_addition(
                    context,
                    event.guild_id,
                    event.member,
                );
                Ok(())
            });
        },
        Event::GuildMemberRemove(mut event) => {
            let _member = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_guild_member_removal(context, event.guild_id, event.user, _member);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_guild_member_removal(context, event.guild_id, event.user);
                    Ok(())
                });
            }}
        },
        Event::GuildMemberUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {
                {
                    // This is safe to unwrap, as the update would have created
                    // the member if it did not exist. So, there is be _no_ way
                    // that this could fail under any circumstance.
                    let after = CACHE.read()
                        .unwrap()
                        .member(event.guild_id, event.user.id)
                        .unwrap()
                        .clone();

                    tokio_handle.spawn_fn(move || {
                        h.on_guild_member_update(context, _before, after);
                        Ok(())
                    });
                } else {
                    tokio_handle.spawn_fn(move || {
                        h.on_guild_member_update(context, event);
                        Ok(())
                    });
                }
            }
        },
        Event::GuildMembersChunk(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_members_chunk(
                    context,
                    event.guild_id,
                    event.members,
                );
                Ok(())
            });
        },
        Event::GuildRoleCreate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_role_create(context, event.guild_id, event.role);
                Ok(())
            });
        },
        Event::GuildRoleDelete(mut event) => {
            let _role = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_guild_role_delete(context, event.guild_id, event.role_id, _role);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_guild_role_delete(context, event.guild_id, event.role_id);
                    Ok(())
                });
            }}
        },
        Event::GuildRoleUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_guild_role_update(context, event.guild_id, _before, event.role);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_guild_role_update(context, event.guild_id, event.role);
                    Ok(())
                });
            }}
        },
        Event::GuildUnavailable(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_guild_unavailable(context, event.guild_id);
                Ok(())
            });
        },
        Event::GuildUpdate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {
                {
                    let before = CACHE.read()
                        .unwrap()
                        .guilds
                        .get(&event.guild.id)
                        .cloned();

                    tokio_handle.spawn_fn(move || {
                        h.on_guild_update(context, before, event.guild);
                        Ok(())
                    });
                } else {
                    tokio_handle.spawn_fn(move || {
                        h.on_guild_update(context, event.guild);
                        Ok(())
                    });
                }
            }
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_message_delete_bulk(
                    context,
                    event.channel_id,
                    event.ids,
                );
                Ok(())
            });
        },
        Event::MessageDelete(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_message_delete(
                    context,
                    event.channel_id,
                    event.message_id,
                );
                Ok(())
            });
        },
        Event::MessageUpdate(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_message_update(context, event);
                Ok(())
            });
        },
        Event::PresencesReplace(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_presence_replace(context, event.presences);
                Ok(())
            });
        },
        Event::PresenceUpdate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_presence_update(context, event);
                Ok(())
            });
        },
        Event::ReactionAdd(mut event) => {
            let h = event_handler.clone();
            let context = context(conn, data, tokio_handle);
            tokio_handle.spawn_fn(move || {
                h.on_reaction_add(context, event.reaction);
                Ok(())
            });
        },
        Event::ReactionRemove(mut event) => {
            let h = event_handler.clone();
            let context = context(conn, data, tokio_handle);
            tokio_handle.spawn_fn(move || {
                h.on_reaction_remove(context, event.reaction);
                Ok(())
            });
        },
        Event::ReactionRemoveAll(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_reaction_remove_all(
                    context,
                    event.channel_id,
                    event.message_id,
                );
                Ok(())
            });
        },
        Event::Ready(mut event) => {
            update!(event);

            feature_cache!{
                {
                    last_guild_create_time = now!();

                    let _ = wait_for_guilds()
                    .map(|_| {
                        let context = context(conn, data, tokio_handle);

                        let h = event_handler.clone();
                        tokio_handle.spawn_fn(move || {
                            h.on_ready(context, event.ready);
                            Ok(())
                        });
                    });
                } else {
                    let context = context(conn, data, tokio_handle);

                    let h = event_handler.clone();
                    tokio_handle.spawn_fn(move || {
                        h.on_ready(context, event.ready);
                        Ok(())
                    });
                }
            }
        },
        Event::Resumed(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_resume(context, event);
                Ok(())
            });
        },
        Event::TypingStart(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_typing_start(context, event);
                Ok(())
            });
        },
        Event::Unknown(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_unknown(context, event.kind, event.value);
                Ok(())
            });
        },
        Event::UserUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            feature_cache! {{
                tokio_handle.spawn_fn(move || {
                    h.on_user_update(context, _before.unwrap(), event.current_user);
                    Ok(())
                });
            } else {
                tokio_handle.spawn_fn(move || {
                    h.on_user_update(context, event.current_user);
                    Ok(())
                });
            }}
        },
        Event::VoiceServerUpdate(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_voice_server_update(context, event);
                Ok(())
            });
        },
        Event::VoiceStateUpdate(mut event) => {
            update!(event);

            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_voice_state_update(
                    context,
                    event.guild_id,
                    event.voice_state,
                );
                Ok(())
            });
        },
        Event::WebhookUpdate(mut event) => {
            let context = context(conn, data, tokio_handle);

            let h = event_handler.clone();
            tokio_handle.spawn_fn(move || {
                h.on_webhook_update(
                    context,
                    event.guild_id,
                    event.channel_id,
                );
                Ok(())
            });
        },
    }
}
