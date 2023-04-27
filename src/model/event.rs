//! All the events this library handles.
//!
//! Every event includes the gateway intent required to receive it, as well as a link to the
//! Discord documentation for the event.

// Just for MessageUpdateEvent (for some reason the #[allow] doesn't work when placed directly)
#![allow(clippy::option_option)]

use std::collections::HashMap;

use serde::de::Error as DeError;
use serde::Serialize;

use super::application::ActionRow;
use super::prelude::*;
use super::utils::{
    deserialize_val,
    emojis,
    ignore_input,
    remove_from_map,
    remove_from_map_opt,
    stickers,
};
use crate::constants::Opcode;
use crate::internal::prelude::*;
use crate::model::application::{CommandPermission, Interaction};
use crate::model::guild::audit_log::AuditLogEntry;
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

/// Requires [`GatewayIntents::GUILD_MODERATION`] and [`Permissions::VIEW_AUDIT_LOG`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-audit-log-entry-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildAuditLogEntryCreateEvent {
    pub guild_id: GuildId,
    #[serde(flatten)]
    pub entry: AuditLogEntry,
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

// Manual impl needed to insert guild_id fields in GuildChannel, Member, Role
impl<'de> Deserialize<'de> for GuildCreateEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut guild: Guild = Guild::deserialize(deserializer)?;
        guild.channels.values_mut().for_each(|x| x.guild_id = guild.id);
        guild.members.values_mut().for_each(|x| x.guild_id = guild.id);
        guild.roles.values_mut().for_each(|x| x.guild_id = guild.id);
        Ok(Self {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct GuildMembersChunkEvent {
    /// ID of the guild.
    pub guild_id: GuildId,
    /// Set of guild members.
    pub members: HashMap<UserId, Member>,
    /// Chunk index in the expected chunks for this response (0 <= chunk_index < chunk_count).
    pub chunk_index: u32,
    /// Total number of expected chunks for this response.
    pub chunk_count: u32,
    /// When passing an invalid ID to [`crate::gateway::ShardRunnerMessage::ChunkGuild`], it will
    /// be returned here.
    #[serde(default)]
    pub not_found: Vec<GenericId>,
    /// When passing true to [`crate::gateway::ShardRunnerMessage::ChunkGuild`], presences of the
    /// returned members will be here.
    pub presences: Option<Vec<Presence>>,
    /// Nonce used in the [`crate::gateway::ShardRunnerMessage::ChunkGuild`] request.
    pub nonce: Option<String>,
}

// Manual impl needed to insert guild_id fields in Member
impl<'de> Deserialize<'de> for GuildMembersChunkEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut event = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        event.members.values_mut().for_each(|m| m.guild_id = event.guild_id);
        Ok(event)
    }
}

impl Serialize for GuildMembersChunkEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

/// Helper to deserialize `GuildRoleCreateEvent` and `GuildRoleUpdateEvent`.
#[derive(Deserialize)]
struct RoleEventHelper {
    guild_id: GuildId,
    role: Role,
}

/// Requires [`GatewayIntents::GUILDS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-role-create).
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
    /// Whether or not the invite is temporary (invited users will be kicked on disconnect unless
    /// Channel the invite is for.
    pub channel_id: ChannelId,
    /// Unique invite [code](Invite::code).
    pub code: String,
    /// Time at which the invite was created.
    pub created_at: Timestamp,
    /// Guild of the invite.
    pub guild_id: Option<GuildId>,
    /// User that created the invite.
    pub inviter: Option<User>,
    /// How long the invite is valid for (in seconds).
    pub max_age: u64,
    /// Maximum number of times the invite can be used.
    pub max_uses: u64,
    /// Type of target for this voice channel invite.
    pub target_type: Option<InviteTargetType>,
    /// User whose stream to display for this voice channel stream invite.
    pub target_user: Option<User>,
    /// Embedded application to open for this voice channel embedded application invite.
    pub target_application: Option<serde_json::Value>,
    /// they're assigned a role).
    pub temporary: bool,
    /// How many times the invite has been used (always will be 0).
    pub uses: u64,
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
    pub guild: Guild,
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

// Any value that is present is considered Some value, including null.
// Taken from https://github.com/serde-rs/serde/issues/984#issuecomment-314143738
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// Requires [`GatewayIntents::GUILD_MESSAGES`].
///
/// Contains identical fields to [`Message`], except everything but `id` and `channel_id` are
/// optional.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageUpdateEvent {
    pub id: MessageId,
    pub channel_id: ChannelId,
    // pub author: User, - cannot be edited
    pub content: Option<String>,
    // pub timestamp: Timestamp, - cannot be edited
    pub edited_timestamp: Option<Timestamp>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub mention_channels: Option<Vec<ChannelMention>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Embed>>,
    pub reactions: Option<Vec<MessageReaction>>,
    pub pinned: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub webhook_id: Option<Option<WebhookId>>,
    // #[serde(rename = "type")] pub kind: MessageType, - cannot be edited
    #[serde(default, deserialize_with = "deserialize_some")]
    pub activity: Option<Option<MessageActivity>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub application: Option<Option<MessageApplication>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub application_id: Option<Option<ApplicationId>>,
    // pub message_reference: Option<MessageReference>, - cannot be edited
    #[serde(default, deserialize_with = "deserialize_some")]
    pub flags: Option<Option<MessageFlags>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub referenced_message: Option<Option<Box<Message>>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub interaction: Option<Option<Box<MessageInteraction>>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub thread: Option<Option<GuildChannel>>,
    pub components: Option<Vec<ActionRow>>,
    pub sticker_items: Option<Vec<StickerItem>>,
    pub position: Option<Option<u64>>,
    // pub role_subscription_data: Option<RoleSubscriptionData>, - cannot be edited
    pub guild_id: GuildId,          // not wrapped in Option, unlike Message!
    pub member: Box<PartialMember>, // not wrapped in Option, unlike Message!
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

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-add).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ReactionAddEvent {
    pub reaction: Reaction,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-remove).
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
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-reaction-remove-all).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ReactionRemoveAllEvent {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
}

/// Requires [`GatewayIntents::GUILD_MESSAGE_REACTIONS`] or
/// [`GatewayIntents::DIRECT_MESSAGE_REACTIONS`].
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
pub struct ResumedEvent {}

/// Requires [`GatewayIntents::GUILD_MESSAGE_TYPING`] or [`GatewayIntents::DIRECT_MESSAGE_TYPING`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#typing-start).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TypingStartEvent {
    /// ID of the channel.
    pub channel_id: ChannelId,
    /// ID of the guild.
    pub guild_id: Option<GuildId>,
    /// ID of the user.
    pub user_id: UserId,
    /// Timestamp of when the user started typing.
    pub timestamp: u64,
    /// Member who started typing if this happened in a guild.
    pub member: Option<Member>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnknownEvent {
    pub kind: String,
    pub value: Value,
}

/// Sent when properties about the current bot's user change.
///
/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#user-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct UserUpdateEvent {
    pub current_user: User,
}

/// Requires no gateway intents.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#voice-server-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceServerUpdateEvent {
    pub token: String,
    pub guild_id: Option<GuildId>,
    pub endpoint: Option<String>,
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
    /// The parent channel Id whose threads are being synced. If omitted, then threads were synced
    /// for the entire guild. This array may contain channel Ids that have no active threads as
    /// well, so you know to clear that data.
    pub channel_ids: Option<Vec<ChannelId>>,
    /// All active threads in the given channels that the current user can access.
    pub threads: Vec<GuildChannel>,
    /// All thread member objects from the synced threads for the current user, indicating which
    /// threads the current user has been added to
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
    ///
    /// NOTE: This count has been observed to be above 50, or below 0.
    /// See: <https://github.com/discord/discord-api-docs/issues/5139>
    pub member_count: i16,
    /// The users who were added to the thread.
    #[serde(default)]
    pub added_members: Vec<ThreadMember>,
    /// The ids of the users who were removed from the thread.
    #[serde(default)]
    pub removed_member_ids: Vec<UserId>,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-create).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventCreateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-update).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventUpdateEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-delete).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GuildScheduledEventDeleteEvent {
    pub event: ScheduledEvent,
}

/// Requires [`GatewayIntents::GUILD_SCHEDULED_EVENTS`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-user-add).
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
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#guild-scheduled-event-user-remove).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildScheduledEventUserRemoveEvent {
    #[serde(rename = "guild_scheduled_event_id")]
    pub scheduled_event_id: ScheduledEventId,
    pub user_id: UserId,
    pub guild_id: GuildId,
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#payload-structure).
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

// Manual impl needed to emulate integer enum tags
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
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#receive-events).
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "t", content = "d")]
#[non_exhaustive]
pub enum Event {
    /// The permissions of an [`Command`] was changed.
    ///
    /// Fires the [`EventHandler::command_permissions_update`] event.
    ///
    /// [`Command`]: crate::model::application::Command
    /// [`EventHandler::command_permissions_update`]: crate::client::EventHandler::command_permissions_update
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
    GuildScheduledEventCreate(GuildScheduledEventCreateEvent),
    /// A scheduled event was updated.
    GuildScheduledEventUpdate(GuildScheduledEventUpdateEvent),
    /// A scheduled event was deleted.
    GuildScheduledEventDelete(GuildScheduledEventDeleteEvent),
    /// A guild member has subscribed to a scheduled event.
    GuildScheduledEventUserAdd(GuildScheduledEventUserAddEvent),
    /// A guild member has unsubscribed from a scheduled event.
    GuildScheduledEventUserRemove(GuildScheduledEventUserRemoveEvent),
    /// An event type not covered by the above
    #[serde(other, deserialize_with = "ignore_input")]
    Unknown,
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
            Self::GuildAuditLogEntryCreate(_) => EventType::GuildAuditLogEntryCreate,
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
            Self::GuildScheduledEventCreate(_) => EventType::GuildScheduledEventCreate,
            Self::GuildScheduledEventUpdate(_) => EventType::GuildScheduledEventUpdate,
            Self::GuildScheduledEventDelete(_) => EventType::GuildScheduledEventDelete,
            Self::GuildScheduledEventUserAdd(_) => EventType::GuildScheduledEventUserAdd,
            Self::GuildScheduledEventUserRemove(_) => EventType::GuildScheduledEventUserRemove,
            Self::Unknown => EventType::Other,
        }
    }
}

/// The type of event dispatch received from the gateway.
///
/// This is useful for deciding how to deserialize a received payload.
///
/// A Deserialization implementation is provided for deserializing raw event dispatch type strings
/// to this enum, e.g. deserializing `"CHANNEL_CREATE"` to [`EventType::ChannelCreate`].
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
    /// Indicator that a new audit log entry was created.
    ///
    /// This maps to [`GuildAuditLogEntryCreateEvent`].
    GuildAuditLogEntryCreate,
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
    /// This maps to [`GuildScheduledEventCreateEvent`].
    GuildScheduledEventCreate,
    /// Indicator that a scheduled event update payload was received.
    ///
    /// This maps to [`GuildScheduledEventUpdateEvent`].
    GuildScheduledEventUpdate,
    /// Indicator that a scheduled event delete payload was received.
    ///
    /// This maps to [`GuildScheduledEventDeleteEvent`].
    GuildScheduledEventDelete,
    /// Indicator that a guild member has subscribed to a scheduled event.
    ///
    /// This maps to [`GuildScheduledEventUserAddEvent`].
    GuildScheduledEventUserAdd,
    /// Indicator that a guild member has unsubscribed from a scheduled event.
    ///
    /// This maps to [`GuildScheduledEventUserRemoveEvent`].
    GuildScheduledEventUserRemove,
    /// An unknown event was received over the gateway.
    Other,
}

impl From<&Event> for EventType {
    fn from(event: &Event) -> EventType {
        event.event_type()
    }
}

impl EventType {
    /// Return the event name of this event. Some events are synthetic, and we lack the information
    /// to recover the original event name for these events, in which case this method returns
    /// [`None`].
    #[must_use]
    pub const fn name(&self) -> Option<&str> {
        match self {
            Self::CommandPermissionsUpdate => Some("APPLICATION_COMMAND_PERMISSIONS_UPDATE"),
            Self::AutoModRuleCreate => Some("AUTO_MODERATION_RULE_CREATE"),
            Self::AutoModRuleUpdate => Some("AUTO_MODERATION_RULE_UPDATE"),
            Self::AutoModRuleDelete => Some("AUTO_MODERATION_RULE_DELETE"),
            Self::AutoModActionExecution => Some("AUTO_MODERATION_ACTION_EXECUTION"),
            Self::ChannelCreate => Some("CHANNEL_CREATE"),
            Self::ChannelDelete => Some("CHANNEL_DELETE"),
            Self::ChannelPinsUpdate => Some("CHANNEL_PINS_UPDATE"),
            Self::ChannelUpdate => Some("CHANNEL_UPDATE"),
            Self::GuildAuditLogEntryCreate => Some("GUILD_AUDIT_LOG_ENTRY_CREATE"),
            Self::GuildBanAdd => Some("GUILD_BAN_ADD"),
            Self::GuildBanRemove => Some("GUILD_BAN_REMOVE"),
            Self::GuildCreate => Some("GUILD_CREATE"),
            Self::GuildDelete => Some("GUILD_DELETE"),
            Self::GuildEmojisUpdate => Some("GUILD_EMOJIS_UPDATE"),
            Self::GuildIntegrationsUpdate => Some("GUILD_INTEGRATIONS_UPDATE"),
            Self::GuildMemberAdd => Some("GUILD_MEMBER_ADD"),
            Self::GuildMemberRemove => Some("GUILD_MEMBER_REMOVE"),
            Self::GuildMemberUpdate => Some("GUILD_MEMBER_UPDATE"),
            Self::GuildMembersChunk => Some("GUILD_MEMBERS_CHUNK"),
            Self::GuildRoleCreate => Some("GUILD_ROLE_CREATE"),
            Self::GuildRoleDelete => Some("GUILD_ROLE_DELETE"),
            Self::GuildRoleUpdate => Some("GUILD_ROLE_UPDATE"),
            Self::GuildStickersUpdate => Some("GUILD_STICKERS_UPDATE"),
            Self::InviteCreate => Some("INVITE_CREATE"),
            Self::InviteDelete => Some("INVITE_DELETE"),
            Self::GuildUpdate => Some("GUILD_UPDATE"),
            Self::MessageCreate => Some("MESSAGE_CREATE"),
            Self::MessageDelete => Some("MESSAGE_DELETE"),
            Self::MessageDeleteBulk => Some("MESSAGE_DELETE_BULK"),
            Self::ReactionAdd => Some("MESSAGE_REACTION_ADD"),
            Self::ReactionRemove => Some("MESSAGE_REACTION_REMOVE"),
            Self::ReactionRemoveAll => Some("MESSAGE_REACTION_REMOVE_ALL"),
            Self::ReactionRemoveEmoji => Some("MESSAGE_REACTION_REMOVE_ALL_EMOJI"),
            Self::MessageUpdate => Some("MESSAGE_UPDATE"),
            Self::PresenceUpdate => Some("PRESENCE_UPDATE"),
            Self::PresencesReplace => Some("PRESENCES_REPLACE"),
            Self::Ready => Some("READY"),
            Self::Resumed => Some("RESUMED"),
            Self::TypingStart => Some("TYPING_START"),
            Self::UserUpdate => Some("USER_UPDATE"),
            Self::VoiceServerUpdate => Some("VOICE_SERVER_UPDATE"),
            Self::VoiceStateUpdate => Some("VOICE_STATE_UPDATE"),
            Self::WebhookUpdate => Some("WEBHOOKS_UPDATE"),
            Self::InteractionCreate => Some("INTERACTION_CREATE"),
            Self::IntegrationCreate => Some("INTEGRATION_CREATE"),
            Self::IntegrationUpdate => Some("INTEGRATION_UPDATE"),
            Self::IntegrationDelete => Some("INTEGRATION_DELETE"),
            Self::StageInstanceCreate => Some("STAGE_INSTANCE_CREATE"),
            Self::StageInstanceUpdate => Some("STAGE_INSTANCE_UPDATE"),
            Self::StageInstanceDelete => Some("STAGE_INSTANCE_DELETE"),
            Self::ThreadCreate => Some("THREAD_CREATE"),
            Self::ThreadUpdate => Some("THREAD_UPDATE"),
            Self::ThreadDelete => Some("THREAD_DELETE"),
            Self::ThreadListSync => Some("THREAD_LIST_SYNC"),
            Self::ThreadMemberUpdate => Some("THREAD_MEMBER_UPDATE"),
            Self::ThreadMembersUpdate => Some("THREAD_MEMBERS_UPDATE"),
            Self::GuildScheduledEventCreate => Some("GUILD_SCHEDULED_EVENT_CREATE"),
            Self::GuildScheduledEventUpdate => Some("GUILD_SCHEDULED_EVENT_UPDATE"),
            Self::GuildScheduledEventDelete => Some("GUILD_SCHEDULED_EVENT_DELETE"),
            Self::GuildScheduledEventUserAdd => Some("GUILD_SCHEDULED_EVENT_USER_ADD"),
            Self::GuildScheduledEventUserRemove => Some("GUILD_SCHEDULED_EVENT_USER_REMOVE"),
            Self::Other => None,
        }
    }
}
