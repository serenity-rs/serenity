use std::collections::HashMap;

use async_trait::async_trait;

use super::context::Context;
use crate::client::bridge::gateway::event::*;
use crate::http::ratelimiting::RatelimitInfo;
use crate::model::application::command::CommandPermission;
use crate::model::application::interaction::Interaction;
use crate::model::guild::automod::{ActionExecution, Rule};
use crate::model::prelude::*;

macro_rules! event_handler {
    ( $(
        $( #[doc = $doc:literal] )*
        $( #[cfg(feature = $feature:literal)] )?
        async fn $method_name:ident(&self, $variant_name:ident { $(
            $arg_name:ident: $arg_type:ty
        ),* $(,)? } );
    )* ) => {
        /// The core trait for handling events by serenity.
        #[async_trait]
        pub trait EventHandler: Send + Sync { $(
            $( #[doc = $doc] )* $( #[cfg(feature = $feature)] )?
            async fn $method_name(&self, $( $arg_name: $arg_type ),* ) {
                // Suppress unused argument warnings
                drop(( $( $arg_name ),* ));
            }
        )* }

        /// This enum stores every possible event that an [`EventHandler`] can receive.
        #[non_exhaustive]
        #[allow(clippy::large_enum_variant)] // TODO: do some boxing to fix this
        #[derive(Clone, Debug)]
        pub enum FullEvent { $(
            $( #[doc = $doc] )* $( #[cfg(feature = $feature)] )?
            $variant_name {
                $( $arg_name: $arg_type ),*
            },
        )* }

        impl FullEvent {
            /// Returns the name of this event as a snake case string
            ///
            /// ```rust,no_run
            /// # use serenity::client::{Context, FullEvent};
            /// # fn _foo(ctx: Context, event: FullEvent) {
            /// if let FullEvent::Message { .. } = &event {
            ///     assert_eq!(event.snake_case_name(), "message");
            /// }
            /// # }
            /// ```
            #[must_use]
            pub fn snake_case_name(&self) -> &'static str {
                match self { $(
                    $( #[cfg(feature = $feature)] )?
                    Self::$variant_name { .. } => stringify!($method_name),
                )* }
            }

            /// Runs the given [`EventHandler`]'s code for this event.
            pub async fn dispatch(self, handler: &dyn EventHandler) {
                match self { $(
                    $( #[cfg(feature = $feature)] )?
                    Self::$variant_name { $( $arg_name ),* } => {
                        handler.$method_name( $( $arg_name ),* ).await;
                    }
                )* }
            }
        }
    };
}

event_handler! {
    /// Dispatched when the permissions of an application command was updated.
    ///
    /// Provides said permission's data.
    async fn application_command_permissions_update(&self, CommandPermissionsUpdate { ctx: Context, permission: CommandPermission });

    /// Dispatched when an auto moderation rule was created.
    ///
    /// Provides said rule's data.
    async fn auto_moderation_rule_create(&self, AutoModRuleCreate { ctx: Context, rule: Rule });

    /// Dispatched when an auto moderation rule was updated.
    ///
    /// Provides said rule's data.
    async fn auto_moderation_rule_update(&self, AutoModRuleUpdate { ctx: Context, rule: Rule });

    /// Dispatched when an auto moderation rule was deleted.
    ///
    /// Provides said rule's data.
    async fn auto_moderation_rule_delete(&self, AutoModRuleDelete { ctx: Context, rule: Rule });

    /// Dispatched when an auto moderation rule was triggered and an action was executed.
    ///
    /// Provides said action execution's data.
    async fn auto_moderation_action_execution(&self, AutoModActionExecution { ctx: Context, execution: ActionExecution });

    /// Dispatched when the cache has received and inserted all data from
    /// guilds.
    ///
    /// This process happens upon starting your bot and should be fairly quick.
    /// However, cache actions performed prior this event may fail as the data
    /// could be not inserted yet.
    ///
    /// Provides the cached guilds' ids.
    #[cfg(feature = "cache")]
    async fn cache_ready(&self, CacheReady { ctx: Context, guilds: Vec<GuildId> });

    /// Dispatched when every shard has received a Ready event
    #[cfg(feature = "cache")]
    async fn shards_ready(&self, ShardsReady { ctx: Context, total_shards: u32 });

    /// Dispatched when a channel is created.
    ///
    /// Provides said channel's data.
    async fn channel_create(&self, ChannelCreate { ctx: Context, channel: GuildChannel });

    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    async fn category_create(&self, CategoryCreate { ctx: Context, category: GuildChannel });

    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    async fn category_delete(&self, CategoryDelete { ctx: Context, category: GuildChannel });

    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    async fn channel_delete(&self, ChannelDelete { ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>> });

    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    async fn channel_pins_update(&self, ChannelPinsUpdate { ctx: Context, pin: ChannelPinsUpdateEvent });

    /// Dispatched when a channel is updated.
    ///
    /// The old channel data is only provided when the cache feature is enabled.
    async fn channel_update(&self, ChannelUpdate { ctx: Context, old: Option<Channel>, new: Channel });

    /// Dispatched when a user is banned from a guild.
    ///
    /// Provides the guild's id and the banned user's data.
    async fn guild_ban_addition(&self, GuildBanAddition { ctx: Context, guild_id: GuildId, banned_user: User });

    /// Dispatched when a user's ban is lifted from a guild.
    ///
    /// Provides the guild's id and the lifted user's data.
    async fn guild_ban_removal(&self, GuildBanRemoval { ctx: Context, guild_id: GuildId, unbanned_user: User });

    /// Dispatched when a guild is created;
    /// or an existing guild's data is sent to us.
    ///
    /// Provides the guild's data and whether the guild is new (only when cache feature is enabled).
    async fn guild_create(&self, GuildCreate { ctx: Context, guild: Guild, is_new: Option<bool> });

    /// Dispatched when a guild is deleted.
    ///
    /// Provides the partial data of the guild sent by discord,
    /// and the full data from the cache, if cache feature is enabled and the data is available.
    ///
    /// The [`unavailable`] flag in the partial data determines the status of the guild.
    /// If the flag is false, the bot was removed from the guild, either by being
    /// kicked or banned. If the flag is true, the guild went offline.
    ///
    /// [`unavailable`]: UnavailableGuild::unavailable
    async fn guild_delete(&self, GuildDelete { ctx: Context, incomplete: UnavailableGuild, full: Option<Guild> });

    // the emojis were updated.

    /// Dispatched when the emojis are updated.
    ///
    /// Provides the guild's id and the new state of the emojis in the guild.
    async fn guild_emojis_update(&self, GuildEmojisUpdate { ctx: Context, guild_id: GuildId, current_state: HashMap<EmojiId, Emoji> });

    /// Dispatched when a guild's integration is added, updated or removed.
    ///
    /// Provides the guild's id.
    async fn guild_integrations_update(&self, GuildIntegrationsUpdate { ctx: Context, guild_id: GuildId });

    /// Dispatched when a user joins a guild.
    ///
    /// Provides the guild's id and the user's member data.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    async fn guild_member_addition(&self, GuildMemberAddition { ctx: Context, new_member: Member });

    /// Dispatched when a user's membership ends by leaving, getting kicked, or being banned.
    ///
    /// Provides the guild's id, the user's data, and the user's member data if cache feature is
    /// enabled and the data is available.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    async fn guild_member_removal(&self, GuildMemberRemoval { ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member> });

    /// Dispatched when a member is updated (e.g their nickname is updated).
    ///
    /// Provides the member's old and new data (if cache feature is enabled and data is available) and the
    /// new raw data about updated fields.
    ///
    /// Note: This event will not trigger unless the "guild members" privileged intent
    /// is enabled on the bot application page.
    async fn guild_member_update(&self, GuildMemberUpdate { ctx: Context, old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent });

    /// Dispatched when the data for offline members was requested.
    ///
    /// Provides the guild's id and the data.
    async fn guild_members_chunk(&self, GuildMembersChunk { ctx: Context, chunk: GuildMembersChunkEvent });

    /// Dispatched when a role is created.
    ///
    /// Provides the guild's id and the new role's data.
    async fn guild_role_create(&self, GuildRoleCreate { ctx: Context, new: Role });

    /// Dispatched when a role is deleted.
    ///
    /// Provides the guild's id, the role's id and its data (if cache feature is enabled and the
    /// data is available).
    async fn guild_role_delete(&self, GuildRoleDelete { ctx: Context, guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> });

    /// Dispatched when a role is updated.
    ///
    /// Provides the guild's id, the role's old (if cache feature is enabled and the data is
    /// available) and new data.
    async fn guild_role_update(&self, GuildRoleUpdate { ctx: Context, old_data_if_available: Option<Role>, new: Role });

    /// Dispatched when the stickers are updated.
    ///
    /// Provides the guild's id and the new state of the stickers in the guild.
    async fn guild_stickers_update(&self, GuildStickersUpdate { ctx: Context, guild_id: GuildId, current_state: HashMap<StickerId, Sticker> });

    /// Dispatched when the guild is updated.
    ///
    /// Provides the guild's old full data (if cache feature is enabled and the data is available)
    /// and the new, albeit partial data.
    async fn guild_update(&self, GuildUpdate { ctx: Context, old_data_if_available: Option<Guild>, new_but_incomplete: PartialGuild });

    /// Dispatched when a invite is created.
    ///
    /// Provides data about the invite.
    async fn invite_create(&self, InviteCreate { ctx: Context, data: InviteCreateEvent });

    /// Dispatched when a invite is deleted.
    ///
    /// Provides data about the invite.
    async fn invite_delete(&self, InviteDelete { ctx: Context, data: InviteDeleteEvent });

    /// Dispatched when a message is created.
    ///
    /// Provides the message's data.
    async fn message(&self, Message { ctx: Context, new_message: Message });

    /// Dispatched when a message is deleted.
    ///
    /// Provides the guild's id, the channel's id and the message's id.
    async fn message_delete(&self, MessageDelete { ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId> });

    /// Dispatched when multiple messages were deleted at once.
    ///
    /// Provides the guild's id, channel's id and the deleted messages' ids.
    async fn message_delete_bulk(&self, MessageDeleteBulk { ctx: Context, channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId> });

    /// Dispatched when a message is updated.
    ///
    /// Provides the message update data, as well as the actual old and new message if cache feature
    /// is enabled and the data is available.
    async fn message_update(&self, MessageUpdate { ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent });

    /// Dispatched when a new reaction is attached to a message.
    ///
    /// Provides the reaction's data.
    async fn reaction_add(&self, ReactionAdd { ctx: Context, add_reaction: Reaction });

    /// Dispatched when a reaction is detached from a message.
    ///
    /// Provides the reaction's data.
    async fn reaction_remove(&self, ReactionRemove { ctx: Context, removed_reaction: Reaction });

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    async fn reaction_remove_all(&self, ReactionRemoveAll { ctx: Context, channel_id: ChannelId, removed_from_message_id: MessageId });

    /// Dispatched when all reactions of a message are detached from a message.
    ///
    /// Provides the channel's id and the message's id.
    async fn reaction_remove_emoji(&self, ReactionRemoveEmoji { ctx: Context, removed_reactions: Reaction });

    /// This event is legacy, and likely no longer sent by discord.
    async fn presence_replace(&self, PresenceReplace { ctx: Context, presences: Vec<Presence> });

    /// Dispatched when a user's presence is updated (e.g off -> on).
    ///
    /// Provides the presence's new data.
    ///
    /// Note: This event will not trigger unless the "guild presences" privileged intent
    /// is enabled on the bot application page.
    async fn presence_update(&self, PresenceUpdate { ctx: Context, new_data: Presence });

    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    async fn ready(&self, Ready { ctx: Context, data_about_bot: Ready });

    /// Dispatched upon reconnection.
    async fn resume(&self, Resume { ctx: Context, event: ResumedEvent });

    /// Dispatched when a shard's connection stage is updated
    ///
    /// Provides the context of the shard and the event information about the update.
    async fn shard_stage_update(&self, ShardStageUpdate { ctx: Context, event: ShardStageUpdateEvent });

    /// Dispatched when a user starts typing.
    async fn typing_start(&self, TypingStart { ctx: Context, event: TypingStartEvent });

    /// Dispatched when the bot's data is updated.
    ///
    /// Provides the old (if cache feature is enabled and the data is available) and new data.
    async fn user_update(&self, UserUpdate { ctx: Context, old_data: Option<CurrentUser>, new: CurrentUser });

    /// Dispatched when a guild's voice server was updated (or changed to another one).
    ///
    /// Provides the voice server's data.
    async fn voice_server_update(&self, VoiceServerUpdate { ctx: Context, event: VoiceServerUpdateEvent });

    /// Dispatched when a user joins, leaves or moves to a voice channel.
    ///
    /// Provides the guild's id (if available) and the old state (if cache feature is enabled and
    /// [`GatewayIntents::GUILDS`] is enabled) and the new state of the guild's voice channels.
    async fn voice_state_update(&self, VoiceStateUpdate { ctx: Context, old: Option<VoiceState>, new: VoiceState });

    /// Dispatched when a guild's webhook is updated.
    ///
    /// Provides the guild's id and the channel's id the webhook belongs in.
    async fn webhook_update(&self, WebhookUpdate { ctx: Context, guild_id: GuildId, belongs_to_channel_id: ChannelId });

    /// Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    ///
    /// Provides the created interaction.
    async fn interaction_create(&self, InteractionCreate { ctx: Context, interaction: Interaction });

    /// Dispatched when a guild integration is created.
    ///
    /// Provides the created integration.
    async fn integration_create(&self, IntegrationCreate { ctx: Context, integration: Integration });

    /// Dispatched when a guild integration is updated.
    ///
    /// Provides the updated integration.
    async fn integration_update(&self, IntegrationUpdate { ctx: Context, integration: Integration });

    /// Dispatched when a guild integration is deleted.
    ///
    /// Provides the integration's id, the id of the guild it belongs to, and its associated application id
    async fn integration_delete(&self, IntegrationDelete { ctx: Context, integration_id: IntegrationId, guild_id: GuildId, application_id: Option<ApplicationId> });

    /// Dispatched when a stage instance is created.
    ///
    /// Provides the created stage instance.
    async fn stage_instance_create(&self, StageInstanceCreate { ctx: Context, stage_instance: StageInstance });

    /// Dispatched when a stage instance is updated.
    ///
    /// Provides the updated stage instance.
    async fn stage_instance_update(&self, StageInstanceUpdate { ctx: Context, stage_instance: StageInstance });

    /// Dispatched when a stage instance is deleted.
    ///
    /// Provides the deleted stage instance.
    async fn stage_instance_delete(&self, StageInstanceDelete { ctx: Context, stage_instance: StageInstance });

    /// Dispatched when a thread is created or the current user is added
    /// to a private thread.
    ///
    /// Provides the thread.
    async fn thread_create(&self, ThreadCreate { ctx: Context, thread: GuildChannel });

    /// Dispatched when a thread is updated.
    ///
    /// Provides the updated thread.
    async fn thread_update(&self, ThreadUpdate { ctx: Context, thread: GuildChannel });

    /// Dispatched when a thread is deleted.
    ///
    /// Provides the partial deleted thread.
    async fn thread_delete(&self, ThreadDelete { ctx: Context, thread: PartialGuildChannel });

    /// Dispatched when the current user gains access to a channel
    ///
    /// Provides the threads the current user can access, the thread members,
    /// the guild Id, and the channel Ids of the parent channels being synced.
    async fn thread_list_sync(&self, ThreadListSync { ctx: Context, thread_list_sync: ThreadListSyncEvent });

    /// Dispatched when the [`ThreadMember`] for the current user is updated.
    ///
    /// Provides the updated thread member.
    async fn thread_member_update(&self, ThreadMemberUpdate { ctx: Context, thread_member: ThreadMember });

    /// Dispatched when anyone is added to or removed from a thread. If the current user does not have the [`GatewayIntents::GUILDS`],
    /// then this event will only be sent if the current user was added to or removed from the thread.
    ///
    /// Provides the added/removed members, the approximate member count of members in the thread,
    /// the thread Id and its guild Id.
    ///
    /// [`GatewayIntents::GUILDS`]: crate::model::gateway::GatewayIntents::GUILDS
    async fn thread_members_update(&self, ThreadMembersUpdate { ctx: Context, thread_members_update: ThreadMembersUpdateEvent });

    /// Dispatched when a scheduled event is created.
    ///
    /// Provides data about the scheduled event.
    async fn guild_scheduled_event_create(&self, ScheduledEventCreate { ctx: Context, event: ScheduledEvent });

    /// Dispatched when a scheduled event is updated.
    ///
    /// Provides data about the scheduled event.
    async fn guild_scheduled_event_update(&self, ScheduledEventUpdate { ctx: Context, event: ScheduledEvent });

    /// Dispatched when a scheduled event is deleted.
    ///
    /// Provides data about the scheduled event.
    async fn guild_scheduled_event_delete(&self, ScheduledEventDelete { ctx: Context, event: ScheduledEvent });

    /// Dispatched when a guild member has subscribed to a scheduled event.
    ///
    /// Provides data about the subscription.
    async fn guild_scheduled_event_user_add(&self, ScheduledEventUserAdd { ctx: Context, subscribed: ScheduledEventUserAddEvent });

    /// Dispatched when a guild member has unsubscribed from a scheduled event.
    ///
    /// Provides data about the cancelled subscription.
    async fn guild_scheduled_event_user_remove(&self, ScheduledEventUserRemove { ctx: Context, unsubscribed: ScheduledEventUserRemoveEvent });

    /// Dispatched when an HTTP rate limit is hit
    async fn ratelimit(&self, Ratelimit { data: RatelimitInfo });
}

/// This core trait for handling raw events
#[async_trait]
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&self, _ctx: Context, _ev: Event) {}
}
