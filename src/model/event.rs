//! All the events this library handles.

use serde::de::Error as DeError;
use serde_json::{self, Error as JsonError};
use std::collections::HashMap;
use super::utils::deserialize_emojis;
use super::*;
use ::constants::{OpCode, VoiceOpCode};
use ::internal::prelude::*;

#[cfg(feature="gateway")]
use ::gateway::GatewayError;

/// Event data for the channel creation event.
///
/// This is fired when:
///
/// - A [`Channel`] is created in a [`Guild`]
/// - A [`PrivateChannel`] is created
/// - The current user is added to a [`Group`]
///
/// [`Channel`]: ../enum.Channel.html
/// [`Group`]: ../struct.Group.html
/// [`Guild`]: ../struct.Guild.html
/// [`PrivateChannel`]: ../struct.PrivateChannel.html
#[derive(Clone, Debug)]
pub struct ChannelCreateEvent {
    /// The channel that was created.
    pub channel: Channel,
}

impl<'de> Deserialize<'de> for ChannelCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

impl<'de> Deserialize<'de> for ChannelDeleteEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelRecipientAddEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelRecipientRemoveEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

impl<'de> Deserialize<'de> for ChannelUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            channel: Channel::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

impl<'de> Deserialize<'de> for GuildCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: Guild::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildDeleteEvent {
    pub guild: PartialGuild,
}

impl<'de> Deserialize<'de> for GuildDeleteEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: PartialGuild::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildEmojisUpdateEvent {
    #[serde(deserialize_with="deserialize_emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildMemberAddEvent {
    pub guild_id: GuildId,
    pub member: Member,
}

impl<'de> Deserialize<'de> for GuildMemberAddEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let guild_id = map.get("guild_id")
            .ok_or_else(|| DeError::custom("missing member add guild id"))
            .and_then(|v| GuildId::deserialize(v.clone()))
            .map_err(DeError::custom)?;

        Ok(GuildMemberAddEvent {
            guild_id: guild_id,
            member: Member::deserialize(Value::Object(map)).map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize)]
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

impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = map.get("guild_id")
            .ok_or_else(|| DeError::custom("missing member chunk guild id"))
            .and_then(|v| GuildId::deserialize(v.clone()))
            .map_err(DeError::custom)?;

        let mut members = map.remove("members").ok_or_else(|| DeError::custom("missing member chunk members"))?;

        if let Some(members) = members.as_array_mut() {
            let num = Value::Number(Number::from(guild_id.0));

            for member in members {
                if let Some(map) = member.as_object_mut() {
                    map.insert("guild_id".to_owned(), num.clone());
                }
            }
        }

        let members: HashMap<UserId, Member> = Deserialize::deserialize(members)
            .map_err(DeError::custom)?;

        Ok(GuildMembersChunkEvent {
            guild_id: guild_id,
            members: members,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildUnavailableEvent {
    #[serde(rename="id")]
    pub guild_id: GuildId,
}

#[derive(Clone, Debug)]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

impl<'de> Deserialize<'de> for GuildUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            guild: PartialGuild::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MessageCreateEvent {
    pub message: Message,
}

impl<'de> Deserialize<'de> for MessageCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            message: Message::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageDeleteBulkEvent {
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MessageDeleteEvent {
    pub channel_id: ChannelId,
    #[serde(rename="id")]
    pub message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize)]
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

impl<'de> Deserialize<'de> for PresenceUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = match map.remove("guild_id") {
            Some(v) => serde_json::from_value::<Option<GuildId>>(v).map_err(DeError::custom)?,
            None => None,
        };
        let roles = match map.remove("roles") {
            Some(v) => serde_json::from_value::<Option<Vec<RoleId>>>(v).map_err(DeError::custom)?,
            None => None,
        };
        let presence = Presence::deserialize(Value::Object(map)).map_err(DeError::custom)?;

        Ok(Self {
            guild_id: guild_id,
            presence: presence,
            roles: roles,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PresencesReplaceEvent {
    pub presences: Vec<Presence>,
}

impl<'de> Deserialize<'de> for PresencesReplaceEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let presences: Vec<Presence> = Deserialize::deserialize(deserializer)?;

        Ok(Self {
            presences: presences,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

impl<'de> Deserialize<'de> for ReactionAddEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            reaction: Reaction::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReactionRemoveEvent {
    pub reaction: Reaction,
}

impl<'de> Deserialize<'de> for ReactionRemoveEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            reaction: Reaction::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ReactionRemoveAllEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

/// The "Ready" event, containing initial ready cache
#[derive(Clone, Debug)]
pub struct ReadyEvent {
    pub ready: Ready,
}

impl<'de> Deserialize<'de> for ReadyEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            ready: Ready::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResumedEvent {
    #[serde(rename="_trace")]
    pub trace: Vec<Option<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TypingStartEvent {
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub struct UnknownEvent {
    pub kind: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

impl<'de> Deserialize<'de> for UserUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            current_user: CurrentUser::deserialize(deserializer)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
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

impl<'de> Deserialize<'de> for VoiceStateUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let guild_id = match map.get("guild_id") {
            Some(v) => Some(GuildId::deserialize(v.clone()).map_err(DeError::custom)?),
            None => None,
        };

        Ok(VoiceStateUpdateEvent {
            guild_id: guild_id,
            voice_state: VoiceState::deserialize(Value::Object(map)).map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

#[allow(large_enum_variant)]
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
    #[cfg(feature="gateway")]
    pub fn decode(value: Value) -> Result<Self> {
        let mut map = JsonMap::deserialize(value)?;

        let op = map.remove("op")
            .ok_or_else(|| DeError::custom("expected gateway event op"))
            .and_then(OpCode::deserialize)?;

        Ok(match op {
            OpCode::Event => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("expected gateway event sequence"))
                    .and_then(u64::deserialize)?;
                let t = map.remove("t")
                    .ok_or_else(|| DeError::custom("expected gateway event type"))
                    .and_then(String::deserialize)?;
                let d = map.remove("d")
                    .ok_or_else(|| Error::Decode("expected gateway event d", Value::Object(map)))?;

                GatewayEvent::Dispatch(s, Event::decode(t, d)?)
            },
            OpCode::Heartbeat => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("Expected heartbeat s"))
                    .and_then(u64::deserialize)?;

                GatewayEvent::Heartbeat(s)
            },
            OpCode::Reconnect => GatewayEvent::Reconnect,
            OpCode::InvalidSession => GatewayEvent::InvalidateSession,
            OpCode::Hello => {
                let mut d = map.remove("d")
                    .ok_or_else(|| DeError::custom("expected gateway hello d"))
                    .and_then(JsonMap::deserialize)?;
                let interval = d.remove("heartbeat_interval")
                    .ok_or_else(|| DeError::custom("expected gateway hello interval"))
                    .and_then(u64::deserialize)?;

                GatewayEvent::Hello(interval)
            },
            OpCode::HeartbeatAck => GatewayEvent::HeartbeatAck,
            _ => return Err(Error::Gateway(GatewayError::InvalidOpCode)),
        })
    }
}

/// Event received over a websocket connection
#[allow(large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Event {
    /// A [`Channel`] was created.
    ///
    /// Fires the [`Client::on_channel_create`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Client::on_channel_create`]: ../../client/struct.Client.html#on_channel_create
    ChannelCreate(ChannelCreateEvent),
    /// A [`Channel`] has been deleted.
    ///
    /// Fires the [`Client::on_channel_delete`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    ChannelDelete(ChannelDeleteEvent),
    /// The pins for a [`Channel`] have been updated.
    ///
    /// Fires the [`Client::on_channel_pins_update`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Client::on_channel_pins_update`]: ../../client/struct.Client.html#on_channel_pins_update
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A [`User`] has been added to a [`Group`].
    ///
    /// Fires the [`Client::on_recipient_add`] event.
    ///
    /// [`Client::on_recipient_add`]: ../../client/struct.Client.html#on_recipient_add
    /// [`User`]: ../struct.User.html
    ChannelRecipientAdd(ChannelRecipientAddEvent),
    /// A [`User`] has been removed from a [`Group`].
    ///
    /// Fires the [`Client::on_recipient_remove`] event.
    ///
    /// [`Client::on_recipient_remove`]: ../../client/struct.Client.html#on_recipient_remove
    /// [`User`]: ../struct.User.html
    ChannelRecipientRemove(ChannelRecipientRemoveEvent),
    /// A [`Channel`] has been updated.
    ///
    /// Fires the [`Client::on_channel_update`] event.
    ///
    /// [`Client::on_channel_update`]: ../../client/struct.Client.html#on_channel_update
    /// [`User`]: ../struct.User.html
    ChannelUpdate(ChannelUpdateEvent),
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
    /// When a guild is unavailable, such as due to a Discord server outage.
    GuildUnavailable(GuildUnavailableEvent),
    GuildUpdate(GuildUpdateEvent),
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
    /// The connection has successfully resumed after a disconnect.
    Resumed(ResumedEvent),
    /// A user is typing; considered to last 5 seconds
    TypingStart(TypingStartEvent),
    /// Update to the logged-in user's information
    UserUpdate(UserUpdateEvent),
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
        Ok(match &kind[..] {
            "CHANNEL_CREATE" => Event::ChannelCreate(ChannelCreateEvent::deserialize(value)?),
            "CHANNEL_DELETE" => Event::ChannelDelete(ChannelDeleteEvent::deserialize(value)?),
            "CHANNEL_PINS_UPDATE" => Event::ChannelPinsUpdate(ChannelPinsUpdateEvent::deserialize(value)?),
            "CHANNEL_RECIPIENT_ADD" => Event::ChannelRecipientAdd(ChannelRecipientAddEvent::deserialize(value)?),
            "CHANNEL_RECIPIENT_REMOVE" => Event::ChannelRecipientRemove(ChannelRecipientRemoveEvent::deserialize(value)?),
            "CHANNEL_UPDATE" => Event::ChannelUpdate(ChannelUpdateEvent::deserialize(value)?),
            "GUILD_BAN_ADD" => Event::GuildBanAdd(GuildBanAddEvent::deserialize(value)?),
            "GUILD_BAN_REMOVE" => Event::GuildBanRemove(GuildBanRemoveEvent::deserialize(value)?),
            "GUILD_CREATE" => {
                let mut map = JsonMap::deserialize(value)?;

                if map.remove("unavailable").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::deserialize(Value::Object(map))?)
                } else {
                    Event::GuildCreate(GuildCreateEvent::deserialize(Value::Object(map))?)
                }
            },
            "GUILD_DELETE" => {
                let mut map = JsonMap::deserialize(value)?;

                if map.remove("unavailable").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::deserialize(Value::Object(map))?)
                } else {
                    Event::GuildDelete(GuildDeleteEvent::deserialize(Value::Object(map))?)
                }
            },
            "GUILD_EMOJIS_UPDATE" => Event::GuildEmojisUpdate(GuildEmojisUpdateEvent::deserialize(value)?),
            "GUILD_INTEGRATIONS_UPDATE" => Event::GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent::deserialize(value)?),
            "GUILD_MEMBER_ADD" => Event::GuildMemberAdd(GuildMemberAddEvent::deserialize(value)?),
            "GUILD_MEMBER_REMOVE" => Event::GuildMemberRemove(GuildMemberRemoveEvent::deserialize(value)?),
            "GUILD_MEMBER_UPDATE" => Event::GuildMemberUpdate(GuildMemberUpdateEvent::deserialize(value)?),
            "GUILD_MEMBERS_CHUNK" => Event::GuildMembersChunk(GuildMembersChunkEvent::deserialize(value)?),
            "GUILD_ROLE_CREATE" => Event::GuildRoleCreate(GuildRoleCreateEvent::deserialize(value)?),
            "GUILD_ROLE_DELETE" => Event::GuildRoleDelete(GuildRoleDeleteEvent::deserialize(value)?),
            "GUILD_ROLE_UPDATE" => Event::GuildRoleUpdate(GuildRoleUpdateEvent::deserialize(value)?),
            "GUILD_UPDATE" => Event::GuildUpdate(GuildUpdateEvent::deserialize(value)?),
            "MESSAGE_CREATE" => Event::MessageCreate(MessageCreateEvent::deserialize(value)?),
            "MESSAGE_DELETE" => Event::MessageDelete(MessageDeleteEvent::deserialize(value)?),
            "MESSAGE_DELETE_BULK" => Event::MessageDeleteBulk(MessageDeleteBulkEvent::deserialize(value)?),
            "MESSAGE_REACTION_ADD" => Event::ReactionAdd(ReactionAddEvent::deserialize(value)?),
            "MESSAGE_REACTION_REMOVE" => Event::ReactionRemove(ReactionRemoveEvent::deserialize(value)?),
            "MESSAGE_REACTION_REMOVE_ALL" => Event::ReactionRemoveAll(ReactionRemoveAllEvent::deserialize(value)?),
            "MESSAGE_UPDATE" => Event::MessageUpdate(MessageUpdateEvent::deserialize(value)?),
            "PRESENCE_UPDATE" => Event::PresenceUpdate(PresenceUpdateEvent::deserialize(value)?),
            "PRESENCES_REPLACE" => Event::PresencesReplace(PresencesReplaceEvent::deserialize(value)?),
            "READY" => Event::Ready(ReadyEvent::deserialize(value)?),
            "RESUMED" => Event::Resumed(ResumedEvent::deserialize(value)?),
            "TYPING_START" => Event::TypingStart(TypingStartEvent::deserialize(value)?),
            "USER_UPDATE" => Event::UserUpdate(UserUpdateEvent::deserialize(value)?),
            "VOICE_SERVER_UPDATE" => Event::VoiceServerUpdate(VoiceServerUpdateEvent::deserialize(value)?),
            "VOICE_STATE_UPDATE" => Event::VoiceStateUpdate(VoiceStateUpdateEvent::deserialize(value)?),
            "WEBHOOKS_UPDATE" => Event::WebhookUpdate(WebhookUpdateEvent::deserialize(value)?),
            _ => Event::Unknown(UnknownEvent {
                kind: kind,
                value: value,
            }),
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize)]
pub struct VoiceSessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

/// A representation of data received for [`voice`] events.
///
/// [`voice`]: ../../voice/index.html
#[derive(Clone, Debug)]
pub enum VoiceEvent {
    /// A voice heartbeat.
    Heartbeat(VoiceHeartbeat),
    /// A "hello" was received with initial voice data, such as the
    /// [`heartbeat_interval`].
    ///
    /// [`heartbeat_interval`]: struct.VoiceHello.html#structfield.heartbeat_interval
    Hello(VoiceHello),
    /// A simple keepalive event.
    KeepAlive,
    /// A voice event describing the current session.
    Ready(VoiceSessionDescription),
    /// A voice event denoting that someone is speaking.
    Speaking(VoiceSpeaking),
    /// An unknown voice event not registered.
    Unknown(VoiceOpCode, Value)
}

impl VoiceEvent {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<VoiceEvent> {
        let mut map = JsonMap::deserialize(value)?;

        let op = match map.remove("op") {
            Some(v) => VoiceOpCode::deserialize(v).map_err(JsonError::from).map_err(Error::from)?,
            None => return Err(Error::Decode("expected voice event op", Value::Object(map))),
        };

        let d = match map.remove("d") {
            Some(v) => JsonMap::deserialize(v).map_err(JsonError::from).map_err(Error::from)?,
            None => return Err(Error::Decode("expected voice gateway d", Value::Object(map))),
        };
        let v = Value::Object(d);

        Ok(match op {
            VoiceOpCode::Heartbeat => VoiceEvent::Heartbeat(VoiceHeartbeat::deserialize(v)?),
            VoiceOpCode::Hello => VoiceEvent::Hello(VoiceHello::deserialize(v)?),
            VoiceOpCode::KeepAlive => VoiceEvent::KeepAlive,
            VoiceOpCode::SessionDescription => VoiceEvent::Ready(VoiceSessionDescription::deserialize(v)?),
            VoiceOpCode::Speaking => VoiceEvent::Speaking(VoiceSpeaking::deserialize(v)?),
            other => VoiceEvent::Unknown(other, v),
        })
    }
}
