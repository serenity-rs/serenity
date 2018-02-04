//! All the events this library handles.

use chrono::{DateTime, FixedOffset};
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_json;
use std::collections::HashMap;
use super::utils::deserialize_emojis;
use super::prelude::*;
use constants::{OpCode, VoiceOpCode};
use internal::prelude::*;

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

impl Serialize for ChannelCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Channel::serialize(&self.channel, serializer)
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

impl Serialize for ChannelDeleteEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Channel::serialize(&self.channel, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelRecipientAddEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Serialize for ChannelUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Channel::serialize(&self.channel, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Serialize for GuildCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Guild::serialize(&self.guild, serializer)
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

impl Serialize for GuildDeleteEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        PartialGuild::serialize(&self.guild, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildEmojisUpdateEvent {
    #[serde(deserialize_with = "deserialize_emojis")] pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

#[derive(Clone, Debug, Serialize)]
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
            member: Member::deserialize(Value::Object(map))
                .map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

#[derive(Clone, Debug, Serialize)]
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

        let mut members = map.remove("members")
            .ok_or_else(|| DeError::custom("missing member chunk members"))?;

        if let Some(members) = members.as_array_mut() {
            let num = Value::Number(Number::from(guild_id.0));

            for member in members {
                if let Some(map) = member.as_object_mut() {
                    map.insert("guild_id".to_string(), num.clone());
                }
            }
        }

        let members: HashMap<UserId, Member> =
            Deserialize::deserialize(members).map_err(DeError::custom)?;

        Ok(GuildMembersChunkEvent {
            guild_id: guild_id,
            members: members,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildUnavailableEvent {
    #[serde(rename = "id")] pub guild_id: GuildId,
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

impl Serialize for GuildUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        PartialGuild::serialize(&self.guild, serializer)
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

impl Serialize for MessageCreateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Message::serialize(&self.message, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageDeleteBulkEvent {
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MessageDeleteEvent {
    pub channel_id: ChannelId,
    #[serde(rename = "id")] pub message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub kind: Option<MessageType>,
    pub content: Option<String>,
    pub nonce: Option<String>,
    pub tts: Option<bool>,
    pub pinned: Option<bool>,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub edited_timestamp: Option<DateTime<FixedOffset>>,
    pub author: Option<User>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Embed>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PresenceUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub presence: Presence,
    pub roles: Option<Vec<RoleId>>,
}

impl<'de> Deserialize<'de> for PresenceUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = match map.remove("guild_id") {
            Some(v) => serde_json::from_value::<Option<GuildId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let roles = match map.remove("roles") {
            Some(v) => serde_json::from_value::<Option<Vec<RoleId>>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let presence = Presence::deserialize(Value::Object(map))
            .map_err(DeError::custom)?;

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

impl Serialize for PresencesReplaceEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.presences.len()))?;

        for value in &self.presences {
            seq.serialize_element(value)?;
        }

        seq.end()
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

impl Serialize for ReactionAddEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Reaction::serialize(&self.reaction, serializer)
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

impl Serialize for ReactionRemoveEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Reaction::serialize(&self.reaction, serializer)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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

impl Serialize for ReadyEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        Ready::serialize(&self.ready, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResumedEvent {
    #[serde(rename = "_trace")] pub trace: Vec<Option<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypingStartEvent {
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Serialize for UserUpdateEvent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        CurrentUser::serialize(&self.current_user, serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}

#[derive(Clone, Debug, Serialize)]
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
            voice_state: VoiceState::deserialize(Value::Object(map))
                .map_err(DeError::custom)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

#[allow(large_enum_variant)]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum GatewayEvent {
    Dispatch(u64, Event),
    Heartbeat(u64),
    Reconnect,
    /// Whether the session can be resumed.
    InvalidateSession(bool),
    Hello(u64),
    HeartbeatAck,
}

impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D)
        -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let op = map.remove("op")
            .ok_or_else(|| DeError::custom("expected op"))
            .and_then(OpCode::deserialize)
            .map_err(DeError::custom)?;

        Ok(match op {
            OpCode::Event => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("expected gateway event sequence"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;
                let kind = map.remove("t")
                    .ok_or_else(|| DeError::custom("expected gateway event type"))
                    .and_then(EventType::deserialize)
                    .map_err(DeError::custom)?;
                let payload = map.remove("d").ok_or_else(|| {
                    Error::Decode("expected gateway event d", Value::Object(map))
                }).map_err(DeError::custom)?;

                let x = deserialize_event_with_type(kind, payload)
                    .map_err(DeError::custom)?;

                GatewayEvent::Dispatch(s, x)
            },
            OpCode::Heartbeat => {
                let s = map.remove("s")
                    .ok_or_else(|| DeError::custom("Expected heartbeat s"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::Heartbeat(s)
            },
            OpCode::Reconnect => GatewayEvent::Reconnect,
            OpCode::InvalidSession => {
                let resumable = map.remove("d")
                    .ok_or_else(|| {
                        DeError::custom("expected gateway invalid session d")
                    })
                    .and_then(bool::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::InvalidateSession(resumable)
            },
            OpCode::Hello => {
                let mut d = map.remove("d")
                    .ok_or_else(|| DeError::custom("expected gateway hello d"))
                    .and_then(JsonMap::deserialize)
                    .map_err(DeError::custom)?;
                let interval = d.remove("heartbeat_interval")
                    .ok_or_else(|| DeError::custom("expected gateway hello interval"))
                    .and_then(u64::deserialize)
                    .map_err(DeError::custom)?;

                GatewayEvent::Hello(interval)
            },
            OpCode::HeartbeatAck => GatewayEvent::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}

/// Event received over a websocket connection
#[allow(large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// [`Client::on_channel_pins_update`]:
    /// ../../client/struct.Client.html#on_channel_pins_update
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
    /// [`on_message_reaction_remove`]:
    /// ../client/struct.Client.html#method.on_message_reaction_remove
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

/// Deserializes a `serde_json::Value` into an `Event`.
///
/// The given `EventType` is used to determine what event to deserialize into.
/// For example, an [`EventType::ChannelCreate`] will cause the given value to
/// attempt to be deserialized into a [`ChannelCreateEvent`].
///
/// Special handling is done in regards to [`EventType::GuildCreate`] and
/// [`EventType::GuildDelete`]: they check for an `"unavailable"` key and, if
/// present and containing a value of `true`, will cause a
/// [`GuildUnavailableEvent`] to be returned. Otherwise, all other event types
/// correlate to the deserialization of their appropriate event.
///
/// [`EventType::ChannelCreate`]: enum.EventType.html#variant.ChannelCreate
/// [`EventType::GuildCreate`]: enum.EventType.html#variant.GuildCreate
/// [`EventType::GuildDelete`]: enum.EventType.html#variant.GuildDelete
/// [`ChannelCreateEvent`]: struct.ChannelCreateEvent.html
/// [`GuildUnavailableEvent`]: struct.GuildUnavailableEvent.html
pub fn deserialize_event_with_type(kind: EventType, v: Value) -> Result<Event> {
    Ok(match kind {
        EventType::ChannelCreate => Event::ChannelCreate(serde_json::from_value(v)?),
        EventType::ChannelDelete => Event::ChannelDelete(serde_json::from_value(v)?),
        EventType::ChannelPinsUpdate => {
            Event::ChannelPinsUpdate(serde_json::from_value(v)?)
        },
        EventType::ChannelRecipientAdd => {
            Event::ChannelRecipientAdd(serde_json::from_value(v)?)
        },
        EventType::ChannelRecipientRemove => {
            Event::ChannelRecipientRemove(serde_json::from_value(v)?)
        },
        EventType::ChannelUpdate => Event::ChannelUpdate(serde_json::from_value(v)?),
        EventType::GuildBanAdd => Event::GuildBanAdd(serde_json::from_value(v)?),
        EventType::GuildBanRemove => Event::GuildBanRemove(serde_json::from_value(v)?),
        EventType::GuildCreate | EventType::GuildUnavailable => {
            // GuildUnavailable isn't actually received from the gateway, so it
            // can be lumped in with GuildCreate's arm.

            let mut map = JsonMap::deserialize(v)?;

            if map.remove("unavailable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false) {
                let guild_data = serde_json::from_value(Value::Object(map))?;

                Event::GuildUnavailable(guild_data)
            } else {
                Event::GuildCreate(serde_json::from_value(Value::Object(map))?)
            }
        },
        EventType::GuildDelete => {
            let mut map = JsonMap::deserialize(v)?;

            if map.remove("unavailable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false) {
                let guild_data = serde_json::from_value(Value::Object(map))?;

                Event::GuildUnavailable(guild_data)
            } else {
                Event::GuildDelete(serde_json::from_value(Value::Object(map))?)
            }
        },
        EventType::GuildEmojisUpdate => {
            Event::GuildEmojisUpdate(serde_json::from_value(v)?)
        },
        EventType::GuildIntegrationsUpdate => {
            Event::GuildIntegrationsUpdate(serde_json::from_value(v)?)
        },
        EventType::GuildMemberAdd => Event::GuildMemberAdd(serde_json::from_value(v)?),
        EventType::GuildMemberRemove => {
            Event::GuildMemberRemove(serde_json::from_value(v)?)
        },
        EventType::GuildMemberUpdate => {
            Event::GuildMemberUpdate(serde_json::from_value(v)?)
        },
        EventType::GuildMembersChunk => {
            Event::GuildMembersChunk(serde_json::from_value(v)?)
        },
        EventType::GuildRoleCreate => {
            Event::GuildRoleCreate(serde_json::from_value(v)?)
        },
        EventType::GuildRoleDelete => {
            Event::GuildRoleDelete(serde_json::from_value(v)?)
        },
        EventType::GuildRoleUpdate => {
            Event::GuildRoleUpdate(serde_json::from_value(v)?)
        },
        EventType::GuildUpdate => Event::GuildUpdate(serde_json::from_value(v)?),
        EventType::MessageCreate => Event::MessageCreate(serde_json::from_value(v)?),
        EventType::MessageDelete => Event::MessageDelete(serde_json::from_value(v)?),
        EventType::MessageDeleteBulk => {
            Event::MessageDeleteBulk(serde_json::from_value(v)?)
        },
        EventType::ReactionAdd => {
            Event::ReactionAdd(serde_json::from_value(v)?)
        },
        EventType::ReactionRemove => {
            Event::ReactionRemove(serde_json::from_value(v)?)
        },
        EventType::ReactionRemoveAll => {
            Event::ReactionRemoveAll(serde_json::from_value(v)?)
        },
        EventType::MessageUpdate => Event::MessageUpdate(serde_json::from_value(v)?),
        EventType::PresenceUpdate => Event::PresenceUpdate(serde_json::from_value(v)?),
        EventType::PresencesReplace => {
            Event::PresencesReplace(serde_json::from_value(v)?)
        },
        EventType::Ready => Event::Ready(serde_json::from_value(v)?),
        EventType::Resumed => Event::Resumed(serde_json::from_value(v)?),
        EventType::TypingStart => Event::TypingStart(serde_json::from_value(v)?),
        EventType::UserUpdate => Event::UserUpdate(serde_json::from_value(v)?),
        EventType::VoiceServerUpdate => {
            Event::VoiceServerUpdate(serde_json::from_value(v)?)
        },
        EventType::VoiceStateUpdate => {
            Event::VoiceStateUpdate(serde_json::from_value(v)?)
        },
        EventType::WebhookUpdate => Event::WebhookUpdate(serde_json::from_value(v)?),
        EventType::Other(kind) => Event::Unknown(UnknownEvent {
            kind: kind.to_owned(),
            value: v,
        }),
    })
}

/// The type of event dispatch received from the gateway.
///
/// This is useful for deciding how to deserialize a received payload.
///
/// A Deserialization implementation is provided for deserializing raw event
/// dispatch type strings to this enum, e.g. deserializing `"CHANNEL_CREATE"` to
/// [`EventType::ChannelCreate`].
///
/// [`EventType::ChannelCreate`]: enum.EventType.html#variant.ChannelCreate
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum EventType {
    /// Indicator that a channel create payload was received.
    ///
    /// This maps to [`ChannelCreateEvent`].
    ///
    /// [`ChannelCreateEvent`]: struct.ChannelCreateEvent.html
    ChannelCreate,
    /// Indicator that a channel delete payload was received.
    ///
    /// This maps to [`ChannelDeleteEvent`].
    ///
    /// [`ChannelDeleteEvent`]: struct.ChannelDeleteEvent.html
    ChannelDelete,
    /// Indicator that a channel pins update payload was received.
    ///
    /// This maps to [`ChannelPinsUpdateEvent`].
    ///
    /// [`ChannelPinsUpdateEvent`]: struct.ChannelPinsUpdateEvent.html
    ChannelPinsUpdate,
    /// Indicator that a channel recipient addition payload was received.
    ///
    /// This maps to [`ChannelRecipientAddEvent`].
    ///
    /// [`ChannelRecipientAddEvent`]: struct.ChannelRecipientAddEvent.html
    ChannelRecipientAdd,
    /// Indicator that a channel recipient removal payload was received.
    ///
    /// This maps to [`ChannelRecipientRemoveEvent`].
    ///
    /// [`ChannelRecipientRemoveEvent`]: struct.ChannelRecipientRemoveEvent.html
    ChannelRecipientRemove,
    /// Indicator that a channel update payload was received.
    ///
    /// This maps to [`ChannelUpdateEvent`].
    ///
    /// [`ChannelUpdateEvent`]: struct.ChannelUpdateEvent.html
    ChannelUpdate,
    /// Indicator that a guild ban addition payload was received.
    ///
    /// This maps to [`GuildBanAddEvent`].
    ///
    /// [`GuildBanAddEvent`]: struct.GuildBanAddEvent.html
    GuildBanAdd,
    /// Indicator that a guild ban removal payload was received.
    ///
    /// This maps to [`GuildBanRemoveEvent`].
    ///
    /// [`GuildBanRemoveEvent`]: struct.GuildBanRemoveEvent.html
    GuildBanRemove,
    /// Indicator that a guild create payload was received.
    ///
    /// This maps to [`GuildCreateEvent`].
    ///
    /// [`GuildCreateEvent`]: struct.GuildCreateEvent.html
    GuildCreate,
    /// Indicator that a guild delete payload was received.
    ///
    /// This maps to [`GuildDeleteEvent`].
    ///
    /// [`GuildDeleteEvent`]: struct.GuildDeleteEvent.html
    GuildDelete,
    /// Indicator that a guild emojis update payload was received.
    ///
    /// This maps to [`GuildEmojisUpdateEvent`].
    ///
    /// [`GuildEmojisUpdateEvent`]: struct.GuildEmojisUpdateEvent.html
    GuildEmojisUpdate,
    /// Indicator that a guild integrations update payload was received.
    ///
    /// This maps to [`GuildIntegrationsUpdateEvent`].
    ///
    /// [`GuildIntegrationsUpdateEvent`]: struct.GuildIntegrationsUpdateEvent.html
    GuildIntegrationsUpdate,
    /// Indicator that a guild member add payload was received.
    ///
    /// This maps to [`GuildMemberAddEvent`].
    ///
    /// [`GuildMemberAddEvent`]: struct.GuildMemberAddEvent.html
    GuildMemberAdd,
    /// Indicator that a guild member remove payload was received.
    ///
    /// This maps to [`GuildMemberRemoveEvent`].
    ///
    /// [`GuildMemberRemoveEvent`]: struct.GuildMemberRemoveEvent.html
    GuildMemberRemove,
    /// Indicator that a guild member update payload was received.
    ///
    /// This maps to [`GuildMemberUpdateEvent`].
    ///
    /// [`GuildMemberUpdateEvent`]: struct.GuildMemberUpdateEvent.html
    GuildMemberUpdate,
    /// Indicator that a guild members chunk payload was received.
    ///
    /// This maps to [`GuildMembersChunkEvent`].
    ///
    /// [`GuildMembersChunkEvent`]: struct.GuildMembersChunkEvent.html
    GuildMembersChunk,
    /// Indicator that a guild role create payload was received.
    ///
    /// This maps to [`GuildRoleCreateEvent`].
    ///
    /// [`GuildRoleCreateEvent`]: struct.GuildRoleCreateEvent.html
    GuildRoleCreate,
    /// Indicator that a guild role delete payload was received.
    ///
    /// This maps to [`GuildRoleDeleteEvent`].
    ///
    /// [`GuildRoleDeleteEvent`]: struct.GuildRoleDeleteEvent.html
    GuildRoleDelete,
    /// Indicator that a guild role update payload was received.
    ///
    /// This maps to [`GuildRoleUpdateEvent`].
    ///
    /// [`GuildRoleUpdateEvent`]: struct.GuildRoleUpdateEvent.html
    GuildRoleUpdate,
    /// Indicator that a guild unavailable payload was received.
    ///
    /// This maps to [`GuildUnavailableEvent`].
    ///
    /// [`GuildUnavailableEvent`]: struct.GuildUnavailableEvent.html
    GuildUnavailable,
    /// Indicator that a guild update payload was received.
    ///
    /// This maps to [`GuildUpdateEvent`].
    ///
    /// [`GuildUpdateEvent`]: struct.GuildUpdateEvent.html
    GuildUpdate,
    /// Indicator that a message create payload was received.
    ///
    /// This maps to [`MessageCreateEvent`].
    ///
    /// [`MessageCreateEvent`]: struct.MessageCreateEvent.html
    MessageCreate,
    /// Indicator that a message delete payload was received.
    ///
    /// This maps to [`MessageDeleteEvent`].
    ///
    /// [`MessageDeleteEvent`]: struct.MessageDeleteEvent.html
    MessageDelete,
    /// Indicator that a message delete bulk payload was received.
    ///
    /// This maps to [`MessageDeleteBulkEvent`].
    ///
    /// [`MessageDeleteBulkEvent`]: struct.MessageDeleteBulkEvent.html
    MessageDeleteBulk,
    /// Indicator that a message update payload was received.
    ///
    /// This maps to [`MessageUpdateEvent`].
    ///
    /// [`MessageUpdateEvent`]: struct.MessageUpdateEvent.html
    MessageUpdate,
    /// Indicator that a presence update payload was received.
    ///
    /// This maps to [`PresenceUpdateEvent`].
    ///
    /// [`PresenceUpdateEvent`]: struct.PresenceUpdateEvent.html
    PresenceUpdate,
    /// Indicator that a presences replace payload was received.
    ///
    /// This maps to [`PresencesReplaceEvent`].
    ///
    /// [`PresencesReplaceEvent`]: struct.PresencesReplaceEvent.html
    PresencesReplace,
    /// Indicator that a reaction add payload was received.
    ///
    /// This maps to [`ReactionAddEvent`].
    ///
    /// [`ReactionAddEvent`]: struct.ReactionAddEvent.html
    ReactionAdd,
    /// Indicator that a reaction remove payload was received.
    ///
    /// This maps to [`ReactionRemoveEvent`].
    ///
    /// [`ReactionRemoveEvent`]: struct.ResumedEvent.html
    ReactionRemove,
    /// Indicator that a reaction remove all payload was received.
    ///
    /// This maps to [`ReactionRemoveAllEvent`].
    ///
    /// [`ReactionRemoveAllEvent`]: struct.ReactionRemoveAllEvent.html
    ReactionRemoveAll,
    /// Indicator that a ready payload was received.
    ///
    /// This maps to [`ReadyEvent`].
    ///
    /// [`ReadyEvent`]: struct.ReadyEvent.html
    Ready,
    /// Indicator that a resumed payload was received.
    ///
    /// This maps to [`ResumedEvent`].
    ///
    /// [`ResumedEvent`]: struct.ResumedEvent.html
    Resumed,
    /// Indicator that a typing start payload was received.
    ///
    /// This maps to [`TypingStartEvent`].
    ///
    /// [`TypingStartEvent`]: struct.TypingStartEvent.html
    TypingStart,
    /// Indicator that a user update payload was received.
    ///
    /// This maps to [`UserUpdateEvent`].
    ///
    /// [`UserUpdateEvent`]: struct.UserUpdateEvent.html
    UserUpdate,
    /// Indicator that a voice state payload was received.
    ///
    /// This maps to [`VoiceStateUpdateEvent`].
    ///
    /// [`VoiceStateUpdateEvent`]: struct.VoiceStateUpdateEvent.html
    VoiceStateUpdate,
    /// Indicator that a voice server update payload was received.
    ///
    /// This maps to [`VoiceServerUpdateEvent`].
    ///
    /// [`VoiceServerUpdateEvent`]: struct.VoiceServerUpdateEvent.html
    VoiceServerUpdate,
    /// Indicator that a webhook update payload was received.
    ///
    /// This maps to [`WebhookUpdateEvent`].
    ///
    /// [`WebhookUpdateEvent`]: struct.WebhookUpdateEvent.html
    WebhookUpdate,
    /// An unknown event was received over the gateway.
    ///
    /// This should be logged so that support for it can be added in the
    /// library.
    Other(String),
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
        where D: Deserializer<'de> {
        struct EventTypeVisitor;

        impl<'de> Visitor<'de> for EventTypeVisitor {
            type Value = EventType;

            fn expecting(&self, f: &mut Formatter) -> FmtResult {
                f.write_str("event type str")
            }

            fn visit_str<E>(self, v: &str) -> StdResult<Self::Value, E>
                where E: DeError {
                Ok(match v {
                    "CHANNEL_CREATE" => EventType::ChannelCreate,
                    "CHANNEL_DELETE" => EventType::ChannelDelete,
                    "CHANNEL_PINS_UPDATE" => EventType::ChannelPinsUpdate,
                    "CHANNEL_RECIPIENT_ADD" => EventType::ChannelRecipientAdd,
                    "CHANNEL_RECIPIENT_REMOVE" => EventType::ChannelRecipientRemove,
                    "CHANNEL_UPDATE" => EventType::ChannelUpdate,
                    "GUILD_BAN_ADD" => EventType::GuildBanAdd,
                    "GUILD_BAN_REMOVE" => EventType::GuildBanRemove,
                    "GUILD_CREATE" => EventType::GuildCreate,
                    "GUILD_DELETE" => EventType::GuildDelete,
                    "GUILD_EMOJIS_UPDATE" => EventType::GuildEmojisUpdate,
                    "GUILD_INTEGRATIONS_UPDATE" => EventType::GuildIntegrationsUpdate,
                    "GUILD_MEMBER_ADD" => EventType::GuildMemberAdd,
                    "GUILD_MEMBER_REMOVE" => EventType::GuildMemberRemove,
                    "GUILD_MEMBER_UPDATE" => EventType::GuildMemberUpdate,
                    "GUILD_MEMBERS_CHUNK" => EventType::GuildMembersChunk,
                    "GUILD_ROLE_CREATE" => EventType::GuildRoleCreate,
                    "GUILD_ROLE_DELETE" => EventType::GuildRoleDelete,
                    "GUILD_ROLE_UPDATE" => EventType::GuildRoleUpdate,
                    "GUILD_UPDATE" => EventType::GuildUpdate,
                    "MESSAGE_CREATE" => EventType::MessageCreate,
                    "MESSAGE_DELETE" => EventType::MessageDelete,
                    "MESSAGE_DELETE_BULK" => EventType::MessageDeleteBulk,
                    "MESSAGE_REACTION_ADD" => EventType::ReactionAdd,
                    "MESSAGE_REACTION_REMOVE" => EventType::ReactionRemove,
                    "MESSAGE_REACTION_REMOVE_ALL" => EventType::ReactionRemoveAll,
                    "MESSAGE_UPDATE" => EventType::MessageUpdate,
                    "PRESENCE_UPDATE" => EventType::PresenceUpdate,
                    "PRESENCES_REPLACE" => EventType::PresencesReplace,
                    "READY" => EventType::Ready,
                    "RESUMED" => EventType::Resumed,
                    "TYPING_START" => EventType::TypingStart,
                    "USER_UPDATE" => EventType::UserUpdate,
                    "VOICE_SERVER_UPDATE" => EventType::VoiceServerUpdate,
                    "VOICE_STATE_UPDATE" => EventType::VoiceStateUpdate,
                    "WEBHOOKS_UPDATE" => EventType::WebhookUpdate,
                    other => EventType::Other(other.to_owned()),
                })
            }
        }

        deserializer.deserialize_str(EventTypeVisitor)
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceSessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

/// A representation of data received for [`voice`] events.
///
/// [`voice`]: ../../voice/index.html
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
    Unknown(VoiceOpCode, Value),
}

impl<'de> Deserialize<'de> for VoiceEvent {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
        where D: Deserializer<'de> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let op = map.remove("op")
            .ok_or_else(|| DeError::custom("expected voice event op"))
            .and_then(VoiceOpCode::deserialize)
            .map_err(DeError::custom)?;

        let v = map.remove("d")
            .ok_or_else(|| DeError::custom("expected voice gateway payload"))
            .and_then(Value::deserialize)
            .map_err(DeError::custom)?;

        Ok(match op {
            VoiceOpCode::Heartbeat => {
                let v = serde_json::from_value(v).map_err(DeError::custom)?;

                VoiceEvent::Heartbeat(v)
            },
            VoiceOpCode::Hello => {
                let v = VoiceHello::deserialize(v).map_err(DeError::custom)?;

                VoiceEvent::Hello(v)
            },
            VoiceOpCode::KeepAlive => VoiceEvent::KeepAlive,
            VoiceOpCode::SessionDescription => {
                let v = VoiceSessionDescription::deserialize(v)
                    .map_err(DeError::custom)?;

                VoiceEvent::Ready(v)
            },
            VoiceOpCode::Speaking => {
                let v = VoiceSpeaking::deserialize(v).map_err(DeError::custom)?;

                VoiceEvent::Speaking(v)
            },
            other => VoiceEvent::Unknown(other, v),
        })
    }
}
