use crate::model::prelude::*;
use parking_lot::RwLock;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc
};
use super::context::Context;
use crate::client::bridge::gateway::event::*;

/// The core trait for handling events by serenity.
pub trait EventHandler: Send + Sync {
    /// Dispatched when the cache has received and inserted all data from
    /// guilds.
    ///
    /// This process happens upon starting your bot and should be fairly quick.
    /// However, cache actions performed prior this event may fail as the data
    /// could be not inserted yet.
    ///
    /// Provides the cached guilds' ids.
    #[cfg(feature = "cache")]
    fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {}

    /// Dispatched when a channel is created.
    ///
    /// Provides said channel's data.
    fn channel_create(&self, _ctx: Context, _channel: Arc<RwLock<GuildChannel>>) {}

    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    fn category_create(&self, _ctx: Context, _category: Arc<RwLock<ChannelCategory>>) {}

    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    fn category_delete(&self, _ctx: Context, _category: Arc<RwLock<ChannelCategory>>) {}

    /// Dispatched when a private channel is created.
    ///
    /// Provides said channel's data.
    fn private_channel_create(&self, _ctx: Context, _channel: Arc<RwLock<PrivateChannel>>) {}

    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    fn channel_delete(&self, _ctx: Context, _channel: Arc<RwLock<GuildChannel>>) {}

    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    fn channel_pins_update(&self, _ctx: Context, _pin: ChannelPinsUpdateEvent) {}

    /// Dispatched when a user is added to a `Group`.
    ///
    /// Provides the group's id and the user's data.
    fn channel_recipient_addition(&self, _ctx: Context, _group_id: ChannelId, _user: User) {}

    /// Dispatched when a user is removed to a `Group`.
    ///
    /// Provides the group's id and the user's data.
    fn channel_recipient_removal(&self, _ctx: Context, _group_id: ChannelId, _user: User) {}

    /// Dispatched when a channel is updated.
    ///
    /// Provides the old channel data, and the new data.
    #[cfg(feature = "cache")]
    fn channel_update(&self, _ctx: Context, _old: Option<Channel>, _new: Channel) {}

    /// Dispatched when a channel is updated.
    ///
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn channel_update(&self, _ctx: Context, _new_data: Channel) {}

    /// Dispatched when a user is banned from a guild.
    ///
    /// Provides the guild's id and the banned user's data.
    fn guild_ban_addition(&self, _ctx: Context, _guild_id: GuildId, _banned_user: User) {}

    /// Dispatched when a user's ban is lifted from a guild.
    ///
    /// Provides the guild's id and the lifted user's data.
    fn guild_ban_removal(&self, _ctx: Context, _guild_id: GuildId, _unbanned_user: User) {}

    /// Dispatched when a guild is created;
    /// or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data and whether the guild is new.
    #[cfg(feature = "cache")]
    fn guild_create(&self, _ctx: Context, _guild: Guild, _is_new: bool) {}

    /// Dispatched when a guild is created;
    /// or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data.
    #[cfg(not(feature = "cache"))]
    fn guild_create(&self, _ctx: Context, _guild: Guild) {}

    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord,
    /// and the full data from the cache, if available.
    #[cfg(feature = "cache")]
    fn guild_delete(&self, _ctx: Context, _incomplete: PartialGuild, _full: Option<Arc<RwLock<Guild>>>) {}

    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord.
    #[cfg(not(feature = "cache"))]
    fn guild_delete(&self, _ctx: Context, _incomplete: PartialGuild) {}

    /* the emojis were updated. */

    /// Dispatched when the emojis are updated.
    ///
    /// Provides the guild's id and the new state of the emojis in the guild.
    fn guild_emojis_update(&self, _ctx: Context, _guild_id: GuildId, _current_state: HashMap<EmojiId, Emoji>) {}

    /// Dispatched when a guild's integration is added, updated or removed.
    ///
    /// Provides the guild's id.
    fn guild_integrations_update(&self, _ctx: Context, _guild_id: GuildId) {}

    /// Dispatched when a user joins a guild.
    ///
    /// Provides the guild's id and the user's member data.
    fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {}

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data, and the user's member data if available.
    #[cfg(feature = "cache")]
    fn guild_member_removal(&self, _ctx: Context, _guild: GuildId, _user: User, _member_data_if_available: Option<Member>) {}

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data.
    #[cfg(not(feature = "cache"))]
    fn guild_member_removal(&self, _ctx: Context, _guild_id: GuildId, _kicked: User) {}

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the member's old data (if available) and the new data.
    #[cfg(feature = "cache")]
    fn guild_member_update(&self, _ctx: Context, _old_if_available: Option<Member>, _new: Member) {}

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn guild_member_update(&self, _ctx: Context, _new: GuildMemberUpdateEvent) {}

    /// Dispatched when the data for offline members was requested.
    ///
    /// Provides the guild's id and the data.
    fn guild_members_chunk(&self, _ctx: Context, _guild_id: GuildId, _offline_members: HashMap<UserId, Member>) {}

    /// Dispatched when a role is created.
    ///
    /// Provides the guild's id and the new role's data.
    fn guild_role_create(&self, _ctx: Context, _guild_id: GuildId, _new: Role) {}

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id and its data if available.
    #[cfg(feature = "cache")]
    fn guild_role_delete(&self, _ctx: Context, _guild_id: GuildId, _removed_role_id: RoleId, _removed_role_data_if_available: Option<Role>) {}

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id.
    #[cfg(not(feature = "cache"))]
    fn guild_role_delete(&self, _ctx: Context, _guild_id: GuildId, _removed_role_id: RoleId) {}

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id, the role's old (if available) and new data.
    #[cfg(feature = "cache")]
    fn guild_role_update(&self, _ctx: Context, _guild_id: GuildId, _old_data_if_available: Option<Role>, _new: Role) {}

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id and the role's new data.
    #[cfg(not(feature = "cache"))]
    fn guild_role_update(&self, _ctx: Context, _guild_id: GuildId, _new_data: Role) {}

    /// Dispatched when a guild became unavailable.
    ///
    /// Provides the guild's id.
    fn guild_unavailable(&self, _ctx: Context, _guild_id: GuildId) {}

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's old full data (if available) and the new, albeit partial data.
    #[cfg(feature = "cache")]
    fn guild_update(&self, _ctx: Context, _old_data_if_available: Option<Arc<RwLock<Guild>>>, _new_but_incomplete: PartialGuild) {}

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's new, albeit partial data.
    #[cfg(not(feature = "cache"))]
    fn guild_update(&self, _ctx: Context, _new_but_incomplete_data: PartialGuild) {}

    /// Dispatched when a message is created.
    ///
    /// Provides the message's data.
    fn message(&self, _ctx: Context, _new_message: Message) {}

    /// Dispatched when a message is deleted.
    ///
    /// Provides the channel's id and the message's id.
    fn message_delete(&self, _ctx: Context, _channel_id: ChannelId, _deleted_message_id: MessageId) {}

    /// Dispatched when multiple messages were deleted at once.
    ///
    /// Provides the channel's id and the deleted messages' ids.
    fn message_delete_bulk(&self, _ctx: Context, _channel_id: ChannelId, _multiple_deleted_messages_ids: Vec<MessageId>) {}

    /// Dispatched when a message is updated.
    ///
    /// Provides the old message if available,
    /// the new message as an option in case of cache inconsistencies,
    /// and the raw [`MessageUpdateEvent`] as a fallback.
    ///
    /// [`MessageUpdateEvent`]: ../model/event/struct.MessageUpdateEvent.html
    #[cfg(feature = "cache")]
    fn message_update(&self, _ctx: Context, _old_if_available: Option<Message>, _new: Option<Message>, _event: MessageUpdateEvent) {}

    /// Dispatched when a message is updated.
    ///
    /// Provides the new data of the message.
    #[cfg(not(feature = "cache"))]
    fn message_update(&self, _ctx: Context, _new_data: MessageUpdateEvent) {}

    /// Dispatched when a new reaction is attached to a message.
    ///
    /// Provides the reaction's data.
    fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}

    /// Dispatched when a reaction is detached from a message.
    ///
    /// Provides the reaction's data.
    fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {}

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    fn reaction_remove_all(&self, _ctx: Context, _channel_id: ChannelId, _removed_from_message_id: MessageId) {}

    fn presence_replace(&self, _ctx: Context, _: Vec<Presence>) {}

    /// Dispatched when a user's presence is updated (e.g off -> on).
    ///
    /// Provides the presence's new data.
    fn presence_update(&self, _ctx: Context, _new_data: PresenceUpdateEvent) {}

    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    fn ready(&self, _ctx: Context, _data_about_bot: Ready) {}

    /// Dispatched upon reconnection.
    fn resume(&self, _ctx: Context, _: ResumedEvent) {}

    /// Dispatched when a shard's connection stage is updated
    ///
    /// Provides the context of the shard and the event information about the update.
    fn shard_stage_update(&self, _ctx: Context, _: ShardStageUpdateEvent) {}

    /// Dispatched when a user starts typing.
    fn typing_start(&self, _ctx: Context, _: TypingStartEvent) {}

    /// Dispatched when an unknown event was sent from discord.
    ///
    /// Provides the event's name and its unparsed data.
    fn unknown(&self, _ctx: Context, _name: String, _raw: Value) {}

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the old and new data.
    #[cfg(feature = "cache")]
    fn user_update(&self, _ctx: Context, _old_data: CurrentUser, _new: CurrentUser) {}

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    fn user_update(&self, _ctx: Context, _new_data: CurrentUser) {}

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    ///
    /// Provides the voice server's data.
    fn voice_server_update(&self, _ctx: Context, _: VoiceServerUpdateEvent) {}

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and
    /// the old and the new state of the guild's voice channels.
    #[cfg(feature = "cache")]
    fn voice_state_update(&self, _ctx: Context, _: Option<GuildId>, _old: Option<VoiceState>, _new: VoiceState) {}

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and
    /// the new state of the guild's voice channels.
    #[cfg(not(feature = "cache"))]
    fn voice_state_update(&self, _ctx: Context, _: Option<GuildId>, _: VoiceState) {}

    /// Dispatched when a guild's webhook is updated.
    ///
    /// Provides the guild's id and the channel's id the webhook belongs in.
    fn webhook_update(&self, _ctx: Context, _guild_id: GuildId, _belongs_to_channel_id: ChannelId) {}
}

/// This core trait for handling raw events
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    fn raw_event(&self, _ctx: Context, _ev: Event) {}
}
