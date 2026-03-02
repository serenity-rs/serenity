use std::collections::VecDeque;
#[cfg(feature = "cache")]
use std::num::NonZeroU16;

use async_trait::async_trait;
use strum::{EnumCount, IntoStaticStr, VariantNames};

use super::context::Context;
#[cfg(doc)]
use crate::gateway::ShardRunner;
use crate::gateway::ShardStageUpdateEvent;
use crate::http::RatelimitInfo;
use crate::model::prelude::*;

#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Checks if the `event` should be dispatched or ignored. Returns `true` by default.
    ///
    /// Returning `false` will drop an event and prevent it being dispatched by any
    /// frameworks and will exclude it from any collectors.
    ///
    /// ## Warning
    ///
    /// Similar to [`RawEventHandler`], this method runs synchronously to the [`ShardRunner`], keep
    /// runtime complexity low.
    fn filter_event(&self, _context: &Context, _event: &Event) -> bool {
        true
    }

    /// Dispatches an event through this handler, allowing for event matching and handling
    /// based on individual event variants.
    async fn dispatch(&self, _context: &Context, _event: &FullEvent) {}

    /// Dispatched when an HTTP rate limit is hit.
    async fn ratelimit(&self, _data: RatelimitInfo) {}
}

macro_rules! full_event {
    ( $(
        $( #[doc = $doc:literal] )*
        $( #[deprecated = $deprecated:literal] )?
        $( #[cfg(feature = $feature:literal)] )?
        $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* };
    )* ) => {
        /// This enum stores every possible event that an [`EventHandler`] can receive.
        #[cfg_attr(not(feature = "unstable"), non_exhaustive)]
        #[derive(Clone, Debug, VariantNames, IntoStaticStr, EnumCount, Serialize)]
        #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
        pub enum FullEvent {
            $(
                $( #[doc = $doc] )*
                $( #[deprecated = $deprecated] )?
                $( #[cfg(feature = $feature)] )?
                #[cfg_attr(not(feature = "unstable"), non_exhaustive)]
                $variant_name {
                    $( $arg_name: $arg_type ),*
                },
            )*
        }
    }
}

full_event! {
    /// Dispatched when the permissions of an application command was updated.
    ///
    /// Provides said permission's data.
    CommandPermissionsUpdate { permission: CommandPermissions };
    /// Dispatched when an auto moderation rule was created.
    ///
    /// Provides said rule's data.
    AutoModRuleCreate { rule: AutoModRule };
    /// Dispatched when an auto moderation rule was updated.
    ///
    /// Provides said rule's data.
    AutoModRuleUpdate { rule: AutoModRule };
    /// Dispatched when an auto moderation rule was deleted.
    ///
    /// Provides said rule's data.
    AutoModRuleDelete { rule: AutoModRule };
    /// Dispatched when an auto moderation rule was triggered and an action was executed.
    ///
    /// Provides said action execution's data.
    AutoModActionExecution { execution: ActionExecution };
    /// Dispatched when the cache has received and inserted all data from guilds.
    ///
    /// This process happens upon starting your bot and should be fairly quick. However, cache
    /// actions performed prior this event may fail as the data could be not inserted yet.
    ///
    /// Provides the cached guilds' ids.
    #[cfg(feature = "cache")]
    CacheReady { guilds: Vec<GuildId> };
    /// Dispatched when every shard has received a Ready event
    #[cfg(feature = "cache")]
    ShardsReady { total_shards: NonZeroU16 };
    /// Dispatched when a channel is created.
    ///
    /// Provides said channel's data.
    ChannelCreate { channel: GuildChannel };
    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    CategoryCreate { category: GuildChannel };
    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    CategoryDelete { category: GuildChannel };
    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    ChannelDelete { channel: GuildChannel, messages: Option<VecDeque<Message>> };
    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    ChannelPinsUpdate { pin: ChannelPinsUpdateEvent };
    /// Dispatched when a channel is updated.
    ///
    /// The old channel data is only provided when the cache feature is enabled.
    ChannelUpdate { old: Option<GuildChannel>, new: GuildChannel };
    /// Dispatched when a new audit log entry is created.
    ///
    /// Provides said entry's data and the id of the guild where it was created.
    GuildAuditLogEntryCreate { entry: AuditLogEntry, guild_id: GuildId };
    /// Dispatched when a user is banned from a guild.
    ///
    /// Provides the guild's id and the banned user's data.
    GuildBanAddition { guild_id: GuildId, banned_user: User };
    /// Dispatched when a user's ban is lifted from a guild.
    ///
    /// Provides the guild's id and the lifted user's data.
    GuildBanRemoval { guild_id: GuildId, unbanned_user: User };
    /// Dispatched when a guild is created; or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data and whether the guild is new (only when cache feature is enabled).
    GuildCreate { guild: Guild, is_new: Option<bool> };
    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord, and the full data from the cache,
    /// if cache feature is enabled and the data is available.
    ///
    /// The [`unavailable`] flag in the partial data determines the status of the guild. If the
    /// flag is false, the bot was removed from the guild, either by being kicked or banned. If the
    /// flag is true, the guild went offline.
    ///
    /// [`unavailable`]: UnavailableGuild::unavailable
    GuildDelete { incomplete: UnavailableGuild, full: Option<Guild> };
    // the emojis were updated.
    /// Dispatched when the emojis are updated.
    ///
    /// Provides the guild's id and the new state of the emojis in the guild.
    GuildEmojisUpdate { guild_id: GuildId, current_state: ExtractMap<EmojiId, Emoji> };
    /// Dispatched when a guild's integration is added, updated or removed.
    ///
    /// Provides the guild's id.
    GuildIntegrationsUpdate { guild_id: GuildId };
    /// Dispatched when a user joins a guild.
    ///
    /// Provides the guild's id and the user's member data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberAddition { new_member: Member };
    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data, and the user's member data if cache feature is
    /// enabled and the data is available.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberRemoval { guild_id: GuildId, user: User, member_data_if_available: Option<Member> };
    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the member's old and new data (if cache feature is enabled and data is available)
    /// and the new raw data about updated fields.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberUpdate { old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent };
    /// Dispatched when the data for offline members was requested.
    ///
    /// Provides the guild's id and the data.
    GuildMembersChunk { chunk: GuildMembersChunkEvent };

    /// Dispatched when a role is created.
    ///
    /// Provides the guild's id and the new role's data.
    GuildRoleCreate { new: Role };

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id and its data (if cache feature is enabled and the
    /// data is available).
    GuildRoleDelete { guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> };

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id, the role's old (if cache feature is enabled and the data is
    /// available) and new data.
    GuildRoleUpdate { old_data_if_available: Option<Role>, new: Role };

    /// Dispatched when the stickers are updated.
    ///
    /// Provides the guild's id and the new state of the stickers in the guild.
    GuildStickersUpdate { guild_id: GuildId, current_state: ExtractMap<StickerId, Sticker> };

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's old data (if cache feature is enabled and the data is available)
    /// and the new data.
    GuildUpdate { old_data_if_available: Option<Guild>, new_data: PartialGuild };

    /// Dispatched when a invite is created.
    ///
    /// Provides data about the invite.
    InviteCreate { data: InviteCreateEvent };

    /// Dispatched when a invite is deleted.
    ///
    /// Provides data about the invite.
    InviteDelete { data: InviteDeleteEvent };

    /// Dispatched when a message is created.
    ///
    /// Provides the message's data.
    Message { new_message: Message };

    /// Dispatched when a message is deleted.
    ///
    /// Provides the guild's id, the channel's id and the message's id.
    MessageDelete { channel_id: GenericChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId> };
    /// Dispatched when multiple messages were deleted at once.
    ///
    /// Provides the guild's id, channel's id and the deleted messages' ids.
    MessageDeleteBulk { channel_id: GenericChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId> };

    /// Dispatched when a message is updated.
    ///
    /// Provides the message update data, as well as the old message if cache feature is enabled and
    /// the data is available.
    MessageUpdate { old_if_available: Option<Message>, event: MessageUpdateEvent };

    /// Dispatched when a new reaction is attached to a message.
    ///
    /// Provides the reaction's data.
    ReactionAdd { old_message_if_available: Option<Message>, add_reaction: Reaction };

    /// Dispatched when a reaction is detached from a message.
    ///
    /// Provides the reaction's data.
    ReactionRemove { old_message_if_available: Option<Message>, removed_reaction: Reaction };

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id, message's id, and guild's id if in a guild.
    ReactionRemoveAll { old_message_if_available: Option<Message>, guild_id: Option<GuildId>, channel_id: GenericChannelId, removed_from_message_id: MessageId };

    /// Dispatched when all reactions of a message for a given emoji are explicitly detached from a message.
    ///
    /// Provides the emoji's data, channel's id, message's id and guild's id if in a guild.
    ReactionRemoveEmoji { old_message_if_available: Option<Message>, removed_reactions: Reaction };

    /// Dispatched when a user's presence is updated (e.g off -> on).
    ///
    /// Provides the presence's new data, as well as the old presence data if the
    /// cache feature is enabled and the data is available.
    ///
    /// Note: This event will not trigger unless the "guild presences" privileged intent is enabled
    /// on the bot application page.
    PresenceUpdate { old_data: Option<Presence>, new_data: Presence };

    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    Ready { data_about_bot: Ready };
    /// Dispatched upon reconnection.
    Resume { event: ResumedEvent };

    /// Dispatched when a shard's connection stage is updated
    ///
    /// Provides the context of the shard and the event information about the update.
    ShardStageUpdate { event: ShardStageUpdateEvent };

    /// Dispatched when the data for soundboard sounds is requested.
    ///
    /// Provides the guild's id and the data.
    SoundboardSounds { event: SoundboardSoundsEvent };

    /// Dispatched when a soundboard sound is created.
    SoundboardSoundCreate { event: SoundboardSoundCreateEvent };

    /// Dispatched when a soundboard sound is updated.
    SoundboardSoundUpdate { event: SoundboardSoundUpdateEvent };

    /// Dispatched when multiple soundboard sounds at once are updated.
    SoundboardSoundsUpdate { event: SoundboardSoundsUpdateEvent };

    /// Dispatched when a soundboard sound is deleted.
    SoundboardSoundDelete { event: SoundboardSoundDeleteEvent };

    /// Dispatched when a user starts typing.
    TypingStart { event: TypingStartEvent };

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the old (if cache feature is enabled and the data is available) and new data.
    UserUpdate { old_data: Option<CurrentUser>, new: CurrentUser };

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    ///
    /// Provides the voice server's data.
    VoiceServerUpdate { event: VoiceServerUpdateEvent };

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and the old state (if cache feature is enabled and
    /// [`GatewayIntents::GUILDS`] is enabled) and the new state of the guild's voice channels.
    VoiceStateUpdate { old: Option<VoiceState>, new: VoiceState };

    /// Dispatched when a voice channel's status is updated.
    ///
    /// Provides the status, channel's id and the guild's id.
    VoiceChannelStatusUpdate { old: Option<String>, status: Option<String>, id: ChannelId,  guild_id: GuildId };
    /// Dispatched when a guild's webhook is updated.
    ///
    /// Provides the guild's id and the channel's id the webhook belongs in.
    WebhookUpdate { guild_id: GuildId, belongs_to_channel_id: ChannelId };
    /// Dispatched when an interaction is created (e.g a slash command was used or a button was
    /// clicked).
    ///
    /// Provides the created interaction.
    InteractionCreate { interaction: Interaction };
    /// Dispatched when a guild integration is created.
    ///
    /// Provides the created integration.
    IntegrationCreate { integration: Integration };
    /// Dispatched when a guild integration is updated.
    ///
    /// Provides the updated integration.
    IntegrationUpdate { integration: Integration };
    /// Dispatched when a guild integration is deleted.
    ///
    /// Provides the integration's id, the id of the guild it belongs to, and its associated
    /// application id
    IntegrationDelete { integration_id: IntegrationId, guild_id: GuildId, application_id: Option<ApplicationId> };
    /// Dispatched when a stage instance is created.
    ///
    /// Provides the created stage instance.
    StageInstanceCreate { stage_instance: StageInstance };
    /// Dispatched when a stage instance is updated.
    ///
    /// Provides the updated stage instance.
    StageInstanceUpdate { stage_instance: StageInstance };
    /// Dispatched when a stage instance is deleted.
    ///
    /// Provides the deleted stage instance.
    StageInstanceDelete { stage_instance: StageInstance };
    /// Dispatched when a thread is created or the current user is added to a private thread.
    ///
    /// Provides the thread and if the thread was newly created.
    ThreadCreate { thread: GuildThread, newly_created: Option<bool> };
    /// Dispatched when a thread is updated.
    ///
    /// Provides the updated thread and the old thread data, provided the thread was cached prior to
    /// dispatch.
    ThreadUpdate { old: Option<GuildThread>, new: GuildThread };
    /// Dispatched when a thread is deleted.
    ///
    /// Provides the partial data about the deleted thread and, if it was present in the cache
    /// before its deletion, its full data.
    ThreadDelete { thread: PartialGuildThread, full_thread_data: Option<GuildThread> };
    /// Dispatched when the current user gains access to a channel.
    ///
    /// Provides the threads the current user can access, the thread members, the guild Id, and the
    /// channel Ids of the parent channels being synced.
    ThreadListSync { thread_list_sync: ThreadListSyncEvent };
    /// Dispatched when the [`ThreadMember`] for the current user is updated.
    ///
    /// Provides the updated thread member.
    ThreadMemberUpdate { thread_member: ThreadMember };
    /// Dispatched when anyone is added to or removed from a thread. If the current user does not
    /// have the [`GatewayIntents::GUILDS`], then this event will only be sent if the current user
    /// was added to or removed from the thread.
    ///
    /// Provides the added/removed members, the approximate member count of members in the thread,
    /// the thread Id and its guild Id.
    ///
    /// [`GatewayIntents::GUILDS`]: crate::model::gateway::GatewayIntents::GUILDS
    ThreadMembersUpdate { thread_members_update: ThreadMembersUpdateEvent };
    /// Dispatched when a scheduled event is created.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventCreate { event: ScheduledEvent };
    /// Dispatched when a scheduled event is updated.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventUpdate { event: ScheduledEvent };
    /// Dispatched when a scheduled event is deleted.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventDelete { event: ScheduledEvent };
    /// Dispatched when a guild member has subscribed to a scheduled event.
    ///
    /// Provides data about the subscription.
    GuildScheduledEventUserAdd { subscribed: GuildScheduledEventUserAddEvent };
    /// Dispatched when a guild member has unsubscribed from a scheduled event.
    ///
    /// Provides data about the cancelled subscription.
    GuildScheduledEventUserRemove { unsubscribed: GuildScheduledEventUserRemoveEvent };
    /// Dispatched when a user subscribes to a SKU.
    ///
    /// Provides data about the subscription.
    EntitlementCreate { entitlement: Entitlement };
    /// Dispatched when a user's entitlement has been updated, such as when a subscription is
    /// renewed for the next billing period.
    ///
    /// Provides data abut the updated subscription. If the entitlement is renewed, the
    /// [`Entitlement::ends_at`] field will have changed.
    EntitlementUpdate { entitlement: Entitlement };
    /// Dispatched when a user's entitlement has been deleted. This happens rarely, but can occur
    /// if a subscription is refunded or otherwise deleted by Discord. Entitlements are not deleted
    /// when they expire.
    ///
    /// Provides data about the subscription. Specifically, the [`Entitlement::deleted`] field will
    /// be set.
    EntitlementDelete { entitlement: Entitlement };
    /// Dispatched when a user votes on a message poll.
    ///
    /// This will be dispatched multiple times if multiple answers are selected.
    MessagePollVoteAdd { event: MessagePollVoteAddEvent };
    /// Dispatched when a user removes a previous vote on a poll.
    MessagePollVoteRemove { event: MessagePollVoteRemoveEvent };
}

/// An event handler that receives raw `dispatch` events.
///
/// ## Warning
/// As this is a low level trait, the methods of this trait are run on the same tokio task as the
/// [`ShardRunner`].
///
/// This means that if any of these methods take too long to return, the shard may drop events or be
/// disconnected entirely.
///
/// It is recommended to clone the fields needed out of [`Event`], then spawn a task to run
/// concurrently to the shard loop.
#[async_trait]
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&self, _ctx: Context, _ev: &Event) {}

    /// Checks if the `event` should be dispatched or ignored. Returns `true` by default.
    ///
    /// Returning `false` will drop an event and prevent it being dispatched by any frameworks and
    /// will exclude it from any collectors.
    fn filter_event(&self, _context: &Context, _event: &Event) -> bool {
        // Suppress unused argument warnings
        true
    }
}
