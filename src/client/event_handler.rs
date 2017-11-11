use model::event::*;
use model::*;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::context::Context;

pub trait EventHandler {
    #[cfg(feature = "cache")]
    fn cached(&self, _: Context, _: Vec<GuildId>) {}
    fn channel_create(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}
    fn category_create(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}
    fn category_delete(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}
    fn private_channel_create(&self, _: Context, _: Arc<RwLock<PrivateChannel>>) {}
    fn channel_delete(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}
    fn channel_pins_update(&self, _: Context, _: ChannelPinsUpdateEvent) {}
    fn channel_recipient_addition(&self, _: Context, _: ChannelId, _: User) {}
    fn channel_recipient_removal(&self, _: Context, _: ChannelId, _: User) {}
    #[cfg(feature = "cache")]
    fn channel_update(&self, _: Context, _: Option<Channel>, _: Channel) {}
    #[cfg(not(feature = "cache"))]
    fn channel_update(&self, _: Context, _: Channel) {}
    fn guild_ban_addition(&self, _: Context, _: GuildId, _: User) {}
    fn guild_ban_removal(&self, _: Context, _: GuildId, _: User) {}
    #[cfg(feature = "cache")]
    fn guild_create(&self, _: Context, _: Guild, _: bool) {}
    #[cfg(not(feature = "cache"))]
    fn guild_create(&self, _: Context, _: Guild) {}
    #[cfg(feature = "cache")]
    fn guild_delete(&self, _: Context, _: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {}
    #[cfg(not(feature = "cache"))]
    fn guild_delete(&self, _: Context, _: PartialGuild) {}
    fn guild_emojis_update(&self, _: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {}
    fn guild_integrations_update(&self, _: Context, _: GuildId) {}
    fn guild_member_addition(&self, _: Context, _: GuildId, _: Member) {}
    #[cfg(feature = "cache")]
    fn guild_member_removal(&self, _: Context, _: GuildId, _: User, _: Option<Member>) {}
    #[cfg(not(feature = "cache"))]
    fn guild_member_removal(&self, _: Context, _: GuildId, _: User) {}
    #[cfg(feature = "cache")]
    fn guild_member_update(&self, _: Context, _: Option<Member>, _: Member) {}
    #[cfg(not(feature = "cache"))]
    fn guild_member_update(&self, _: Context, _: GuildMemberUpdateEvent) {}
    fn guild_members_chunk(&self, _: Context, _: GuildId, _: HashMap<UserId, Member>) {}
    fn guild_role_create(&self, _: Context, _: GuildId, _: Role) {}
    #[cfg(feature = "cache")]
    fn guild_role_delete(&self, _: Context, _: GuildId, _: RoleId, _: Option<Role>) {}
    #[cfg(not(feature = "cache"))]
    fn guild_role_delete(&self, _: Context, _: GuildId, _: RoleId) {}
    #[cfg(feature = "cache")]
    fn guild_role_update(&self, _: Context, _: GuildId, _: Option<Role>, _: Role) {}
    #[cfg(not(feature = "cache"))]
    fn guild_role_update(&self, _: Context, _: GuildId, _: Role) {}
    fn guild_unavailable(&self, _: Context, _: GuildId) {}
    #[cfg(feature = "cache")]
    fn guild_update(&self, _: Context, _: Option<Arc<RwLock<Guild>>>, _: PartialGuild) {}
    #[cfg(not(feature = "cache"))]
    fn guild_update(&self, _: Context, _: PartialGuild) {}
    fn message(&self, _: Context, _: Message) {}
    fn message_delete(&self, _: Context, _: ChannelId, _: MessageId) {}
    fn message_delete_bulk(&self, _: Context, _: ChannelId, _: Vec<MessageId>) {}
    fn reaction_add(&self, _: Context, _: Reaction) {}
    fn reaction_remove(&self, _: Context, _: Reaction) {}
    fn reaction_remove_all(&self, _: Context, _: ChannelId, _: MessageId) {}
    fn message_update(&self, _: Context, _: MessageUpdateEvent) {}
    fn presence_replace(&self, _: Context, _: Vec<Presence>) {}
    fn presence_update(&self, _: Context, _: PresenceUpdateEvent) {}
    fn ready(&self, _: Context, _: Ready) {}
    fn resume(&self, _: Context, _: ResumedEvent) {}
    fn typing_start(&self, _: Context, _: TypingStartEvent) {}
    fn unknown(&self, _: Context, _: String, _: Value) {}
    #[cfg(feature = "cache")]
    fn user_update(&self, _: Context, _: CurrentUser, _: CurrentUser) {}
    #[cfg(not(feature = "cache"))]
    fn user_update(&self, _: Context, _: CurrentUser) {}
    fn voice_server_update(&self, _: Context, _: VoiceServerUpdateEvent) {}
    fn voice_state_update(&self, _: Context, _: Option<GuildId>, _: VoiceState) {}
    fn webhook_update(&self, _: Context, _: GuildId, _: ChannelId) {}
}
