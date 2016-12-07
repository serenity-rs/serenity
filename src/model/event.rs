//! All the events this library handles.

use std::collections::{BTreeMap, HashMap};
use super::utils::*;
use super::*;
use ::constants::{OpCode, VoiceOpCode};
use ::internal::prelude::*;
use ::utils::decode_array;

#[derive(Clone, Debug)]
pub struct CallCreateEvent {
    pub call: Call,
}

#[derive(Clone, Debug)]
pub struct CallDeleteEvent {
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug)]
pub struct CallUpdateEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub region: String,
    pub ringing: Vec<UserId>,
}

#[derive(Clone, Debug)]
pub struct ChannelCreateEvent {
    pub channel: Channel,
}

#[derive(Clone, Debug)]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

#[derive(Clone, Debug)]
pub struct ChannelPinsAckEvent {
    pub channel_id: ChannelId,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ChannelRecipientAddEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct ChannelRecipientRemoveEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

#[derive(Clone, Debug)]
pub struct FriendSuggestionCreateEvent {
    pub reasons: Vec<SuggestionReason>,
    pub suggested_user: User,
}

#[derive(Clone, Copy, Debug)]
pub struct FriendSuggestionDeleteEvent {
    pub suggested_user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

#[derive(Clone, Debug)]
pub struct GuildDeleteEvent {
    pub guild: PartialGuild,
}

#[derive(Clone, Debug)]
pub struct GuildEmojisUpdateEvent {
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildMemberAddEvent {
    pub guild_id: GuildId,
    pub member: Member,
}

#[derive(Clone, Debug)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildMembersChunkEvent {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, Member>,
}

#[derive(Clone, Debug)]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[derive(Clone, Debug)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug)]
pub struct GuildSyncEvent {
    pub guild_id: GuildId,
    pub large: bool,
    pub members: HashMap<UserId, Member>,
    pub presences: HashMap<UserId, Presence>,
}

#[derive(Clone, Debug)]
pub struct GuildUnavailableEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

#[derive(Clone, Copy, Debug)]
pub struct MessageAckEvent {
    pub channel_id: ChannelId,
    /// May be `None` if a private channel with no messages has closed.
    pub message_id: Option<MessageId>,
}

#[derive(Clone, Debug)]
pub struct MessageCreateEvent {
    pub message: Message,
}

#[derive(Clone, Debug)]
pub struct MessageDeleteBulkEvent {
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

#[derive(Clone, Copy, Debug)]
pub struct MessageDeleteEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

#[derive(Clone, Debug)]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub kind: Option<MessageType>,
    pub content: Option<String>,
    pub nonce: Option<String>,
    pub tts: Option<bool>,
    pub pinned: Option<bool>,
    pub timestamp: Option<String>,
    pub edited_timestamp: Option<String>,
    pub author: Option<User>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Value>>,
}

#[derive(Clone, Debug)]
pub struct PresenceUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub presence: Presence,
    pub roles: Option<Vec<RoleId>>,
}

#[derive(Clone, Debug)]
pub struct PresencesReplaceEvent {
    pub presences: Vec<Presence>,
}

#[derive(Clone, Debug)]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

#[derive(Clone, Debug)]
pub struct ReactionRemoveEvent {
    pub reaction: Reaction,
}

#[derive(Clone, Copy, Debug)]
pub struct ReactionRemoveAllEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

/// The "Ready" event, containing initial ready cache
#[derive(Clone, Debug)]
pub struct ReadyEvent {
    pub ready: Ready,
}

#[derive(Clone, Debug)]
pub struct RelationshipAddEvent {
    pub relationship: Relationship,
}

#[derive(Clone, Copy, Debug)]
pub struct RelationshipRemoveEvent {
    pub kind: RelationshipType,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct ResumedEvent {
    pub heartbeat_interval: u64,
    pub trace: Vec<Option<String>>,
}

#[derive(Clone, Debug)]
pub struct TypingStartEvent {
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct UnknownEvent {
    pub kind: String,
    pub value: BTreeMap<String, Value>
}

#[derive(Clone, Debug)]
pub struct UserGuildSettingsUpdateEvent {
    pub settings: UserGuildSettings,
}

#[derive(Clone, Debug)]
pub struct UserNoteUpdateEvent {
    pub note: String,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

#[derive(Clone, Debug)]
pub struct UserSettingsUpdateEvent {
    pub enable_tts_command: Option<bool>,
    pub inline_attachment_media: Option<bool>,
    pub inline_embed_media: Option<bool>,
    pub locale: Option<String>,
    pub message_display_compact: Option<bool>,
    pub render_embeds: Option<bool>,
    pub show_current_game: Option<bool>,
    pub theme: Option<String>,
    pub convert_emoticons: Option<bool>,
    pub friend_source_flags: Option<FriendSourceFlags>,
}

#[derive(Clone, Debug)]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct VoiceStateUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub voice_state: VoiceState,
}

#[derive(Clone, Debug)]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

#[derive(Debug, Clone)]
pub enum GatewayEvent {
    Dispatch(u64, Event),
    Heartbeat(u64),
    Reconnect,
    InvalidateSession,
    Hello(u64),
    HeartbeatAck,
}

impl GatewayEvent {
    pub fn decode(value: Value) -> Result<Self> {
        let mut value = into_map(value)?;

        let op = req!(value.get("op").and_then(|x| x.as_u64()));

        match OpCode::from_num(op).ok_or(Error::Client(ClientError::InvalidOpCode))? {
            OpCode::Event => Ok(GatewayEvent::Dispatch(
                req!(remove(&mut value, "s")?.as_u64()),
                Event::decode(
                    remove(&mut value, "t").and_then(into_string)?,
                    remove(&mut value, "d")?
                )?
            )),
            OpCode::Heartbeat => {
                Ok(GatewayEvent::Heartbeat(req!(remove(&mut value, "s")?
                    .as_u64())))
            },
            OpCode::Reconnect => Ok(GatewayEvent::Reconnect),
            OpCode::InvalidSession => Ok(GatewayEvent::InvalidateSession),
            OpCode::Hello => {
                let mut data = remove(&mut value, "d").and_then(into_map)?;
                let interval = req!(remove(&mut data, "heartbeat_interval")?.as_u64());
                Ok(GatewayEvent::Hello(interval))
            },
            OpCode::HeartbeatAck => Ok(GatewayEvent::HeartbeatAck),
            _ => Err(Error::Decode("Unexpected opcode", Value::Object(value))),
        }
    }
}

/// Event received over a websocket connection
#[derive(Clone, Debug)]
pub enum Event {
    /// A new group call has been created
    CallCreate(CallCreateEvent),
    /// A group call has been deleted (the call ended)
    CallDelete(CallDeleteEvent),
    /// A group call has been updated
    CallUpdate(CallUpdateEvent),
    ChannelCreate(ChannelCreateEvent),
    ChannelDelete(ChannelDeleteEvent),
    ChannelPinsAck(ChannelPinsAckEvent),
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A user has been added to a group
    ChannelRecipientAdd(ChannelRecipientAddEvent),
    /// A user has been removed from a group
    ChannelRecipientRemove(ChannelRecipientRemoveEvent),
    ChannelUpdate(ChannelUpdateEvent),
    /// When a suggestion for a friend is created, due to a connection like
    /// [`Skype`].
    ///
    /// [`Connection::Skype`]: enum.Connection.html#variant.Skype
    FriendSuggestionCreate(FriendSuggestionCreateEvent),
    /// When a suggestion for a friend is removed.
    FriendSuggestionDelete(FriendSuggestionDeleteEvent),
    GuildBanAdd(GuildBanAddEvent),
    GuildBanRemove(GuildBanRemoveEvent),
    GuildCreate(GuildCreateEvent),
    GuildDelete(GuildDeleteEvent),
    GuildEmojisUpdate(GuildEmojisUpdateEvent),
    GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent),
    GuildMemberAdd(GuildMemberAddEvent),
    GuildMemberRemove(GuildMemberRemoveEvent),
    /// A member's roles have changed
    GuildMemberUpdate(GuildMemberUpdateEvent),
    GuildMembersChunk(GuildMembersChunkEvent),
    GuildRoleCreate(GuildRoleCreateEvent),
    GuildRoleDelete(GuildRoleDeleteEvent),
    GuildRoleUpdate(GuildRoleUpdateEvent),
    GuildSync(GuildSyncEvent),
    /// When a guild is unavailable, such as due to a Discord server outage.
    GuildUnavailable(GuildUnavailableEvent),
    GuildUpdate(GuildUpdateEvent),
    /// Another logged-in device acknowledged this message
    MessageAck(MessageAckEvent),
    MessageCreate(MessageCreateEvent),
    MessageDelete(MessageDeleteEvent),
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// A message has been edited, either by the user or the system
    MessageUpdate(MessageUpdateEvent),
    /// A member's presence state (or username or avatar) has changed
    PresenceUpdate(PresenceUpdateEvent),
    /// The precense list of the user's friends should be replaced entirely
    PresencesReplace(PresencesReplaceEvent),
    /// A reaction was added to a message.
    ///
    /// Fires the [`on_message_reaction_add`] event handler.
    ///
    /// [`on_message_reaction_add`]: ../client/struct.Client.html#method.on_message_reaction_add
    ReactionAdd(ReactionAddEvent),
    /// A reaction was removed to a message.
    ///
    /// Fires the [`on_message_reaction_remove`] event handler.
    ///
    /// [`on_message_reaction_remove`]: ../client/struct.Client.html#method.on_message_reaction_remove
    ReactionRemove(ReactionRemoveEvent),
    /// A request was issued to remove all [`Reaction`]s from a [`Message`].
    ///
    /// Fires the [`on_reaction_remove_all`] event handler.
    ///
    /// [`Message`]: struct.Message.html
    /// [`Reaction`]: struct.Reaction.html
    /// [`on_reaction_remove_all`]: ../client/struct.Clint.html#method.on_reaction_remove_all
    ReactionRemoveAll(ReactionRemoveAllEvent),
    /// The first event in a connection, containing the initial ready cache.
    ///
    /// May also be received at a later time in the event of a reconnect.
    Ready(ReadyEvent),
    RelationshipAdd(RelationshipAddEvent),
    RelationshipRemove(RelationshipRemoveEvent),
    /// The connection has successfully resumed after a disconnect.
    Resumed(ResumedEvent),
    /// A user is typing; considered to last 5 seconds
    TypingStart(TypingStartEvent),
    /// Update to the logged-in user's guild-specific notification settings
    UserGuildSettingsUpdate(UserGuildSettingsUpdateEvent),
    /// Update to a note that the logged-in user has set for another user.
    UserNoteUpdate(UserNoteUpdateEvent),
    /// Update to the logged-in user's information
    UserUpdate(UserUpdateEvent),
    /// Update to the logged-in user's preferences or client settings
    UserSettingsUpdate(UserSettingsUpdateEvent),
    /// A member's voice state has changed
    VoiceStateUpdate(VoiceStateUpdateEvent),
    /// Voice server information is available
    VoiceServerUpdate(VoiceServerUpdateEvent),
    /// A webhook for a [channel][`GuildChannel`] was updated in a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    WebhookUpdate(WebhookUpdateEvent),
    /// An event type not covered by the above
    Unknown(UnknownEvent),
}

impl Event {
    #[allow(cyclomatic_complexity)]
    fn decode(kind: String, value: Value) -> Result<Event> {
        if kind == "PRESENCES_REPLACE" {
            return Ok(Event::PresencesReplace(PresencesReplaceEvent {
                presences: decode_array(value, Presence::decode)?,
            }));
        }

        let mut value = into_map(value)?;

        if kind == "CALL_CREATE" {
            Ok(Event::CallCreate(CallCreateEvent {
                call: Call::decode(Value::Object(value))?,
            }))
        } else if kind == "CALL_DELETE" {
            Ok(Event::CallDelete(CallDeleteEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
            }))
        } else if kind == "CALL_UPDATE" {
            Ok(Event::CallUpdate(CallUpdateEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                message_id: remove(&mut value, "message_id").and_then(MessageId::decode)?,
                region: remove(&mut value, "region").and_then(into_string)?,
                ringing: decode_array(remove(&mut value, "ringing")?, UserId::decode)?,
            }))
        } else if kind == "CHANNEL_CREATE" {
            Ok(Event::ChannelCreate(ChannelCreateEvent {
                channel: Channel::decode(Value::Object(value))?,
            }))
        } else if kind == "CHANNEL_DELETE" {
            Ok(Event::ChannelDelete(ChannelDeleteEvent {
                channel: Channel::decode(Value::Object(value))?,
            }))
        } else if kind == "CHANNEL_PINS_ACK" {
            Ok(Event::ChannelPinsAck(ChannelPinsAckEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                timestamp: remove(&mut value, "timestamp").and_then(into_string)?,
            }))
        } else if kind == "CHANNEL_PINS_UPDATE" {
            Ok(Event::ChannelPinsUpdate(ChannelPinsUpdateEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                last_pin_timestamp: opt(&mut value, "last_pin_timestamp", into_string)?,
            }))
        } else if kind == "CHANNEL_RECIPIENT_ADD" {
            Ok(Event::ChannelRecipientAdd(ChannelRecipientAddEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "CHANNEL_RECIPIENT_REMOVE" {
            Ok(Event::ChannelRecipientRemove(ChannelRecipientRemoveEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "CHANNEL_UPDATE" {
            Ok(Event::ChannelUpdate(ChannelUpdateEvent {
                channel: Channel::decode(Value::Object(value))?,
            }))
        } else if kind == "FRIEND_SUGGESTION_CREATE" {
            Ok(Event::FriendSuggestionCreate(FriendSuggestionCreateEvent {
                reasons: decode_array(remove(&mut value, "reasons")?, SuggestionReason::decode)?,
                suggested_user: remove(&mut value, "suggested_user").and_then(User::decode)?,
            }))
        } else if kind == "FRIEND_SUGGESTION_DELETE" {
            Ok(Event::FriendSuggestionDelete(FriendSuggestionDeleteEvent {
                suggested_user_id: remove(&mut value, "suggested_user_id").and_then(UserId::decode)?,
            }))
        } else if kind == "GUILD_BAN_ADD" {
            Ok(Event::GuildBanAdd(GuildBanAddEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "GUILD_BAN_REMOVE" {
            Ok(Event::GuildBanRemove(GuildBanRemoveEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "GUILD_CREATE" {
            if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(Event::GuildUnavailable(GuildUnavailableEvent {
                    guild_id: remove(&mut value, "id").and_then(GuildId::decode)?,
                }))
            } else {
                Ok(Event::GuildCreate(GuildCreateEvent {
                    guild: Guild::decode(Value::Object(value))?,
                }))
            }
        } else if kind == "GUILD_DELETE" {
            if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(Event::GuildUnavailable(GuildUnavailableEvent {
                    guild_id: remove(&mut value, "id").and_then(GuildId::decode)?,
                }))
            } else {
                Ok(Event::GuildDelete(GuildDeleteEvent {
                    guild: PartialGuild::decode(Value::Object(value))?,
                }))
            }
        } else if kind == "GUILD_EMOJIS_UPDATE" {
            Ok(Event::GuildEmojisUpdate(GuildEmojisUpdateEvent {
                emojis: remove(&mut value, "emojis").and_then(decode_emojis)?,
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
            }))
        } else if kind == "GUILD_INTEGRATIONS_UPDATE" {
            Ok(Event::GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
            }))
        } else if kind == "GUILD_MEMBER_ADD" {
            Ok(Event::GuildMemberAdd(GuildMemberAddEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                member: Member::decode(Value::Object(value))?,
            }))
        } else if kind == "GUILD_MEMBER_REMOVE" {
            Ok(Event::GuildMemberRemove(GuildMemberRemoveEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "GUILD_MEMBER_UPDATE" {
            Ok(Event::GuildMemberUpdate(GuildMemberUpdateEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                nick: opt(&mut value, "nick", into_string)?,
                roles: decode_array(remove(&mut value, "roles")?, RoleId::decode)?,
                user: remove(&mut value, "user").and_then(User::decode)?,
            }))
        } else if kind == "GUILD_MEMBERS_CHUNK" {
            Ok(Event::GuildMembersChunk(GuildMembersChunkEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                members: remove(&mut value, "members").and_then(decode_members)?,
            }))
        } else if kind == "GUILD_ROLE_CREATE" {
            Ok(Event::GuildRoleCreate(GuildRoleCreateEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                role: remove(&mut value, "role").and_then(Role::decode)?,
            }))
        } else if kind == "GUILD_ROLE_DELETE" {
            Ok(Event::GuildRoleDelete(GuildRoleDeleteEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                role_id: remove(&mut value, "role_id").and_then(RoleId::decode)?,
            }))
        } else if kind == "GUILD_ROLE_UPDATE" {
            Ok(Event::GuildRoleUpdate(GuildRoleUpdateEvent {
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
                role: remove(&mut value, "role").and_then(Role::decode)?,
            }))
        } else if kind == "GUILD_SYNC" {
            Ok(Event::GuildSync(GuildSyncEvent {
                guild_id: remove(&mut value, "id").and_then(GuildId::decode)?,
                large: req!(remove(&mut value, "large")?.as_bool()),
                members: remove(&mut value, "members").and_then(decode_members)?,
                presences: remove(&mut value, "presences").and_then(decode_presences)?,
            }))
        } else if kind == "GUILD_UPDATE" {
            Ok(Event::GuildUpdate(GuildUpdateEvent {
                guild: PartialGuild::decode(Value::Object(value))?,
            }))
        } else if kind == "MESSAGE_ACK" {
            Ok(Event::MessageAck(MessageAckEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                message_id: opt(&mut value, "message_id", MessageId::decode)?,
            }))
        } else if kind == "MESSAGE_CREATE" {
            Ok(Event::MessageCreate(MessageCreateEvent {
                message: Message::decode(Value::Object(value))?,
            }))
        } else if kind == "MESSAGE_DELETE" {
            Ok(Event::MessageDelete(MessageDeleteEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                message_id: remove(&mut value, "id").and_then(MessageId::decode)?,
            }))
        } else if kind == "MESSAGE_DELETE_BULK" {
            Ok(Event::MessageDeleteBulk(MessageDeleteBulkEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                ids: decode_array(remove(&mut value, "ids")?, MessageId::decode)?,
            }))
        } else if kind == "MESSAGE_REACTION_ADD" {
            Ok(Event::ReactionAdd(ReactionAddEvent {
                reaction: Reaction::decode(Value::Object(value))?
            }))
        } else if kind == "MESSAGE_REACTION_REMOVE" {
            Ok(Event::ReactionRemove(ReactionRemoveEvent {
                reaction: Reaction::decode(Value::Object(value))?
            }))
        } else if kind == "MESSAGE_REACTION_REMOVE_ALL" {
            Ok(Event::ReactionRemoveAll(ReactionRemoveAllEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                message_id: remove(&mut value, "message_id").and_then(MessageId::decode)?,
            }))
        } else if kind == "MESSAGE_UPDATE" {
            Ok(Event::MessageUpdate(MessageUpdateEvent {
                id: remove(&mut value, "id").and_then(MessageId::decode)?,
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                kind: opt(&mut value, "type", MessageType::decode)?,
                content: opt(&mut value, "content", into_string)?,
                nonce: remove(&mut value, "nonce").and_then(into_string).ok(),
                tts: remove(&mut value, "tts").ok().and_then(|v| v.as_bool()),
                pinned: remove(&mut value, "pinned").ok().and_then(|v| v.as_bool()),
                timestamp: opt(&mut value, "timestamp", into_string)?,
                edited_timestamp: opt(&mut value, "edited_timestamp", into_string)?,
                author: opt(&mut value, "author", User::decode)?,
                mention_everyone: remove(&mut value, "mention_everyone").ok().and_then(|v| v.as_bool()),
                mentions: opt(&mut value, "mentions", |v| decode_array(v, User::decode))?,
                mention_roles: opt(&mut value, "mention_roles", |v| decode_array(v, RoleId::decode))?,
                attachments: opt(&mut value, "attachments", |v| decode_array(v, Attachment::decode))?,
                embeds: opt(&mut value, "embeds", |v| decode_array(v, Ok))?,
            }))
        } else if kind == "PRESENCE_UPDATE" {
            let guild_id = opt(&mut value, "guild_id", GuildId::decode)?;
            let roles = opt(&mut value, "roles", |v| decode_array(v, RoleId::decode))?;
            let presence = Presence::decode(Value::Object(value))?;
            Ok(Event::PresenceUpdate(PresenceUpdateEvent {
                guild_id: guild_id,
                presence: presence,
                roles: roles,
            }))
        } else if kind == "RELATIONSHIP_ADD" {
            Ok(Event::RelationshipAdd(RelationshipAddEvent {
                relationship: Relationship::decode(Value::Object(value))?,
            }))
        } else if kind == "RELATIONSHIP_REMOVE" {
            Ok(Event::RelationshipRemove(RelationshipRemoveEvent {
                kind: remove(&mut value, "type").and_then(RelationshipType::decode)?,
                user_id: remove(&mut value, "id").and_then(UserId::decode)?,
            }))
        } else if kind == "READY" {
            Ok(Event::Ready(ReadyEvent {
                ready: Ready::decode(Value::Object(value))?,
            }))
        } else if kind == "RESUMED" {
            Ok(Event::Resumed(ResumedEvent {
                heartbeat_interval: req!(remove(&mut value, "heartbeat_interval")?.as_u64()),
                trace: remove(&mut value, "_trace").and_then(|v| decode_array(v, |v| Ok(into_string(v).ok())))?,
            }))
        } else if kind == "TYPING_START" {
            Ok(Event::TypingStart(TypingStartEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                timestamp: req!(remove(&mut value, "timestamp")?.as_u64()),
                user_id: remove(&mut value, "user_id").and_then(UserId::decode)?,
            }))
        } else if kind == "USER_GUILD_SETTINGS_UPDATE" {
            Ok(Event::UserGuildSettingsUpdate(UserGuildSettingsUpdateEvent {
                settings: UserGuildSettings::decode(Value::Object(value))?,
            }))
        } else if kind == "USER_NOTE_UPDATE" {
            Ok(Event::UserNoteUpdate(UserNoteUpdateEvent {
                note: remove(&mut value, "note").and_then(into_string)?,
                user_id: remove(&mut value, "id").and_then(UserId::decode)?,
            }))
        } else if kind == "USER_SETTINGS_UPDATE" {
            Ok(Event::UserSettingsUpdate(UserSettingsUpdateEvent {
                enable_tts_command: remove(&mut value, "enable_tts_command").ok().and_then(|v| v.as_bool()),
                inline_attachment_media: remove(&mut value, "inline_attachment_media").ok().and_then(|v| v.as_bool()),
                inline_embed_media: remove(&mut value, "inline_embed_media").ok().and_then(|v| v.as_bool()),
                locale: opt(&mut value, "locale", into_string)?,
                message_display_compact: remove(&mut value, "message_display_compact").ok().and_then(|v| v.as_bool()),
                render_embeds: remove(&mut value, "render_embeds").ok().and_then(|v| v.as_bool()),
                show_current_game: remove(&mut value, "show_current_game").ok().and_then(|v| v.as_bool()),
                theme: opt(&mut value, "theme", into_string)?,
                convert_emoticons: remove(&mut value, "convert_emoticons").ok().and_then(|v| v.as_bool()),
                friend_source_flags: opt(&mut value, "friend_source_flags", FriendSourceFlags::decode)?,
            }))
        } else if kind == "USER_UPDATE" {
            Ok(Event::UserUpdate(UserUpdateEvent {
                current_user: CurrentUser::decode(Value::Object(value))?,
            }))
        } else if kind == "VOICE_SERVER_UPDATE" {
            Ok(Event::VoiceServerUpdate(VoiceServerUpdateEvent {
                guild_id: opt(&mut value, "guild_id", GuildId::decode)?,
                channel_id: opt(&mut value, "channel_id", ChannelId::decode)?,
                endpoint: opt(&mut value, "endpoint", into_string)?,
                token: remove(&mut value, "token").and_then(into_string)?,
            }))
        } else if kind == "VOICE_STATE_UPDATE" {
            Ok(Event::VoiceStateUpdate(VoiceStateUpdateEvent {
                guild_id: opt(&mut value, "guild_id", GuildId::decode)?,
                voice_state: VoiceState::decode(Value::Object(value))?,
            }))
        } else if kind == "WEBHOOKS_UPDATE" {
            Ok(Event::WebhookUpdate(WebhookUpdateEvent {
                channel_id: remove(&mut value, "channel_id").and_then(ChannelId::decode)?,
                guild_id: remove(&mut value, "guild_id").and_then(GuildId::decode)?,
            }))
        } else {
            Ok(Event::Unknown(UnknownEvent {
                kind: kind,
                value: value,
            }))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

#[derive(Clone, Debug)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

#[derive(Clone, Debug)]
pub struct VoiceReady {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub enum VoiceEvent {
    Heartbeat(VoiceHeartbeat),
    Hello(VoiceHello),
    Ready(VoiceReady),
    Speaking(VoiceSpeaking),
    KeepAlive,
    Unknown(VoiceOpCode, Value)
}

impl VoiceEvent {
    pub fn decode(value: Value) -> Result<VoiceEvent> {
        let mut value = into_map(value)?;
        let op = req!(remove(&mut value, "op")?.as_u64());
        let mut map = remove(&mut value, "d").and_then(into_map)?;

        let opcode = VoiceOpCode::from_num(op)
            .ok_or(Error::Client(ClientError::InvalidOpCode))?;

        match opcode {
            VoiceOpCode::Heartbeat => {
                Ok(VoiceEvent::Heartbeat(VoiceHeartbeat {
                    heartbeat_interval: req!(remove(&mut map, "heartbeat_interval")?.as_u64()),
                }))
            },
            VoiceOpCode::Hello => {
                Ok(VoiceEvent::Hello(VoiceHello {
                    heartbeat_interval: req!(remove(&mut map, "heartbeat_interval")?
                        .as_u64()),
                    ip: remove(&mut map, "ip").and_then(into_string)?,
                    modes: decode_array(remove(&mut map, "modes")?,
                                             into_string)?,
                    port: req!(remove(&mut map, "port")?
                        .as_u64()) as u16,
                    ssrc: req!(remove(&mut map, "ssrc")?
                        .as_u64()) as u32,
                }))
            },
            VoiceOpCode::KeepAlive => Ok(VoiceEvent::KeepAlive),
            VoiceOpCode::SessionDescription => {
                Ok(VoiceEvent::Ready(VoiceReady {
                    mode: remove(&mut map, "mode")
                        .and_then(into_string)?,
                    secret_key: decode_array(remove(&mut map, "secret_key")?,
                        |v| Ok(req!(v.as_u64()) as u8)
                    )?,
                }))
            },
            VoiceOpCode::Speaking => {
                Ok(VoiceEvent::Speaking(VoiceSpeaking {
                    speaking: req!(remove(&mut map, "speaking")?.as_bool()),
                    ssrc: req!(remove(&mut map, "ssrc")?.as_u64()) as u32,
                    user_id: remove(&mut map, "user_id").and_then(UserId::decode)?,
                }))
            }
            other => Ok(VoiceEvent::Unknown(other, Value::Object(map))),
        }
    }
}
