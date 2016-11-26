use std::sync::{Arc, Mutex};
use std::thread;
use super::event_store::EventStore;
use super::login_type::LoginType;
use super::Context;
use super::gateway::Shard;
use ::model::event::Event;
use ::model::{ChannelId, Message};

#[cfg(feature="framework")]
use ::ext::framework::Framework;

#[cfg(feature = "cache")]
use super::CACHE;

macro_rules! handler {
    ($field:ident, $event_store:ident) => {
        $event_store.lock()
            .unwrap()
            .$field
            .as_ref()
            .cloned()
    }
}

macro_rules! update {
    ($method:ident, $event:expr) => {
        feature_cache_enabled! {{
            CACHE.write().unwrap().$method(&$event)
        }}
    };
    ($method:ident, $event:expr, $old:expr) => {
        feature_cache_enabled! {{
            CACHE.write().unwrap().$method(&$event, $old)
        }}
    };
}

fn context(channel_id: Option<ChannelId>,
           conn: Arc<Mutex<Shard>>,
           login_type: LoginType) -> Context {
    Context::new(channel_id, conn, login_type)
}

#[cfg(feature="framework")]
pub fn dispatch(event: Event,
                conn: Arc<Mutex<Shard>>,
                framework: Arc<Mutex<Framework>>,
                login_type: LoginType,
                event_store: Arc<Mutex<EventStore>>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(Some(event.message.channel_id),
                                  conn,
                                  login_type);
            let mut framework = framework.lock().expect("framework poisoned");

            if framework.initialized {
                dispatch_message(context.clone(),
                                 event.message.clone(),
                                 event_store);

                framework.dispatch(context, event.message);
            } else {
                dispatch_message(context, event.message, event_store);
            }
        },
        other => handle_event(other, conn, login_type, event_store),
    }
}

#[cfg(not(feature="framework"))]
pub fn dispatch(event: Event,
                conn: Arc<Mutex<Shard>>,
                login_type: LoginType,
                event_store: Arc<Mutex<EventStore>>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(Some(event.message.channel_id),
                                  conn,
                                  login_type);
            dispatch_message(context.clone(),
                             event.message.clone(),
                             event_store);
        },
        other => handle_event(other, conn, login_type, event_store),
    }
}

fn dispatch_message(context: Context,
                    message: Message,
                    event_store: Arc<Mutex<EventStore>>) {
    if let Some(ref handler) = handler!(on_message, event_store) {
        let handler = handler.clone();

        thread::spawn(move || {
            (handler)(context, message);
        });
    }
}

#[allow(cyclomatic_complexity)]
fn handle_event(event: Event,
                conn: Arc<Mutex<Shard>>,
                login_type: LoginType,
                event_store: Arc<Mutex<EventStore>>) {
    match event {
        Event::CallCreate(event) => {
            if let Some(ref handler) = handler!(on_call_create, event_store) {
                update!(update_with_call_create, event);

                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.call);
                });
            } else {
                update!(update_with_call_create, event);
            }
        },
        Event::CallDelete(event) => {
            if let Some(ref handler) = handler!(on_call_delete, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let call = update!(update_with_call_delete, event);

                    thread::spawn(move || {
                        (handler)(context, event.channel_id, call);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.channel_id);
                    });
                }}
            } else {
                update!(update_with_call_delete, event);
            }
        },
        Event::CallUpdate(event) => {
            if let Some(ref handler) = handler!(on_call_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_call_update, event, true);
                    let after = CACHE
                        .read()
                        .unwrap()
                        .calls
                        .get(&event.channel_id)
                        .cloned();

                    thread::spawn(move || {
                        (handler)(context, before, after);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    update!(update_with_call_update, event, false);
                }}
            }
        },
        Event::ChannelCreate(event) => {
            if let Some(ref handler) = handler!(on_channel_create, event_store) {
                update!(update_with_channel_create, event);
                let context = context(Some(event.channel.id()),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel);
                });
            } else {
                update!(update_with_channel_create, event);
            }
        },
        Event::ChannelDelete(event) => {
            if let Some(ref handler) = handler!(on_channel_delete, event_store) {
                update!(update_with_channel_delete, event);
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel);
                });
            } else {
                update!(update_with_channel_delete, event);
            }
        },
        Event::ChannelPinsAck(event) => {
            if let Some(ref handler) = handler!(on_channel_pins_ack, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::ChannelPinsUpdate(event) => {
            if let Some(ref handler) = handler!(on_channel_pins_update, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::ChannelRecipientAdd(event) => {
            update!(update_with_channel_recipient_add, event);

            if let Some(ref handler) = handler!(on_channel_recipient_addition, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.user);
                });
            }
        },
        Event::ChannelRecipientRemove(event) => {
            update!(update_with_channel_recipient_remove, event);

            if let Some(ref handler) = handler!(on_channel_recipient_removal, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.user);
                });
            }
        },
        Event::ChannelUpdate(event) => {
            if let Some(ref handler) = handler!(on_channel_update, event_store) {
                let context = context(Some(event.channel.id()),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = CACHE.read()
                        .unwrap()
                        .get_channel(event.channel.id())
                        .map(|x| x.clone_inner());
                    update!(update_with_channel_update, event);

                    thread::spawn(move || {
                        (handler)(context, before, event.channel);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.channel);
                    });
                }}
            } else {
                update!(update_with_channel_update, event);
            }
        },
        Event::FriendSuggestionCreate(event) => {
            if let Some(ref handler) = handler!(on_friend_suggestion_create, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.suggested_user, event.reasons);
                });
            }
        },
        Event::FriendSuggestionDelete(event) => {
            if let Some(ref handler) = handler!(on_friend_suggestion_delete, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.suggested_user_id);
                });
            }
        },
        Event::GuildBanAdd(event) => {
            if let Some(ref handler) = handler!(on_guild_ban_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.user);
                });
            }
        },
        Event::GuildBanRemove(event) => {
            if let Some(ref handler) = handler!(on_guild_ban_removal, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.user);
                });
            }
        },
        Event::GuildCreate(event) => {
            update!(update_with_guild_create, event);

            if let Some(ref handler) = handler!(on_guild_create, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild);
                });
            }
        },
        Event::GuildDelete(event) => {
            if let Some(ref handler) = handler!(on_guild_delete, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let full = update!(update_with_guild_delete, event);

                    thread::spawn(move || {
                        (handler)(context, event.guild, full);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.guild);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _full = update!(update_with_guild_delete, event);
                }}
            }
        },
        Event::GuildEmojisUpdate(event) => {
            update!(update_with_guild_emojis_update, event);

            if let Some(ref handler) = handler!(on_guild_emojis_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.emojis);
                });
            }
        },
        Event::GuildIntegrationsUpdate(event) => {
            if let Some(ref handler) = handler!(on_guild_integrations_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id);
                });
            }
        },
        Event::GuildMemberAdd(event) => {
            update!(update_with_guild_member_add, event);

            if let Some(ref handler) = handler!(on_guild_member_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.member);
                });
            }
        },
        Event::GuildMemberRemove(event) => {
            if let Some(ref handler) = handler!(on_guild_member_removal, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let member = update!(update_with_guild_member_remove, event);

                    thread::spawn(move || {
                        (handler)(context, event.guild_id, event.user, member);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.guild_id, event.user);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _member = update!(update_with_guild_member_remove, event);
                }}
            }
        },
        Event::GuildMemberUpdate(event) => {
            if let Some(ref handler) = handler!(on_guild_member_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_guild_member_update, event);

                    // This is safe, as the update would have created the member
                    // if it did not exist. Thus, there _should_ be no way that this
                    // could fail under any circumstance.
                    let after = CACHE.read()
                        .unwrap()
                        .get_member(event.guild_id, event.user.id)
                        .unwrap()
                        .clone();

                    thread::spawn(move || {
                        (handler)(context, before, after);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event);
                    });
                }}
            } else {
                let _ = update!(update_with_guild_member_update, event);
            }
        },
        Event::GuildMembersChunk(event) => {
            update!(update_with_guild_members_chunk, event);

            if let Some(ref handler) = handler!(on_guild_members_chunk, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.members);
                });
            }
        },
        Event::GuildRoleCreate(event) => {
            update!(update_with_guild_role_create, event);

            if let Some(ref handler) = handler!(on_guild_role_create, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.role);
                });
            }
        },
        Event::GuildRoleDelete(event) => {
            if let Some(ref handler) = handler!(on_guild_role_delete, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let role = update!(update_with_guild_role_delete, event);

                    thread::spawn(move || {
                        (handler)(context, event.guild_id, event.role_id, role);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.guild_id, event.role_id);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _role = update!(update_with_guild_role_delete, event);
                }}
            }
        },
        Event::GuildRoleUpdate(event) => {
            if let Some(ref handler) = handler!(on_guild_role_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_guild_role_update, event);

                    thread::spawn(move || {
                        (handler)(context, event.guild_id, before, event.role);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.guild_id, event.role);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _before = update!(update_with_guild_role_update, event);
                }}
            }
        },
        Event::GuildSync(event) => {
            if let Some(ref handler) = handler!(on_guild_sync, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::GuildUnavailable(event) => {
            update!(update_with_guild_unavailable, event);

            if let Some(ref handler) = handler!(on_guild_unavailable, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id);
                });
            }
        },
        Event::GuildUpdate(event) => {
            if let Some(ref handler) = handler!(on_guild_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = CACHE.read()
                        .unwrap()
                        .guilds
                        .get(&event.guild.id)
                        .cloned();
                    update!(update_with_guild_update, event);

                    thread::spawn(move || {
                        (handler)(context, before, event.guild);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.guild);
                    });
                }}
            } else {
                update!(update_with_guild_update, event);
            }
        }
        Event::MessageAck(event) => {
            if let Some(ref handler) = handler!(on_message_ack, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.message_id);
                });
            }
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_event) => {},
        Event::MessageDeleteBulk(event) => {
            if let Some(ref handler) = handler!(on_message_delete_bulk, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.ids);
                });
            }
        },
        Event::MessageDelete(event) => {
            if let Some(ref handler) = handler!(on_message_delete, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.message_id);
                });
            }
        },
        Event::MessageUpdate(event) => {
            if let Some(ref handler) = handler!(on_message_update, event_store) {
                let context = context(Some(event.channel_id),
                                           conn,
                                           login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::PresencesReplace(event) => {
            update!(update_with_presences_replace, event);

            if let Some(handler) = handler!(on_presence_replace, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.presences);
                });
            }
        },
        Event::PresenceUpdate(event) => {
            update!(update_with_presence_update, event);

            if let Some(handler) = handler!(on_presence_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::ReactionAdd(event) => {
            if let Some(ref handler) = handler!(on_reaction_add, event_store) {
                let context = context(Some(event.reaction.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.reaction);
                });
            }
        },
        Event::ReactionRemove(event) => {
            if let Some(ref handler) = handler!(on_reaction_remove, event_store) {
                let context = context(Some(event.reaction.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.reaction);
                });
            }
        },
        Event::ReactionRemoveAll(event) => {
            if let Some(ref handler) = handler!(on_reaction_remove_all, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.channel_id, event.message_id);
                });
            }
        },
        Event::Ready(event) => {
            if let Some(ref handler) = handler!(on_ready, event_store) {
                update!(update_with_ready, event);

                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.ready);
                });
            } else {
                update!(update_with_ready, event);
            }
        },
        Event::RelationshipAdd(event) => {
            update!(update_with_relationship_add, event);

            if let Some(ref handler) = handler!(on_relationship_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.relationship);
                });
            }
        },
        Event::RelationshipRemove(event) => {
            update!(update_with_relationship_remove, event);

            if let Some(ref handler) = handler!(on_relationship_removal, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.user_id, event.kind);
                });
            }
        },
        Event::Resumed(event) => {
            if let Some(ref handler) = handler!(on_resume, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::TypingStart(event) => {
            if let Some(ref handler) = handler!(on_typing_start, event_store) {
                let context = context(Some(event.channel_id),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::Unknown(event) => {
            if let Some(ref handler) = handler!(on_unknown, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.kind, event.value);
                });
            }
        },
        Event::UserGuildSettingsUpdate(event) => {
            if let Some(ref handler) = handler!(on_user_guild_settings_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_user_guild_settings_update, event);

                    thread::spawn(move || {
                        (handler)(context, before, event.settings);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.settings);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _before = update!(update_with_user_guild_settings_update, event);
                }}
            }
        },
        Event::UserNoteUpdate(event) => {
            if let Some(ref handler) = handler!(on_note_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_user_note_update, event);

                    thread::spawn(move || {
                        (handler)(context, event.user_id, before, event.note);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.user_id, event.note);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _before = update!(update_with_user_note_update, event);
                }}
            }
        },
        Event::UserSettingsUpdate(event) => {
            if let Some(ref handler) = handler!(on_user_settings_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_user_settings_update, event, true);
                    let after = CACHE.read().unwrap().settings.clone();

                    thread::spawn(move || {
                        (handler)(context, before.unwrap(), after.unwrap());
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    update!(update_with_user_settings_update, event, false);
                }}
            }
        },
        Event::UserUpdate(event) => {
            if let Some(ref handler) = handler!(on_user_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                feature_cache! {{
                    let before = update!(update_with_user_update, event);

                    thread::spawn(move || {
                        (handler)(context, before, event.current_user);
                    });
                } else {
                    thread::spawn(move || {
                        (handler)(context, event.current_user);
                    });
                }}
            } else {
                feature_cache_enabled! {{
                    let _before = update!(update_with_user_update, event);
                }}
            }
        },
        Event::VoiceServerUpdate(event) => {
            if let Some(ref handler) = handler!(on_voice_server_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Event::VoiceStateUpdate(event) => {
            update!(update_with_voice_state_update, event);

            if let Some(ref handler) = handler!(on_voice_state_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.voice_state);
                });
            }
        },
        Event::WebhookUpdate(event) => {
            if let Some(ref handler) = handler!(on_webhook_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.channel_id);
                });
            }
        },
    }
}
