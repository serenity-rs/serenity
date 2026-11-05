//! All the events this library handles.
//!
//! Every event includes the gateway intent required to receive it, as well as a link to the
//! Discord documentation for the event.

mod full_event;

use serde::de::Error as DeError;
use serde::{Serialize, Serializer};
use serde_json::value::RawValue;
use strum::{EnumCount, IntoStaticStr, VariantNames};

pub use self::full_event::*;
use crate::constants::Opcode;
use crate::model::prelude::*;
use crate::model::utils::deserialize_null_as_default;

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#application-command-permissions-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct CommandPermissionsUpdateEvent {
    pub permission: CommandPermissions,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#auto-moderation-rule-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleCreateEvent {
    pub rule: AutoModRule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#auto-moderation-rule-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleUpdateEvent {
    pub rule: AutoModRule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_CONFIGURATION`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#auto-moderation-rule-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModRuleDeleteEvent {
    pub rule: AutoModRule,
}

/// Requires [`GatewayIntents::AUTO_MODERATION_EXECUTION`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#auto-moderation-action-execution).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct AutoModActionExecutionEvent {
    pub execution: ActionExecution,
}

/// Event data for the channel creation event.
///
/// This is fired when:
/// - A [`Channel`] is created in a [`Guild`]
///
/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelCreateEvent {
    /// The channel that was created.
    pub channel: GuildChannel,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelDeleteEvent {
    pub channel: GuildChannel,
}

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-info).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelInfoEvent {
    pub guild_id: GuildId,
    pub channels: Vec<ChannelInfoChannel>,
}

/// Requires [`GatewayIntents::GUILDS`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-pins-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelPinsUpdateEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: GenericChannelId,
    pub last_pin_timestamp: Option<Timestamp>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ChannelUpdateEvent {
    pub channel: GuildChannel,
}

/// Requires [`GatewayIntents::GUILD_MODERATION`] and [`Permissions::VIEW_AUDIT_LOG`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-audit-log-entry-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildAuditLogEntryCreateEvent {
    pub guild_id: GuildId,
    #[serde(flatten)]
    pub entry: AuditLogEntry,
}

/// Requires [`GatewayIntents::GUILD_MODERATION`] or [`Permissions::VIEW_AUDIT_LOG`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-ban-add).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanAddEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILD_MODERATION`] or [`Permissions::VIEW_AUDIT_LOG`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-ban-remove).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildBanRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildCreateEvent {
    pub guild: GuildCreateGuild,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildDeleteEvent {
    pub guild: UnavailableGuild,
}

/// Requires [`GatewayIntents::GUILD_EMOJIS_AND_STICKERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-emojis-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildEmojisUpdateEvent {
    pub emojis: ExtractMap<EmojiId, Emoji>,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-integrations-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildIntegrationsUpdateEvent {
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-member-add).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildMemberAddEvent {
    pub member: Member,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-member-remove).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildMemberRemoveEvent {
    pub guild_id: GuildId,
    pub user: User,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-member-update).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct GuildMemberUpdateEvent {
    pub guild_id: GuildId,
    pub nick: Option<FixedString<u8>>,
    pub joined_at: Option<Timestamp>,
    pub roles: FixedArray<RoleId>,
    pub user: User,
    pub premium_since: Option<Timestamp>,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    pub avatar: Option<ImageHash>,
    pub banner: Option<ImageHash>,
    pub communication_disabled_until: Option<Timestamp>,
    // This is not documented but present on the event?
    pub unusual_dm_activity_until: Option<Timestamp>,
    pub flags: Option<GuildMemberFlags>,
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_as_default")]
    pub collectibles: Collectibles,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-members-chunk).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct GuildMembersChunkEvent {
    /// ID of the guild.
    pub guild_id: GuildId,
    /// Set of guild members.
    pub members: ExtractMap<UserId, Member>,
    /// Chunk index in the expected chunks for this response (0 <= chunk_index < chunk_count).
    pub chunk_index: u32,
    /// Total number of expected chunks for this response.
    pub chunk_count: u32,
    /// When passing an invalid ID to [`crate::gateway::ShardRunnerMessage::ChunkGuild`], it will
    /// be returned here.
    #[serde(default)]
    pub not_found: FixedArray<GenericId>,
    /// When passing true to [`crate::gateway::ShardRunnerMessage::ChunkGuild`], presences of the
    /// returned members will be here.
    pub presences: Option<Vec<Presence>>,
    /// Nonce used in the [`crate::gateway::ShardRunnerMessage::ChunkGuild`] request.
    pub nonce: Option<FixedString>,
}

// Manual impl needed to insert guild_id fields in Member
impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut event = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        event.members.iter_mut().for_each(|m| m.guild_id = event.guild_id);
        Ok(event)
    }
}

impl Serialize for GuildMembersChunkEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#soundboard-sounds).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SoundboardSoundsEvent {
    /// ID of the guild.
    pub guild_id: GuildId,
    /// The guild's soundboard sounds.
    pub soundboard_sounds: Vec<Soundboard>,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-soundboard-sound-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct SoundboardSoundCreateEvent {
    pub soundboard: Soundboard,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-soundboard-sound-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct SoundboardSoundUpdateEvent {
    pub soundboard: Soundboard,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-soundboard-sounds-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SoundboardSoundsUpdateEvent {
    pub guild_id: GuildId,
    pub soundboard_sounds: Vec<Soundboard>,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-soundboard-sound-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SoundboardSoundDeleteEvent {
    pub guild_id: GuildId,
    pub sound_id: SoundId,
}

/// Helper to deserialize `GuildRoleCreateEvent` and `GuildRoleUpdateEvent`.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Deserialize)]
struct RoleEventHelper {
    guild_id: GuildId,
    role: Role,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-role-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleCreateEvent {
    pub role: Role,
}

// Manual impl needed to insert guild_id field in Role
impl<'de> Deserialize<'de> for GuildRoleCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut event = RoleEventHelper::deserialize(deserializer)?;
        event.role.guild_id = event.guild_id;
        Ok(Self {
            role: event.role,
        })
    }
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-role-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildRoleDeleteEvent {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-role-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct GuildRoleUpdateEvent {
    pub role: Role,
}

// Manual impl needed to insert guild_id field in Role
impl<'de> Deserialize<'de> for GuildRoleUpdateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut event = RoleEventHelper::deserialize(deserializer)?;
        event.role.guild_id = event.guild_id;
        Ok(Self {
            role: event.role,
        })
    }
}

/// Requires [`GatewayIntents::GUILD_EMOJIS_AND_STICKERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-stickers-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildStickersUpdateEvent {
    pub stickers: ExtractMap<StickerId, Sticker>,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_INVITES`] and [`Permissions::MANAGE_CHANNELS`] permission.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#invite-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteCreateEvent {
    /// Whether or not the invite is temporary (invited users will be kicked on disconnect unless
    /// Channel the invite is for.
    pub channel_id: ChannelId,
    /// Unique invite [code](Invite::code).
    pub code: FixedString,
    /// Time at which the invite was created.
    pub created_at: Timestamp,
    /// Guild of the invite.
    pub guild_id: Option<GuildId>,
    /// User that created the invite.
    pub inviter: Option<User>,
    /// How long the invite is valid for (in seconds).
    pub max_age: u32,
    /// Maximum number of times the invite can be used.
    pub max_uses: u8,
    /// Type of target for this voice channel invite.
    pub target_type: Option<InviteTargetType>,
    /// User whose stream to display for this voice channel stream invite.
    pub target_user: Option<User>,
    /// Embedded application to open for this voice channel embedded application invite.
    pub target_application: Option<Value>,
    /// they're assigned a role).
    pub temporary: bool,
    /// How many times the invite has been used (always will be 0).
    pub uses: u64,
}

/// Requires [`GatewayIntents::GUILD_INVITES`] and [`Permissions::MANAGE_CHANNELS`] permission.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#invite-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteDeleteEvent {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub code: FixedString,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildUpdateEvent {
    /// GuildUpdateEvent doesn't have GuildCreate's extra fields, so this is a partial guild
    pub guild: PartialGuild,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct MessageCreateEvent {
    pub message: Message,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-delete-bulk).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteBulkEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: GenericChannelId,
    pub ids: FixedArray<MessageId>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`] or [`GatewayIntents::DIRECT_MESSAGES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageDeleteEvent {
    pub guild_id: Option<GuildId>,
    pub channel_id: GenericChannelId,
    #[serde(rename = "id")]
    pub message_id: MessageId,
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
// This ensures that `RawValue` is further supported in nested fields of `Message`.
// Fields underneath #[serde(flatten)] cannot be deserialized as `RawValue`.
#[serde(transparent)]
#[non_exhaustive]
pub struct MessageUpdateEvent {
    pub message: Message,
}

/// Requires [`GatewayIntents::GUILD_PRESENCES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#presence-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct PresenceUpdateEvent {
    pub presence: Presence,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-reaction-add).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-reaction-remove).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionRemoveEvent {
    // The Discord API doesn't share the same schema for Reaction Remove Event and Reaction Add
    // Event (which [`Reaction`] is), but the two currently match up well enough, so re-using the
    // [`Reaction`] struct here is fine.
    pub reaction: Reaction,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-reaction-remove-all).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ReactionRemoveAllEvent {
    pub channel_id: GenericChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-reaction-remove-emoji-message-reaction-remove-emoji-event-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#ready).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReadyEvent {
    pub ready: Ready,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#resumed).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ResumedEvent {}

/// Requires [`GatewayIntents::GUILD_MESSAGE_TYPING`] or [`GatewayIntents::DIRECT_MESSAGE_TYPING`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#typing-start).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TypingStartEvent {
    /// ID of the channel.
    pub channel_id: GenericChannelId,
    /// ID of the guild.
    pub guild_id: Option<GuildId>,
    /// ID of the user.
    pub user_id: UserId,
    /// Timestamp of when the user started typing.
    pub timestamp: u64,
    /// Member who started typing if this happened in a guild.
    pub member: Option<Member>,
}

/// Sent when properties about the current bot's user change.
///
/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#user-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-server-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceServerUpdateEvent {
    pub token: FixedString,
    pub guild_id: GuildId,
    pub endpoint: Option<FixedString>,
}

/// Requires [`GatewayIntents::GUILD_VOICE_STATES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-state-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct VoiceStateUpdateEvent {
    pub voice_state: VoiceState,
}

/// Requires [`GatewayIntents::GUILD_VOICE_STATES`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-channel-effect-send).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceChannelEffectSendEvent {
    #[serde(flatten)]
    pub effect: VoiceChannelEffect,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-channel-start-time-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceChannelStartTimeUpdateEvent {
    pub id: ChannelId,
    pub guild_id: GuildId,
    pub voice_start_time: Option<i64>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-channel-status-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceChannelStatusUpdateEvent {
    pub status: Option<FixedString<u16>>,
    pub id: ChannelId,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_WEBHOOKS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#webhooks-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookUpdateEvent {
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#interaction-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct InteractionCreateEvent {
    pub interaction: Interaction,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#integration-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct IntegrationCreateEvent {
    pub integration: Integration,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#integration-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct IntegrationUpdateEvent {
    pub integration: Integration,
}

/// Requires [`GatewayIntents::GUILD_INTEGRATIONS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#integration-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IntegrationDeleteEvent {
    pub id: IntegrationId,
    pub guild_id: GuildId,
    pub application_id: Option<ApplicationId>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#stage-instance-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceCreateEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#stage-instance-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceUpdateEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#stage-instance-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct StageInstanceDeleteEvent {
    pub stage_instance: StageInstance,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadCreateEvent {
    #[serde(flatten)]
    pub thread: GuildThread,
    pub newly_created: Option<bool>,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadUpdateEvent {
    pub thread: GuildThread,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadDeleteEvent {
    pub thread: PartialGuildThread,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-list-sync).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadListSyncEvent {
    /// The guild Id.
    pub guild_id: GuildId,
    /// The parent channel Id whose threads are being synced. If omitted, then threads were synced
    /// for the entire guild. This array may contain channel Ids that have no active threads as
    /// well, so you know to clear that data.
    pub channel_ids: Option<Vec<ChannelId>>,
    /// All active threads in the given channels that the current user can access.
    pub threads: ExtractMap<ThreadId, GuildThread>,
    /// All thread member objects from the synced threads for the current user, indicating which
    /// threads the current user has been added to
    pub members: FixedArray<ThreadMember>,
}

/// Requires [`GatewayIntents::GUILDS`], and, to receive this event for other users,
/// [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-member-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ThreadMemberUpdateEvent {
    pub member: ThreadMember,
}

/// Requires [`GatewayIntents::GUILD_MEMBERS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#thread-members-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMembersUpdateEvent {
    /// The id of the thread.
    pub id: ThreadId,
    /// The id of the Guild.
    pub guild_id: GuildId,
    /// The approximate number of members in the thread, capped at 50.
    ///
    /// NOTE: This count has been observed to be above 50, or below 0.
    /// See: <https://github.com/discord/discord-api-docs/issues/5139>
    pub member_count: i16,
    /// The users who were added to the thread.
    #[serde(default)]
    pub added_members: FixedArray<ThreadMember>,
    /// The ids of the users who were removed from the thread.
    #[serde(default)]
    pub removed_member_ids: FixedArray<UserId>,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-scheduled-event-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventCreateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-scheduled-event-update).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventUpdateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-scheduled-event-delete).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventDeleteEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-scheduled-event-user-add).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildScheduledEventUserAddEvent {
    #[serde(rename = "guild_scheduled_event_id")]
    pub scheduled_event_id: ScheduledEventId,
    pub user_id: UserId,
    pub guild_id: GuildId,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#guild-scheduled-event-user-remove).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildScheduledEventUserRemoveEvent {
    #[serde(rename = "guild_scheduled_event_id")]
    pub scheduled_event_id: ScheduledEventId,
    pub user_id: UserId,
    pub guild_id: GuildId,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#entitlement-create)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct EntitlementCreateEvent {
    pub entitlement: Entitlement,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#entitlement-update)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct EntitlementUpdateEvent {
    pub entitlement: Entitlement,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#entitlement-delete)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct EntitlementDeleteEvent {
    pub entitlement: Entitlement,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_POLLS`] or [`GatewayIntents::DIRECT_MESSAGE_POLLS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-poll-vote-add)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessagePollVoteAddEvent {
    pub user_id: UserId,
    pub channel_id: GenericChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
    pub answer_id: AnswerId,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_POLLS`] or [`GatewayIntents::DIRECT_MESSAGE_POLLS`].
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-poll-vote-remove)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessagePollVoteRemoveEvent {
    pub user_id: UserId,
    pub channel_id: GenericChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
    pub answer_id: AnswerId,
}

/// An internal event denoting that a shard's connection stage was changed.
///
/// # Examples
///
/// This might happen when a shard changes from [`ConnectionStage::Identifying`] to
/// [`ConnectionStage::Connected`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShardStageUpdateEvent {
    /// The new connection stage.
    pub new: ConnectionStage,
    /// The old connection stage.
    pub old: ConnectionStage,
    /// The ID of the shard that had its connection stage change.
    pub shard_id: ShardId,
}

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#payload-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GatewayEvent {
    Dispatch {
        seq: u64,
        event: DeserializedEvent,
    },
    Heartbeat,
    Reconnect,
    /// Whether the session can be resumed.
    InvalidateSession(bool),
    Hello(u64),
    HeartbeatAck,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum DeserializedEvent {
    Success(Box<Event>),
    Unknown(UnknownEvent),
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct UnknownEvent {
    #[cfg_attr(feature = "typesize", typesize(with = raw_value_len))]
    pub data: Box<RawValue>,
    pub err: String,
}

impl Serialize for UnknownEvent {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "typesize")]
fn raw_value_len(val: &RawValue) -> usize {
    val.get().len()
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct GatewayEventRaw<'a> {
            op: Opcode,
            #[serde(rename = "s")]
            seq: Option<u64>,
            #[serde(rename = "d")]
            data: &'a RawValue,
            #[serde(rename = "t")]
            ty: Option<&'a str>,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;

        let raw = GatewayEventRaw::deserialize(raw_data).map_err(DeError::custom)?;

        Ok(match raw.op {
            Opcode::Dispatch => {
                if raw.ty.is_none() {
                    return Err(DeError::missing_field("t"));
                }

                Self::Dispatch {
                    seq: raw.seq.ok_or_else(|| DeError::missing_field("s"))?,
                    event: match Deserialize::deserialize(raw_data) {
                        Ok(event) => DeserializedEvent::Success(event),
                        Err(e) => DeserializedEvent::Unknown(UnknownEvent {
                            data: Deserialize::deserialize(raw_data).map_err(DeError::custom)?,
                            err: e.to_string(),
                        }),
                    },
                }
            },
            Opcode::Heartbeat => Self::Heartbeat,
            Opcode::InvalidSession => {
                Self::InvalidateSession(bool::deserialize(raw.data).map_err(DeError::custom)?)
            },
            Opcode::Hello => {
                #[derive(Deserialize)]
                struct HelloPayload {
                    heartbeat_interval: u64,
                }

                let inner = HelloPayload::deserialize(raw.data).map_err(DeError::custom)?;

                Self::Hello(inner.heartbeat_interval)
            },
            Opcode::Reconnect => Self::Reconnect,
            Opcode::HeartbeatAck => Self::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}

/// Event received over a websocket connection
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#receive-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, EnumCount, VariantNames, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "t", content = "d")]
#[non_exhaustive]
pub enum Event {
    /// The permissions of an [`Command`] was changed.
    #[serde(rename = "APPLICATION_COMMAND_PERMISSIONS_UPDATE")]
    CommandPermissionsUpdate(CommandPermissionsUpdateEvent),
    /// A [`AutoModRule`] was created.
    #[serde(rename = "AUTO_MODERATION_RULE_CREATE")]
    AutoModRuleCreate(AutoModRuleCreateEvent),
    /// A [`AutoModRule`] has been updated.
    #[serde(rename = "AUTO_MODERATION_RULE_UPDATE")]
    AutoModRuleUpdate(AutoModRuleUpdateEvent),
    /// A [`AutoModRule`] was deleted.
    #[serde(rename = "AUTO_MODERATION_RULE_DELETE")]
    AutoModRuleDelete(AutoModRuleDeleteEvent),
    /// A [`AutoModRule`] was triggered and an action was executed.
    #[serde(rename = "AUTO_MODERATION_ACTION_EXECUTION")]
    AutoModActionExecution(AutoModActionExecutionEvent),
    /// A [`Channel`] was created.
    ChannelCreate(ChannelCreateEvent),
    /// A [`Channel`] has been deleted.
    ChannelDelete(ChannelDeleteEvent),
    /// Sent in response to [Opcode::RequestChannelInfo].
    ChannelInfo(ChannelInfoEvent),
    /// The pins for a [`Channel`] have been updated.
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// A [`Channel`] has been updated.
    ChannelUpdate(ChannelUpdateEvent),
    GuildAuditLogEntryCreate(GuildAuditLogEntryCreateEvent),
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
    InviteCreate(InviteCreateEvent),
    /// An [`Invite`] was deleted.
    InviteDelete(InviteDeleteEvent),
    MessageCreate(MessageCreateEvent),
    MessageDelete(MessageDeleteEvent),
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// A message has been edited, either by the user or the system
    MessageUpdate(MessageUpdateEvent),
    /// A member's presence state (or username or avatar) has changed
    PresenceUpdate(PresenceUpdateEvent),
    /// A reaction was added to a message.
    #[serde(rename = "MESSAGE_REACTION_ADD")]
    ReactionAdd(ReactionAddEvent),
    /// A reaction was removed to a message.
    #[serde(rename = "MESSAGE_REACTION_REMOVE")]
    ReactionRemove(ReactionRemoveEvent),
    /// A request was issued to remove all [`Reaction`]s from a [`Message`].
    #[serde(rename = "MESSAGE_REACTION_REMOVE_ALL")]
    ReactionRemoveAll(ReactionRemoveAllEvent),
    /// Sent when a bot removes all instances of a given emoji from the reactions of a message.
    #[serde(rename = "MESSAGE_REACTION_REMOVE_EMOJI")]
    ReactionRemoveEmoji(ReactionRemoveEmojiEvent),
    /// The first event in a connection, containing the initial ready cache.
    ///
    /// May also be received at a later time in the event of a reconnect.
    Ready(ReadyEvent),
    /// The connection has successfully resumed after a disconnect.
    Resumed(ResumedEvent),
    SoundboardSounds(SoundboardSoundsEvent),
    #[serde(rename = "GUILD_SOUNDBOARD_SOUND_CREATE")]
    SoundboardSoundCreate(SoundboardSoundCreateEvent),
    #[serde(rename = "GUILD_SOUNDBOARD_SOUND_UPDATE")]
    SoundboardSoundUpdate(SoundboardSoundUpdateEvent),
    #[serde(rename = "GUILD_SOUNDBOARD_SOUNDS_UPDATE")]
    SoundboardSoundsUpdate(SoundboardSoundsUpdateEvent),
    #[serde(rename = "GUILD_SOUNDBOARD_SOUND_DELETE")]
    SoundboardSoundDelete(SoundboardSoundDeleteEvent),
    /// A user is typing; considered to last 5 seconds
    TypingStart(TypingStartEvent),
    /// Update to the logged-in user's information
    UserUpdate(UserUpdateEvent),
    /// A member's voice state has changed
    VoiceStateUpdate(VoiceStateUpdateEvent),
    /// Voice server information is available
    VoiceServerUpdate(VoiceServerUpdateEvent),
    /// Sent when someone sends an effect, such as an emoji reaction or a soundboard sound,
    /// in a voice channel the current user is connected to.
    VoiceChannelEffectSend(VoiceChannelEffectSendEvent),
    /// Sent when the voice channel start time changes.
    VoiceChannelStartTimeUpdate(VoiceChannelStartTimeUpdateEvent),
    /// Sent when the voice channel status changes.
    VoiceChannelStatusUpdate(VoiceChannelStatusUpdateEvent),
    /// A webhook for a [channel][`GuildChannel`] was updated in a [`Guild`].
    #[serde(rename = "WEBHOOKS_UPDATE")]
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
    GuildScheduledEventCreate(GuildScheduledEventCreateEvent),
    /// A scheduled event was updated.
    GuildScheduledEventUpdate(GuildScheduledEventUpdateEvent),
    /// A scheduled event was deleted.
    GuildScheduledEventDelete(GuildScheduledEventDeleteEvent),
    /// A guild member has subscribed to a scheduled event.
    GuildScheduledEventUserAdd(GuildScheduledEventUserAddEvent),
    /// A guild member has unsubscribed from a scheduled event.
    GuildScheduledEventUserRemove(GuildScheduledEventUserRemoveEvent),
    /// A user subscribed to a SKU.
    EntitlementCreate(EntitlementCreateEvent),
    /// A user's entitlement was updated or renewed.
    EntitlementUpdate(EntitlementUpdateEvent),
    /// A user's entitlement was deleted by Discord, or refunded.
    EntitlementDelete(EntitlementDeleteEvent),
    /// A user has voted on a Message Poll.
    MessagePollVoteAdd(MessagePollVoteAddEvent),
    /// A user has removed a previous vote on a Message Poll.
    MessagePollVoteRemove(MessagePollVoteRemoveEvent),
    /// A shard has changed its connection stage.
    ShardStageUpdate(ShardStageUpdateEvent),
}

impl Event {
    /// Returns the event name of this event.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.into()
    }
}
