use model::prelude::*;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::context::Context;
use ::client::bridge::gateway::event::*;

/// The core trait for handling events by serenity.
pub trait EventHandler {
    /// Dispatched when the cache gets full.
    /// 
    /// Provides the cached guilds' ids.
    #[cfg(feature = "cache")]
    fn cached(&self, _: Context, _: Vec<GuildId>) {}

    /// Dispatched when a channel is created.
    /// 
    /// Provides said channel's data.
    fn channel_create(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}

    /// Dispatched when a category is created.
    /// 
    /// Provides said category's data.
    fn category_create(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}

    /// Dispatched when a category is deleted.
    /// 
    /// Provides said category's data.
    fn category_delete(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}

    /// Dispatched when a private channel is created.
    /// 
    /// Provides said channel's data.
    fn private_channel_create(&self, _: Context, _: Arc<RwLock<PrivateChannel>>) {}

    /// Dispatched when a channel is deleted.
    /// 
    /// Provides said channel's data.
    fn channel_delete(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}

    /// Dispatched when a pin is added, deleted.
    /// 
    /// Provides said pin's data.
    fn channel_pins_update(&self, _: Context, _: ChannelPinsUpdateEvent) {}

    /// Dispatched when a user is added to a `Group`.
    /// 
    /// Provides the group's id and the user's data.
    fn channel_recipient_addition(&self, _: Context, _: ChannelId, _: User) {}

    /// Dispatched when a user is removed to a `Group`.
    /// 
    /// Provides the group's id and the user's data.
    fn channel_recipient_removal(&self, _: Context, _: ChannelId, _: User) {}

    /// Dispatched when a channel is updated.
    /// 
    /// Provides the old channel data, and the new data.
    #[cfg(feature = "cache")]
    fn channel_update(&self, _: Context, _: Option<Channel>, _: Channel) {}

    /// Dispatched when a channel is updated.
    /// 
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn channel_update(&self, _: Context, _: Channel) {}

    /// Dispatched when a user is banned from a guild.
    /// 
    /// Provides the guild's id and the banned user's data.
    fn guild_ban_addition(&self, _: Context, _: GuildId, _: User) {}

    /// Dispatched when a user's ban is lifted from a guild.
    /// 
    /// /// Provides the guild's id and the lifted user's data.
    fn guild_ban_removal(&self, _: Context, _: GuildId, _: User) {}

    /// Dispatched when a guild is created; 
    /// or an existing guild's data is sent to us.
    /// 
    /// Provides the guild's data and whether the guild is new.
    #[cfg(feature = "cache")]
    fn guild_create(&self, _: Context, _: Guild, _: bool) {}

    /// Dispatched when a guild is created; 
    /// or an existing guild's data is sent to us.
    /// 
    /// Provides the guild's data.
    #[cfg(not(feature = "cache"))]
    fn guild_create(&self, _: Context, _: Guild) {}

    /// Dispatched when a guild is deleted.
    /// 
    /// Provides the partial data of the guild sent by discord,
    /// and the full data from the cache, if available.
    #[cfg(feature = "cache")]
    fn guild_delete(&self, _: Context, _: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {}

    /// Dispatched when a guild is deleted.
    /// 
    /// Provides the partial data of the guild sent by discord.
    #[cfg(not(feature = "cache"))]
    fn guild_delete(&self, _: Context, _: PartialGuild) {}

    /* the emojis were updated. */

    /// Dispatched when the emojis are updated.
    /// 
    /// Provides the guild's id and the new state of the emojis in the guild.
    fn guild_emojis_update(&self, _: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {}

    /// Dispatched when a guild's integration is added, updated or removed.
    /// 
    /// Provides the guild's id.
    fn guild_integrations_update(&self, _: Context, _: GuildId) {}

    /// Dispatched when a user joins a guild.
    /// 
    /// Provides the guild's id and the user's member data.
    fn guild_member_addition(&self, _: Context, _: GuildId, _: Member) {}

    /// Dispatched when a user is removed (kicked).
    /// 
    /// Provides the guild's id, the user's data, and the user's member data if available.
    #[cfg(feature = "cache")]
    fn guild_member_removal(&self, _: Context, _: GuildId, _: User, _: Option<Member>) {}

    /// Dispatched when a user is removed (kicked).
    /// 
    /// Provides the guild's id, the user's data.
    #[cfg(not(feature = "cache"))]
    fn guild_member_removal(&self, _: Context, _: GuildId, _: User) {}

    /// Dispatched when a member is updated (e.g their nickname is updated)
    /// 
    /// Provides the member's old data (if available) and the new data.
    #[cfg(feature = "cache")]
    fn guild_member_update(&self, _: Context, _: Option<Member>, _: Member) {}

    /// Dispatched when a member is updated (e.g their nickname is updated)
    /// 
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn guild_member_update(&self, _: Context, _: GuildMemberUpdateEvent) {}

    /// Dispatched when the data for offline members was requested.
    /// 
    /// Provides the guild's id and the data. 
    fn guild_members_chunk(&self, _: Context, _: GuildId, _: HashMap<UserId, Member>) {}

    /// Dispatched when a role is created.
    /// 
    /// Provides the guild's id and the new role's data.
    fn guild_role_create(&self, _: Context, _: GuildId, _: Role) {}

    /// Dispatched when a role is deleted.
    /// 
    /// Provides the guild's id, the role's id and its data if available.
    #[cfg(feature = "cache")]
    fn guild_role_delete(&self, _: Context, _: GuildId, _: RoleId, _: Option<Role>) {}

    /// Dispatched when a role is deleted.
    /// 
    /// Provides the guild's id, the role's id.
    #[cfg(not(feature = "cache"))]
    fn guild_role_delete(&self, _: Context, _: GuildId, _: RoleId) {}

    /// Dispatched when a role is updated.
    /// 
    /// Provides the guild's id, the role's old (if available) and new data.
    #[cfg(feature = "cache")]
    fn guild_role_update(&self, _: Context, _: GuildId, _: Option<Role>, _: Role) {}

    /// Dispatched when a role is updated.
    /// 
    /// Provides the guild's id and the role's new data.
    #[cfg(not(feature = "cache"))]
    fn guild_role_update(&self, _: Context, _: GuildId, _: Role) {}

    /// Dispatched when a guild became unavailable.
    /// 
    /// Provides the guild's id. 
    fn guild_unavailable(&self, _: Context, _: GuildId) {}

    /// Dispatched when the guild is updated.
    /// 
    /// Provides the guild's old full data (if available) and the new, albeit partial data.
    #[cfg(feature = "cache")]
    fn guild_update(&self, _: Context, _: Option<Arc<RwLock<Guild>>>, _: PartialGuild) {}

    /// Dispatched when the guild is updated.
    /// 
    /// Provides the guild's new, albeit partial data.
    #[cfg(not(feature = "cache"))]
    fn guild_update(&self, _: Context, _: PartialGuild) {}

    /// Dispatched when a message is created.
    /// 
    /// Provides the message's data.
    fn message(&self, _: Context, _: Message) {}

    /// Dispatched when a message is deleted.
    /// 
    /// Provides the channel's id and the message's id.
    fn message_delete(&self, _: Context, _: ChannelId, _: MessageId) {}

    /// Dispatched when multiple messages were deleted at once.
    /// 
    /// Provides the channel's id and the deleted messages' ids.
    fn message_delete_bulk(&self, _: Context, _: ChannelId, _: Vec<MessageId>) {}

    /// Dispatched when a new reaction is attached to a message.
    /// 
    /// Provides the reaction's data.
    fn reaction_add(&self, _: Context, _: Reaction) {}

    /// Dispatched when a reaction is dettached from a message.
    /// 
    /// Provides the reaction's data.
    fn reaction_remove(&self, _: Context, _: Reaction) {}

    /// Dispatched when all reactions of a message are dettached from a message.
    /// 
    /// Provides the channel's id and the message's id.
    fn reaction_remove_all(&self, _: Context, _: ChannelId, _: MessageId) {}

    /// Dispatched when a message is updated.
    /// 
    /// Provides the new data of the message.
    fn message_update(&self, _: Context, _: MessageUpdateEvent) {}

    fn presence_replace(&self, _: Context, _: Vec<Presence>) {}

    /// Dispatched when a user's presence is updated (i.e off -> on).
    /// 
    /// Provides the presence's new data.
    fn presence_update(&self, _: Context, _: PresenceUpdateEvent) {}

    /// Dispatched upon startup.
    /// 
    /// Provides data about the bot and the guilds it's in.
    fn ready(&self, _: Context, _: Ready) {}

    /// Dispatched upon reconnection.
    fn resume(&self, _: Context, _: ResumedEvent) {}

    /// Dispatched when a shard's connection stage is updated
    /// 
    /// Provides the context of the shard and the event information about the update.
    fn shard_stage_update(&self, _: Context, _: ShardStageUpdateEvent) {}

    /// Dispatched when a user starts typing.
    fn typing_start(&self, _: Context, _: TypingStartEvent) {}

    /// Dispatched when an unknown event was sent from discord.
    /// 
    /// Provides the event's name and its unparsed data.
    fn unknown(&self, _: Context, _: String, _: Value) {}

    /// Dispatched when the bot's data is updated.
    /// 
    /// Provides the old and new data.
    #[cfg(feature = "cache")]
    fn user_update(&self, _: Context, _: CurrentUser, _: CurrentUser) {}

    /// Dispatched when the bot's data is updated.
    /// 
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn user_update(&self, _: Context, _: CurrentUser) {}

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    /// 
    /// Provides the voice server's data.
    fn voice_server_update(&self, _: Context, _: VoiceServerUpdateEvent) {}

    /// Dispatched when a user joins, leaves or moves a voice channel.
    /// 
    /// Provides the guild's id (if available) and 
    /// the new state of the guild's voice channels.
    fn voice_state_update(&self, _: Context, _: Option<GuildId>, _: VoiceState) {}

    /// Dispatched when a guild's webhook is updated.
    /// 
    /// Provides the guild's id and the channel's id the webhook belongs in.
    fn webhook_update(&self, _: Context, _: GuildId, _: ChannelId) {}
}
