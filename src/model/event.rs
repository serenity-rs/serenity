//! All the events this library handles.

use std::collections::{BTreeMap, HashMap};
use super::utils::*;
use super::*;
use ::constants::{OpCode, VoiceOpCode};
use ::internal::prelude::*;
use ::utils::decode_array;

type Map = BTreeMap<String, Value>;

/// Event data for the call creation event.
///
/// This is fired when:
///
/// - a [`Call`] in a [`Group`] is created
/// - a [`Call`] in a [`PrivateChannel`] is created
///
/// [`Call`]: ../struct.Call.html
/// [`Group`]: ../struct.Group.html
/// [`PrivateChannel`]: ../struct.PrivateChannel.html
#[derive(Clone, Debug)]
pub struct CallCreateEvent {
    /// Information about the created call.
    pub call: Call,
}

impl CallCreateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(CallCreateEvent {
            call: Call::decode(Value::Object(map))?,
        })
    }
}

/// Event data for the call deletion event.
///
/// This is fired when:
///
/// - A [`Call`] in a [`Group`] has ended
/// - A [`Call`] in a [`PrivateChannel`] has ended
///
/// [`Call`]: ../struct.Call.html
/// [`Group`]: ../struct.Group.html
/// [`PrivateChannel`]: ../struct.PrivateChannel.html
#[derive(Clone, Debug)]
pub struct CallDeleteEvent {
    /// The Id of the [`Channel`] that the call took place in.
    ///
    /// [`Channel`]: ../enum.Channel.html
    pub channel_id: ChannelId,
}

impl CallDeleteEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(CallDeleteEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
        })
    }
}

/// Event data for the call update event.
///
/// This is fired when:
///
/// - A member of a [`Group`] has been rung
/// - The voice [`region`] of the [`Call`] has been updated
///
/// [`Call`]: ../struct.Call.html
/// [`Group`]: ../srruct.Group.html
/// [`region`]: #structfield.region
#[derive(Clone, Debug)]
pub struct CallUpdateEvent {
    /// The Id of the [channel][`Channel`] that the [call][`Call`] is taking
    /// place in.
    ///
    /// [`Call`]: ../struct.Call.html
    /// [`Channel`]: ../enum.Channel.html
    pub channel_id: ChannelId,
    /// The Id in the [channel][`Channel`] - mapped by the [`channel_id`] -
    /// denoting the beginning of the [call][`Call`].
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Call`]: ../struct.Call.html
    /// [`channel_id`]: #structfield.channel_id
    pub message_id: MessageId,
    /// The voice region that the [call][`Call`] is hosted in.
    ///
    /// [`Call`]: ../struct.Call.html
    pub region: String,
    /// A list of [user][`User`]s currently being rung who have not declined or
    /// accepted the [call][`Call`].
    ///
    /// [`Call`]: ../struct.Call.html
    /// [`User`]: ../struct.User.html
    pub ringing: Vec<UserId>,
}

impl CallUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(CallUpdateEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            message_id: remove(&mut map, "message_id").and_then(MessageId::decode)?,
            region: remove(&mut map, "region").and_then(into_string)?,
            ringing: decode_array(remove(&mut map, "ringing")?, UserId::decode)?,
        })
    }
}

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

impl ChannelCreateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ChannelCreateEvent {
            channel: Channel::decode(Value::Object(map))?,
        })
    }
}


#[derive(Clone, Debug)]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

impl ChannelDeleteEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ChannelDeleteEvent {
            channel: Channel::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelPinsAckEvent {
    pub channel_id: ChannelId,
    pub timestamp: String,
}

impl ChannelPinsAckEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ChannelPinsAckEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            timestamp: remove(&mut map, "timestamp").and_then(into_string)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<String>,
}

impl ChannelPinsUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ChannelPinsUpdateEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            last_pin_timestamp: opt(&mut map, "last_pin_timestamp", into_string)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelRecipientAddEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

impl ChannelRecipientAddEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ChannelRecipientAddEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelRecipientRemoveEvent {
    pub channel_id: ChannelId,
    pub user: User,
}

impl ChannelRecipientRemoveEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ChannelRecipientRemoveEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

impl ChannelUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ChannelUpdateEvent {
            channel: Channel::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

impl GuildBanAddEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildBanAddEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

impl GuildBanRemoveEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildBanRemoveEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

impl GuildCreateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(GuildCreateEvent {
            guild: Guild::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildDeleteEvent {
    pub guild: PartialGuild,
}

impl GuildDeleteEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(GuildDeleteEvent {
            guild: PartialGuild::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildEmojisUpdateEvent {
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

impl GuildEmojisUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildEmojisUpdateEvent {
            emojis: remove(&mut map, "emojis").and_then(decode_emojis)?,
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

impl GuildIntegrationsUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildIntegrationsUpdateEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildMemberAddEvent {
    pub guild_id: GuildId,
    pub member: Member,
}

impl GuildMemberAddEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        let guild_id = remove(&mut map, "guild_id").and_then(GuildId::decode)?;

        Ok(GuildMemberAddEvent {
            guild_id: guild_id,
            member: Member::decode_guild(guild_id, Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

impl GuildMemberRemoveEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildMemberRemoveEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

impl GuildMemberUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildMemberUpdateEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            nick: opt(&mut map, "nick", into_string)?,
            roles: decode_array(remove(&mut map, "roles")?, RoleId::decode)?,
            user: remove(&mut map, "user").and_then(User::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildMembersChunkEvent {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, Member>,
}

impl GuildMembersChunkEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        let guild_id = remove(&mut map, "guild_id").and_then(GuildId::decode)?;

        Ok(GuildMembersChunkEvent {
            guild_id: guild_id,
            members: remove(&mut map, "members").and_then(|x| decode_guild_members(guild_id, x))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildRoleCreateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

impl GuildRoleCreateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildRoleCreateEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            role: remove(&mut map, "role").and_then(Role::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

impl GuildRoleDeleteEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildRoleDeleteEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            role_id: remove(&mut map, "role_id").and_then(RoleId::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: GuildId,
    pub role: Role,
}

impl GuildRoleUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildRoleUpdateEvent {
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
            role: remove(&mut map, "role").and_then(Role::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildSyncEvent {
    pub guild_id: GuildId,
    pub large: bool,
    pub members: HashMap<UserId, Member>,
    pub presences: HashMap<UserId, Presence>,
}

impl GuildSyncEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildSyncEvent {
            guild_id: remove(&mut map, "id").and_then(GuildId::decode)?,
            large: req!(remove(&mut map, "large")?.as_bool()),
            members: remove(&mut map, "members").and_then(decode_members)?,
            presences: remove(&mut map, "presences").and_then(decode_presences)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildUnavailableEvent {
    pub guild_id: GuildId,
}

impl GuildUnavailableEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(GuildUnavailableEvent {
            guild_id: remove(&mut map, "id").and_then(GuildId::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

impl GuildUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(GuildUpdateEvent {
            guild: PartialGuild::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MessageCreateEvent {
    pub message: Message,
}

impl MessageCreateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(MessageCreateEvent {
            message: Message::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MessageDeleteBulkEvent {
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

impl MessageDeleteBulkEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(MessageDeleteBulkEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            ids: decode_array(remove(&mut map, "ids")?, MessageId::decode)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MessageDeleteEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

impl MessageDeleteEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(MessageDeleteEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            message_id: remove(&mut map, "id").and_then(MessageId::decode)?,
        })
    }
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

impl MessageUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(MessageUpdateEvent {
            id: remove(&mut map, "id").and_then(MessageId::decode)?,
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            kind: opt(&mut map, "type", MessageType::decode)?,
            content: opt(&mut map, "content", into_string)?,
            nonce: remove(&mut map, "nonce").and_then(into_string).ok(),
            tts: remove(&mut map, "tts").ok().and_then(|v| v.as_bool()),
            pinned: remove(&mut map, "pinned").ok().and_then(|v| v.as_bool()),
            timestamp: opt(&mut map, "timestamp", into_string)?,
            edited_timestamp: opt(&mut map, "edited_timestamp", into_string)?,
            author: opt(&mut map, "author", User::decode)?,
            mention_everyone: remove(&mut map, "mention_everyone").ok().and_then(|v| v.as_bool()),
            mentions: opt(&mut map, "mentions", |v| decode_array(v, User::decode))?,
            mention_roles: opt(&mut map, "mention_roles", |v| decode_array(v, RoleId::decode))?,
            attachments: opt(&mut map, "attachments", |v| decode_array(v, Attachment::decode))?,
            embeds: opt(&mut map, "embeds", |v| decode_array(v, Ok))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PresenceUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub presence: Presence,
    pub roles: Option<Vec<RoleId>>,
}

impl PresenceUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        let guild_id = opt(&mut map, "guild_id", GuildId::decode)?;
        let roles = opt(&mut map, "roles", |v| decode_array(v, RoleId::decode))?;
        let presence = Presence::decode(Value::Object(map))?;
        Ok(PresenceUpdateEvent {
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

impl PresencesReplaceEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(value: Value) -> Result<Self> {
        Ok(PresencesReplaceEvent {
            presences: decode_array(value, Presence::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

impl ReactionAddEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ReactionAddEvent {
            reaction: Reaction::decode(Value::Object(map))?
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReactionRemoveEvent {
    pub reaction: Reaction,
}

impl ReactionRemoveEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ReactionRemoveEvent {
            reaction: Reaction::decode(Value::Object(map))?
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ReactionRemoveAllEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

impl ReactionRemoveAllEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ReactionRemoveAllEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            message_id: remove(&mut map, "message_id").and_then(MessageId::decode)?,
        })
    }
}

/// The "Ready" event, containing initial ready cache
#[derive(Clone, Debug)]
pub struct ReadyEvent {
    pub ready: Ready,
}

impl ReadyEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(ReadyEvent {
            ready: Ready::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ResumedEvent {
    pub trace: Vec<Option<String>>,
}

impl ResumedEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(ResumedEvent {
            trace: remove(&mut map, "_trace").and_then(|v| decode_array(v, |v| Ok(into_string(v).ok())))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TypingStartEvent {
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

impl TypingStartEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(TypingStartEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            timestamp: req!(remove(&mut map, "timestamp")?.as_u64()),
            user_id: remove(&mut map, "user_id").and_then(UserId::decode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UnknownEvent {
    pub kind: String,
    pub value: BTreeMap<String, Value>
}

#[derive(Clone, Debug)]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

impl UserUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(map: Map) -> Result<Self> {
        Ok(UserUpdateEvent {
            current_user: CurrentUser::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}

impl VoiceServerUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceServerUpdateEvent {
            guild_id: opt(&mut map, "guild_id", GuildId::decode)?,
            channel_id: opt(&mut map, "channel_id", ChannelId::decode)?,
            endpoint: opt(&mut map, "endpoint", into_string)?,
            token: remove(&mut map, "token").and_then(into_string)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct VoiceStateUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub voice_state: VoiceState,
}

impl VoiceStateUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceStateUpdateEvent {
            guild_id: opt(&mut map, "guild_id", GuildId::decode)?,
            voice_state: VoiceState::decode(Value::Object(map))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

impl WebhookUpdateEvent {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(WebhookUpdateEvent {
            channel_id: remove(&mut map, "channel_id").and_then(ChannelId::decode)?,
            guild_id: remove(&mut map, "guild_id").and_then(GuildId::decode)?,
        })
    }
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
#[allow(large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Event {
    /// A new [`Call`] has been created.
    ///
    /// Fires the [`Client::on_call_create`] event.
    ///
    /// [`Call`]: ../struct.Call.html
    /// [`Client::on_call_create`]: ../../client/struct.Client.html#on_call_create
    CallCreate(CallCreateEvent),
    /// A [`Call`] has ended.
    ///
    /// Fires the [`Client::on_call_delete`] event.
    ///
    /// [`Call`]: ../struct.Call.html
    /// [`Client::on_call_delete`]: ../../client/struct.Client.html#method.on_call_delete
    CallDelete(CallDeleteEvent),
    /// A [`Call`] was updated.
    ///
    /// Fires the [`Client::on_call_update`] event.
    ///
    /// [`Call`]: ../struct.Call.html
    /// [`Client::on_call_update`]: ../../client/struct.Client.html#method.on_call_update
    CallUpdate(CallUpdateEvent),
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
    /// The pins for a [`Channel`] have been acked.
    ///
    /// Fires the [`Client::on_channel_pins_ack`] event.
    ///
    /// [`Channel`]: ../enum.Channel.html
    /// [`Client::on_channel_pins_ack`]: ../../client/struct.Client.html#on_channel_pins_ack
    ChannelPinsAck(ChannelPinsAckEvent),
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
    GuildSync(GuildSyncEvent),
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
        if kind == "PRESENCES_REPLACE" {
            return Ok(Event::PresencesReplace(PresencesReplaceEvent::decode(value)?));
        }

        let mut value = into_map(value)?;

        Ok(match &kind[..] {
            "CALL_CREATE" => Event::CallCreate(CallCreateEvent::decode(value)?),
            "CALL_DELETE" => Event::CallDelete(CallDeleteEvent::decode(value)?),
            "CALL_UPDATE" => Event::CallUpdate(CallUpdateEvent::decode(value)?),
            "CHANNEL_CREATE" => Event::ChannelCreate(ChannelCreateEvent::decode(value)?),
            "CHANNEL_DELETE" => Event::ChannelDelete(ChannelDeleteEvent::decode(value)?),
            "CHANNEL_PINS_ACK" => Event::ChannelPinsAck(ChannelPinsAckEvent::decode(value)?),
            "CHANNEL_PINS_UPDATE" => Event::ChannelPinsUpdate(ChannelPinsUpdateEvent::decode(value)?),
            "CHANNEL_RECIPIENT_ADD" => Event::ChannelRecipientAdd(ChannelRecipientAddEvent::decode(value)?),
            "CHANNEL_RECIPIENT_REMOVE" => Event::ChannelRecipientRemove(ChannelRecipientRemoveEvent::decode(value)?),
            "CHANNEL_UPDATE" => Event::ChannelUpdate(ChannelUpdateEvent::decode(value)?),
            "GUILD_BAN_ADD" => Event::GuildBanAdd(GuildBanAddEvent::decode(value)?),
            "GUILD_BAN_REMOVE" => Event::GuildBanRemove(GuildBanRemoveEvent::decode(value)?),
            "GUILD_CREATE" => {
                if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::decode(value)?)
                } else {
                    Event::GuildCreate(GuildCreateEvent::decode(value)?)
                }
            },
            "GUILD_DELETE" => {
                if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
                    Event::GuildUnavailable(GuildUnavailableEvent::decode(value)?)
                } else {
                    Event::GuildDelete(GuildDeleteEvent::decode(value)?)
                }
            },
            "GUILD_EMOJIS_UPDATE" => Event::GuildEmojisUpdate(GuildEmojisUpdateEvent::decode(value)?),
            "GUILD_INTEGRATIONS_UPDATE" => Event::GuildIntegrationsUpdate(GuildIntegrationsUpdateEvent::decode(value)?),
            "GUILD_MEMBER_ADD" => Event::GuildMemberAdd(GuildMemberAddEvent::decode(value)?),
            "GUILD_MEMBER_REMOVE" => Event::GuildMemberRemove(GuildMemberRemoveEvent::decode(value)?),
            "GUILD_MEMBER_UPDATE" => Event::GuildMemberUpdate(GuildMemberUpdateEvent::decode(value)?),
            "GUILD_MEMBERS_CHUNK" => Event::GuildMembersChunk(GuildMembersChunkEvent::decode(value)?),
            "GUILD_ROLE_CREATE" => Event::GuildRoleCreate(GuildRoleCreateEvent::decode(value)?),
            "GUILD_ROLE_DELETE" => Event::GuildRoleDelete(GuildRoleDeleteEvent::decode(value)?),
            "GUILD_ROLE_UPDATE" => Event::GuildRoleUpdate(GuildRoleUpdateEvent::decode(value)?),
            "GUILD_SYNC" => Event::GuildSync(GuildSyncEvent::decode(value)?),
            "GUILD_UPDATE" => Event::GuildUpdate(GuildUpdateEvent::decode(value)?),
            "MESSAGE_CREATE" => Event::MessageCreate(MessageCreateEvent::decode(value)?),
            "MESSAGE_DELETE" => Event::MessageDelete(MessageDeleteEvent::decode(value)?),
            "MESSAGE_DELETE_BULK" => Event::MessageDeleteBulk(MessageDeleteBulkEvent::decode(value)?),
            "MESSAGE_REACTION_ADD" => Event::ReactionAdd(ReactionAddEvent::decode(value)?),
            "MESSAGE_REACTION_REMOVE" => Event::ReactionRemove(ReactionRemoveEvent::decode(value)?),
            "MESSAGE_REACTION_REMOVE_ALL" => Event::ReactionRemoveAll(ReactionRemoveAllEvent::decode(value)?),
            "MESSAGE_UPDATE" => Event::MessageUpdate(MessageUpdateEvent::decode(value)?),
            "PRESENCE_UPDATE" => Event::PresenceUpdate(PresenceUpdateEvent::decode(value)?),
            "READY" => Event::Ready(ReadyEvent::decode(value)?),
            "RESUMED" => Event::Resumed(ResumedEvent::decode(value)?),
            "TYPING_START" => Event::TypingStart(TypingStartEvent::decode(value)?),
            "USER_UPDATE" => Event::UserUpdate(UserUpdateEvent::decode(value)?),
            "VOICE_SERVER_UPDATE" => Event::VoiceServerUpdate(VoiceServerUpdateEvent::decode(value)?),
            "VOICE_STATE_UPDATE" => Event::VoiceStateUpdate(VoiceStateUpdateEvent::decode(value)?),
            "WEBHOOKS_UPDATE" => Event::WebhookUpdate(WebhookUpdateEvent::decode(value)?),
            _ => Event::Unknown(UnknownEvent {
                kind: kind,
                value: value,
            }),
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

impl VoiceHeartbeat {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceHeartbeat {
            heartbeat_interval: req!(remove(&mut map, "heartbeat_interval")?.as_u64()),
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

impl VoiceHello {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceHello {
            heartbeat_interval: req!(remove(&mut map, "heartbeat_interval")?.as_u64()),
            ip: remove(&mut map, "ip").and_then(into_string)?,
            modes: decode_array(remove(&mut map, "modes")?, into_string)?,
            port: req!(remove(&mut map, "port")?.as_u64()) as u16,
            ssrc: req!(remove(&mut map, "ssrc")?.as_u64()) as u32,
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct VoiceSessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

impl VoiceSessionDescription {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceSessionDescription {
            mode: remove(&mut map, "mode")
                .and_then(into_string)?,
            secret_key: decode_array(remove(&mut map, "secret_key")?,
                |v| Ok(req!(v.as_u64()) as u8)
            )?,
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

impl VoiceSpeaking {
    #[doc(hidden)]
    #[inline]
    pub fn decode(mut map: Map) -> Result<Self> {
        Ok(VoiceSpeaking {
            speaking: req!(remove(&mut map, "speaking")?.as_bool()),
            ssrc: req!(remove(&mut map, "ssrc")?.as_u64()) as u32,
            user_id: remove(&mut map, "user_id").and_then(UserId::decode)?,
        })
    }
}

/// A representation of data received for [`voice`] events.
///
/// [`voice`]: ../../ext/voice/index.html
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
        let mut value = into_map(value)?;
        let op = req!(remove(&mut value, "op")?.as_u64());
        let map = remove(&mut value, "d").and_then(into_map)?;

        let opcode = VoiceOpCode::from_num(op)
            .ok_or(Error::Client(ClientError::InvalidOpCode))?;

        Ok(match opcode {
            VoiceOpCode::Heartbeat => VoiceEvent::Heartbeat(VoiceHeartbeat::decode(map)?),
            VoiceOpCode::Hello => VoiceEvent::Hello(VoiceHello::decode(map)?),
            VoiceOpCode::KeepAlive => VoiceEvent::KeepAlive,
            VoiceOpCode::SessionDescription => VoiceEvent::Ready(VoiceSessionDescription::decode(map)?),
            VoiceOpCode::Speaking => VoiceEvent::Speaking(VoiceSpeaking::decode(map)?),
            other => VoiceEvent::Unknown(other, Value::Object(map)),
        })
    }
}
