use std::collections::VecDeque;
#[cfg(feature = "cache")]
use std::num::NonZeroU16;

use async_trait::async_trait;
use strum::{EnumCount, IntoStaticStr, VariantNames};

use super::context::Context;
use crate::gateway::ShardStageUpdateEvent;
use crate::http::RatelimitInfo;
use crate::model::prelude::*;

macro_rules! event_handler {
    ( $(
        $( #[doc = $doc:literal] )*
        $( #[deprecated = $deprecated:literal] )?
        $( #[cfg(feature = $feature:literal)] )?
        $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* } => async fn $method_name:ident(&self $(, $context:ident: Context)?);
    )* ) => {
        /// The core trait for handling events by serenity.
        #[async_trait]
        pub trait EventHandler: Send + Sync {
            $(
                $( #[doc = $doc] )*
                $( #[cfg(feature = $feature)] )?
                $( #[deprecated = $deprecated] )?
                async fn $method_name(&self, $($context: Context,)? $( $arg_name: $arg_type ),*) {
                    // Suppress unused argument warnings
                    drop(( $($context,)? $($arg_name),* ))
                }
            )*

            /// Checks if the `event` should be dispatched or ignored. Returns `true` by default.
            ///
            /// Returning `false` will drop an event and prevent it being dispatched by any
            /// frameworks and will exclude it from any collectors.
            ///
            /// ## Warning
            ///
            /// This will run synchronously on every event in the dispatch loop
            /// of the shard that is receiving the event. If your filter code
            /// takes too long, it may delay other events from being dispatched
            /// in a timely manner. It is recommended to keep the runtime
            /// complexity of the filter code low to avoid unnecessarily blocking
            /// your bot.
            fn filter_event(&self, _context: &Context, _event: &Event) -> bool {
                true
            }
        }

        /// This enum stores every possible event that an [`EventHandler`] can receive.
        #[cfg_attr(not(feature = "unstable"), non_exhaustive)]
        #[derive(Clone, Debug, VariantNames, IntoStaticStr, EnumCount, Serialize)]
        #[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
        pub enum FullEvent {
            $(
                $( #[doc = $doc] )*
                $( #[cfg(feature = $feature)] )?
                $( #[deprecated = $deprecated] )?
                $variant_name {
                    $( $arg_name: $arg_type ),*
                },
            )*
        }

        impl FullEvent {
            /// Returns the name of this event as a snake case string
            ///
            /// ```rust,no_run
            /// # use serenity::gateway::client::{Context, FullEvent};
            /// # fn foo_(ctx: Context, event: FullEvent) {
            /// if let FullEvent::Message { .. } = &event {
            ///     assert_eq!(event.snake_case_name(), "message");
            /// }
            /// # }
            /// ```
            #[must_use]
            pub fn snake_case_name(&self) -> &'static str {
                match self {
                    $(
                        $( #[cfg(feature = $feature)] )?
                        Self::$variant_name { .. } => stringify!($method_name),
                    )*
                }
            }

            /// Runs the given [`EventHandler`]'s code for this event.
            pub async fn dispatch(self, ctx: Context, handler: &dyn EventHandler) {
                match self {
                    $(
                        $( #[cfg(feature = $feature)] )?
                        Self::$variant_name { $( $arg_name ),* } => {
                            $( let $context = ctx; )?
                            handler.$method_name( $($context,)? $( $arg_name ),* ).await;
                        }
                    )*
                }
            }
        }
    };
}

event_handler! {
    /// Dispatched when the permissions of an application command was updated.
    ///
    /// Provides said permission's data.
    CommandPermissionsUpdate { permission: CommandPermissions } => async fn command_permissions_update(&self, ctx: Context);

    /// Dispatched when an auto moderation rule was created.
    ///
    /// Provides said rule's data.
    AutoModRuleCreate { rule: AutoModRule } => async fn auto_moderation_rule_create(&self, ctx: Context);

    /// Dispatched when an auto moderation rule was updated.
    ///
    /// Provides said rule's data.
    AutoModRuleUpdate { rule: AutoModRule } => async fn auto_moderation_rule_update(&self, ctx: Context);

    /// Dispatched when an auto moderation rule was deleted.
    ///
    /// Provides said rule's data.
    AutoModRuleDelete { rule: AutoModRule } => async fn auto_moderation_rule_delete(&self, ctx: Context);

    /// Dispatched when an auto moderation rule was triggered and an action was executed.
    ///
    /// Provides said action execution's data.
    AutoModActionExecution { execution: ActionExecution } => async fn auto_moderation_action_execution(&self, ctx: Context);

    /// Dispatched when the cache has received and inserted all data from guilds.
    ///
    /// This process happens upon starting your bot and should be fairly quick. However, cache
    /// actions performed prior this event may fail as the data could be not inserted yet.
    ///
    /// Provides the cached guilds' ids.
    #[cfg(feature = "cache")]
    CacheReady { guilds: Vec<GuildId> } => async fn cache_ready(&self, ctx: Context);

    /// Dispatched when every shard has received a Ready event
    #[cfg(feature = "cache")]
    ShardsReady { total_shards: NonZeroU16 } => async fn shards_ready(&self, ctx: Context);

    /// Dispatched when a channel is created.
    ///
    /// Provides said channel's data.
    ChannelCreate { channel: GuildChannel } => async fn channel_create(&self, ctx: Context);

    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    CategoryCreate { category: GuildChannel } => async fn category_create(&self, ctx: Context);

    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    CategoryDelete { category: GuildChannel } => async fn category_delete(&self, ctx: Context);

    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    ChannelDelete { channel: GuildChannel, messages: Option<VecDeque<Message>> } => async fn channel_delete(&self, ctx: Context);

    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    ChannelPinsUpdate { pin: ChannelPinsUpdateEvent } => async fn channel_pins_update(&self, ctx: Context);

    /// Dispatched when a channel is updated.
    ///
    /// The old channel data is only provided when the cache feature is enabled.
    ChannelUpdate { old: Option<GuildChannel>, new: GuildChannel } => async fn channel_update(&self, ctx: Context);

    /// Dispatched when a new audit log entry is created.
    ///
    /// Provides said entry's data and the id of the guild where it was created.
    GuildAuditLogEntryCreate { entry: AuditLogEntry, guild_id: GuildId } => async fn guild_audit_log_entry_create(&self, ctx: Context);

    /// Dispatched when a user is banned from a guild.
    ///
    /// Provides the guild's id and the banned user's data.
    GuildBanAddition { guild_id: GuildId, banned_user: User } => async fn guild_ban_addition(&self, ctx: Context);

    /// Dispatched when a user's ban is lifted from a guild.
    ///
    /// Provides the guild's id and the lifted user's data.
    GuildBanRemoval { guild_id: GuildId, unbanned_user: User } => async fn guild_ban_removal(&self, ctx: Context);

    /// Dispatched when a guild is created; or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data and whether the guild is new (only when cache feature is enabled).
    GuildCreate { guild: Guild, is_new: Option<bool> } => async fn guild_create(&self, ctx: Context);

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
    GuildDelete { incomplete: UnavailableGuild, full: Option<Guild> } => async fn guild_delete(&self, ctx: Context);

    // the emojis were updated.

    /// Dispatched when the emojis are updated.
    ///
    /// Provides the guild's id and the new state of the emojis in the guild.
    GuildEmojisUpdate { guild_id: GuildId, current_state: ExtractMap<EmojiId, Emoji> } => async fn guild_emojis_update(&self, ctx: Context);

    /// Dispatched when a guild's integration is added, updated or removed.
    ///
    /// Provides the guild's id.
    GuildIntegrationsUpdate { guild_id: GuildId } => async fn guild_integrations_update(&self, ctx: Context);

    /// Dispatched when a user joins a guild.
    ///
    /// Provides the guild's id and the user's member data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberAddition { new_member: Member } => async fn guild_member_addition(&self, ctx: Context);

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data, and the user's member data if cache feature is
    /// enabled and the data is available.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberRemoval { guild_id: GuildId, user: User, member_data_if_available: Option<Member> } => async fn guild_member_removal(&self, ctx: Context);

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the member's old and new data (if cache feature is enabled and data is available)
    /// and the new raw data about updated fields.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent is enabled
    /// on the bot application page.
    GuildMemberUpdate { old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent } => async fn guild_member_update(&self, ctx: Context);

    /// Dispatched when the data for offline members was requested.
    ///
    /// Provides the guild's id and the data.
    GuildMembersChunk { chunk: GuildMembersChunkEvent } => async fn guild_members_chunk(&self, ctx: Context);

    /// Dispatched when a role is created.
    ///
    /// Provides the guild's id and the new role's data.
    GuildRoleCreate { new: Role } => async fn guild_role_create(&self, ctx: Context);

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id and its data (if cache feature is enabled and the
    /// data is available).
    GuildRoleDelete { guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> } => async fn guild_role_delete(&self, ctx: Context);

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id, the role's old (if cache feature is enabled and the data is
    /// available) and new data.
    GuildRoleUpdate { old_data_if_available: Option<Role>, new: Role } => async fn guild_role_update(&self, ctx: Context);

    /// Dispatched when the stickers are updated.
    ///
    /// Provides the guild's id and the new state of the stickers in the guild.
    GuildStickersUpdate { guild_id: GuildId, current_state: ExtractMap<StickerId, Sticker> } => async fn guild_stickers_update(&self, ctx: Context);

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's old data (if cache feature is enabled and the data is available)
    /// and the new data.
    GuildUpdate { old_data_if_available: Option<Guild>, new_data: PartialGuild } => async fn guild_update(&self, ctx: Context);

    /// Dispatched when a invite is created.
    ///
    /// Provides data about the invite.
    InviteCreate { data: InviteCreateEvent } => async fn invite_create(&self, ctx: Context);

    /// Dispatched when a invite is deleted.
    ///
    /// Provides data about the invite.
    InviteDelete { data: InviteDeleteEvent } => async fn invite_delete(&self, ctx: Context);

    /// Dispatched when a message is created.
    ///
    /// Provides the message's data.
    Message { new_message: Message } => async fn message(&self, ctx: Context);

    /// Dispatched when a message is deleted.
    ///
    /// Provides the guild's id, the channel's id and the message's id.
    MessageDelete { channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId> } => async fn message_delete(&self, ctx: Context);

    /// Dispatched when multiple messages were deleted at once.
    ///
    /// Provides the guild's id, channel's id and the deleted messages' ids.
    MessageDeleteBulk { channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId> } => async fn message_delete_bulk(&self, ctx: Context);

    /// Dispatched when a message is updated.
    ///
    /// Provides the message update data, as well as the actual old and new message if cache
    /// feature is enabled and the data is available.
    MessageUpdate { old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent } => async fn message_update(&self, ctx: Context);

    /// Dispatched when a new reaction is attached to a message.
    ///
    /// Provides the reaction's data.
    ReactionAdd { add_reaction: Reaction } => async fn reaction_add(&self, ctx: Context);

    /// Dispatched when a reaction is detached from a message.
    ///
    /// Provides the reaction's data.
    ReactionRemove { removed_reaction: Reaction } => async fn reaction_remove(&self, ctx: Context);

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    ReactionRemoveAll { channel_id: ChannelId, removed_from_message_id: MessageId } => async fn reaction_remove_all(&self, ctx: Context);

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    ReactionRemoveEmoji { removed_reactions: Reaction } => async fn reaction_remove_emoji(&self, ctx: Context);

    /// Dispatched when a user's presence is updated (e.g off -> on).
    ///
    /// Provides the presence's new data, as well as the old presence data if the
    /// cache feature is enabled and the data is available.
    ///
    /// Note: This event will not trigger unless the "guild presences" privileged intent is enabled
    /// on the bot application page.
    PresenceUpdate { old_data: Option<Presence>, new_data: Presence } => async fn presence_update(&self, ctx: Context);

    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    Ready { data_about_bot: Ready } => async fn ready(&self, ctx: Context);

    /// Dispatched upon reconnection.
    Resume { event: ResumedEvent } => async fn resume(&self, ctx: Context);

    /// Dispatched when a shard's connection stage is updated
    ///
    /// Provides the context of the shard and the event information about the update.
    ShardStageUpdate { event: ShardStageUpdateEvent } => async fn shard_stage_update(&self, ctx: Context);

    /// Dispatched when a user starts typing.
    TypingStart { event: TypingStartEvent } => async fn typing_start(&self, ctx: Context);

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the old (if cache feature is enabled and the data is available) and new data.
    UserUpdate { old_data: Option<CurrentUser>, new: CurrentUser } => async fn user_update(&self, ctx: Context);

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    ///
    /// Provides the voice server's data.
    VoiceServerUpdate { event: VoiceServerUpdateEvent } => async fn voice_server_update(&self, ctx: Context);

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and the old state (if cache feature is enabled and
    /// [`GatewayIntents::GUILDS`] is enabled) and the new state of the guild's voice channels.
    VoiceStateUpdate { old: Option<VoiceState>, new: VoiceState } => async fn voice_state_update(&self, ctx: Context);

    /// Dispatched when a voice channel's status is updated.
    ///
    /// Provides the status, channel's id and the guild's id.
    VoiceChannelStatusUpdate { old: Option<String>, status: Option<String>, id: ChannelId, guild_id: GuildId } => async fn voice_channel_status_update(&self, ctx: Context);

    /// Dispatched when a guild's webhook is updated.
    ///
    /// Provides the guild's id and the channel's id the webhook belongs in.
    WebhookUpdate { guild_id: GuildId, belongs_to_channel_id: ChannelId } => async fn webhook_update(&self, ctx: Context);

    /// Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    ///
    /// Provides the created interaction.
    InteractionCreate { interaction: Interaction } => async fn interaction_create(&self, ctx: Context);

    /// Dispatched when a guild integration is created.
    ///
    /// Provides the created integration.
    IntegrationCreate { integration: Integration } => async fn integration_create(&self, ctx: Context);

    /// Dispatched when a guild integration is updated.
    ///
    /// Provides the updated integration.
    IntegrationUpdate { integration: Integration } => async fn integration_update(&self, ctx: Context);

    /// Dispatched when a guild integration is deleted.
    ///
    /// Provides the integration's id, the id of the guild it belongs to, and its associated application id
    IntegrationDelete { integration_id: IntegrationId, guild_id: GuildId, application_id: Option<ApplicationId> } => async fn integration_delete(&self, ctx: Context);

    /// Dispatched when a stage instance is created.
    ///
    /// Provides the created stage instance.
    StageInstanceCreate { stage_instance: StageInstance } => async fn stage_instance_create(&self, ctx: Context);

    /// Dispatched when a stage instance is updated.
    ///
    /// Provides the updated stage instance.
    StageInstanceUpdate { stage_instance: StageInstance } => async fn stage_instance_update(&self, ctx: Context);

    /// Dispatched when a stage instance is deleted.
    ///
    /// Provides the deleted stage instance.
    StageInstanceDelete { stage_instance: StageInstance } => async fn stage_instance_delete(&self, ctx: Context);

    /// Dispatched when a thread is created or the current user is added to a private thread.
    ///
    /// Provides the thread.
    ThreadCreate { thread: GuildChannel } => async fn thread_create(&self, ctx: Context);

    /// Dispatched when a thread is updated.
    ///
    /// Provides the updated thread and the old thread data, provided the thread was cached prior to dispatch.
    ThreadUpdate { old: Option<GuildChannel>, new: GuildChannel } => async fn thread_update(&self, ctx: Context);

    /// Dispatched when a thread is deleted.
    ///
    /// Provides the partial data about the deleted thread and, if it was present in the cache
    /// before its deletion, its full data.
    ThreadDelete { thread: PartialGuildChannel, full_thread_data: Option<GuildChannel> } => async fn thread_delete(&self, ctx: Context);

    /// Dispatched when the current user gains access to a channel.
    ///
    /// Provides the threads the current user can access, the thread members, the guild Id, and the
    /// channel Ids of the parent channels being synced.
    ThreadListSync { thread_list_sync: ThreadListSyncEvent } => async fn thread_list_sync(&self, ctx: Context);

    /// Dispatched when the [`ThreadMember`] for the current user is updated.
    ///
    /// Provides the updated thread member.
    ThreadMemberUpdate { thread_member: ThreadMember } => async fn thread_member_update(&self, ctx: Context);

    /// Dispatched when anyone is added to or removed from a thread. If the current user does not
    /// have the [`GatewayIntents::GUILDS`], then this event will only be sent if the current user
    /// was added to or removed from the thread.
    ///
    /// Provides the added/removed members, the approximate member count of members in the thread,
    /// the thread Id and its guild Id.
    ///
    /// [`GatewayIntents::GUILDS`]: crate::model::gateway::GatewayIntents::GUILDS
    ThreadMembersUpdate { thread_members_update: ThreadMembersUpdateEvent } => async fn thread_members_update(&self, ctx: Context);

    /// Dispatched when a scheduled event is created.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventCreate { event: ScheduledEvent } => async fn guild_scheduled_event_create(&self, ctx: Context);

    /// Dispatched when a scheduled event is updated.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventUpdate { event: ScheduledEvent } => async fn guild_scheduled_event_update(&self, ctx: Context);

    /// Dispatched when a scheduled event is deleted.
    ///
    /// Provides data about the scheduled event.
    GuildScheduledEventDelete { event: ScheduledEvent } => async fn guild_scheduled_event_delete(&self, ctx: Context);

    /// Dispatched when a guild member has subscribed to a scheduled event.
    ///
    /// Provides data about the subscription.
    GuildScheduledEventUserAdd { subscribed: GuildScheduledEventUserAddEvent } => async fn guild_scheduled_event_user_add(&self, ctx: Context);

    /// Dispatched when a guild member has unsubscribed from a scheduled event.
    ///
    /// Provides data about the cancelled subscription.
    GuildScheduledEventUserRemove { unsubscribed: GuildScheduledEventUserRemoveEvent } => async fn guild_scheduled_event_user_remove(&self, ctx: Context);

    /// Dispatched when a user subscribes to a SKU.
    ///
    /// Provides data about the subscription.
    EntitlementCreate { entitlement: Entitlement } => async fn entitlement_create(&self, ctx: Context);

    /// Dispatched when a user's entitlement has been updated, such as when a subscription is
    /// renewed for the next billing period.
    ///
    /// Provides data abut the updated subscription. If the entitlement is renewed, the
    /// [`Entitlement::ends_at`] field will have changed.
    EntitlementUpdate { entitlement: Entitlement } => async fn entitlement_update(&self, ctx: Context);

    /// Dispatched when a user's entitlement has been deleted. This happens rarely, but can occur
    /// if a subscription is refunded or otherwise deleted by Discord. Entitlements are not deleted
    /// when they expire.
    ///
    /// Provides data about the subscription. Specifically, the [`Entitlement::deleted`] field will
    /// be set.
    EntitlementDelete { entitlement: Entitlement } => async fn entitlement_delete(&self, ctx: Context);

    /// Dispatched when a user votes on a message poll.
    ///
    /// This will be dispatched multiple times if multiple answers are selected.
    MessagePollVoteAdd { event: MessagePollVoteAddEvent } => async fn poll_vote_add(&self, ctx: Context);

    /// Dispatched when a user removes a previous vote on a poll.
    MessagePollVoteRemove { event: MessagePollVoteRemoveEvent } => async fn poll_vote_remove(&self, ctx: Context);

    /// Dispatched when an HTTP rate limit is hit
    Ratelimit { data: RatelimitInfo } => async fn ratelimit(&self);
}

/// This core trait for handling raw events
#[async_trait]
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&self, _ctx: Context, _ev: &Event) {}

    /// Checks if the `event` should be dispatched or ignored. Returns `true` by default.
    ///
    /// Returning `false` will drop an event and prevent it being dispatched by any frameworks and
    /// will exclude it from any collectors.
    ///
    /// ## Warning
    ///
    /// This will run synchronously on every event in the dispatch loop
    /// of the shard that is receiving the event. If your filter code
    /// takes too long, it may delay other events from being dispatched
    /// in a timely manner. It is recommended to keep the runtime
    /// complexity of the filter code low to avoid unnecessarily blocking
    /// your bot.
    fn filter_event(&self, _context: &Context, _event: &Event) -> bool {
        // Suppress unused argument warnings
        true
    }
}
