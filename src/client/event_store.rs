use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use super::context::Context;
use ::model::event::{
    ChannelPinsAckEvent,
    ChannelPinsUpdateEvent,
    GuildSyncEvent,
    MessageUpdateEvent,
    PresenceUpdateEvent,
    ResumedEvent,
    TypingStartEvent,
    VoiceServerUpdateEvent,
};
use ::model::*;

// This should use type macros when stable receives the type macro
// stabilization patch.
//
// This implementation should be:
//
// ```rust,ignore
// macro_rules! efn {
//     ($def:ty) => {
//         Option<Arc<Box<$def> + Send + Sync + 'static>>
//     }
// }
// ```
//
// Where each field will look like:
//
// ```rust,ignore
// pub something: efn!(Fn(Context, ...)),
// ```
#[allow(type_complexity)]
#[derive(Default)]
pub struct EventStore {
    pub on_call_create: Option<Arc<Fn(Context, Call) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_call_delete: Option<Arc<Fn(Context, ChannelId, Option<Call>) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_call_delete: Option<Arc<Fn(Context, ChannelId) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_call_update: Option<Arc<Fn(Context, Option<Call>, Option<Call>) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_call_update: Option<Arc<Fn(Context, CallUpdateEvent) + Send + Sync + 'static>>,
    pub on_channel_create: Option<Arc<Fn(Context, Channel) + Send + Sync + 'static>>,
    pub on_channel_delete: Option<Arc<Fn(Context, Channel) + Send + Sync + 'static>>,
    pub on_channel_pins_ack: Option<Arc<Fn(Context, ChannelPinsAckEvent) + Send + Sync + 'static>>,
    pub on_channel_pins_update: Option<Arc<Fn(Context, ChannelPinsUpdateEvent) + Send + Sync + 'static>>,
    pub on_channel_recipient_addition: Option<Arc<Fn(Context, ChannelId, User) + Send + Sync + 'static>>,
    pub on_channel_recipient_removal: Option<Arc<Fn(Context, ChannelId, User) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_channel_update: Option<Arc<Fn(Context, Option<Channel>, Channel) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_channel_update: Option<Arc<Fn(Context, Channel) + Send + Sync + 'static>>,
    pub on_friend_suggestion_create: Option<Arc<Fn(Context, User, Vec<SuggestionReason>) + Send + Sync + 'static>>,
    pub on_friend_suggestion_delete: Option<Arc<Fn(Context, UserId) + Send + Sync + 'static>>,
    pub on_guild_ban_addition: Option<Arc<Fn(Context, GuildId, User) + Send + Sync + 'static>>,
    pub on_guild_ban_removal: Option<Arc<Fn(Context, GuildId, User) + Send + Sync + 'static>>,
    pub on_guild_create: Option<Arc<Fn(Context, Guild) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_delete: Option<Arc<Fn(Context, PartialGuild, Option<Guild>) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_delete: Option<Arc<Fn(Context, PartialGuild) + Send + Sync + 'static>>,
    pub on_guild_emojis_update: Option<Arc<Fn(Context, GuildId, HashMap<EmojiId, Emoji>) + Send + Sync + 'static>>,
    pub on_guild_integrations_update: Option<Arc<Fn(Context, GuildId) + Send + Sync + 'static>>,
    pub on_guild_member_addition: Option<Arc<Fn(Context, GuildId, Member) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_member_removal: Option<Arc<Fn(Context, GuildId, User, Option<Member>) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_member_removal: Option<Arc<Fn(Context, GuildId, User) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_member_update: Option<Arc<Fn(Context, Option<Member>, Member) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_member_update: Option<Arc<Fn(Context, GuildMemberUpdateEvent) + Send + Sync + 'static>>,
    pub on_guild_members_chunk: Option<Arc<Fn(Context, GuildId, HashMap<UserId, Member>) + Send + Sync + 'static>>,
    pub on_guild_role_create: Option<Arc<Fn(Context, GuildId, Role) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_role_delete: Option<Arc<Fn(Context, GuildId, RoleId, Option<Role>) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_role_delete: Option<Arc<Fn(Context, GuildId, RoleId) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_role_update: Option<Arc<Fn(Context, GuildId, Option<Role>, Role) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_role_update: Option<Arc<Fn(Context, GuildId, Role) + Send + Sync + 'static>>,
    pub on_guild_sync: Option<Arc<Fn(Context, GuildSyncEvent) + Send + Sync + 'static>>,
    pub on_guild_unavailable: Option<Arc<Fn(Context, GuildId) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_guild_update: Option<Arc<Fn(Context, Option<Guild>, PartialGuild) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_guild_update: Option<Arc<Fn(Context, PartialGuild) + Send + Sync + 'static>>,
    pub on_message: Option<Arc<Fn(Context, Message) + Send + Sync + 'static>>,
    pub on_message_ack: Option<Arc<Fn(Context, ChannelId, Option<MessageId>) + Send + Sync + 'static>>,
    pub on_message_delete: Option<Arc<Fn(Context, ChannelId, MessageId) + Send + Sync + 'static>>,
    pub on_message_delete_bulk: Option<Arc<Fn(Context, ChannelId, Vec<MessageId>) + Send + Sync + 'static>>,
    pub on_reaction_add: Option<Arc<Fn(Context, Reaction) + Send + Sync + 'static>>,
    pub on_reaction_remove: Option<Arc<Fn(Context, Reaction) + Send + Sync + 'static>>,
    pub on_reaction_remove_all: Option<Arc<Fn(Context, ChannelId, MessageId) + Send + Sync + 'static>>,
    pub on_message_update: Option<Arc<Fn(Context, MessageUpdateEvent) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_note_update: Option<Arc<Fn(Context, UserId, Option<String>, String) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_note_update: Option<Arc<Fn(Context, UserId, String) + Send + Sync + 'static>>,
    pub on_presence_replace: Option<Arc<Fn(Context, Vec<Presence>) + Send + Sync + 'static>>,
    pub on_presence_update: Option<Arc<Fn(Context, PresenceUpdateEvent) + Send + Sync + 'static>>,
    pub on_ready: Option<Arc<Fn(Context, Ready) + Send + Sync + 'static>>,
    pub on_relationship_addition: Option<Arc<Fn(Context, Relationship) + Send + Sync + 'static>>,
    pub on_relationship_removal: Option<Arc<Fn(Context, UserId, RelationshipType) + Send + Sync + 'static>>,
    pub on_resume: Option<Arc<Fn(Context, ResumedEvent) + Send + Sync + 'static>>,
    pub on_typing_start: Option<Arc<Fn(Context, TypingStartEvent) + Send + Sync + 'static>>,
    pub on_unknown: Option<Arc<Fn(Context, String, BTreeMap<String, Value>) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_user_guild_settings_update: Option<Arc<Fn(Context, Option<UserGuildSettings>, UserGuildSettings) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_user_guild_settings_update: Option<Arc<Fn(Context, UserGuildSettings) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_user_update: Option<Arc<Fn(Context, CurrentUser, CurrentUser) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_user_update: Option<Arc<Fn(Context, CurrentUser) + Send + Sync + 'static>>,
    #[cfg(feature = "cache")]
    pub on_user_settings_update: Option<Arc<Fn(Context, UserSettings, UserSettings) + Send + Sync + 'static>>,
    #[cfg(not(feature = "cache"))]
    pub on_user_settings_update: Option<Arc<Fn(Context, UserSettingsUpdateEvent) + Send + Sync + 'static>>,
    pub on_voice_server_update: Option<Arc<Fn(Context, VoiceServerUpdateEvent) + Send + Sync + 'static>>,
    pub on_voice_state_update: Option<Arc<Fn(Context, Option<GuildId>, VoiceState) + Send + Sync + 'static>>,
    pub on_webhook_update: Option<Arc<Fn(Context, GuildId, ChannelId) + Send + Sync + 'static>>,
}
