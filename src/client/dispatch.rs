use std::sync::Arc;
use parking_lot::Mutex;
use super::event_handler::EventHandler;
use super::Context;
use typemap::ShareMap;
use gateway::Shard;
use model::event::Event;
use model::{Channel, Message};

#[cfg(feature = "cache")]
use chrono::{Timelike, Utc};
#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "cache")]
use model::GuildId;
#[cfg(feature = "cache")]
use std::{thread, time};
#[cfg(feature = "framework")]
use std::sync;

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

#[cfg(feature = "cache")]
macro_rules! now {
    () => (Utc::now().time().second() * 1000)
}

fn context(conn: Arc<Mutex<Shard>>, data: Arc<Mutex<ShareMap>>) -> Context {
    Context::new(conn, data)
}

#[cfg(feature = "framework")]
pub fn dispatch<H: EventHandler + 'static>(event: Event,
                                           conn: Arc<Mutex<Shard>>,
                                           framework: Arc<sync::Mutex<Option<Box<Framework + Send>>>>,
                                           data: Arc<Mutex<ShareMap>>,
                                           event_handler: Arc<H>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data);
            dispatch_message(
                context.clone(),
                event.message.clone(),
                event_handler,
            );

            if let Some(ref mut framework) = *framework.lock().unwrap() {
                framework.dispatch(context, event.message);
            }
        },
        other => handle_event(other, conn, data, event_handler),
    }
}

#[cfg(not(feature = "framework"))]
pub fn dispatch<H: EventHandler + 'static>(event: Event,
                                           conn: Arc<Mutex<Shard>>,
                                           data: Arc<Mutex<ShareMap>>,
                                           event_handler: Arc<H>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data);
            dispatch_message(context, event.message, event_handler);
        },
        other => handle_event(other, conn, data, event_handler),
    }
}

#[allow(unused_mut)]
fn dispatch_message<H>(
    context: Context,
    mut message: Message,
    event_handler: Arc<H>
) where H: EventHandler + 'static {
    #[cfg(feature = "model")]
    {
        message.transform_content();
    }

    event_handler.on_message(context, message);
}

#[allow(cyclomatic_complexity, unused_assignments, unused_mut)]
fn handle_event<H: EventHandler + 'static>(event: Event,
                                           conn: Arc<Mutex<Shard>>,
                                           data: Arc<Mutex<ShareMap>>,
                                           event_handler: Arc<H>) {
    #[cfg(feature = "cache")]
    let mut last_guild_create_time = now!();

    #[cfg(feature = "cache")]
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

            let context = context(conn, data);

            // This different channel_create dispatching is only due to the fact that
            // each time the bot receives a dm, this event is also fired.
            // So in short, only exists to reduce unnecessary clutter.
            match event.channel {
                Channel::Private(channel) => {
                    event_handler.on_private_channel_create(context, channel);
                },
                Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    event_handler.on_channel_create(context, channel);
                },
                Channel::Category(channel) => {
                    event_handler.on_category_create(context, channel);
                },
            }
        },
        Event::ChannelDelete(mut event) => {
            update!(event);

            let context = context(conn, data);

            match event.channel {
                Channel::Private(_) | Channel::Group(_) => {},
                Channel::Guild(channel) => {
                    event_handler.on_channel_delete(context, channel);
                },
                Channel::Category(channel) => {
                    event_handler.on_category_delete(context, channel);
                },
            }
        },
        Event::ChannelPinsUpdate(mut event) => {
            let context = context(conn, data);

            event_handler.on_channel_pins_update(context, event);
        },
        Event::ChannelRecipientAdd(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_channel_recipient_addition(context, event.channel_id, event.user);
        },
        Event::ChannelRecipientRemove(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_channel_recipient_removal(context, event.channel_id, event.user);
        },
        Event::ChannelUpdate(mut event) => {
            update!(event);

            let context = context(conn, data);

            feature_cache! {{
                let before = CACHE.read().unwrap().channel(event.channel.id());
                event_handler.on_channel_update(context, before, event.channel);
            } else {
                event_handler.on_channel_update(context, event.channel);
            }}
        },
        Event::GuildBanAdd(mut event) => {
            let context = context(conn, data);

                event_handler.on_guild_ban_addition(context, event.guild_id, event.user);
        },
        Event::GuildBanRemove(mut event) => {
            let context = context(conn, data);

            event_handler.on_guild_ban_removal(context, event.guild_id, event.user);
        },
        Event::GuildCreate(mut event) => {
            #[cfg(feature = "cache")]
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
                    let context = context(conn.clone(), data.clone());

                    let guild_amount = cache
                        .guilds
                        .iter()
                        .map(|(&id, _)| id)
                        .collect::<Vec<GuildId>>();

                    event_handler.on_cached(context, guild_amount);
                }
            }

            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_guild_create(context, event.guild, _is_new);
            } else {
                event_handler.on_guild_create(context, event.guild);
            }}
        },
        Event::GuildDelete(mut event) => {
            let _full = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_guild_delete(context, event.guild, _full);
            } else {
                event_handler.on_guild_delete(context, event.guild);
            }}
        },
        Event::GuildEmojisUpdate(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_guild_emojis_update(context, event.guild_id, event.emojis);
        },
        Event::GuildIntegrationsUpdate(mut event) => {
            let context = context(conn, data);

            event_handler.on_guild_integrations_update(context, event.guild_id);
        },
        Event::GuildMemberAdd(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_guild_member_addition(context, event.guild_id, event.member);
        },
        Event::GuildMemberRemove(mut event) => {
            let _member = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_guild_member_removal(context, event.guild_id, event.user, _member);
            } else {
                event_handler.on_guild_member_removal(context, event.guild_id, event.user);
            }}
        },
        Event::GuildMemberUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                // This is safe to unwrap, as the update would have created
                // the member if it did not exist. So, there is be _no_ way
                // that this could fail under any circumstance.
                let after = CACHE.read()
                    .unwrap()
                    .member(event.guild_id, event.user.id)
                    .unwrap()
                    .clone();

                event_handler.on_guild_member_update(context, _before, after);
            } else {
                event_handler.on_guild_member_update(context, event);
            }}
        },
        Event::GuildMembersChunk(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_guild_members_chunk(context, event.guild_id, event.members);
        },
        Event::GuildRoleCreate(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_guild_role_create(context, event.guild_id, event.role);
        },
        Event::GuildRoleDelete(mut event) => {
            let _role = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_guild_role_delete(context, event.guild_id, event.role_id, _role);
            } else {
                event_handler.on_guild_role_delete(context, event.guild_id, event.role_id);
            }}
        },
        Event::GuildRoleUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_guild_role_update(context, event.guild_id, _before, event.role);
            } else {
                event_handler.on_guild_role_update(context, event.guild_id, event.role);
            }}
        },
        Event::GuildUnavailable(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_guild_unavailable(context, event.guild_id);
        },
        Event::GuildUpdate(mut event) => {
            update!(event);

            let context = context(conn, data);

            feature_cache! {{
                let before = CACHE.read()
                    .unwrap()
                    .guilds
                    .get(&event.guild.id)
                    .cloned();

                event_handler.on_guild_update(context, before, event.guild);
            } else {
                event_handler.on_guild_update(context, event.guild);
            }}
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(mut event) => {
            let context = context(conn, data);

            event_handler.on_message_delete_bulk(context, event.channel_id, event.ids);
        },
        Event::MessageDelete(mut event) => {
            let context = context(conn, data);

            event_handler.on_message_delete(context, event.channel_id, event.message_id);
        },
        Event::MessageUpdate(mut event) => {
            let context = context(conn, data);

            event_handler.on_message_update(context, event);
        },
        Event::PresencesReplace(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_presence_replace(context, event.presences);
        },
        Event::PresenceUpdate(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_presence_update(context, event);
        },
        Event::ReactionAdd(mut event) => {
            let context = context(conn, data);

            event_handler.on_reaction_add(context, event.reaction);
        },
        Event::ReactionRemove(mut event) => {
            let context = context(conn, data);

            event_handler.on_reaction_remove(context, event.reaction);
        },
        Event::ReactionRemoveAll(mut event) => {
            let context = context(conn, data);

            event_handler.on_reaction_remove_all(context, event.channel_id, event.message_id);
        },
        Event::Ready(mut event) => {
            update!(event);

            feature_cache!{
                {
                    last_guild_create_time = now!();

                    let _ = wait_for_guilds()
                        .map(move |_| {
                            let context = context(conn, data);

                            event_handler.on_ready(context, event.ready);
                        });
                } else {
                    let context = context(conn, data);

                    event_handler.on_ready(context, event.ready);
                }
            }
        },
        Event::Resumed(mut event) => {
            let context = context(conn, data);

            event_handler.on_resume(context, event);
        },
        Event::TypingStart(mut event) => {
            let context = context(conn, data);

            event_handler.on_typing_start(context, event);
        },
        Event::Unknown(mut event) => {
            let context = context(conn, data);

            event_handler.on_unknown(context, event.kind, event.value);
        },
        Event::UserUpdate(mut event) => {
            let _before = update!(event);
            let context = context(conn, data);

            feature_cache! {{
                event_handler.on_user_update(context, _before.unwrap(), event.current_user);
            } else {
                event_handler.on_user_update(context, event.current_user);
            }}
        },
        Event::VoiceServerUpdate(mut event) => {
            let context = context(conn, data);

            event_handler.on_voice_server_update(context, event);
        },
        Event::VoiceStateUpdate(mut event) => {
            update!(event);

            let context = context(conn, data);

            event_handler.on_voice_state_update(context, event.guild_id, event.voice_state);
        },
        Event::WebhookUpdate(mut event) => {
            let context = context(conn, data);

            event_handler.on_webhook_update(context, event.guild_id, event.channel_id);
        },
    }
}
