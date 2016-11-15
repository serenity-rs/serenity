use std::sync::{Arc, Mutex};
use std::thread;
use super::event_store::EventStore;
use super::login_type::LoginType;
use super::{STATE, Connection, Context};
use ::ext::framework::Framework;
use ::internal::prelude::*;
use ::model::{ChannelId, Event, Message};

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
        STATE.lock().unwrap().$method(&$event);
    };
    ($method:ident, $event:expr, $old:expr) => {
        STATE.lock().unwrap().$method(&$event, $old);
    };
}

fn context(channel_id: Option<ChannelId>,
           conn: Arc<Mutex<Connection>>,
           login_type: LoginType) -> Context {
    Context::new(channel_id, conn, login_type)
}

#[allow(cyclomatic_complexity)]
pub fn dispatch(event: Result<Event>,
                conn: Arc<Mutex<Connection>>,
                framework: Arc<Mutex<Framework>>,
                login_type: LoginType,
                event_store: Arc<Mutex<EventStore>>) {
    match event {
        Ok(Event::CallCreate(event)) => {
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
        Ok(Event::CallDelete(event)) => {
            if let Some(ref handler) = handler!(on_call_delete, event_store) {
                let call = STATE
                    .lock()
                    .unwrap()
                    .calls
                    .remove(&event.channel_id);
                update!(update_with_call_delete, event);

                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, call);
                });
            } else {
                update!(update_with_call_delete, event);
            }
        },
        Ok(Event::CallUpdate(event)) => {
            if let Some(ref handler) = handler!(on_call_update, event_store) {
                let before = update!(update_with_call_update, event, true);
                let after = STATE
                    .lock()
                    .unwrap()
                    .calls
                    .get(&event.channel_id)
                    .cloned();

                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, after);
                });
            } else {
                update!(update_with_call_update, event, false);
            }
        },
        Ok(Event::ChannelCreate(event)) => {
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
        Ok(Event::ChannelDelete(event)) => {
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
        Ok(Event::ChannelPinsAck(event)) => {
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
        Ok(Event::ChannelPinsUpdate(event)) => {
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
        Ok(Event::ChannelRecipientAdd(event)) => {
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
        Ok(Event::ChannelRecipientRemove(event)) => {
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
        Ok(Event::ChannelUpdate(event)) => {
            if let Some(ref handler) = handler!(on_channel_update, event_store) {
                let before = STATE.lock()
                    .unwrap()
                    .find_channel(event.channel.id());
                update!(update_with_channel_update, event);
                let context = context(Some(event.channel.id()),
                                      conn,
                                      login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, event.channel);
                });
            } else {
                update!(update_with_channel_update, event);
            }
        },
        Ok(Event::GuildBanAdd(event)) => {
            if let Some(ref handler) = handler!(on_guild_ban_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.user);
                });
            }
        },
        Ok(Event::GuildBanRemove(event)) => {
            if let Some(ref handler) = handler!(on_guild_ban_removal, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.user);
                });
            }
        },
        Ok(Event::GuildCreate(event)) => {
            update!(update_with_guild_create, event);

            if let Some(ref handler) = handler!(on_guild_create, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild);
                });
            }
        },
        Ok(Event::GuildDelete(event)) => {
            if let Some(ref handler) = handler!(on_guild_delete, event_store) {
                let full = update!(update_with_guild_delete, event);
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild, full);
                });
            } else {
                let _full = update!(update_with_guild_delete, event);
            }
        },
        Ok(Event::GuildEmojisUpdate(event)) => {
            update!(update_with_guild_emojis_update, event);

            if let Some(ref handler) = handler!(on_guild_emojis_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.emojis);
                });
            }
        },
        Ok(Event::GuildIntegrationsUpdate(event)) => {
            if let Some(ref handler) = handler!(on_guild_integrations_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id);
                });
            }
        },
        Ok(Event::GuildMemberAdd(event)) => {
            update!(update_with_guild_member_add, event);

            if let Some(ref handler) = handler!(on_guild_member_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.member);
                });
            }
        },
        Ok(Event::GuildMemberRemove(event)) => {
            if let Some(ref handler) = handler!(on_guild_member_removal, event_store) {
                let member = update!(update_with_guild_member_remove, event);
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.user, member);
                });
            } else {
                let _member = update!(update_with_guild_member_remove, event);
            }
        },
        Ok(Event::GuildMemberUpdate(event)) => {
            if let Some(ref handler) = handler!(on_guild_member_update, event_store) {
                let before = update!(update_with_guild_member_update, event, true);

                // This is safe, as the update would have created the member
                // if it did not exist. Thus, there _should_ be no way that this
                // could fail under any circumstance.
                let after = STATE.lock()
                    .unwrap()
                    .find_member(event.guild_id, event.user.id)
                    .unwrap()
                    .clone();
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, after);
                });
            } else {
                let _ = update!(update_with_guild_member_update, event, false);
            }
        },
        Ok(Event::GuildMembersChunk(event)) => {
            update!(update_with_guild_members_chunk, event);

            if let Some(ref handler) = handler!(on_guild_members_chunk, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.members);
                });
            }
        },
        Ok(Event::GuildRoleCreate(event)) => {
            update!(update_with_guild_role_create, event);

            if let Some(ref handler) = handler!(on_guild_role_create, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.role);
                });
            }
        },
        Ok(Event::GuildRoleDelete(event)) => {
            if let Some(ref handler) = handler!(on_guild_role_delete, event_store) {
                let role = update!(update_with_guild_role_delete, event);
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.role_id, role);
                });
            }
        },
        Ok(Event::GuildRoleUpdate(event)) => {
            let before = update!(update_with_guild_role_update, event);

            if let Some(ref handler) = handler!(on_guild_role_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, before, event.role);
                });
            }
        },
        Ok(Event::GuildSync(event)) => {
            if let Some(ref handler) = handler!(on_guild_sync, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Ok(Event::GuildUnavailable(event)) => {
            update!(update_with_guild_unavailable, event);

            if let Some(ref handler) = handler!(on_guild_unavailable, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id);
                });
            }
        },
        Ok(Event::GuildUpdate(event)) => {
            if let Some(ref handler) = handler!(on_guild_update, event_store) {
                let before = STATE.lock()
                    .unwrap()
                    .guilds
                    .get(&event.guild.id)
                    .cloned();
                update!(update_with_guild_update, event);
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, event.guild);
                });
            } else {
                update!(update_with_guild_update, event);
            }
        }
        Ok(Event::MessageAck(event)) => {
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
        Ok(Event::MessageCreate(event)) => {
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
        Ok(Event::MessageDeleteBulk(event)) => {
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
        Ok(Event::MessageDelete(event)) => {
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
        Ok(Event::MessageUpdate(event)) => {
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
        Ok(Event::PresencesReplace(event)) => {
            update!(update_with_presences_replace, event);

            if let Some(handler) = handler!(on_presence_replace, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.presences);
                });
            }
        },
        Ok(Event::PresenceUpdate(event)) => {
            update!(update_with_presence_update, event);

            if let Some(handler) = handler!(on_presence_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Ok(Event::ReactionAdd(event)) => {
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
        Ok(Event::ReactionRemove(event)) => {
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
        Ok(Event::ReactionRemoveAll(event)) => {
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
        Ok(Event::Ready(event)) => {
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
        Ok(Event::RelationshipAdd(event)) => {
            update!(update_with_relationship_add, event);

            if let Some(ref handler) = handler!(on_relationship_addition, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.relationship);
                });
            }
        },
        Ok(Event::RelationshipRemove(event)) => {
            update!(update_with_relationship_remove, event);

            if let Some(ref handler) = handler!(on_relationship_removal, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.user_id, event.kind);
                });
            }
        },
        Ok(Event::Resumed(event)) => {
            if let Some(ref handler) = handler!(on_resume, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Ok(Event::TypingStart(event)) => {
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
        Ok(Event::Unknown(event)) => {
            if let Some(ref handler) = handler!(on_unknown, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.kind, event.value);
                });
            }
        },
        Ok(Event::UserGuildSettingsUpdate(event)) => {
            let before = update!(update_with_user_guild_settings_update, event);

            if let Some(ref handler) = handler!(on_user_guild_settings_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, event.settings);
                });
            }
        },
        Ok(Event::UserNoteUpdate(event)) => {
            let before = update!(update_with_user_note_update, event);

            if let Some(ref handler) = handler!(on_note_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.user_id, before, event.note);
                });
            }
        },
        Ok(Event::UserSettingsUpdate(event)) => {
            if let Some(ref handler) = handler!(on_user_settings_update, event_store) {
                let before = update!(update_with_user_settings_update, event, true);
                let after = STATE.lock().unwrap().settings.clone();

                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before.unwrap(), after.unwrap());
                });
            } else {
                update!(update_with_user_settings_update, event, false);
            }
        },
        Ok(Event::UserUpdate(event)) => {
            let before = update!(update_with_user_update, event);

            if let Some(ref handler) = handler!(on_user_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, before, event.current_user);
                });
            }
        },
        Ok(Event::VoiceServerUpdate(event)) => {
            if let Some(ref handler) = handler!(on_voice_server_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event);
                });
            }
        },
        Ok(Event::VoiceStateUpdate(event)) => {
            update!(update_with_voice_state_update, event);

            if let Some(ref handler) = handler!(on_voice_state_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.voice_state);
                });
            }
        },
        Ok(Event::WebhookUpdate(event)) => {
            if let Some(ref handler) = handler!(on_webhook_update, event_store) {
                let context = context(None, conn, login_type);
                let handler = handler.clone();

                thread::spawn(move || {
                    (handler)(context, event.guild_id, event.channel_id);
                });
            }
        },
        Err(_why) => {},
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
