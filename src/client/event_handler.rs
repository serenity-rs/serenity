use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::context::Context;
use ::model::event::*;
use ::model::*;

#[cfg(feature="cache")]
use std::sync::RwLock;

#[allow(type_complexity)]
pub trait EventHandler {
    #[cfg(feature="cache")]
    fn on_cached(&self, Context, Vec<GuildId>) {}
    fn on_channel_create(&self, Context, Channel) {}
    fn on_channel_delete(&self, Context, Channel) {}
    fn on_channel_pins_update(&self, Context, ChannelPinsUpdateEvent) {}
    fn on_channel_recipient_addition(&self, Context, ChannelId, User) {}
    fn on_channel_recipient_removal(&self, Context, ChannelId, User) {}
    #[cfg(feature="cache")]
    fn on_channel_update(&self, Context, Option<Channel>, Channel) {}
    #[cfg(not(feature="cache"))]
    fn on_channel_update(&self, Context, Channel) {}
    fn on_guild_ban_addition(&self, Context, GuildId, User) {}
    fn on_guild_ban_removal(&self, Context, GuildId, User) {}
    fn on_guild_create(&self, Context, Guild) {}
    #[cfg(feature="cache")]
    fn on_guild_delete(&self, Context, PartialGuild, Option<Arc<RwLock<Guild>>>) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_delete(&self, Context, PartialGuild) {}
    fn on_guild_emojis_update(&self, Context, GuildId, HashMap<EmojiId, Emoji>) {}
    fn on_guild_integrations_update(&self, Context, GuildId) {}
    fn on_guild_member_addition(&self, Context, GuildId, Member) {}
    #[cfg(feature="cache")]
    fn on_guild_member_removal(&self, Context, GuildId, User, Option<Member>) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_member_removal(&self, Context, GuildId, User) {}
    #[cfg(feature="cache")]
    fn on_guild_member_update(&self, Context, Option<Member>, Member) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_member_update(&self, Context, GuildMemberUpdateEvent) {}
    fn on_guild_members_chunk(&self, Context, GuildId, HashMap<UserId, Member>) {}
    fn on_guild_role_create(&self, Context, GuildId, Role) {}
    #[cfg(feature="cache")]
    fn on_guild_role_delete(&self, Context, GuildId, RoleId, Option<Role>) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_role_delete(&self, Context, GuildId, RoleId) {}
    #[cfg(feature="cache")]
    fn on_guild_role_update(&self, Context, GuildId, Option<Role>, Role) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_role_update(&self, Context, GuildId, Role) {}
    fn on_guild_unavailable(&self, Context, GuildId) {}
    #[cfg(feature="cache")]
    fn on_guild_update(&self, Context, Option<Arc<RwLock<Guild>>>, PartialGuild) {}
    #[cfg(not(feature="cache"))]
    fn on_guild_update(&self, Context, PartialGuild) {}
    fn on_message(&self, Context, Message) {}
    fn on_message_delete(&self, Context, ChannelId, MessageId) {}
    fn on_message_delete_bulk(&self, Context, ChannelId, Vec<MessageId>) {}
    fn on_reaction_add(&self, Context, Reaction) {}
    fn on_reaction_remove(&self, Context, Reaction) {}
    fn on_reaction_remove_all(&self, Context, ChannelId, MessageId) {}
    fn on_message_update(&self, Context, MessageUpdateEvent) {}
    fn on_presence_replace(&self, Context, Vec<Presence>) {}
    fn on_presence_update(&self, Context, PresenceUpdateEvent) {}
    fn on_ready(&self, Context, Ready) {}
    fn on_resume(&self, Context, ResumedEvent) {}
    fn on_typing_start(&self, Context, TypingStartEvent) {}
    fn on_unknown(&self, Context, String, Value) {}
    #[cfg(feature="cache")]
    fn on_user_update(&self, Context, CurrentUser, CurrentUser) {}
    #[cfg(not(feature="cache"))]
    fn on_user_update(&self, Context, CurrentUser) {}
    fn on_voice_server_update(&self, Context, VoiceServerUpdateEvent) {}
    fn on_voice_state_update(&self, Context, Option<GuildId>, VoiceState) {}
    fn on_webhook_update(&self, Context, GuildId, ChannelId) {}
}
