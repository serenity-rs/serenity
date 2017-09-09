use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::context::Context;
use model::event::*;
use model::*;

use std::sync::RwLock;

pub trait EventHandler {
    #[cfg(feature = "cache")]
    fn on_cached(&self, _: Context, _: Vec<GuildId>) {}
    fn on_channel_create(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}
    fn on_category_create(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}
    fn on_category_delete(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}
    fn on_private_channel_create(&self, _: Context, _: Arc<RwLock<PrivateChannel>>) {}
    fn on_channel_delete(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}
    fn on_channel_pins_update(&self, _: Context, _: ChannelPinsUpdateEvent) {}
    fn on_channel_recipient_addition(&self, _: Context, _: ChannelId, _: User) {}
    fn on_channel_recipient_removal(&self, _: Context, _: ChannelId, _: User) {}
    #[cfg(feature = "cache")]
    fn on_channel_update(&self, _: Context, _: Option<Channel>, _: Channel) {}
    #[cfg(not(feature = "cache"))]
    fn on_channel_update(&self, _: Context, _: Channel) {}
    fn on_guild_ban_addition(&self, _: Context, _: GuildId, _: User) {}
    fn on_guild_ban_removal(&self, _: Context, _: GuildId, _: User) {}
    #[cfg(feature = "cache")]
    fn on_guild_create(&self, _: Context, _: Guild, _: bool) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_create(&self, _: Context, _: Guild) {}
    #[cfg(feature = "cache")]
    fn on_guild_delete(&self, _: Context, _: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_delete(&self, _: Context, _: PartialGuild) {}
    fn on_guild_emojis_update(&self, _: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {}
    fn on_guild_integrations_update(&self, _: Context, _: GuildId) {}
    fn on_guild_member_addition(&self, _: Context, _: GuildId, _: Member) {}
    #[cfg(feature = "cache")]
    fn on_guild_member_removal(&self, _: Context, _: GuildId, _: User, _: Option<Member>) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_member_removal(&self, _: Context, _: GuildId, _: User) {}
    #[cfg(feature = "cache")]
    fn on_guild_member_update(&self, _: Context, _: Option<Member>, _: Member) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_member_update(&self, _: Context, _: GuildMemberUpdateEvent) {}
    fn on_guild_members_chunk(&self, _: Context, _: GuildId, _: HashMap<UserId, Member>) {}
    fn on_guild_role_create(&self, _: Context, _: GuildId, _: Role) {}
    #[cfg(feature = "cache")]
    fn on_guild_role_delete(&self, _: Context, _: GuildId, _: RoleId, _: Option<Role>) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_role_delete(&self, _: Context, _: GuildId, _: RoleId) {}
    #[cfg(feature = "cache")]
    fn on_guild_role_update(&self, _: Context, _: GuildId, _: Option<Role>, _: Role) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_role_update(&self, _: Context, _: GuildId, _: Role) {}
    fn on_guild_unavailable(&self, _: Context, _: GuildId) {}
    #[cfg(feature = "cache")]
    fn on_guild_update(&self, _: Context, _: Option<Arc<RwLock<Guild>>>, _: PartialGuild) {}
    #[cfg(not(feature = "cache"))]
    fn on_guild_update(&self, _: Context, _: PartialGuild) {}
    fn on_message(&self, _: Context, _: Message) {}
    fn on_message_delete(&self, _: Context, _: ChannelId, _: MessageId) {}
    fn on_message_delete_bulk(&self, _: Context, _: ChannelId, _: Vec<MessageId>) {}
    fn on_reaction_add(&self, _: Context, _: Reaction) {}
    fn on_reaction_remove(&self, _: Context, _: Reaction) {}
    fn on_reaction_remove_all(&self, _: Context, _: ChannelId, _: MessageId) {}
    fn on_message_update(&self, _: Context, _: MessageUpdateEvent) {}
    fn on_presence_replace(&self, _: Context, _: Vec<Presence>) {}
    fn on_presence_update(&self, _: Context, _: PresenceUpdateEvent) {}
    fn on_ready(&self, _: Context, _: Ready) {}
    fn on_resume(&self, _: Context, _: ResumedEvent) {}
    fn on_typing_start(&self, _: Context, _: TypingStartEvent) {}
    fn on_unknown(&self, _: Context, _: String, _: Value) {}
    #[cfg(feature = "cache")]
    fn on_user_update(&self, _: Context, _: CurrentUser, _: CurrentUser) {}
    #[cfg(not(feature = "cache"))]
    fn on_user_update(&self, _: Context, _: CurrentUser) {}
    fn on_voice_server_update(&self, _: Context, _: VoiceServerUpdateEvent) {}
    fn on_voice_state_update(&self, _: Context, _: Option<GuildId>, _: VoiceState) {}
    fn on_webhook_update(&self, _: Context, _: GuildId, _: ChannelId) {}
}
