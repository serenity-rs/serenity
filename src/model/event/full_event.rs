use std::collections::VecDeque;
#[cfg(feature = "cache")]
use std::num::NonZeroU16;

use strum::{EnumCount, IntoStaticStr, VariantNames};

#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheUpdate};
use crate::model::prelude::*;

macro_rules! full_event {
    ( $(
        $( #[doc = $doc:literal] )*
        $( #[deprecated = $deprecated:literal] )?
        $( #[cfg(feature = $feature:literal)] )?
        $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* };
    )* ) => {
        /// This enum stores every possible event that an `EventHandler` can receive.
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
    ChannelCreate { channel: MaybeObfuscated };
    /// Dispatched when a category is created.
    ///
    /// Provides said category's data.
    CategoryCreate { category: MaybeObfuscated };
    /// Dispatched when a category is deleted.
    ///
    /// Provides said category's data.
    CategoryDelete { category: MaybeObfuscated };
    /// Dispatched when a channel is deleted.
    ///
    /// Provides said channel's data.
    ChannelDelete { channel: MaybeObfuscated, messages: Option<VecDeque<Message>> };
    /// Dispatched when channel info is requested for a guild.
    ///
    /// Provides said channel info.
    ChannelInfo { old: Option<Vec<ChannelInfoChannel>>, guild_id: GuildId, channels: Vec<ChannelInfoChannel> };
    /// Dispatched when a pin is added, deleted.
    ///
    /// Provides said pin's data.
    ChannelPinsUpdate { pin: ChannelPinsUpdateEvent };
    /// Dispatched when a channel is updated.
    ///
    /// The old channel data is only provided when the cache feature is enabled.
    ChannelUpdate { old: Option<MaybeObfuscated>, new: MaybeObfuscated };
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
    /// This event may also be sent for users who are already members of the guild.
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

    /// Dispatched when an effect, such as an emoji reaction or a soundboard sound, is sent in a
    /// voice channel the current user is connected to.
    ///
    /// Provides data about the voice channel effect.
    VoiceChannelEffectSend { effect: VoiceChannelEffect };
    /// Dispatched when a voice channel's start time is updated.
    ///
    /// Provides the start time, channel's id and the guild's id.
    VoiceChannelStartTimeUpdate { old: Option<i64>, id: ChannelId,  guild_id: GuildId, voice_start_time: Option<i64> };
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
    /// This will be dispatched multiple times if multiple answers are selected
    MessagePollVoteAdd { event: MessagePollVoteAddEvent };
    /// Dispatched when a user removes a previous vote on a poll
    MessagePollVoteRemove { event: MessagePollVoteRemoveEvent };
}

impl FullEvent {
    #[cfg_attr(not(feature = "cache"), expect(unused_variables))]
    pub fn from_event(
        event: Box<Event>,
        extra_event: &mut Option<Self>,
        #[cfg(feature = "cache")] cache: &Cache,
    ) -> Self {
        match *event {
            Event::CommandPermissionsUpdate(event) => Self::CommandPermissionsUpdate {
                permission: event.permission,
            },
            Event::AutoModRuleCreate(event) => Self::AutoModRuleCreate {
                rule: event.rule,
            },
            Event::AutoModRuleUpdate(event) => Self::AutoModRuleUpdate {
                rule: event.rule,
            },
            Event::AutoModRuleDelete(event) => Self::AutoModRuleDelete {
                rule: event.rule,
            },
            Event::AutoModActionExecution(event) => Self::AutoModActionExecution {
                execution: event.execution,
            },
            Event::ChannelCreate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                let channel = event.channel;
                if channel.base.kind == ChannelType::Category {
                    Self::CategoryCreate {
                        category: channel.into(),
                    }
                } else {
                    Self::ChannelCreate {
                        channel: channel.into(),
                    }
                }
            },
            Event::ChannelDelete(event) => {
                #[cfg(feature = "cache")]
                let cached_messages = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let cached_messages = None;

                let channel = event.channel;
                if channel.base.kind == ChannelType::Category {
                    Self::CategoryDelete {
                        category: channel.into(),
                    }
                } else {
                    Self::ChannelDelete {
                        channel: channel.into(),
                        messages: cached_messages,
                    }
                }
            },
            Event::ChannelInfo(event) => {
                #[cfg(feature = "cache")]
                let old = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old = None;

                Self::ChannelInfo {
                    old,
                    guild_id: event.guild_id,
                    channels: event.channels,
                }
            },
            Event::ChannelPinsUpdate(event) => Self::ChannelPinsUpdate {
                pin: event,
            },
            Event::ChannelUpdate(event) => {
                #[cfg(feature = "cache")]
                let old_channel = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_channel = None;

                Self::ChannelUpdate {
                    old: old_channel,
                    new: event.channel.into(),
                }
            },
            Event::GuildAuditLogEntryCreate(event) => Self::GuildAuditLogEntryCreate {
                entry: event.entry,
                guild_id: event.guild_id,
            },
            Event::GuildBanAdd(event) => Self::GuildBanAddition {
                guild_id: event.guild_id,
                banned_user: event.user,
            },
            Event::GuildBanRemove(event) => Self::GuildBanRemoval {
                guild_id: event.guild_id,
                unbanned_user: event.user,
            },
            Event::GuildCreate(event) => {
                #[cfg(feature = "cache")]
                let is_new = Some(is_guild_new(cache, event.guild.id));
                #[cfg(not(feature = "cache"))]
                let is_new = None;

                #[cfg(feature = "cache")]
                if let Some(guilds) = event.update(cache) {
                    *extra_event = Some(Self::CacheReady {
                        guilds,
                    });
                }

                Self::GuildCreate {
                    guild: event.guild,
                    is_new,
                }
            },
            Event::GuildDelete(event) => {
                #[cfg(feature = "cache")]
                let full = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let full = None;

                Self::GuildDelete {
                    incomplete: event.guild,
                    full,
                }
            },
            Event::GuildEmojisUpdate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildEmojisUpdate {
                    guild_id: event.guild_id,
                    current_state: event.emojis,
                }
            },
            Event::GuildIntegrationsUpdate(event) => Self::GuildIntegrationsUpdate {
                guild_id: event.guild_id,
            },
            Event::GuildMemberAdd(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildMemberAddition {
                    new_member: event.member,
                }
            },
            Event::GuildMemberRemove(event) => {
                #[cfg(feature = "cache")]
                let member = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let member = None;

                Self::GuildMemberRemoval {
                    guild_id: event.guild_id,
                    user: event.user,
                    member_data_if_available: member,
                }
            },
            Event::GuildMemberUpdate(event) => {
                #[cfg(feature = "cache")]
                let (before, after) = (
                    event.update(cache),
                    cache
                        .guild(event.guild_id)
                        .and_then(|g| g.members.get(&event.user.id).cloned()),
                );
                #[cfg(not(feature = "cache"))]
                let (before, after) = (None, None);

                Self::GuildMemberUpdate {
                    old_if_available: before,
                    new: after,
                    event,
                }
            },
            Event::GuildMembersChunk(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildMembersChunk {
                    chunk: event,
                }
            },
            Event::GuildRoleCreate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildRoleCreate {
                    new: event.role,
                }
            },
            Event::GuildRoleDelete(event) => {
                #[cfg(feature = "cache")]
                let role = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let role = None;

                Self::GuildRoleDelete {
                    guild_id: event.guild_id,
                    removed_role_id: event.role_id,
                    removed_role_data_if_available: role,
                }
            },
            Event::GuildRoleUpdate(event) => {
                #[cfg(feature = "cache")]
                let before = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let before = None;

                Self::GuildRoleUpdate {
                    old_data_if_available: before,
                    new: event.role,
                }
            },
            Event::GuildStickersUpdate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildStickersUpdate {
                    guild_id: event.guild_id,
                    current_state: event.stickers,
                }
            },
            Event::GuildUpdate(event) => {
                #[cfg(feature = "cache")]
                let before = cache.guild(event.guild.id).map(|g| g.clone());
                #[cfg(not(feature = "cache"))]
                let before = None;

                Self::GuildUpdate {
                    old_data_if_available: before,
                    new_data: event.guild,
                }
            },
            Event::InviteCreate(event) => Self::InviteCreate {
                data: event,
            },
            Event::InviteDelete(event) => Self::InviteDelete {
                data: event,
            },
            Event::MessageCreate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::Message {
                    new_message: event.message,
                }
            },
            Event::MessageDeleteBulk(event) => Self::MessageDeleteBulk {
                channel_id: event.channel_id,
                multiple_deleted_messages_ids: event.ids.into_vec(),
                guild_id: event.guild_id,
            },
            Event::MessageDelete(event) => Self::MessageDelete {
                channel_id: event.channel_id,
                deleted_message_id: event.message_id,
                guild_id: event.guild_id,
            },
            Event::MessageUpdate(event) => {
                #[cfg(feature = "cache")]
                let before = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let before = None;

                Self::MessageUpdate {
                    old_if_available: before,
                    event,
                }
            },
            Event::PresenceUpdate(event) => {
                #[cfg(feature = "cache")]
                let old_data = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_data = None;

                Self::PresenceUpdate {
                    old_data,
                    new_data: event.presence,
                }
            },
            Event::ReactionAdd(event) => {
                #[cfg(feature = "cache")]
                let old_message_if_available = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_message_if_available = None;

                Self::ReactionAdd {
                    add_reaction: event.reaction,
                    old_message_if_available,
                }
            },
            Event::ReactionRemove(event) => {
                #[cfg(feature = "cache")]
                let old_message_if_available = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_message_if_available = None;

                Self::ReactionRemove {
                    removed_reaction: event.reaction,
                    old_message_if_available,
                }
            },
            Event::ReactionRemoveAll(event) => {
                #[cfg(feature = "cache")]
                let old_message_if_available = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_message_if_available = None;

                Self::ReactionRemoveAll {
                    guild_id: event.guild_id,
                    channel_id: event.channel_id,
                    removed_from_message_id: event.message_id,
                    old_message_if_available,
                }
            },
            Event::ReactionRemoveEmoji(event) => {
                #[cfg(feature = "cache")]
                let old_message_if_available = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old_message_if_available = None;

                Self::ReactionRemoveEmoji {
                    removed_reactions: event.reaction,
                    old_message_if_available,
                }
            },
            Event::Ready(event) => {
                #[cfg(feature = "cache")]
                if let Some(total_shards) = event.update(cache) {
                    *extra_event = Some(Self::ShardsReady {
                        total_shards,
                    });
                }

                Self::Ready {
                    data_about_bot: event.ready,
                }
            },
            Event::Resumed(event) => Self::Resume {
                event,
            },
            Event::SoundboardSounds(event) => Self::SoundboardSounds {
                event,
            },
            Event::SoundboardSoundCreate(event) => Self::SoundboardSoundCreate {
                event,
            },
            Event::SoundboardSoundUpdate(event) => Self::SoundboardSoundUpdate {
                event,
            },
            Event::SoundboardSoundsUpdate(event) => Self::SoundboardSoundsUpdate {
                event,
            },
            Event::SoundboardSoundDelete(event) => Self::SoundboardSoundDelete {
                event,
            },
            Event::TypingStart(event) => Self::TypingStart {
                event,
            },
            Event::UserUpdate(event) => {
                #[cfg(feature = "cache")]
                let before = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let before = None;

                Self::UserUpdate {
                    old_data: before,
                    new: event.current_user,
                }
            },
            Event::VoiceServerUpdate(event) => Self::VoiceServerUpdate {
                event,
            },
            Event::VoiceStateUpdate(event) => {
                #[cfg(feature = "cache")]
                let before = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let before = None;

                Self::VoiceStateUpdate {
                    old: before,
                    new: event.voice_state,
                }
            },
            Event::VoiceChannelEffectSend(event) => Self::VoiceChannelEffectSend {
                effect: event.effect,
            },
            Event::VoiceChannelStartTimeUpdate(event) => {
                #[cfg(feature = "cache")]
                let old = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old = None;

                Self::VoiceChannelStartTimeUpdate {
                    old,
                    id: event.id,
                    guild_id: event.guild_id,
                    voice_start_time: event.voice_start_time,
                }
            },
            Event::VoiceChannelStatusUpdate(event) => {
                #[cfg(feature = "cache")]
                let old = event.update(cache).map(Into::into);
                #[cfg(not(feature = "cache"))]
                let old = None;

                let status: Option<String> = event.status.map(Into::into);
                let status =
                    if status.as_ref().is_some_and(String::is_empty) { None } else { status };

                Self::VoiceChannelStatusUpdate {
                    old,
                    status,
                    id: event.id,
                    guild_id: event.guild_id,
                }
            },

            Event::WebhookUpdate(event) => Self::WebhookUpdate {
                guild_id: event.guild_id,
                belongs_to_channel_id: event.channel_id,
            },
            Event::InteractionCreate(event) => Self::InteractionCreate {
                interaction: event.interaction,
            },
            Event::IntegrationCreate(event) => Self::IntegrationCreate {
                integration: event.integration,
            },
            Event::IntegrationUpdate(event) => Self::IntegrationUpdate {
                integration: event.integration,
            },
            Event::IntegrationDelete(event) => Self::IntegrationDelete {
                integration_id: event.id,
                guild_id: event.guild_id,
                application_id: event.application_id,
            },
            Event::StageInstanceCreate(event) => Self::StageInstanceCreate {
                stage_instance: event.stage_instance,
            },
            Event::StageInstanceUpdate(event) => Self::StageInstanceUpdate {
                stage_instance: event.stage_instance,
            },
            Event::StageInstanceDelete(event) => Self::StageInstanceDelete {
                stage_instance: event.stage_instance,
            },
            Event::ThreadCreate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::ThreadCreate {
                    thread: event.thread,
                    newly_created: event.newly_created,
                }
            },
            Event::ThreadUpdate(event) => {
                #[cfg(feature = "cache")]
                let old = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let old = None;

                Self::ThreadUpdate {
                    old,
                    new: event.thread,
                }
            },
            Event::ThreadDelete(event) => {
                #[cfg(feature = "cache")]
                let full_thread_data = event.update(cache);
                #[cfg(not(feature = "cache"))]
                let full_thread_data = None;

                Self::ThreadDelete {
                    thread: event.thread,
                    full_thread_data,
                }
            },
            Event::ThreadListSync(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::ThreadListSync {
                    thread_list_sync: event,
                }
            },
            Event::ThreadMemberUpdate(event) => Self::ThreadMemberUpdate {
                thread_member: event.member,
            },
            Event::ThreadMembersUpdate(event) => Self::ThreadMembersUpdate {
                thread_members_update: event,
            },
            Event::GuildScheduledEventCreate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildScheduledEventCreate {
                    event: event.event,
                }
            },
            Event::GuildScheduledEventUpdate(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildScheduledEventUpdate {
                    event: event.event,
                }
            },
            Event::GuildScheduledEventDelete(event) => {
                #[cfg(feature = "cache")]
                event.update(cache);

                Self::GuildScheduledEventDelete {
                    event: event.event,
                }
            },
            Event::GuildScheduledEventUserAdd(event) => Self::GuildScheduledEventUserAdd {
                subscribed: event,
            },
            Event::GuildScheduledEventUserRemove(event) => Self::GuildScheduledEventUserRemove {
                unsubscribed: event,
            },
            Event::EntitlementCreate(event) => Self::EntitlementCreate {
                entitlement: event.entitlement,
            },
            Event::EntitlementUpdate(event) => Self::EntitlementUpdate {
                entitlement: event.entitlement,
            },
            Event::EntitlementDelete(event) => Self::EntitlementDelete {
                entitlement: event.entitlement,
            },
            Event::MessagePollVoteAdd(event) => Self::MessagePollVoteAdd {
                event,
            },
            Event::MessagePollVoteRemove(event) => Self::MessagePollVoteRemove {
                event,
            },
            Event::ShardStageUpdate(event) => Self::ShardStageUpdate {
                event,
            },
        }
    }
}

#[cfg(feature = "cache")]
fn is_guild_new(cache: &Cache, guild_id: GuildId) -> bool {
    !cache.unavailable_guilds().contains(&guild_id)
}

#[cfg(all(test, feature = "cache"))]
mod tests {
    use extract_map::ExtractMap;
    use small_fixed_array::{FixedArray, FixedString};

    use super::is_guild_new;
    use crate::cache::Cache;
    use crate::model::Timestamp;
    use crate::model::application::{ApplicationFlags, PartialCurrentApplicationInfo};
    use crate::model::event::{Event, FullEvent, GuildCreateEvent, ReadyEvent};
    use crate::model::gateway::Ready;
    use crate::model::guild::{
        DefaultMessageNotificationLevel,
        ExplicitContentFilter,
        Guild,
        GuildGeneratedFlags,
        MemberCount,
        MfaLevel,
        NsfwLevel,
        PremiumTier,
        SystemChannelFlags,
        UnavailableGuild,
        VerificationLevel,
    };
    use crate::model::id::{ApplicationId, GuildId, UserId};
    use crate::model::user::CurrentUser;

    #[test]
    fn guild_is_new() {
        let cache = Cache::new();

        let guild_id = GuildId::new(1);
        let event = Box::new(Event::Ready(ReadyEvent {
            ready: Ready {
                version: 0,
                user: CurrentUser::default(),
                guilds: FixedArray::from_vec_trunc(vec![UnavailableGuild {
                    id: guild_id,
                    unavailable: true,
                }]),
                session_id: FixedString::new(),
                resume_gateway_url: FixedString::new(),
                shard: None,
                application: PartialCurrentApplicationInfo {
                    id: ApplicationId::new(1),
                    flags: ApplicationFlags::default(),
                },
            },
        }));

        assert_eq!(cache.unavailable_guilds().len(), 0);

        assert!(is_guild_new(&cache, guild_id));

        let mut extra_event = None;
        let _ = FullEvent::from_event(event, &mut extra_event, &cache);

        assert_eq!(cache.unavailable_guilds().len(), 1);
        assert!(!is_guild_new(&cache, guild_id));

        let event = Box::new(Event::GuildCreate(GuildCreateEvent {
            guild: Guild {
                __generated_flags: GuildGeneratedFlags::default(),
                id: guild_id,
                name: FixedString::new(),
                icon: None,
                icon_hash: None,
                splash: None,
                discovery_splash: None,
                owner_id: UserId::new(1),
                afk_metadata: None,
                widget_channel_id: None,
                verification_level: VerificationLevel::None,
                default_message_notifications: DefaultMessageNotificationLevel::Mentions,
                explicit_content_filter: ExplicitContentFilter::None,
                roles: ExtractMap::new(),
                emojis: ExtractMap::new(),
                features: FixedArray::new(),
                mfa_level: MfaLevel::None,
                application_id: None,
                system_channel_id: None,
                system_channel_flags: SystemChannelFlags::default(),
                rules_channel_id: None,
                max_presences: None,
                max_members: None,
                vanity_url_code: None,
                description: None,
                banner: None,
                premium_tier: PremiumTier::Tier0,
                premium_subscription_count: None,
                preferred_locale: FixedString::new(),
                public_updates_channel_id: None,
                max_video_channel_users: None,
                max_stage_video_channel_users: None,
                approximate_member_count: None,
                approximate_presence_count: None,
                welcome_screen: None,
                nsfw_level: NsfwLevel::Default,
                stickers: ExtractMap::new(),
                joined_at: Timestamp::now(),
                viewable_channels: ExtractMap::new(),
                obfuscated_channels: ExtractMap::new(),
                member_count: MemberCount::ZERO,
                members: ExtractMap::new(),
                incidents_data: None,
                presences: ExtractMap::new(),
                safety_alerts_channel_id: None,
                scheduled_events: ExtractMap::new(),
                stage_instances: FixedArray::new(),
                threads: ExtractMap::new(),
                voice_states: ExtractMap::new(),
            },
        }));

        assert_eq!(cache.unavailable_guilds().len(), 1);

        assert!(!is_guild_new(&cache, guild_id));

        let full_event = FullEvent::from_event(event, &mut extra_event, &cache);

        assert!(matches!(full_event, FullEvent::GuildCreate {
            is_new: Some(false),
            ..
        }));

        assert_eq!(cache.unavailable_guilds().len(), 0);

        assert!(is_guild_new(&cache, guild_id));

        let guild_id2 = GuildId::new(2);

        let event = Box::new(Event::GuildCreate(GuildCreateEvent {
            guild: Guild {
                __generated_flags: GuildGeneratedFlags::default(),
                id: guild_id2,
                name: FixedString::new(),
                icon: None,
                icon_hash: None,
                splash: None,
                discovery_splash: None,
                owner_id: UserId::new(1),
                afk_metadata: None,
                widget_channel_id: None,
                verification_level: VerificationLevel::None,
                default_message_notifications: DefaultMessageNotificationLevel::Mentions,
                explicit_content_filter: ExplicitContentFilter::None,
                roles: ExtractMap::new(),
                emojis: ExtractMap::new(),
                features: FixedArray::new(),
                mfa_level: MfaLevel::None,
                application_id: None,
                system_channel_id: None,
                system_channel_flags: SystemChannelFlags::default(),
                rules_channel_id: None,
                max_presences: None,
                max_members: None,
                vanity_url_code: None,
                description: None,
                banner: None,
                premium_tier: PremiumTier::Tier0,
                premium_subscription_count: None,
                preferred_locale: FixedString::new(),
                public_updates_channel_id: None,
                max_video_channel_users: None,
                max_stage_video_channel_users: None,
                approximate_member_count: None,
                approximate_presence_count: None,
                welcome_screen: None,
                nsfw_level: NsfwLevel::Default,
                stickers: ExtractMap::new(),
                joined_at: Timestamp::now(),
                viewable_channels: ExtractMap::new(),
                obfuscated_channels: ExtractMap::new(),
                member_count: MemberCount::ZERO,
                members: ExtractMap::new(),
                incidents_data: None,
                presences: ExtractMap::new(),
                safety_alerts_channel_id: None,
                scheduled_events: ExtractMap::new(),
                stage_instances: FixedArray::new(),
                threads: ExtractMap::new(),
                voice_states: ExtractMap::new(),
            },
        }));

        assert_eq!(cache.unavailable_guilds().len(), 0);

        let full_event = FullEvent::from_event(event, &mut extra_event, &cache);

        assert!(matches!(full_event, FullEvent::GuildCreate {
            is_new: Some(true),
            ..
        }));

        assert_eq!(cache.unavailable_guilds().len(), 0);

        assert!(is_guild_new(&cache, guild_id2));
    }
}
