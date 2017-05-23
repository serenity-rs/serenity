use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use super::event_store::EventStore;
use super::Context;
use typemap::ShareMap;
use ::gateway::Shard;
use ::model::event::Event;
use ::model::{ChannelId, Message};

#[cfg(feature="framework")]
use ::ext::framework::Framework;

#[cfg(feature="cache")]
use super::CACHE;

macro_rules! handler {
    ($field:ident, $event_store:ident) => {
        $event_store.read()
            .unwrap()
            .$field
            .as_ref()
            .cloned()
    }
}

macro_rules! update {
    ($method:ident, @$event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&mut $event)
            }
        }
    };
    ($method:ident, @$event:expr, $old:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&mut $event, $old)
            }
        }
    };
    ($method:ident, $event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&$event)
            }
        }
    };
    ($method:ident, $event:expr, $old:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&$event, $old)
            }
        }
    };
}

fn context(channel_id: Option<ChannelId>,
           conn: &Arc<Mutex<Shard>>,
           data: &Arc<Mutex<ShareMap>>) -> Context {
    Context::new(channel_id, conn.clone(), data.clone())
}

#[cfg(feature="framework")]
pub fn dispatch(event: Event,
                conn: &Arc<Mutex<Shard>>,
                framework: &Arc<Mutex<Framework>>,
                data: &Arc<Mutex<ShareMap>>,
                event_store: &Arc<RwLock<EventStore>>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(Some(event.message.channel_id),
                                  conn,
                                  data);
            let mut framework = framework.lock().unwrap();

            if framework.initialized {
                dispatch_message(context.clone(),
                                 event.message.clone(),
                                 event_store);

                framework.dispatch(context, event.message);
            } else {
                dispatch_message(context, event.message, event_store);
            }
        },
        other => handle_event(other, conn, data, event_store),
    }
}

#[cfg(not(feature="framework"))]
pub fn dispatch(event: Event,
                conn: &Arc<Mutex<Shard>>,
                data: &Arc<Mutex<ShareMap>>,
                event_store: &Arc<RwLock<EventStore>>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(Some(event.message.channel_id), conn, data);
            dispatch_message(context,
                             event.message,
                             event_store);
        },
        other => handle_event(other, conn, data, event_store),
    }
}

fn dispatch_message(context: Context,
                    mut message: Message,
                    event_store: &Arc<RwLock<EventStore>>) {
    if let Some(handler) = handler!(on_message, event_store) {
        thread::spawn(move || {
            message.transform_content();

            (handler)(context, message);
        });
    }
}

#[allow(cyclomatic_complexity, unused_mut)]
fn handle_event(event: Event,
                conn: &Arc<Mutex<Shard>>,
                data: &Arc<Mutex<ShareMap>>,
                event_store: &Arc<RwLock<EventStore>>) {
    match event {
        Event::ChannelCreate(event) => {
            if let Some(handler) = handler!(on_channel_create, event_store) {
                update!(update_with_channel_create, event);
                let context = context(Some(event.channel.id()), conn, data);

                thread::spawn(move || (handler)(context, event.channel));
            } else {
                update!(update_with_channel_create, event);
            }
        },
        Event::ChannelDelete(event) => {
            if let Some(handler) = handler!(on_channel_delete, event_store) {
                update!(update_with_channel_delete, event);
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.channel));
            } else {
                update!(update_with_channel_delete, event);
            }
        },
        Event::ChannelPinsUpdate(event) => {
            if let Some(handler) = handler!(on_channel_pins_update, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::ChannelRecipientAdd(mut event) => {
            update!(update_with_channel_recipient_add, @event);

            if let Some(handler) = handler!(on_channel_recipient_addition, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.channel_id, event.user));
            }
        },
        Event::ChannelRecipientRemove(event) => {
            update!(update_with_channel_recipient_remove, event);

            if let Some(handler) = handler!(on_channel_recipient_removal, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.channel_id, event.user));
            }
        },
        Event::ChannelUpdate(event) => {
            if let Some(handler) = handler!(on_channel_update, event_store) {
                let context = context(Some(event.channel.id()), conn, data);

                feature_cache! {{
                    let before = CACHE.read().unwrap().channel(event.channel.id());
                    update!(update_with_channel_update, event);

                    thread::spawn(move || (handler)(context, before, event.channel));
                } else {
                    thread::spawn(move || (handler)(context, event.channel));
                }}
            } else {
                update!(update_with_channel_update, event);
            }
        },
        Event::GuildBanAdd(event) => {
            if let Some(handler) = handler!(on_guild_ban_addition, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.user));
            }
        },
        Event::GuildBanRemove(event) => {
            if let Some(handler) = handler!(on_guild_ban_removal, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.user));
            }
        },
        Event::GuildCreate(event) => {
            update!(update_with_guild_create, event);

            if let Some(handler) = handler!(on_guild_create, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild));
            }
        },
        Event::GuildDelete(event) => {
            if let Some(handler) = handler!(on_guild_delete, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let full = update!(update_with_guild_delete, event);

                    thread::spawn(move || (handler)(context, event.guild, full));
                } else {
                    thread::spawn(move || (handler)(context, event.guild));
                }}
            } else {
                #[cfg(feature="cache")]
                {
                    let _ = update!(update_with_guild_delete, event);
                }
            }
        },
        Event::GuildEmojisUpdate(event) => {
            update!(update_with_guild_emojis_update, event);

            if let Some(handler) = handler!(on_guild_emojis_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.emojis));
            }
        },
        Event::GuildIntegrationsUpdate(event) => {
            if let Some(handler) = handler!(on_guild_integrations_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id));
            }
        },
        Event::GuildMemberAdd(mut event) => {
            update!(update_with_guild_member_add, @event);

            if let Some(handler) = handler!(on_guild_member_addition, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.member));
            }
        },
        Event::GuildMemberRemove(event) => {
            if let Some(handler) = handler!(on_guild_member_removal, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let member = update!(update_with_guild_member_remove, event);

                    thread::spawn(move || (handler)(context, event.guild_id, event.user, member));
                } else {
                    thread::spawn(move || (handler)(context, event.guild_id, event.user));
                }}
            } else {
                #[cfg(feature="cache")]
                {
                    let _ = update!(update_with_guild_member_remove, event);
                }
            }
        },
        Event::GuildMemberUpdate(event) => {
            if let Some(handler) = handler!(on_guild_member_update, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let before = update!(update_with_guild_member_update, event);

                    // This is safe to unwrap, as the update would have created
                    // the member if it did not exist. So, there is be _no_ way
                    // that this could fail under any circumstance.
                    let after = CACHE.read()
                        .unwrap()
                        .member(event.guild_id, event.user.id)
                        .unwrap()
                        .clone();

                    thread::spawn(move || (handler)(context, before, after));
                } else {
                    thread::spawn(move || (handler)(context, event));
                }}
            } else {
                update!(update_with_guild_member_update, event);
            }
        },
        Event::GuildMembersChunk(event) => {
            update!(update_with_guild_members_chunk, event);

            if let Some(handler) = handler!(on_guild_members_chunk, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.members));
            }
        },
        Event::GuildRoleCreate(event) => {
            update!(update_with_guild_role_create, event);

            if let Some(handler) = handler!(on_guild_role_create, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.role));
            }
        },
        Event::GuildRoleDelete(event) => {
            if let Some(handler) = handler!(on_guild_role_delete, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let role = update!(update_with_guild_role_delete, event);

                    thread::spawn(move || (handler)(context, event.guild_id, event.role_id, role));
                } else {
                    thread::spawn(move || (handler)(context, event.guild_id, event.role_id));
                }}
            } else {
                #[cfg(feature="cache")]
                {
                    let _ = update!(update_with_guild_role_delete, event);
                }
            }
        },
        Event::GuildRoleUpdate(event) => {
            if let Some(handler) = handler!(on_guild_role_update, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let before = update!(update_with_guild_role_update, event);

                    thread::spawn(move || (handler)(context, event.guild_id, before, event.role));
                } else {
                    thread::spawn(move || (handler)(context, event.guild_id, event.role));
                }}
            } else {
                #[cfg(feature="cache")]
                {
                    let _ = update!(update_with_guild_role_update, event);
                }
            }
        },
        Event::GuildUnavailable(event) => {
            update!(update_with_guild_unavailable, event);

            if let Some(handler) = handler!(on_guild_unavailable, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id));
            }
        },
        Event::GuildUpdate(event) => {
            if let Some(handler) = handler!(on_guild_update, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let before = CACHE.read()
                        .unwrap()
                        .guilds
                        .get(&event.guild.id)
                        .cloned();
                    update!(update_with_guild_update, event);

                    thread::spawn(move || (handler)(context, before, event.guild));
                } else {
                    thread::spawn(move || (handler)(context, event.guild));
                }}
            } else {
                update!(update_with_guild_update, event);
            }
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(event) => {
            if let Some(handler) = handler!(on_message_delete_bulk, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.channel_id, event.ids));
            }
        },
        Event::MessageDelete(event) => {
            if let Some(handler) = handler!(on_message_delete, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.channel_id, event.message_id));
            }
        },
        Event::MessageUpdate(event) => {
            if let Some(handler) = handler!(on_message_update, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::PresencesReplace(event) => {
            update!(update_with_presences_replace, event);

            if let Some(handler) = handler!(on_presence_replace, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.presences));
            }
        },
        Event::PresenceUpdate(mut event) => {
            update!(update_with_presence_update, @event);

            if let Some(handler) = handler!(on_presence_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::ReactionAdd(event) => {
            if let Some(handler) = handler!(on_reaction_add, event_store) {
                let context = context(Some(event.reaction.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.reaction));
            }
        },
        Event::ReactionRemove(event) => {
            if let Some(handler) = handler!(on_reaction_remove, event_store) {
                let context = context(Some(event.reaction.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.reaction));
            }
        },
        Event::ReactionRemoveAll(event) => {
            if let Some(handler) = handler!(on_reaction_remove_all, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event.channel_id, event.message_id));
            }
        },
        Event::Ready(event) => {
            if let Some(handler) = handler!(on_ready, event_store) {
                update!(update_with_ready, event);

                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.ready));
            } else {
                update!(update_with_ready, event);
            }
        },
        Event::Resumed(event) => {
            if let Some(handler) = handler!(on_resume, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::TypingStart(event) => {
            if let Some(handler) = handler!(on_typing_start, event_store) {
                let context = context(Some(event.channel_id), conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::Unknown(event) => {
            if let Some(handler) = handler!(on_unknown, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.kind, event.value));
            }
        },
        Event::UserUpdate(event) => {
            if let Some(handler) = handler!(on_user_update, event_store) {
                let context = context(None, conn, data);

                feature_cache! {{
                    let before = update!(update_with_user_update, event);

                    thread::spawn(move || (handler)(context, before, event.current_user));
                } else {
                    thread::spawn(move || (handler)(context, event.current_user));
                }}
            } else {
                #[cfg(feature="cache")]
                {
                    let _ = update!(update_with_user_update, event);
                }
            }
        },
        Event::VoiceServerUpdate(event) => {
            if let Some(handler) = handler!(on_voice_server_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event));
            }
        },
        Event::VoiceStateUpdate(event) => {
            update!(update_with_voice_state_update, event);

            if let Some(handler) = handler!(on_voice_state_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.voice_state));
            }
        },
        Event::WebhookUpdate(event) => {
            if let Some(handler) = handler!(on_webhook_update, event_store) {
                let context = context(None, conn, data);

                thread::spawn(move || (handler)(context, event.guild_id, event.channel_id));
            }
        },
    }
}
