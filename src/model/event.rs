//! All the events this library handles.
//!
//! Every event includes the gateway intent required to receive it, as well as a link to the Discord
//! documentation for the event.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use serde::de::{Error as DeError, IgnoredAny, MapAccess};

use super::application::component::ActionRow;
use super::prelude::*;
use super::utils::{
    add_guild_id_to_map,
    deserialize_val,
    emojis,
    ignore_input,
    remove_from_map,
    remove_from_map_opt,
    roles,
    stickers,
};
use crate::constants::Opcode;
use crate::internal::prelude::*;
use crate::model::application::command::CommandPermission;
use crate::model::application::interaction::Interaction;
use crate::model::guild::automod::{ActionExecution, Rule};

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#application-command-permissions-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct CommandPermissionsUpdateEvent {
    pub permission: CommandPermission,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#auto-moderation-rule-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleCreateEvent {
    pub rule: Rule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#auto-moderation-rule-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleUpdateEvent {
    pub rule: Rule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#auto-moderation-rule-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleDeleteEvent {
    pub rule: Rule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_EXECUTION`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#auto-moderation-action-execution).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModActionExecutionEvent {
    pub execution: ActionExecution,
}

/// Event data for the channel creation event.
///
/// This is fired when:
///
/// - A [`Channel`] is created in a [`Guild`]
/// - A [`PrivateChannel`] is created
///
/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#channel-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelCreateEvent {
    /// The channel that was created.
    pub channel: Channel,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#channel-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelDeleteEvent {
    pub channel: Channel,
}

/// Requires [`GatewayIntents::GUILDS`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#channel-pins-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelPinsUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<Timestamp>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#channel-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelUpdateEvent {
    pub channel: Channel,
}

/// Requires [`GatewayIntents::GUILD_BANS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-ban-add).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILD_BANS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-ban-remove).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-create).
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildCreateEvent {
    pub guild: Guild,
}

impl<'de> Deserialize<'de> for GuildCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id_val = map.get("id").ok_or_else(|| DeError::missing_field("id"))?;
        let id = deserialize_val(id_val.clone())?;

        add_guild_id_to_map(&mut map, "channels", id);
        add_guild_id_to_map(&mut map, "members", id);
        add_guild_id_to_map(&mut map, "roles", id);

        deserialize_val(Value::from(map)).map(|guild| Self {
            guild,
        })
    }
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildDeleteEvent {
    pub guild: UnavailableGuild,
}

/// Requires [`GatewayIntents::GUILD_EMOJIS_AND_STICKERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-emojis-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildEmojisUpdateEvent {
    #[serde(with = "emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-integrations-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-member-add).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildMemberAddEvent {
    pub member: Member,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-member-remove).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-member-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<String>,
    pub joined_at: Timestamp,
    pub roles: Vec<RoleId>,
    pub user: User,
    pub premium_since: Option<Timestamp>,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    pub avatar: Option<String>,
    pub communication_disabled_until: Option<Timestamp>,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-members-chunk).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildMembersChunkEvent {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, Member>,
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub nonce: Option<String>,
}

impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            GuildId,
            ChunkIndex,
            ChunkCount,
            Members,
            Nonce,
            Unknown(String),
        }

        struct GuildMembersChunkVisitor;

        impl<'de> Visitor<'de> for GuildMembersChunkVisitor {
            type Value = GuildMembersChunkEvent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct GuildMembersChunkEvent")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> StdResult<Self::Value, A::Error> {
                let mut guild_id = None;
                let mut chunk_index = None;
                let mut chunk_count = None;
                let mut members = None;
                let mut nonce = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::GuildId => {
                            if guild_id.is_some() {
                                return Err(DeError::duplicate_field("guild_id"));
                            }
                            guild_id = Some(map.next_value()?);
                        },
                        Field::ChunkIndex => {
                            if chunk_index.is_some() {
                                return Err(DeError::duplicate_field("chunk_index"));
                            }
                            chunk_index = Some(map.next_value()?);
                        },
                        Field::ChunkCount => {
                            if chunk_count.is_some() {
                                return Err(DeError::duplicate_field("chunk_count"));
                            }
                            chunk_count = Some(map.next_value()?);
                        },
                        Field::Members => {
                            if members.is_some() {
                                return Err(DeError::duplicate_field("members"));
                            }
                            members = Some(map.next_value::<Vec<InterimMember>>()?);
                        },
                        Field::Nonce => {
                            if nonce.is_some() {
                                return Err(DeError::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        },
                        Field::Unknown(_) => {
                            // ignore unknown keys
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }

                let guild_id = guild_id.ok_or_else(|| DeError::missing_field("guild_id"))?;
                let chunk_index =
                    chunk_index.ok_or_else(|| DeError::missing_field("chunk_index"))?;
                let chunk_count =
                    chunk_count.ok_or_else(|| DeError::missing_field("chunk_count"))?;
                let members = members.ok_or_else(|| DeError::missing_field("members"))?;

                let members = members
                    .into_iter()
                    .map(|mut m| {
                        m.guild_id = Some(guild_id);
                        (m.user.id, Member::from(m))
                    })
                    .collect();

                Ok(GuildMembersChunkEvent {
                    guild_id,
                    members,
                    chunk_index,
                    chunk_count,
                    nonce,
                })
            }
        }

        const FIELDS: &[&str] = &["guild_id", "chunk_index", "chunk_count", "members", "nonce"];
        deserializer.deserialize_struct("GuildMembersChunkEvent", FIELDS, GuildMembersChunkVisitor)
    }
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-role-create).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleCreateEvent {
    pub role: Role,
}

impl<'de> Deserialize<'de> for GuildRoleCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            role: roles::deserialize_event(deserializer)?,
        })
    }
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-role-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-role-update).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleUpdateEvent {
    pub role: Role,
}

impl<'de> Deserialize<'de> for GuildRoleUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(Self {
            role: roles::deserialize_event(deserializer)?,
        })
    }
}

/// Requires [`GatewayIntents::GUILD_EMOJIS_AND_STICKERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-stickers-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildStickersUpdateEvent {
    #[serde(with = "stickers")]
    pub stickers: HashMap<StickerId, Sticker>,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_INVITES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#invite-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteCreateEvent {
    pub channel_id: ChannelId,
    pub code: String,
    pub guild_id: Option<GuildId>,
    pub inviter: Option<User>,
    pub max_age: u64,
    pub max_uses: u64,
    pub temporary: bool,
}

/// Requires [`GatewayIntents::GUILD_INVITES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#invite-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteDeleteEvent {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub code: String,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildUpdateEvent {
    pub guild: PartialGuild,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct MessageCreateEvent {
    pub message: Message,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-delete-bulk).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteBulkEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub ids: Vec<MessageId>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-delete).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    #[serde(rename = "id")]
    pub message_id: MessageId,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author: Option<User>, // TODO: Is this a Message field that can even change?
    pub content: Option<String>,
    pub timestamp: Option<Timestamp>, // TODO: Is this a Message field that can even change?
    pub edited_timestamp: Option<Timestamp>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub mention_channels: Option<Vec<ChannelMention>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Embed>>,
    pub reactions: Option<Vec<MessageReaction>>,
    pub nonce: Option<String>, // TODO: Is this a Message field that can even change?
    pub pinned: Option<bool>,
    pub kind: Option<MessageType>, // TODO: Is this a Message field that can even change?
    pub flags: Option<MessageFlags>,
    pub components: Option<Vec<ActionRow>>,
    #[deprecated(note = "deprecated by Discord")]
    pub stickers: Option<Vec<StickerItem>>,
    pub sticker_items: Option<Vec<StickerItem>>,

    pub guild_id: Option<GuildId>, // TODO: Is this a Message field that can even change?
}

/// Requires [`GatewayIntents::GUILD_PRESENCES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#presence-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct PresenceUpdateEvent {
    pub presence: Presence,
}

/// Not officially documented.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct PresencesReplaceEvent {
    pub presences: Vec<Presence>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-add).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-remove).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionRemoveEvent {
    pub reaction: Reaction,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-remove-all).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ReactionRemoveAllEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-remove-emoji-message-reaction-remove-emoji-event-fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionRemoveEmojiEvent {
    pub reaction: Reaction,
}

/// The "Ready" event, containing initial ready cache
///
/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#ready).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReadyEvent {
    pub ready: Ready,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#resumed).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ResumedEvent {
    #[serde(rename = "_trace")]
    pub trace: Vec<Option<String>>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_TYPING`] or [`GatewayIntents::DIRECT_MESSAGE_TYPING`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#typing-start).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TypingStartEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub timestamp: u64,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnknownEvent {
    pub kind: String,
    pub value: Value,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#user-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#voice-server-update).
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceServerUpdateEvent {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: Option<GuildId>,
    pub token: String,
}

impl fmt::Debug for VoiceServerUpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VoiceServerUpdateEvent")
            .field("channel_id", &self.channel_id)
            .field("endpoint", &self.endpoint)
            .field("guild_id", &self.guild_id)
            .finish()
    }
}

/// Requires [`GatewayIntents::GUILD_VOICE_STATES`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#voice-state-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct VoiceStateUpdateEvent {
    pub voice_state: VoiceState,
}

/// Requires [`GatewayIntents::GUILD_WEBHOOKS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#webhooks-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#interaction-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct InteractionCreateEvent {
    pub interaction: Interaction,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#integration-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct IntegrationCreateEvent {
    pub integration: Integration,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#integration-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct IntegrationUpdateEvent {
    pub integration: Integration,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#integration-delete).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IntegrationDeleteEvent {
    pub id: IntegrationId,
    pub guild_id: GuildId,
    pub application_id: Option<ApplicationId>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#stage-instance-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceCreateEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#stage-instance-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceUpdateEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#stage-instance-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceDeleteEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadCreateEvent {
    pub thread: GuildChannel,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadUpdateEvent {
    pub thread: GuildChannel,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadDeleteEvent {
    pub thread: PartialGuildChannel,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-list-sync).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadListSyncEvent {
    /// The guild Id.
    pub guild_id: GuildId,
    /// The parent channel Id whose threads are being synced. If empty, then threads were synced for the entire guild.
    /// This array may contain channel Ids that have no active threads as well, so you know to clear that data.
    #[serde(default)]
    pub channels_id: Vec<ChannelId>,
    /// All active threads in the given channels that the current user can access.
    pub threads: Vec<GuildChannel>,
    /// All thread member objects from the synced threads for the current user,
    /// indicating which threads the current user has been added to
    pub members: Vec<ThreadMember>,
}

/// Requires [`GatewayIntents::GUILDS`], and, to receive this event for other users,
/// [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-member-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadMemberUpdateEvent {
    pub member: ThreadMember,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#thread-members-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMembersUpdateEvent {
    /// The id of the thread.
    pub id: ChannelId,
    /// The id of the Guild.
    pub guild_id: GuildId,
    /// The approximate number of members in the thread, capped at 50.
    pub member_count: u8,
    /// The users who were added to the thread.
    #[serde(default)]
    pub added_members: Vec<ThreadMember>,
    /// The ids of the users who were removed from the thread.
    #[serde(default)]
    pub removed_members_ids: Vec<UserId>,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ScheduledEventCreateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ScheduledEventUpdateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ScheduledEventDeleteEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-user-add).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEventUserAddEvent {
    #[serde(rename = "guild_scheduled_event_id")]
    pub scheduled_event_id: ScheduledEventId,
    pub guild_id: GuildId,
    pub user_id: UserId,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-user-remove).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEventUserRemoveEvent {
    #[serde(rename = "guild_scheduled_event_id")]
    pub scheduled_event_id: ScheduledEventId,
    pub guild_id: GuildId,
    pub user_id: UserId,
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#payloads-gateway-payload-structure).
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;
        let seq = remove_from_map_opt(&mut map, "s")?.flatten();

        Ok(match remove_from_map(&mut map, "op")? {
            Opcode::Dispatch => Self::Dispatch(
                seq.ok_or_else(|| DeError::missing_field("s"))?,
                deserialize_val(Value::from(map))?,
            ),
            Opcode::Heartbeat => {
                GatewayEvent::Heartbeat(seq.ok_or_else(|| DeError::missing_field("s"))?)
            },
            Opcode::InvalidSession => {
                GatewayEvent::InvalidateSession(remove_from_map(&mut map, "d")?)
            },
            Opcode::Hello => {
                #[derive(Deserialize)]
                struct HelloPayload {
                    heartbeat_interval: u64,
                }

                let inner: HelloPayload = remove_from_map(&mut map, "d")?;
                GatewayEvent::Hello(inner.heartbeat_interval)
            },
            Opcode::Reconnect => GatewayEvent::Reconnect,
            Opcode::HeartbeatAck => GatewayEvent::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}

/// Event received over a websocket connection
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#commands-and-events-gateway-events).
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "t", content = "d")]
#[non_exhaustive]
pub enum Event {
    /// The permissions of an [`Command`] was changed.
    ///
    /// Fires the [`EventHandler::application_command_permissions_update`] event.
    ///
    /// [`Command`]: crate::model::application::command::Command
    /// [`EventHandler::application_command_permissions_update`]: crate::client::EventHandler::application_command_permissions_update
    CommandPermissionsUpdate(CommandPermissionsUpdateEvent),
    /// A [`Rule`] was created.
    ///
    /// Fires the [`EventHandler::auto_moderation_rule_create`] event.
    ///
    /// [`EventHandler::auto_moderation_rule_create`]:
    /// crate::client::EventHandler::auto_moderation_rule_create
    AutoModRuleCreate(AutoModRuleCreateEvent),
    /// A [`Rule`] has been updated.
    ///
    /// Fires the [`EventHandler::auto_moderation_rule_update`] event.
    ///
    /// [`EventHandler::auto_moderation_rule_update`]:
    /// crate::client::EventHandler::auto_moderation_rule_update
    AutoModRuleUpdate(AutoModRuleUpdateEvent),
    /// A [`Rule`] was deleted.
    ///
    /// Fires the [`EventHandler::auto_moderation_rule_delete`] event.
    ///
    /// [`EventHandler::auto_moderation_rule_delete`]:
    /// crate::client::EventHandler::auto_moderation_rule_delete
    AutoModRuleDelete(AutoModRuleDeleteEvent),
    /// A [`Rule`] was triggered and an action was executed.
    ///
    /// Fires the [`EventHandler::auto_moderation_action_execution`] event.
    ///
    /// [`EventHandler::auto_moderation_action_execution`]:
    /// crate::client::EventHandler::auto_moderation_action_execution
    AutoModActionExecution(AutoModActionExecutionEvent),
    /// A [`Channel`] was created.
    ///
    /// Fires the [`EventHandler::channel_create`] event.
    ///
    /// [`EventHandler::channel_create`]: crate::client::EventHandler::channel_create
    ChannelCreate(ChannelCreateEvent),
    /// A [`Channel`] has been deleted.
    ///
    /// Fires the [`EventHandler::channel_delete`] event.
    ///
    /// [`EventHandler::channel_delete`]: crate::client::EventHandler::channel_delete
    ChannelDelete(ChannelDeleteEvent),
    /// The pins for a [`Channel`] have been updated.
    ///
    /// Fires the [`EventHandler::channel_pins_update`] event.
    ///
    /// [`EventHandler::channel_pins_update`]: crate::client::EventHandler::channel_pins_update
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A [`Channel`] has been updated.
    ///
    /// Fires the [`EventHandler::channel_update`] event.
    ///
    /// [`EventHandler::channel_update`]: crate::client::EventHandler::channel_update
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
    /// A [`Sticker`] was created, updated, or deleted
    GuildStickersUpdate(GuildStickersUpdateEvent),
    GuildUpdate(GuildUpdateEvent),
    /// An [`Invite`] was created.
    ///
    /// Fires the [`EventHandler::invite_create`] event handler.
    ///
    /// [`EventHandler::invite_create`]: crate::client::EventHandler::invite_create
    InviteCreate(InviteCreateEvent),
    /// An [`Invite`] was deleted.
    ///
    /// Fires the [`EventHandler::invite_delete`] event handler.
    ///
    /// [`EventHandler::invite_delete`]: crate::client::EventHandler::invite_delete
    InviteDelete(InviteDeleteEvent),
    MessageCreate(MessageCreateEvent),
    MessageDelete(MessageDeleteEvent),
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// A message has been edited, either by the user or the system
    MessageUpdate(MessageUpdateEvent),
    /// A member's presence state (or username or avatar) has changed
    PresenceUpdate(PresenceUpdateEvent),
    /// The presence list of the user's friends should be replaced entirely
    PresencesReplace(PresencesReplaceEvent),
    /// A reaction was added to a message.
    ///
    /// Fires the [`EventHandler::reaction_add`] event handler.
    ///
    /// [`EventHandler::reaction_add`]: crate::client::EventHandler::reaction_add
    #[serde(rename = "MESSAGE_REACTION_ADD")]
    ReactionAdd(ReactionAddEvent),
    /// A reaction was removed to a message.
    ///
    /// Fires the [`EventHandler::reaction_remove`] event handler.
    ///
    /// [`EventHandler::reaction_remove`]: crate::client::EventHandler::reaction_remove
    #[serde(rename = "MESSAGE_REACTION_REMOVE")]
    ReactionRemove(ReactionRemoveEvent),
    /// A request was issued to remove all [`Reaction`]s from a [`Message`].
    ///
    /// Fires the [`EventHandler::reaction_remove_all`] event handler.
    ///
    /// [`EventHandler::reaction_remove_all`]: crate::client::EventHandler::reaction_remove_all
    #[serde(rename = "MESSAGE_REACTION_REMOVE_ALL")]
    ReactionRemoveAll(ReactionRemoveAllEvent),
    /// Sent when a bot removes all instances of a given emoji from the reactions of a message.
    ///
    /// Fires the [`EventHandler::reaction_remove_emoji`] event handler.
    ///
    /// [`EventHandler::reaction_remove_emoji`]: crate::client::EventHandler::reaction_remove_emoji
    #[serde(rename = "MESSAGE_REACTION_REMOVE_EMOJI")]
    ReactionRemoveEmoji(ReactionRemoveEmojiEvent),
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
    WebhookUpdate(WebhookUpdateEvent),
    /// An interaction was created.
    InteractionCreate(InteractionCreateEvent),
    /// A guild integration was created
    IntegrationCreate(IntegrationCreateEvent),
    /// A guild integration was updated
    IntegrationUpdate(IntegrationUpdateEvent),
    /// A guild integration was deleted
    IntegrationDelete(IntegrationDeleteEvent),
    /// A stage instance was created.
    StageInstanceCreate(StageInstanceCreateEvent),
    /// A stage instance was updated.
    StageInstanceUpdate(StageInstanceUpdateEvent),
    /// A stage instance was deleted.
    StageInstanceDelete(StageInstanceDeleteEvent),
    /// A thread was created or the current user was added
    /// to a private thread.
    ThreadCreate(ThreadCreateEvent),
    /// A thread was updated.
    ThreadUpdate(ThreadUpdateEvent),
    /// A thread was deleted.
    ThreadDelete(ThreadDeleteEvent),
    /// The current user gains access to a channel.
    ThreadListSync(ThreadListSyncEvent),
    /// The [`ThreadMember`] object for the current user is updated.
    ThreadMemberUpdate(ThreadMemberUpdateEvent),
    /// Anyone is added to or removed from a thread.
    ThreadMembersUpdate(ThreadMembersUpdateEvent),
    /// A scheduled event was created.
    ScheduledEventCreate(ScheduledEventCreateEvent),
    /// A scheduled event was updated.
    ScheduledEventUpdate(ScheduledEventUpdateEvent),
    /// A scheduled event was deleted.
    ScheduledEventDelete(ScheduledEventDeleteEvent),
    /// A guild member has subscribed to a scheduled event.
    ScheduledEventUserAdd(ScheduledEventUserAddEvent),
    /// A guild member has unsubscribed from a scheduled event.
    ScheduledEventUserRemove(ScheduledEventUserRemoveEvent),
    /// An event type not covered by the above
    #[serde(other, deserialize_with = "ignore_input")]
    Unknown,
}

#[cfg(feature = "model")]
const fn gid_from_channel(c: &Channel) -> RelatedId<GuildId> {
    match c {
        Channel::Guild(g) => RelatedId::Some(g.guild_id),
        _ => RelatedId::None,
    }
}

macro_rules! with_related_ids_for_event_types {
    ($macro:ident) => {
        $macro! {
            Self::CommandPermissionsUpdate, Self::CommandPermissionsUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.permission.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::AutoModRuleCreate, Self::AutoModRuleCreate(e) => {
                user_id: Some(e.rule.creator_id),
                guild_id: Some(e.rule.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::AutoModRuleUpdate, Self::AutoModRuleUpdate(e) => {
                user_id: Some(e.rule.creator_id),
                guild_id: Some(e.rule.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::AutoModRuleDelete, Self::AutoModRuleDelete(e) => {
                user_id: Some(e.rule.creator_id),
                guild_id: Some(e.rule.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::AutoModActionExecution, Self::AutoModActionExecution(e) => {
                user_id: Some(e.execution.user_id),
                guild_id: Some(e.execution.guild_id),
                channel_id: e.execution.channel_id.into(),
                message_id: e.execution.message_id.into(),
            },
            Self::ChannelCreate, Self::ChannelCreate(e) => {
                user_id: Never,
                guild_id: gid_from_channel(&e.channel),
                channel_id: Some(e.channel.id()),
                message_id: Never,
            },
            Self::ChannelDelete, Self::ChannelDelete(e) => {
                user_id: Never,
                guild_id: gid_from_channel(&e.channel),
                channel_id: Some(e.channel.id()),
                message_id: Never,
            },
            Self::ChannelPinsUpdate, Self::ChannelPinsUpdate(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Never,
            },
            Self::ChannelUpdate, Self::ChannelUpdate(e) => {
                user_id: Never,
                guild_id: gid_from_channel(&e.channel),
                channel_id: Some(e.channel.id()),
                message_id: Never,
            },
            Self::GuildBanAdd, Self::GuildBanAdd(e) => {
                user_id: Some(e.user.id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildBanRemove, Self::GuildBanRemove(e) => {
                user_id: Some(e.user.id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildCreate, Self::GuildCreate(e) => {
                user_id: Never,
                guild_id: Some(e.guild.id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildDelete, Self::GuildDelete(e) => {
                user_id: Never,
                guild_id: Some(e.guild.id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildEmojisUpdate, Self::GuildEmojisUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildIntegrationsUpdate, Self::GuildIntegrationsUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildMemberAdd, Self::GuildMemberAdd(e) => {
                user_id: Some(e.member.user.id),
                guild_id: Some(e.member.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildMemberRemove, Self::GuildMemberRemove(e) => {
                user_id: Some(e.user.id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildMemberUpdate, Self::GuildMemberUpdate(e) => {
                user_id: Some(e.user.id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildMembersChunk, Self::GuildMembersChunk(e) => {
                user_id: Multiple(e.members.keys().copied().collect()),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildRoleCreate, Self::GuildRoleCreate(e) => {
                user_id: Never,
                guild_id: Some(e.role.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildRoleDelete, Self::GuildRoleDelete(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildRoleUpdate, Self::GuildRoleUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.role.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::ScheduledEventCreate, Self::ScheduledEventCreate(e) => {
                user_id: e.event.creator_id.into(),
                guild_id: Some(e.event.guild_id),
                channel_id: e.event.channel_id.into(),
                message_id: Never,
            },
            Self::ScheduledEventUpdate, Self::ScheduledEventUpdate(e) => {
                user_id: e.event.creator_id.into(),
                guild_id: Some(e.event.guild_id),
                channel_id: e.event.channel_id.into(),
                message_id: Never,
            },
            Self::ScheduledEventDelete, Self::ScheduledEventDelete(e) => {
                user_id: e.event.creator_id.into(),
                guild_id: Some(e.event.guild_id),
                channel_id: e.event.channel_id.into(),
                message_id: Never,
            },
            Self::ScheduledEventUserAdd, Self::ScheduledEventUserAdd(e) => {
                user_id: Some(e.user_id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::ScheduledEventUserRemove, Self::ScheduledEventUserRemove(e) => {
                user_id: Some(e.user_id),
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildStickersUpdate, Self::GuildStickersUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::GuildUpdate, Self::GuildUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.guild.id),
                channel_id: Never,
                message_id: Never,
            },
            Self::InviteCreate, Self::InviteCreate(e) => {
                user_id: e.inviter.as_ref().map(|u| u.id).into(),
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Never,
            },
            Self::InviteDelete, Self::InviteDelete(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Never,
            },
            Self::MessageCreate, Self::MessageCreate(e) => {
                user_id: Some(e.message.author.id),
                guild_id: e.message.guild_id.into(),
                channel_id: Some(e.message.channel_id),
                message_id: Some(e.message.id),
            },
            Self::MessageDelete, Self::MessageDelete(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Some(e.message_id),
            },
            Self::MessageDeleteBulk, Self::MessageDeleteBulk(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Multiple(e.ids.clone()),
            },
            Self::MessageUpdate, Self::MessageUpdate(e) => {
                user_id: e.author.as_ref().map(|u| u.id).into(),
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Some(e.id),
            },
            Self::PresenceUpdate, Self::PresenceUpdate(e) => {
                user_id: Some(e.presence.user.id),
                guild_id: e.presence.guild_id.into(),
                channel_id: Never,
                message_id: Never,
            },
            Self::PresencesReplace, Self::PresencesReplace(e) => {
                user_id: Multiple(e.presences.iter().map(|p| p.user.id).collect()),
                guild_id: Never,
                channel_id: Never,
                message_id: Never,
            },
            Self::ReactionAdd, Self::ReactionAdd(e) => {
                user_id: e.reaction.user_id.into(),
                guild_id: e.reaction.guild_id.into(),
                channel_id: Some(e.reaction.channel_id),
                message_id: Some(e.reaction.message_id),
            },
            Self::ReactionRemove, Self::ReactionRemove(e) => {
                user_id: e.reaction.user_id.into(),
                guild_id: e.reaction.guild_id.into(),
                channel_id: Some(e.reaction.channel_id),
                message_id: Some(e.reaction.message_id),
            },
            Self::ReactionRemoveAll, Self::ReactionRemoveAll(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Some(e.message_id),
            },
            Self::ReactionRemoveEmoji, Self::ReactionRemoveEmoji(e) => {
                user_id: e.reaction.user_id.into(),
                guild_id: e.reaction.guild_id.into(),
                channel_id: Some(e.reaction.channel_id),
                message_id: Some(e.reaction.message_id),
            },
            Self::Ready, Self::Ready(e) => {
                user_id: Never,
                guild_id: Never,
                channel_id: Never,
                message_id: Never,
            },
            Self::Resumed, Self::Resumed(e) => {
                user_id: Never,
                guild_id: Never,
                channel_id: Never,
                message_id: Never,
            },
            Self::StageInstanceCreate, Self::StageInstanceCreate(e) => {
                user_id: Never,
                guild_id: Some(e.stage_instance.guild_id),
                channel_id: Some(e.stage_instance.channel_id),
                message_id: Never,
            },
            Self::StageInstanceDelete, Self::StageInstanceDelete(e) => {
                user_id: Never,
                guild_id: Some(e.stage_instance.guild_id),
                channel_id: Some(e.stage_instance.channel_id),
                message_id: Never,
            },
            Self::StageInstanceUpdate, Self::StageInstanceUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.stage_instance.guild_id),
                channel_id: Some(e.stage_instance.channel_id),
                message_id: Never,
            },
            Self::ThreadCreate, Self::ThreadCreate(e) => {
                user_id: Never,
                guild_id: Some(e.thread.guild_id),
                channel_id: Some(e.thread.id),
                message_id: Never,
            },
            Self::ThreadDelete, Self::ThreadDelete(e) => {
                user_id: Never,
                guild_id: Some(e.thread.guild_id),
                channel_id: Some(e.thread.id),
                message_id: Never,
            },
            Self::ThreadListSync, Self::ThreadListSync(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Multiple(e.threads.iter().map(|c| c.id).collect()),
                message_id: Never,
            },
            Self::ThreadMembersUpdate, Self::ThreadMembersUpdate(e) => {
                user_id: Multiple(e.added_members
                        .iter()
                        .filter_map(|m| m.user_id)
                        .chain(e.removed_members_ids.iter().copied())
                        .collect(),
                    ),
                guild_id: Some(e.guild_id),
                channel_id: Some(e.id),
                message_id: Never,
            },
            Self::ThreadMemberUpdate, Self::ThreadMemberUpdate(e) => {
                user_id: e.member.user_id.into(),
                guild_id: Never,
                channel_id: e.member.id.into(),
                message_id: Never,
            },
            Self::ThreadUpdate, Self::ThreadUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.thread.guild_id),
                channel_id: Some(e.thread.id),
                message_id: Never,
            },
            Self::TypingStart, Self::TypingStart(e) => {
                user_id: Some(e.user_id),
                guild_id: e.guild_id.into(),
                channel_id: Some(e.channel_id),
                message_id: Never,
            },
            Self::UserUpdate, Self::UserUpdate(e) => {
                user_id: Some(e.current_user.id),
                guild_id: Never,
                channel_id: Never,
                message_id: Never,
            },
            Self::VoiceServerUpdate, Self::VoiceServerUpdate(e) => {
                user_id: Never,
                guild_id: e.guild_id.into(),
                channel_id: e.channel_id.into(),
                message_id: Never,
            },
            Self::VoiceStateUpdate, Self::VoiceStateUpdate(e) => {
                user_id: Some(e.voice_state.user_id),
                guild_id: e.voice_state.guild_id.into(),
                channel_id: Never,
                message_id: Never,
            },
            Self::WebhookUpdate, Self::WebhookUpdate(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Some(e.channel_id),
                message_id: Never,
            },
            Self::InteractionCreate, Self::InteractionCreate(e) => {
                user_id: match &e.interaction {
                    Interaction::Ping(_) => None,
                    Interaction::Command(i) => Some(i.user.id),
                    Interaction::Component(i) => Some(i.user.id),
                    Interaction::Autocomplete(i) => Some(i.user.id),
                    Interaction::Modal(i) => Some(i.user.id),
                },
                guild_id: match &e.interaction {
                    Interaction::Ping(_) => None,
                    Interaction::Command(i) => i.guild_id.into(),
                    Interaction::Component(i) => i.guild_id.into(),
                    Interaction::Autocomplete(i) => i.guild_id.into(),
                    Interaction::Modal(i) => i.guild_id.into(),
                },
                channel_id: match &e.interaction {
                    Interaction::Ping(_) => None,
                    Interaction::Command(i) => Some(i.channel_id),
                    Interaction::Component(i) => Some(i.channel_id),
                    Interaction::Autocomplete(i) => Some(i.channel_id),
                    Interaction::Modal(i) => Some(i.channel_id),
                },
                message_id: match &e.interaction {
                    Interaction::Ping(_) => None,
                    Interaction::Command(_) => None,
                    Interaction::Component(i) => Some(i.message.id),
                    Interaction::Autocomplete(i) => None,
                    Interaction::Modal(i) => i.message.as_ref().map(|m| m.id).into(),
                },
            },
            Self::IntegrationCreate, Self::IntegrationCreate(e) => {
                user_id: e.integration.user.as_ref().map(|u| u.id).into(),
                guild_id: Some(e.integration.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::IntegrationUpdate, Self::IntegrationUpdate(e) => {
                user_id: e.integration.user.as_ref().map(|u| u.id).into(),
                guild_id: Some(e.integration.guild_id),
                channel_id: Never,
                message_id: Never,
            },
            Self::IntegrationDelete, Self::IntegrationDelete(e) => {
                user_id: Never,
                guild_id: Some(e.guild_id),
                channel_id: Never,
                message_id: Never,
            },
        }
    };
}

#[cfg(feature = "model")]
macro_rules! define_event_related_id_methods {
    ($(
        $(#[$attr:meta])?
        $_:path, $variant:pat => {
            user_id: $user_id:expr,
            guild_id: $guild_id:expr,
            channel_id: $channel_id:expr,
            message_id: $message_id:expr,
        }
    ),+ $(,)?) => {
        /// User ID(s) related to this event.
        #[must_use]
        pub fn user_id(&self) -> RelatedId<UserId> {
            use RelatedId::*;
            #[allow(unused_variables)]
            match self {
                Self::Unknown => Never,
                $(
                    $(#[$attr])?
                    $variant => $user_id
                ),+
            }
        }

        /// Guild ID related to this event.
        #[must_use]
        pub fn guild_id(&self) -> RelatedId<GuildId> {
            use RelatedId::*;
            #[allow(unused_variables)]
            match self {
                Self::Unknown => Never,
                $(
                    $(#[$attr])?
                    $variant => $guild_id
                ),+
            }
        }

        /// Channel ID(s) related to this event.
        #[must_use]
        pub fn channel_id(&self) -> RelatedId<ChannelId> {
            use RelatedId::*;
            #[allow(unused_variables)]
            match self {
                Self::Unknown => Never,
                $(
                    $(#[$attr])?
                    $variant => $channel_id
                ),+
            }
        }

        /// Message ID(s) related to this event.
        #[must_use]
        pub fn message_id(&self) -> RelatedId<MessageId> {
            use RelatedId::*;
            #[allow(unused_variables)]
            match self {
                Self::Unknown => Never,
                $(
                    $(#[$attr])?
                    $variant => $message_id
                ),+
            }
        }
    };
}

impl Event {
    /// Return the type of this event.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        match self {
            Self::CommandPermissionsUpdate(_) => EventType::CommandPermissionsUpdate,
            Self::AutoModRuleCreate(_) => EventType::AutoModRuleCreate,
            Self::AutoModRuleUpdate(_) => EventType::AutoModRuleUpdate,
            Self::AutoModRuleDelete(_) => EventType::AutoModRuleDelete,
            Self::AutoModActionExecution(_) => EventType::AutoModActionExecution,
            Self::ChannelCreate(_) => EventType::ChannelCreate,
            Self::ChannelDelete(_) => EventType::ChannelDelete,
            Self::ChannelPinsUpdate(_) => EventType::ChannelPinsUpdate,
            Self::ChannelUpdate(_) => EventType::ChannelUpdate,
            Self::GuildBanAdd(_) => EventType::GuildBanAdd,
            Self::GuildBanRemove(_) => EventType::GuildBanRemove,
            Self::GuildCreate(_) => EventType::GuildCreate,
            Self::GuildDelete(_) => EventType::GuildDelete,
            Self::GuildEmojisUpdate(_) => EventType::GuildEmojisUpdate,
            Self::GuildIntegrationsUpdate(_) => EventType::GuildIntegrationsUpdate,
            Self::GuildMemberAdd(_) => EventType::GuildMemberAdd,
            Self::GuildMemberRemove(_) => EventType::GuildMemberRemove,
            Self::GuildMemberUpdate(_) => EventType::GuildMemberUpdate,
            Self::GuildMembersChunk(_) => EventType::GuildMembersChunk,
            Self::GuildRoleCreate(_) => EventType::GuildRoleCreate,
            Self::GuildRoleDelete(_) => EventType::GuildRoleDelete,
            Self::GuildRoleUpdate(_) => EventType::GuildRoleUpdate,
            Self::GuildStickersUpdate(_) => EventType::GuildStickersUpdate,
            Self::GuildUpdate(_) => EventType::GuildUpdate,
            Self::InviteCreate(_) => EventType::InviteCreate,
            Self::InviteDelete(_) => EventType::InviteDelete,
            Self::MessageCreate(_) => EventType::MessageCreate,
            Self::MessageDelete(_) => EventType::MessageDelete,
            Self::MessageDeleteBulk(_) => EventType::MessageDeleteBulk,
            Self::MessageUpdate(_) => EventType::MessageUpdate,
            Self::PresenceUpdate(_) => EventType::PresenceUpdate,
            Self::PresencesReplace(_) => EventType::PresencesReplace,
            Self::ReactionAdd(_) => EventType::ReactionAdd,
            Self::ReactionRemove(_) => EventType::ReactionRemove,
            Self::ReactionRemoveAll(_) => EventType::ReactionRemoveAll,
            Self::ReactionRemoveEmoji(_) => EventType::ReactionRemoveEmoji,
            Self::Ready(_) => EventType::Ready,
            Self::Resumed(_) => EventType::Resumed,
            Self::TypingStart(_) => EventType::TypingStart,
            Self::UserUpdate(_) => EventType::UserUpdate,
            Self::VoiceStateUpdate(_) => EventType::VoiceStateUpdate,
            Self::VoiceServerUpdate(_) => EventType::VoiceServerUpdate,
            Self::WebhookUpdate(_) => EventType::WebhookUpdate,
            Self::InteractionCreate(_) => EventType::InteractionCreate,
            Self::IntegrationCreate(_) => EventType::IntegrationCreate,
            Self::IntegrationUpdate(_) => EventType::IntegrationUpdate,
            Self::IntegrationDelete(_) => EventType::IntegrationDelete,
            Self::StageInstanceCreate(_) => EventType::StageInstanceCreate,
            Self::StageInstanceUpdate(_) => EventType::StageInstanceUpdate,
            Self::StageInstanceDelete(_) => EventType::StageInstanceDelete,
            Self::ThreadCreate(_) => EventType::ThreadCreate,
            Self::ThreadUpdate(_) => EventType::ThreadUpdate,
            Self::ThreadDelete(_) => EventType::ThreadDelete,
            Self::ThreadListSync(_) => EventType::ThreadListSync,
            Self::ThreadMemberUpdate(_) => EventType::ThreadMemberUpdate,
            Self::ThreadMembersUpdate(_) => EventType::ThreadMembersUpdate,
            Self::ScheduledEventCreate(_) => EventType::ScheduledEventCreate,
            Self::ScheduledEventUpdate(_) => EventType::ScheduledEventUpdate,
            Self::ScheduledEventDelete(_) => EventType::ScheduledEventDelete,
            Self::ScheduledEventUserAdd(_) => EventType::ScheduledEventUserAdd,
            Self::ScheduledEventUserRemove(_) => EventType::ScheduledEventUserRemove,
            Self::Unknown => EventType::Other,
        }
    }

    #[cfg(feature = "model")]
    with_related_ids_for_event_types!(define_event_related_id_methods);
}

/// Similar to [`Option`], but with additional variants relevant to [`Event`]'s id methods (such as
/// [`Event::user_id`]).
pub enum RelatedId<T> {
    /// This event type will never have this kind of related ID
    Never,
    /// This particular event has no related ID of this type, but other events of this type may.
    None,
    /// A single related ID
    Some(T),
    /// Multiple related IDs
    Multiple(Vec<T>),
}

impl<T> RelatedId<T> {
    pub fn contains(&self, value: &T) -> bool
    where
        T: std::cmp::PartialEq,
    {
        match self {
            Self::Never | RelatedId::None => false,
            Self::Some(id) => id == value,
            Self::Multiple(ids) => ids.contains(value),
        }
    }
}

impl<T> From<Option<T>> for RelatedId<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None => RelatedId::None,
            Some(t) => RelatedId::Some(t),
        }
    }
}

impl<T> TryFrom<RelatedId<T>> for Option<T> {
    type Error = Vec<T>;

    fn try_from(value: RelatedId<T>) -> StdResult<Self, Self::Error> {
        match value {
            RelatedId::Never | RelatedId::None => Ok(None),
            RelatedId::Some(t) => Ok(Some(t)),
            RelatedId::Multiple(t) => Err(t),
        }
    }
}

/// The type of event dispatch received from the gateway.
///
/// This is useful for deciding how to deserialize a received payload.
///
/// A Deserialization implementation is provided for deserializing raw event
/// dispatch type strings to this enum, e.g. deserializing `"CHANNEL_CREATE"` to
/// [`EventType::ChannelCreate`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#commands-and-events-gateway-events).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum EventType {
    /// Indicator that an application command permission update payload was received.
    ///
    /// This maps to [`CommandPermissionsUpdateEvent`].
    CommandPermissionsUpdate,
    /// Indicator that an auto moderation rule create payload was received.
    ///
    /// This maps to [`AutoModRuleCreateEvent`].
    AutoModRuleCreate,
    /// Indicator that an auto moderation rule update payload was received.
    ///
    /// This maps to [`AutoModRuleCreateEvent`].
    AutoModRuleUpdate,
    /// Indicator that an auto moderation rule delete payload was received.
    ///
    /// This maps to [`AutoModRuleDeleteEvent`].
    AutoModRuleDelete,
    /// Indicator that an auto moderation action execution payload was received.
    ///
    /// This maps to [`AutoModActionExecutionEvent`].
    AutoModActionExecution,
    /// Indicator that a channel create payload was received.
    ///
    /// This maps to [`ChannelCreateEvent`].
    ChannelCreate,
    /// Indicator that a channel delete payload was received.
    ///
    /// This maps to [`ChannelDeleteEvent`].
    ChannelDelete,
    /// Indicator that a channel pins update payload was received.
    ///
    /// This maps to [`ChannelPinsUpdateEvent`].
    ChannelPinsUpdate,
    /// Indicator that a channel update payload was received.
    ///
    /// This maps to [`ChannelUpdateEvent`].
    ChannelUpdate,
    /// Indicator that a guild ban addition payload was received.
    ///
    /// This maps to [`GuildBanAddEvent`].
    GuildBanAdd,
    /// Indicator that a guild ban removal payload was received.
    ///
    /// This maps to [`GuildBanRemoveEvent`].
    GuildBanRemove,
    /// Indicator that a guild create payload was received.
    ///
    /// This maps to [`GuildCreateEvent`].
    GuildCreate,
    /// Indicator that a guild delete payload was received.
    ///
    /// This maps to [`GuildDeleteEvent`].
    GuildDelete,
    /// Indicator that a guild emojis update payload was received.
    ///
    /// This maps to [`GuildEmojisUpdateEvent`].
    GuildEmojisUpdate,
    /// Indicator that a guild integrations update payload was received.
    ///
    /// This maps to [`GuildIntegrationsUpdateEvent`].
    GuildIntegrationsUpdate,
    /// Indicator that a guild member add payload was received.
    ///
    /// This maps to [`GuildMemberAddEvent`].
    GuildMemberAdd,
    /// Indicator that a guild member remove payload was received.
    ///
    /// This maps to [`GuildMemberRemoveEvent`].
    GuildMemberRemove,
    /// Indicator that a guild member update payload was received.
    ///
    /// This maps to [`GuildMemberUpdateEvent`].
    GuildMemberUpdate,
    /// Indicator that a guild members chunk payload was received.
    ///
    /// This maps to [`GuildMembersChunkEvent`].
    GuildMembersChunk,
    /// Indicator that a guild role create payload was received.
    ///
    /// This maps to [`GuildRoleCreateEvent`].
    GuildRoleCreate,
    /// Indicator that a guild role delete payload was received.
    ///
    /// This maps to [`GuildRoleDeleteEvent`].
    GuildRoleDelete,
    /// Indicator that a guild role update payload was received.
    ///
    /// This maps to [`GuildRoleUpdateEvent`].
    GuildRoleUpdate,
    /// Indicator that a guild sticker update payload was received.
    ///
    /// This maps to [`GuildStickersUpdateEvent`].
    GuildStickersUpdate,
    /// Indicator that a guild update payload was received.
    ///
    /// This maps to [`GuildUpdateEvent`].
    GuildUpdate,
    /// Indicator that an invite was created.
    ///
    /// This maps to [`InviteCreateEvent`].
    InviteCreate,
    /// Indicator that an invite was deleted.
    ///
    /// This maps to [`InviteDeleteEvent`].
    InviteDelete,
    /// Indicator that a message create payload was received.
    ///
    /// This maps to [`MessageCreateEvent`].
    MessageCreate,
    /// Indicator that a message delete payload was received.
    ///
    /// This maps to [`MessageDeleteEvent`].
    MessageDelete,
    /// Indicator that a message delete bulk payload was received.
    ///
    /// This maps to [`MessageDeleteBulkEvent`].
    MessageDeleteBulk,
    /// Indicator that a message update payload was received.
    ///
    /// This maps to [`MessageUpdateEvent`].
    MessageUpdate,
    /// Indicator that a presence update payload was received.
    ///
    /// This maps to [`PresenceUpdateEvent`].
    PresenceUpdate,
    /// Indicator that a presences replace payload was received.
    ///
    /// This maps to [`PresencesReplaceEvent`].
    PresencesReplace,
    /// Indicator that a reaction add payload was received.
    ///
    /// This maps to [`ReactionAddEvent`].
    ReactionAdd,
    /// Indicator that a reaction remove payload was received.
    ///
    /// This maps to [`ReactionRemoveEvent`].
    ReactionRemove,
    /// Indicator that a reaction remove all payload was received.
    ///
    /// This maps to [`ReactionRemoveAllEvent`].
    ReactionRemoveAll,
    /// Indicator that a reaction remove emoji payload was received.
    ///
    /// This maps to [`ReactionRemoveEmojiEvent`].
    ReactionRemoveEmoji,
    /// Indicator that a ready payload was received.
    ///
    /// This maps to [`ReadyEvent`].
    Ready,
    /// Indicator that a resumed payload was received.
    ///
    /// This maps to [`ResumedEvent`].
    Resumed,
    /// Indicator that a typing start payload was received.
    ///
    /// This maps to [`TypingStartEvent`].
    TypingStart,
    /// Indicator that a user update payload was received.
    ///
    /// This maps to [`UserUpdateEvent`].
    UserUpdate,
    /// Indicator that a voice state payload was received.
    ///
    /// This maps to [`VoiceStateUpdateEvent`].
    VoiceStateUpdate,
    /// Indicator that a voice server update payload was received.
    ///
    /// This maps to [`VoiceServerUpdateEvent`].
    VoiceServerUpdate,
    /// Indicator that a webhook update payload was received.
    ///
    /// This maps to [`WebhookUpdateEvent`].
    WebhookUpdate,
    /// Indicator that an interaction was created.
    ///
    /// This maps to [`InteractionCreateEvent`].
    InteractionCreate,
    /// Indicator that an integration was created.
    ///
    /// This maps to [`IntegrationCreateEvent`].
    IntegrationCreate,
    /// Indicator that an integration was created.
    ///
    /// This maps to [`IntegrationUpdateEvent`].
    IntegrationUpdate,
    /// Indicator that an integration was created.
    ///
    /// This maps to [`IntegrationDeleteEvent`].
    IntegrationDelete,
    /// Indicator that a stage instance was created.
    ///
    /// This maps to [`StageInstanceCreateEvent`].
    StageInstanceCreate,
    /// Indicator that a stage instance was updated.
    ///
    /// This maps to [`StageInstanceUpdateEvent`].
    StageInstanceUpdate,
    /// Indicator that a stage instance was deleted.
    ///
    /// This maps to [`StageInstanceDeleteEvent`].
    StageInstanceDelete,
    /// Indicator that a thread was created or the current user
    /// was added to a private thread.
    ///
    /// This maps to [`ThreadCreateEvent`].
    ThreadCreate,
    /// Indicator that a thread was updated.
    ///
    /// This maps to [`ThreadUpdateEvent`].
    ThreadUpdate,
    /// Indicator that a thread was deleted.
    ///
    /// This maps to [`ThreadDeleteEvent`].
    ThreadDelete,
    /// Indicator that the current user gains access to a channel.
    ///
    /// This maps to [`ThreadListSyncEvent`]
    ThreadListSync,
    /// Indicator that the [`ThreadMember`] object for the current user is updated.
    ///
    /// This maps to [`ThreadMemberUpdateEvent`]
    ThreadMemberUpdate,
    /// Indicator that anyone is added to or removed from a thread.
    ///
    /// This maps to [`ThreadMembersUpdateEvent`]
    ThreadMembersUpdate,
    /// Indicator that a scheduled event create payload was received.
    ///
    /// This maps to [`ScheduledEventCreateEvent`].
    ScheduledEventCreate,
    /// Indicator that a scheduled event update payload was received.
    ///
    /// This maps to [`ScheduledEventUpdateEvent`].
    ScheduledEventUpdate,
    /// Indicator that a scheduled event delete payload was received.
    ///
    /// This maps to [`ScheduledEventDeleteEvent`].
    ScheduledEventDelete,
    /// Indicator that a guild member has subscribed to a scheduled event.
    ///
    /// This maps to [`ScheduledEventUserAddEvent`].
    ScheduledEventUserAdd,
    /// Indicator that a guild member has unsubscribed from a scheduled event.
    ///
    /// This maps to [`ScheduledEventUserRemoveEvent`].
    ScheduledEventUserRemove,
    /// An unknown event was received over the gateway.
    Other,
}

impl From<&Event> for EventType {
    fn from(event: &Event) -> EventType {
        event.event_type()
    }
}

/// Defines the related IDs that may exist for an [`EventType`].
///
/// If a field equals `false`, the corresponding [`Event`] method (i.e. [`Event::user_id`] for the
/// `user_id` field ) will always return [`RelatedId::Never`] for this [`EventType`]. Otherwise, an
/// event of this type may have one or more related IDs.
#[derive(Debug, Default)]
pub struct RelatedIdsForEventType {
    pub user_id: bool,
    pub guild_id: bool,
    pub channel_id: bool,
    pub message_id: bool,
}

macro_rules! define_related_ids_for_event_type {
    (
        $(
            $(#[$attr:meta])?
            $variant:path, $_:pat => { $($input:tt)* }
        ),+ $(,)?
    ) => {
        #[must_use]
        pub fn related_ids(&self) -> RelatedIdsForEventType {
            match self {
                Self::Other => Default::default(),
                $(
                    $(#[$attr])?
                    $variant =>
                        define_related_ids_for_event_type!{ @munch ($($input)*) -> {} }
                ),+
            }
        }
    };
    // Use tt munching to consume "fields" from macro input one at a time and generate the
    // true/false values we actually want based on whether the input is "Never" or some other
    // arbitrary expression.
    (@munch ($id:ident: Never, $($next:tt)*) -> {$($output:tt)*}) => {
        define_related_ids_for_event_type!{ @munch ($($next)*) -> {$($output)* ($id: false)} }
    };
    (@munch ($id:ident: $_:expr, $($next:tt)*) -> {$($output:tt)*}) => {
        define_related_ids_for_event_type!{ @munch ($($next)*) -> {$($output)* ($id: true)} }
    };
    // All input fields consumed; create the struct.
    (@munch () -> {$(($id:ident: $value:literal))+}) => {
        RelatedIdsForEventType {
            $(
                $id: $value
            ),+
        }
    };
}

impl EventType {
    const APPLICATION_COMMAND_PERMISSIONS_UPDATE: &'static str =
        "APPLICATION_COMMAND_PERMISSIONS_UPDATE";
    const AUTO_MODERATION_RULE_CREATE: &'static str = "AUTO_MODERATION_RULE_CREATE";
    const AUTO_MODERATION_RULE_UPDATE: &'static str = "AUTO_MODERATION_RULE_UPDATE";
    const AUTO_MODERATION_RULE_DELETE: &'static str = "AUTO_MODERATION_RULE_DELETE";
    const AUTO_MODERATION_ACTION_EXECUTION: &'static str = "AUTO_MODERATION_ACTION_EXECUTION";
    const CHANNEL_CREATE: &'static str = "CHANNEL_CREATE";
    const CHANNEL_DELETE: &'static str = "CHANNEL_DELETE";
    const CHANNEL_PINS_UPDATE: &'static str = "CHANNEL_PINS_UPDATE";
    const CHANNEL_UPDATE: &'static str = "CHANNEL_UPDATE";
    const GUILD_BAN_ADD: &'static str = "GUILD_BAN_ADD";
    const GUILD_BAN_REMOVE: &'static str = "GUILD_BAN_REMOVE";
    const GUILD_CREATE: &'static str = "GUILD_CREATE";
    const GUILD_DELETE: &'static str = "GUILD_DELETE";
    const GUILD_EMOJIS_UPDATE: &'static str = "GUILD_EMOJIS_UPDATE";
    const GUILD_INTEGRATIONS_UPDATE: &'static str = "GUILD_INTEGRATIONS_UPDATE";
    const GUILD_MEMBER_ADD: &'static str = "GUILD_MEMBER_ADD";
    const GUILD_MEMBER_REMOVE: &'static str = "GUILD_MEMBER_REMOVE";
    const GUILD_MEMBER_UPDATE: &'static str = "GUILD_MEMBER_UPDATE";
    const GUILD_MEMBERS_CHUNK: &'static str = "GUILD_MEMBERS_CHUNK";
    const GUILD_ROLE_CREATE: &'static str = "GUILD_ROLE_CREATE";
    const GUILD_ROLE_DELETE: &'static str = "GUILD_ROLE_DELETE";
    const GUILD_ROLE_UPDATE: &'static str = "GUILD_ROLE_UPDATE";
    const GUILD_STICKERS_UPDATE: &'static str = "GUILD_STICKERS_UPDATE";
    const INVITE_CREATE: &'static str = "INVITE_CREATE";
    const INVITE_DELETE: &'static str = "INVITE_DELETE";
    const GUILD_UPDATE: &'static str = "GUILD_UPDATE";
    const MESSAGE_CREATE: &'static str = "MESSAGE_CREATE";
    const MESSAGE_DELETE: &'static str = "MESSAGE_DELETE";
    const MESSAGE_DELETE_BULK: &'static str = "MESSAGE_DELETE_BULK";
    const MESSAGE_REACTION_ADD: &'static str = "MESSAGE_REACTION_ADD";
    const MESSAGE_REACTION_REMOVE: &'static str = "MESSAGE_REACTION_REMOVE";
    const MESSAGE_REACTION_REMOVE_ALL: &'static str = "MESSAGE_REACTION_REMOVE_ALL";
    const MESSAGE_REACTION_REMOVE_EMOJI: &'static str = "MESSAGE_REACTION_REMOVE_ALL_EMOJI";
    const MESSAGE_UPDATE: &'static str = "MESSAGE_UPDATE";
    const PRESENCE_UPDATE: &'static str = "PRESENCE_UPDATE";
    const PRESENCES_REPLACE: &'static str = "PRESENCES_REPLACE";
    const READY: &'static str = "READY";
    const RESUMED: &'static str = "RESUMED";
    const TYPING_START: &'static str = "TYPING_START";
    const USER_UPDATE: &'static str = "USER_UPDATE";
    const VOICE_SERVER_UPDATE: &'static str = "VOICE_SERVER_UPDATE";
    const VOICE_STATE_UPDATE: &'static str = "VOICE_STATE_UPDATE";
    const WEBHOOKS_UPDATE: &'static str = "WEBHOOKS_UPDATE";
    const INTERACTION_CREATE: &'static str = "INTERACTION_CREATE";
    const INTEGRATION_CREATE: &'static str = "INTEGRATION_CREATE";
    const INTEGRATION_UPDATE: &'static str = "INTEGRATION_UPDATE";
    const INTEGRATION_DELETE: &'static str = "INTEGRATION_DELETE";
    const STAGE_INSTANCE_CREATE: &'static str = "STAGE_INSTANCE_CREATE";
    const STAGE_INSTANCE_UPDATE: &'static str = "STAGE_INSTANCE_UPDATE";
    const STAGE_INSTANCE_DELETE: &'static str = "STAGE_INSTANCE_DELETE";
    const THREAD_CREATE: &'static str = "THREAD_CREATE";
    const THREAD_UPDATE: &'static str = "THREAD_UPDATE";
    const THREAD_DELETE: &'static str = "THREAD_DELETE";
    const THREAD_LIST_SYNC: &'static str = "THREAD_LIST_SYNC";
    const THREAD_MEMBER_UPDATE: &'static str = "THREAD_MEMBER_UPDATE";
    const THREAD_MEMBERS_UPDATE: &'static str = "THREAD_MEMBERS_UPDATE";
    const GUILD_SCHEDULED_EVENT_CREATE: &'static str = "GUILD_SCHEDULED_EVENT_CREATE";
    const GUILD_SCHEDULED_EVENT_UPDATE: &'static str = "GUILD_SCHEDULED_EVENT_UPDATE";
    const GUILD_SCHEDULED_EVENT_DELETE: &'static str = "GUILD_SCHEDULED_EVENT_DELETE";
    const GUILD_SCHEDULED_EVENT_USER_ADD: &'static str = "GUILD_SCHEDULED_EVENT_USER_ADD";
    const GUILD_SCHEDULED_EVENT_USER_REMOVE: &'static str = "GUILD_SCHEDULED_EVENT_USER_REMOVE";

    /// Return the event name of this event. Some events are synthetic, and we lack
    /// the information to recover the original event name for these events, in which
    /// case this method returns [`None`].
    #[must_use]
    pub const fn name(&self) -> Option<&str> {
        match self {
            Self::CommandPermissionsUpdate => Some(Self::APPLICATION_COMMAND_PERMISSIONS_UPDATE),
            Self::AutoModRuleCreate => Some(Self::AUTO_MODERATION_RULE_CREATE),
            Self::AutoModRuleUpdate => Some(Self::AUTO_MODERATION_RULE_UPDATE),
            Self::AutoModRuleDelete => Some(Self::AUTO_MODERATION_RULE_DELETE),
            Self::AutoModActionExecution => Some(Self::AUTO_MODERATION_ACTION_EXECUTION),
            Self::ChannelCreate => Some(Self::CHANNEL_CREATE),
            Self::ChannelDelete => Some(Self::CHANNEL_DELETE),
            Self::ChannelPinsUpdate => Some(Self::CHANNEL_PINS_UPDATE),
            Self::ChannelUpdate => Some(Self::CHANNEL_UPDATE),
            Self::GuildBanAdd => Some(Self::GUILD_BAN_ADD),
            Self::GuildBanRemove => Some(Self::GUILD_BAN_REMOVE),
            Self::GuildCreate => Some(Self::GUILD_CREATE),
            Self::GuildDelete => Some(Self::GUILD_DELETE),
            Self::GuildEmojisUpdate => Some(Self::GUILD_EMOJIS_UPDATE),
            Self::GuildIntegrationsUpdate => Some(Self::GUILD_INTEGRATIONS_UPDATE),
            Self::GuildMemberAdd => Some(Self::GUILD_MEMBER_ADD),
            Self::GuildMemberRemove => Some(Self::GUILD_MEMBER_REMOVE),
            Self::GuildMemberUpdate => Some(Self::GUILD_MEMBER_UPDATE),
            Self::GuildMembersChunk => Some(Self::GUILD_MEMBERS_CHUNK),
            Self::GuildRoleCreate => Some(Self::GUILD_ROLE_CREATE),
            Self::GuildRoleDelete => Some(Self::GUILD_ROLE_DELETE),
            Self::GuildRoleUpdate => Some(Self::GUILD_ROLE_UPDATE),
            Self::GuildStickersUpdate => Some(Self::GUILD_STICKERS_UPDATE),
            Self::InviteCreate => Some(Self::INVITE_CREATE),
            Self::InviteDelete => Some(Self::INVITE_DELETE),
            Self::GuildUpdate => Some(Self::GUILD_UPDATE),
            Self::MessageCreate => Some(Self::MESSAGE_CREATE),
            Self::MessageDelete => Some(Self::MESSAGE_DELETE),
            Self::MessageDeleteBulk => Some(Self::MESSAGE_DELETE_BULK),
            Self::ReactionAdd => Some(Self::MESSAGE_REACTION_ADD),
            Self::ReactionRemove => Some(Self::MESSAGE_REACTION_REMOVE),
            Self::ReactionRemoveAll => Some(Self::MESSAGE_REACTION_REMOVE_ALL),
            Self::ReactionRemoveEmoji => Some(Self::MESSAGE_REACTION_REMOVE_EMOJI),
            Self::MessageUpdate => Some(Self::MESSAGE_UPDATE),
            Self::PresenceUpdate => Some(Self::PRESENCE_UPDATE),
            Self::PresencesReplace => Some(Self::PRESENCES_REPLACE),
            Self::Ready => Some(Self::READY),
            Self::Resumed => Some(Self::RESUMED),
            Self::TypingStart => Some(Self::TYPING_START),
            Self::UserUpdate => Some(Self::USER_UPDATE),
            Self::VoiceServerUpdate => Some(Self::VOICE_SERVER_UPDATE),
            Self::VoiceStateUpdate => Some(Self::VOICE_STATE_UPDATE),
            Self::WebhookUpdate => Some(Self::WEBHOOKS_UPDATE),
            Self::InteractionCreate => Some(Self::INTERACTION_CREATE),
            Self::IntegrationCreate => Some(Self::INTEGRATION_CREATE),
            Self::IntegrationUpdate => Some(Self::INTEGRATION_UPDATE),
            Self::IntegrationDelete => Some(Self::INTEGRATION_DELETE),
            Self::StageInstanceCreate => Some(Self::STAGE_INSTANCE_CREATE),
            Self::StageInstanceUpdate => Some(Self::STAGE_INSTANCE_UPDATE),
            Self::StageInstanceDelete => Some(Self::STAGE_INSTANCE_DELETE),
            Self::ThreadCreate => Some(Self::THREAD_CREATE),
            Self::ThreadUpdate => Some(Self::THREAD_UPDATE),
            Self::ThreadDelete => Some(Self::THREAD_DELETE),
            Self::ThreadListSync => Some(Self::THREAD_LIST_SYNC),
            Self::ThreadMemberUpdate => Some(Self::THREAD_MEMBER_UPDATE),
            Self::ThreadMembersUpdate => Some(Self::THREAD_MEMBERS_UPDATE),
            Self::ScheduledEventCreate => Some(Self::GUILD_SCHEDULED_EVENT_CREATE),
            Self::ScheduledEventUpdate => Some(Self::GUILD_SCHEDULED_EVENT_UPDATE),
            Self::ScheduledEventDelete => Some(Self::GUILD_SCHEDULED_EVENT_DELETE),
            Self::ScheduledEventUserAdd => Some(Self::GUILD_SCHEDULED_EVENT_USER_ADD),
            Self::ScheduledEventUserRemove => Some(Self::GUILD_SCHEDULED_EVENT_USER_REMOVE),
            Self::Other => None,
        }
    }

    with_related_ids_for_event_types!(define_related_ids_for_event_type);
}
