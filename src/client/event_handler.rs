use std::collections::HashMap;

use async_trait::async_trait;

use super::context::Context;
use crate::client::bridge::gateway::event::*;
use crate::json::Value;
use crate::model::prelude::*;

/// The core trait for handling events by serenity.
#[async_trait]
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
    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {}

    /// Dispatched when a channel is created.
    ///
    /// Provides said channel's data.
    async fn channel_create(&self, _ctx: Context, _channel: &GuildChannel) {}

    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    async fn category_create(&self, _ctx: Context, _category: &ChannelCategory) {}

    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    async fn category_delete(&self, _ctx: Context, _category: &ChannelCategory) {}

    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    async fn channel_delete(&self, _ctx: Context, _channel: &GuildChannel) {}

    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    async fn channel_pins_update(&self, _ctx: Context, _pin: ChannelPinsUpdateEvent) {}

    /// Dispatched when a channel is updated.
    ///
    /// Provides the old channel data, and the new data.
    #[cfg(feature = "cache")]
    async fn channel_update(&self, _ctx: Context, _old: Option<Channel>, _new: Channel) {}

    /// Dispatched when a channel is updated.
    ///
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    async fn channel_update(&self, _ctx: Context, _new_data: Channel) {}

    /// Dispatched when a user is banned from a guild.
    ///
    /// Provides the guild's id and the banned user's data.
    async fn guild_ban_addition(&self, _ctx: Context, _guild_id: GuildId, _banned_user: User) {}

    /// Dispatched when a user's ban is lifted from a guild.
    ///
    /// Provides the guild's id and the lifted user's data.
    async fn guild_ban_removal(&self, _ctx: Context, _guild_id: GuildId, _unbanned_user: User) {}

    /// Dispatched when a guild is created;
    /// or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data and whether the guild is new.
    #[cfg(feature = "cache")]
    async fn guild_create(&self, _ctx: Context, _guild: Guild, _is_new: bool) {}

    /// Dispatched when a guild is created;
    /// or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data.
    #[cfg(not(feature = "cache"))]
    async fn guild_create(&self, _ctx: Context, _guild: Guild) {}

    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord,
    /// and the full data from the cache, if available.
    ///
    /// The `unavailable` flag in the partial data determines the status of the guild.
    /// If the flag is false, the bot was removed from the guild, either by being
    /// kicked or banned. If the flag is true, the guild went offline.
    #[cfg(feature = "cache")]
    async fn guild_delete(
        &self,
        _ctx: Context,
        _incomplete: GuildUnavailable,
        _full: Option<Guild>,
    ) {
    }

    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord.
    ///
    /// The `unavailable` flag in the partial data determines the status of the guild.
    /// If the flag is false, the bot was removed from the guild, either by being
    /// kicked or banned. If the flag is true, the guild went offline.
    #[cfg(not(feature = "cache"))]
    async fn guild_delete(&self, _ctx: Context, _incomplete: GuildUnavailable) {}

    // the emojis were updated.

    /// Dispatched when the emojis are updated.
    ///
    /// Provides the guild's id and the new state of the emojis in the guild.
    async fn guild_emojis_update(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _current_state: HashMap<EmojiId, Emoji>,
    ) {
    }

    /// Dispatched when a guild's integration is added, updated or removed.
    ///
    /// Provides the guild's id.
    async fn guild_integrations_update(&self, _ctx: Context, _guild_id: GuildId) {}

    /// Dispatched when a user joins a guild.
    ///
    /// Provides the guild's id and the user's member data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    async fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {}

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data, and the user's member data if available.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    #[cfg(feature = "cache")]
    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _user: User,
        _member_data_if_available: Option<Member>,
    ) {
    }

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    #[cfg(not(feature = "cache"))]
    async fn guild_member_removal(&self, _ctx: Context, _guild_id: GuildId, _kicked: User) {}

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the member's old data (if available) and the new data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    #[cfg(feature = "cache")]
    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Member>,
        _new: Member,
    ) {
    }

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the new data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    #[cfg(not(feature = "cache"))]
    async fn guild_member_update(&self, _ctx: Context, _new: GuildMemberUpdateEvent) {}

    /// Dispatched when the data for offline members was requested.
    ///
    /// Provides the guild's id and the data.
    async fn guild_members_chunk(&self, _ctx: Context, _chunk: GuildMembersChunkEvent) {}

    /// Dispatched when a role is created.
    ///
    /// Provides the guild's id and the new role's data.
    async fn guild_role_create(&self, _ctx: Context, _guild_id: GuildId, _new: Role) {}

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id and its data if available.
    #[cfg(feature = "cache")]
    async fn guild_role_delete(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _removed_role_id: RoleId,
        _removed_role_data_if_available: Option<Role>,
    ) {
    }

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id.
    #[cfg(not(feature = "cache"))]
    async fn guild_role_delete(&self, _ctx: Context, _guild_id: GuildId, _removed_role_id: RoleId) {
    }

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id, the role's old (if available) and new data.
    #[cfg(feature = "cache")]
    async fn guild_role_update(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _old_data_if_available: Option<Role>,
        _new: Role,
    ) {
    }

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id and the role's new data.
    #[cfg(not(feature = "cache"))]
    async fn guild_role_update(&self, _ctx: Context, _guild_id: GuildId, _new_data: Role) {}

    /// Dispatched when a guild became unavailable.
    ///
    /// Provides the guild's id.
    async fn guild_unavailable(&self, _ctx: Context, _guild_id: GuildId) {}

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's old full data (if available) and the new, albeit partial data.
    #[cfg(feature = "cache")]
    async fn guild_update(
        &self,
        _ctx: Context,
        _old_data_if_available: Option<Guild>,
        _new_but_incomplete: PartialGuild,
    ) {
    }

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's new, albeit partial data.
    #[cfg(not(feature = "cache"))]
    async fn guild_update(&self, _ctx: Context, _new_but_incomplete_data: PartialGuild) {}

    /// Dispatched when a invite is created.
    ///
    /// Provides data about the invite.
    async fn invite_create(&self, _ctx: Context, _data: InviteCreateEvent) {}

    /// Dispatched when a invite is deleted.
    ///
    /// Provides data about the invite.
    async fn invite_delete(&self, _ctx: Context, _data: InviteDeleteEvent) {}

    /// Dispatched when a message is created.
    ///
    /// Provides the message's data.
    async fn message(&self, _ctx: Context, _new_message: Message) {}

    /// Dispatched when a message is deleted.
    ///
    /// Provides the guild's id, the channel's id and the message's id.
    async fn message_delete(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _deleted_message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
    }

    /// Dispatched when multiple messages were deleted at once.
    ///
    /// Provides the guild's id, channel's id and the deleted messages' ids.
    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _multiple_deleted_messages_ids: Vec<MessageId>,
        _guild_id: Option<GuildId>,
    ) {
    }

    /// Dispatched when a message is updated.
    ///
    /// Provides the old message if available,
    /// the new message as an option in case of cache inconsistencies,
    /// and the raw [`MessageUpdateEvent`] as a fallback.
    #[cfg(feature = "cache")]
    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        _event: MessageUpdateEvent,
    ) {
    }

    /// Dispatched when a message is updated.
    ///
    /// Provides the new data of the message.
    #[cfg(not(feature = "cache"))]
    async fn message_update(&self, _ctx: Context, _new_data: MessageUpdateEvent) {}

    /// Dispatched when a new reaction is attached to a message.
    ///
    /// Provides the reaction's data.
    async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}

    /// Dispatched when a reaction is detached from a message.
    ///
    /// Provides the reaction's data.
    async fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {}

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _removed_from_message_id: MessageId,
    ) {
    }

    /// This event is legacy, and likely no longer sent by discord.
    async fn presence_replace(&self, _ctx: Context, _: Vec<Presence>) {}

    /// Dispatched when a user's presence is updated (e.g off -> on).
    ///
    /// Provides the presence's new data.
    ///
    /// Note: This event will not trigger unless the "guild presences" privileged intent
    /// is enabled on the bot application page.
    async fn presence_update(&self, _ctx: Context, _new_data: PresenceUpdateEvent) {}

    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {}

    /// Dispatched upon reconnection.
    async fn resume(&self, _ctx: Context, _: ResumedEvent) {}

    /// Dispatched when a shard's connection stage is updated
    ///
    /// Provides the context of the shard and the event information about the update.
    async fn shard_stage_update(&self, _ctx: Context, _: ShardStageUpdateEvent) {}

    /// Dispatched when a user starts typing.
    async fn typing_start(&self, _ctx: Context, _: TypingStartEvent) {}

    /// Dispatched when an unknown event was sent from discord.
    ///
    /// Provides the event's name and its unparsed data.
    async fn unknown(&self, _ctx: Context, _name: String, _raw: Value) {}

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the old and new data.
    #[cfg(feature = "cache")]
    async fn user_update(&self, _ctx: Context, _old_data: CurrentUser, _new: CurrentUser) {}

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the new data.
    #[cfg(not(feature = "cache"))]
    async fn user_update(&self, _ctx: Context, _new_data: CurrentUser) {}

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    ///
    /// Provides the voice server's data.
    async fn voice_server_update(&self, _ctx: Context, _: VoiceServerUpdateEvent) {}

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and
    /// the old and the new state of the guild's voice channels.
    #[cfg(feature = "cache")]
    async fn voice_state_update(
        &self,
        _ctx: Context,
        _: Option<GuildId>,
        _old: Option<VoiceState>,
        _new: VoiceState,
    ) {
    }

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and
    /// the new state of the guild's voice channels.
    #[cfg(not(feature = "cache"))]
    async fn voice_state_update(&self, _ctx: Context, _: Option<GuildId>, _: VoiceState) {}

    /// Dispatched when a guild's webhook is updated.
    ///
    /// Provides the guild's id and the channel's id the webhook belongs in.
    async fn webhook_update(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _belongs_to_channel_id: ChannelId,
    ) {
    }

    /// Dispatched when a user used a slash command.
    ///
    /// Provides the created interaction.
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {}
}

/// This core trait for handling raw events
#[async_trait]
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&self, _ctx: Context, _ev: Event) {}
}
