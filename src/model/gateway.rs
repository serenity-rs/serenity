use std::collections::{BTreeMap, HashMap};
use super::utils::*;
use super::*;
use ::constants::OpCode;
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
    pub guild: LiveGuild,
}

#[derive(Clone, Debug)]
pub struct GuildDeleteEvent {
    pub guild: Guild,
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
    pub guild: Guild,
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

/// The "Ready" event, containing initial state
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
        let mut value = try!(into_map(value));

        let op = req!(value.get("op").and_then(|x| x.as_u64()));

        match try!(OpCode::from_num(op).ok_or(Error::Client(ClientError::InvalidOpCode))) {
            OpCode::Event => Ok(GatewayEvent::Dispatch(
                req!(try!(remove(&mut value, "s")).as_u64()),
                try!(Event::decode(
                    try!(remove(&mut value, "t").and_then(into_string)),
                    try!(remove(&mut value, "d"))
                ))
            )),
            OpCode::Heartbeat => {
                Ok(GatewayEvent::Heartbeat(req!(try!(remove(&mut value, "s"))
                    .as_u64())))
            },
            OpCode::Reconnect => Ok(GatewayEvent::Reconnect),
            OpCode::InvalidSession => Ok(GatewayEvent::InvalidateSession),
            OpCode::Hello => {
                let mut data = try!(remove(&mut value, "d").and_then(into_map));
                let interval = req!(try!(remove(&mut data, "heartbeat_interval")).as_u64());
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
    /// The first event in a connection, containing the initial state.
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
    /// A webhook for a [channel][`PublicChannel`] was updated in a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`PublicChannel`]: struct.PublicChannel.html
    WebhookUpdate(WebhookUpdateEvent),
    /// An event type not covered by the above
    Unknown(UnknownEvent),
}

impl Event {
    #[allow(cyclomatic_complexity)]
    fn decode(kind: String, value: Value) -> Result<Event> {
        if kind == "PRESENCES_REPLACE" {
            return Ok(Event::PresencesReplace(PresencesReplaceEvent {
                presences: try!(decode_array(value, Presence::decode)),
            }));
        }

        let mut value = try!(into_map(value));

        if kind == "CALL_CREATE" {
            Ok(Event::CallCreate(CallCreateEvent {
                call: try!(Call::decode(Value::Object(value))),
            }))
        } else if kind == "CALL_DELETE" {
            missing!(value, Event::CallDelete(CallDeleteEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
            }))
        } else if kind == "CALL_UPDATE" {
            missing!(value, Event::CallUpdate(CallUpdateEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                message_id: try!(remove(&mut value, "message_id").and_then(MessageId::decode)),
                region: try!(remove(&mut value, "region").and_then(into_string)),
                ringing: try!(decode_array(try!(remove(&mut value, "ringing")), UserId::decode)),
            }))
        } else if kind == "CHANNEL_CREATE" {
            Ok(Event::ChannelCreate(ChannelCreateEvent {
                channel: try!(Channel::decode(Value::Object(value))),
            }))
        } else if kind == "CHANNEL_DELETE" {
            Ok(Event::ChannelDelete(ChannelDeleteEvent {
                channel: try!(Channel::decode(Value::Object(value))),
            }))
        } else if kind == "CHANNEL_PINS_ACK" {
            missing!(value, Event::ChannelPinsAck(ChannelPinsAckEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                timestamp: try!(remove(&mut value, "timestamp").and_then(into_string)),
            }))
        } else if kind == "CHANNEL_PINS_UPDATE" {
            missing!(value, Event::ChannelPinsUpdate(ChannelPinsUpdateEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                last_pin_timestamp: try!(opt(&mut value, "last_pin_timestamp", into_string)),
            }))
        } else if kind == "CHANNEL_RECIPIENT_ADD" {
            missing!(value, Event::ChannelRecipientAdd(ChannelRecipientAddEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "CHANNEL_RECIPIENT_REMOVE" {
            missing!(value, Event::ChannelRecipientRemove(ChannelRecipientRemoveEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "CHANNEL_UPDATE" {
            Ok(Event::ChannelUpdate(ChannelUpdateEvent {
                channel: try!(Channel::decode(Value::Object(value))),
            }))
        } else if kind == "FRIEND_SUGGESTION_CREATE" {
            missing!(value, Event::FriendSuggestionCreate(FriendSuggestionCreateEvent {
                reasons: try!(decode_array(try!(remove(&mut value, "reasons")), SuggestionReason::decode)),
                suggested_user: try!(remove(&mut value, "suggested_user").and_then(User::decode)),
            }))
        } else if kind == "FRIEND_SUGGESTION_DELETE" {
            missing!(value, Event::FriendSuggestionDelete(FriendSuggestionDeleteEvent {
                suggested_user_id: try!(remove(&mut value, "suggested_user_id").and_then(UserId::decode)),
            }))
        } else if kind == "GUILD_BAN_ADD" {
            missing!(value, Event::GuildBanAdd(GuildBanAddEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "GUILD_BAN_REMOVE" {
            missing!(value, Event::GuildBanRemove(GuildBanRemoveEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "GUILD_CREATE" {
            if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(Event::GuildUnavailable(GuildUnavailableEvent {
                    guild_id: try!(remove(&mut value, "id").and_then(GuildId::decode)),
                }))
            } else {
                Ok(Event::GuildCreate(GuildCreateEvent {
                    guild: try!(LiveGuild::decode(Value::Object(value))),
                }))
            }
        } else if kind == "GUILD_DELETE" {
            if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(Event::GuildUnavailable(GuildUnavailableEvent {
                    guild_id: try!(remove(&mut value, "id").and_then(GuildId::decode)),
                }))
            } else {
                Ok(Event::GuildDelete(GuildDeleteEvent {
                    guild: try!(Guild::decode(Value::Object(value))),
                }))
            }
        } else if kind == "GUILD_EMOJIS_UPDATE" {
            missing!(value, Event::GuildEmojisUpdate(GuildEmojisUpdateEvent {
                emojis: try!(remove(&mut value, "emojis").and_then(decode_emojis)),
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
            }))
        } else if kind == "GUILD_INTEGRATIONS_UPDATE" {
            missing!(value, Event::GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
            }))
        } else if kind == "GUILD_MEMBER_ADD" {
            Ok(Event::GuildMemberAdd(GuildMemberAddEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                member: try!(Member::decode(Value::Object(value))),
            }))
        } else if kind == "GUILD_MEMBER_REMOVE" {
            missing!(value, Event::GuildMemberRemove(GuildMemberRemoveEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "GUILD_MEMBER_UPDATE" {
            missing!(value, Event::GuildMemberUpdate(GuildMemberUpdateEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                nick: try!(opt(&mut value, "nick", into_string)),
                roles: try!(decode_array(try!(remove(&mut value, "roles")), RoleId::decode)),
                user: try!(remove(&mut value, "user").and_then(User::decode)),
            }))
        } else if kind == "GUILD_MEMBERS_CHUNK" {
            missing!(value, Event::GuildMembersChunk(GuildMembersChunkEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                members: try!(remove(&mut value, "members").and_then(decode_members)),
            }))
        } else if kind == "GUILD_ROLE_CREATE" {
            missing!(value, Event::GuildRoleCreate(GuildRoleCreateEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                role: try!(remove(&mut value, "role").and_then(Role::decode)),
            }))
        } else if kind == "GUILD_ROLE_DELETE" {
            missing!(value, Event::GuildRoleDelete(GuildRoleDeleteEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                role_id: try!(remove(&mut value, "role_id").and_then(RoleId::decode)),
            }))
        } else if kind == "GUILD_ROLE_UPDATE" {
            missing!(value, Event::GuildRoleUpdate(GuildRoleUpdateEvent {
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
                role: try!(remove(&mut value, "role").and_then(Role::decode)),
            }))
        } else if kind == "GUILD_SYNC" {
            missing!(value, Event::GuildSync(GuildSyncEvent {
                guild_id: try!(remove(&mut value, "id").and_then(GuildId::decode)),
                large: req!(try!(remove(&mut value, "large")).as_bool()),
                members: try!(remove(&mut value, "members").and_then(decode_members)),
                presences: try!(remove(&mut value, "presences").and_then(decode_presences)),
            }))
        } else if kind == "GUILD_UPDATE" {
            Ok(Event::GuildUpdate(GuildUpdateEvent {
                guild: try!(Guild::decode(Value::Object(value))),
            }))
        } else if kind == "MESSAGE_ACK" {
            missing!(value, Event::MessageAck(MessageAckEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                message_id: try!(opt(&mut value, "message_id", MessageId::decode)),
            }))
        } else if kind == "MESSAGE_CREATE" {
            Ok(Event::MessageCreate(MessageCreateEvent {
                message: try!(Message::decode(Value::Object(value))),
            }))
        } else if kind == "MESSAGE_DELETE" {
            missing!(value, Event::MessageDelete(MessageDeleteEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                message_id: try!(remove(&mut value, "id").and_then(MessageId::decode)),
            }))
        } else if kind == "MESSAGE_DELETE_BULK" {
            missing!(value, Event::MessageDeleteBulk(MessageDeleteBulkEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                ids: try!(decode_array(try!(remove(&mut value, "ids")), MessageId::decode)),
            }))
        } else if kind == "MESSAGE_REACTION_ADD" {
            Ok(Event::ReactionAdd(ReactionAddEvent {
                reaction: try!(Reaction::decode(Value::Object(value)))
            }))
        } else if kind == "MESSAG_REACTION_REMOVE" {
            Ok(Event::ReactionRemove(ReactionRemoveEvent {
                reaction: try!(Reaction::decode(Value::Object(value)))
            }))
        } else if kind == "MESSAGE_REACTION_REMOVE_ALL" {
            Ok(Event::ReactionRemoveAll(ReactionRemoveAllEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                message_id: try!(remove(&mut value, "message_id").and_then(MessageId::decode)),
            }))
        } else if kind == "MESSAGE_UPDATE" {
            missing!(value, Event::MessageUpdate(MessageUpdateEvent {
                id: try!(remove(&mut value, "id").and_then(MessageId::decode)),
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                kind: try!(opt(&mut value, "type", MessageType::decode)),
                content: try!(opt(&mut value, "content", into_string)),
                nonce: remove(&mut value, "nonce").and_then(into_string).ok(),
                tts: remove(&mut value, "tts").ok().and_then(|v| v.as_bool()),
                pinned: remove(&mut value, "pinned").ok().and_then(|v| v.as_bool()),
                timestamp: try!(opt(&mut value, "timestamp", into_string)),
                edited_timestamp: try!(opt(&mut value, "edited_timestamp", into_string)),
                author: try!(opt(&mut value, "author", User::decode)),
                mention_everyone: remove(&mut value, "mention_everyone").ok().and_then(|v| v.as_bool()),
                mentions: try!(opt(&mut value, "mentions", |v| decode_array(v, User::decode))),
                mention_roles: try!(opt(&mut value, "mention_roles", |v| decode_array(v, RoleId::decode))),
                attachments: try!(opt(&mut value, "attachments", |v| decode_array(v, Attachment::decode))),
                embeds: try!(opt(&mut value, "embeds", |v| decode_array(v, Ok))),
            }))
        } else if kind == "PRESENCE_UPDATE" {
            let guild_id = try!(opt(&mut value, "guild_id", GuildId::decode));
            let roles = try!(opt(&mut value, "roles", |v| decode_array(v, RoleId::decode)));
            let presence = try!(Presence::decode(Value::Object(value)));
            Ok(Event::PresenceUpdate(PresenceUpdateEvent {
                guild_id: guild_id,
                presence: presence,
                roles: roles,
            }))
        } else if kind == "RELATIONSHIP_ADD" {
            Ok(Event::RelationshipAdd(RelationshipAddEvent {
                relationship: try!(Relationship::decode(Value::Object(value))),
            }))
        } else if kind == "RELATIONSHIP_REMOVE" {
            missing!(value, Event::RelationshipRemove(RelationshipRemoveEvent {
                kind: try!(remove(&mut value, "type").and_then(RelationshipType::decode)),
                user_id: try!(remove(&mut value, "id").and_then(UserId::decode)),
            }))
        } else if kind == "READY" {
            Ok(Event::Ready(ReadyEvent {
                ready: try!(Ready::decode(Value::Object(value))),
            }))
        } else if kind == "RESUMED" {
            missing!(value, Event::Resumed(ResumedEvent {
                heartbeat_interval: req!(try!(remove(&mut value, "heartbeat_interval")).as_u64()),
                trace: try!(remove(&mut value, "_trace").and_then(|v| decode_array(v, |v| Ok(into_string(v).ok())))),
            }))
        } else if kind == "TYPING_START" {
            missing!(value, Event::TypingStart(TypingStartEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                timestamp: req!(try!(remove(&mut value, "timestamp")).as_u64()),
                user_id: try!(remove(&mut value, "user_id").and_then(UserId::decode)),
            }))
        } else if kind == "USER_GUILD_SETTINGS_UPDATE" {
            Ok(Event::UserGuildSettingsUpdate(UserGuildSettingsUpdateEvent {
                settings: try!(UserGuildSettings::decode(Value::Object(value))),
            }))
        } else if kind == "USER_NOTE_UPDATE" {
            missing!(value, Event::UserNoteUpdate(UserNoteUpdateEvent {
                note: try!(remove(&mut value, "note").and_then(into_string)),
                user_id: try!(remove(&mut value, "id").and_then(UserId::decode)),
            }))
        } else if kind == "USER_SETTINGS_UPDATE" {
            missing!(value, Event::UserSettingsUpdate(UserSettingsUpdateEvent {
                enable_tts_command: remove(&mut value, "enable_tts_command").ok().and_then(|v| v.as_bool()),
                inline_attachment_media: remove(&mut value, "inline_attachment_media").ok().and_then(|v| v.as_bool()),
                inline_embed_media: remove(&mut value, "inline_embed_media").ok().and_then(|v| v.as_bool()),
                locale: try!(opt(&mut value, "locale", into_string)),
                message_display_compact: remove(&mut value, "message_display_compact").ok().and_then(|v| v.as_bool()),
                render_embeds: remove(&mut value, "render_embeds").ok().and_then(|v| v.as_bool()),
                show_current_game: remove(&mut value, "show_current_game").ok().and_then(|v| v.as_bool()),
                theme: try!(opt(&mut value, "theme", into_string)),
                convert_emoticons: remove(&mut value, "convert_emoticons").ok().and_then(|v| v.as_bool()),
                friend_source_flags: try!(opt(&mut value, "friend_source_flags", FriendSourceFlags::decode)),
            }))
        } else if kind == "USER_UPDATE" {
            Ok(Event::UserUpdate(UserUpdateEvent {
                current_user: try!(CurrentUser::decode(Value::Object(value))),
            }))
        } else if kind == "VOICE_SERVER_UPDATE" {
            missing!(value, Event::VoiceServerUpdate(VoiceServerUpdateEvent {
                guild_id: try!(opt(&mut value, "guild_id", GuildId::decode)),
                channel_id: try!(opt(&mut value, "channel_id", ChannelId::decode)),
                endpoint: try!(opt(&mut value, "endpoint", into_string)),
                token: try!(remove(&mut value, "token").and_then(into_string)),
            }))
        } else if kind == "VOICE_STATE_UPDATE" {
            Ok(Event::VoiceStateUpdate(VoiceStateUpdateEvent {
                guild_id: try!(opt(&mut value, "guild_id", GuildId::decode)),
                voice_state: try!(VoiceState::decode(Value::Object(value))),
            }))
        } else if kind == "WEBHOOKS_UPDATE" {
            Ok(Event::WebhookUpdate(WebhookUpdateEvent {
                channel_id: try!(remove(&mut value, "channel_id").and_then(ChannelId::decode)),
                guild_id: try!(remove(&mut value, "guild_id").and_then(GuildId::decode)),
            }))
        } else {
            Ok(Event::Unknown(UnknownEvent {
                kind: kind,
                value: value,
            }))
        }
    }
}

impl Game {
    #[cfg(feature="methods")]
    pub fn playing(name: &str) -> Game {
        Game {
            kind: GameType::Playing,
            name: name.to_owned(),
            url: None,
        }
    }

    #[cfg(feature="methods")]
    pub fn streaming(name: &str, url: &str) -> Game {
        Game {
            kind: GameType::Streaming,
            name: name.to_owned(),
            url: Some(url.to_owned()),
        }
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Option<Game>> {
        let mut map = try!(into_map(value));

        let name = match map.remove("name") {
            Some(Value::Null) | None => return Ok(None),
            Some(v) => try!(into_string(v)),
        };

        if name.trim().is_empty() {
            return Ok(None);
        }

        missing!(map, Some(Game {
            name: name,
            kind: try!(opt(&mut map, "type", GameType::decode)).unwrap_or(GameType::Playing),
            url: try!(opt(&mut map, "url", into_string)),
        }))
    }
}

impl Presence {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Presence> {
        let mut value = try!(into_map(value));
        let mut user_map = try!(remove(&mut value, "user").and_then(into_map));

        let (user_id, user) = if user_map.len() > 1 {
            let user = try!(User::decode(Value::Object(user_map)));
            (user.id, Some(user))
        } else {
            (try!(remove(&mut user_map, "id").and_then(UserId::decode)), None)
        };

        let game = match value.remove("game") {
            None | Some(Value::Null) => None,
            Some(v) => try!(Game::decode(v)),
        };

        missing!(value, Presence {
            user_id: user_id,
            status: try!(remove(&mut value, "status").and_then(OnlineStatus::decode_str)),
            last_modified: try!(opt(&mut value, "last_modified", |v| Ok(req!(v.as_u64())))),
            game: game,
            user: user,
            nick: try!(opt(&mut value, "nick", into_string)),
        })
    }
}
