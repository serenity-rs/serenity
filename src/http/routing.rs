use super::LightMethod;
use crate::model::id::*;

/// A representation of all routes registered within the library. These are safe
/// and memory-efficient representations of each path that functions exist for
/// in the [`http`] module.
///
/// Used as ratelimit buckets.
///
/// [`http`]: crate::http
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RatelimitBucket {
    /// Route for the `/channels/:channel_id` path.
    ChannelsId(ChannelId),
    /// Route for the `/channels/:channel_id/invites` path.
    ChannelsIdInvites(ChannelId),
    /// Route for the `/channels/:channel_id/messages` path.
    ChannelsIdMessages(ChannelId),
    /// Route for the `/channels/:channel_id/messages/bulk-delete` path.
    ChannelsIdMessagesBulkDelete(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id` path.
    ///
    /// This route is a unique case. The ratelimit for message _deletions_ is
    /// different than the overall route ratelimit.
    ///
    /// Refer to the docs on [Rate Limits] in the yellow warning section.
    /// [Rate Limits]: <https://discord.com/developers/docs/topics/rate-limits>
    ChannelsIdMessagesId(LightMethod, ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/ack` path.
    ChannelsIdMessagesIdAck(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/reactions`
    /// path.
    ChannelsIdMessagesIdReactions(ChannelId),
    /// Route for the
    /// `/channels/:channel_id/messages/:message_id/reactions/:reaction/@me`
    /// path.
    ChannelsIdMessagesIdReactionsUserIdType(ChannelId),
    /// Route for the `/channels/:channel_id/permissions/:target_id` path.
    ChannelsIdPermissionsOverwriteId(ChannelId),
    /// Route for the `/channels/:channel_id/pins` path.
    ChannelsIdPins(ChannelId),
    /// Route for the `/channels/:channel_id/pins/:message_id` path.
    ChannelsIdPinsMessageId(ChannelId),
    /// Route for the `/channels/:channel_id/message/:message_id/crosspost` path.
    ChannelsIdCrosspostsMessageId(ChannelId),
    /// Route for the `/channels/:channel_id/typing` path.
    ChannelsIdTyping(ChannelId),
    /// Route for the `/channels/:channel_id/webhooks` path.
    ChannelsIdWebhooks(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/threads` path.
    ChannelsIdMessagesIdThreads(ChannelId),
    /// Route for the `/channels/:channel_id/threads` path.
    ChannelsIdThreads(ChannelId),
    /// Route for the `/channels/:channel_id/thread-members/@me` path.
    ChannelsIdThreadMembersMe(ChannelId),
    /// Route for the `/channels/:channel_id/thread-members/:user_id` path.
    ChannelsIdThreadMembersUserId(ChannelId),
    /// Route for the `/channels/channel_id/thread-members` path.
    ChannelsIdThreadMembers(ChannelId),
    /// Route for the `/channels/:channel_id/threads/archived/public` path.
    ChannelsIdArchivedPublicThreads(ChannelId),
    /// Route for the `/channels/:channel_id/threads/archived/private` path.
    ChannelsIdArchivedPrivateThreads(ChannelId),
    /// Route for the `/channels/:channel_id/users/@me/threads/archived/private` path.
    ChannelsIdMeJoindedArchivedPrivateThreads(ChannelId),
    /// Route for the `/channels/{channel.id}/followers` path.
    FollowNewsChannel(ChannelId),
    /// Route for the `/gateway` path.
    Gateway,
    /// Route for the `/gateway/bot` path.
    GatewayBot,
    /// Route for the `/guilds` path.
    Guilds,
    /// Route for the `/guilds/:guild_id` path.
    GuildsId(GuildId),
    /// Route for the `/guilds/:guild_id/auto-moderation/rules` path.
    GuildsIdAutoModRules(GuildId),
    /// Route for the `/guilds/:guild_id/auto-moderation/rules/:rule_id` path.
    GuildsIdAutoModRulesId(GuildId),
    /// Route for the `/guilds/:guild_id/bans` path.
    GuildsIdBans(GuildId),
    GuildsIdAuditLogs(GuildId),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    GuildsIdBansUserId(GuildId),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    GuildsIdChannels(GuildId),
    /// Route for the `/guilds/:guild_id/widget` path.
    GuildsIdWidget(GuildId),
    /// Route for the `/guilds/:guild_id/preview` path.
    ///
    /// [`GuildPreview`]: crate::model::guild::GuildPreview
    GuildsIdPreview(GuildId),
    /// Route for the `/guilds/:guild_id/emojis` path.
    GuildsIdEmojis(GuildId),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    GuildsIdEmojisId(GuildId),
    /// Route for the `/guilds/:guild_id/integrations` path.
    GuildsIdIntegrations(GuildId),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    GuildsIdIntegrationsId(GuildId),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    GuildsIdIntegrationsIdSync(GuildId),
    /// Route for the `/guilds/:guild_id/invites` path.
    GuildsIdInvites(GuildId),
    /// Route for the `/guilds/:guild_id/members` path.
    GuildsIdMembers(GuildId),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    GuildsIdMembersId(GuildId),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    GuildsIdMembersIdRolesId(GuildId),
    /// Route for the `/guilds/:guild_id/members/@me` path.
    GuildsIdMembersMe(GuildId),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    GuildsIdMembersMeNick(GuildId),
    /// Route for the `/guilds/:guild_id/members/search` path.
    GuildsIdMembersSearch(GuildId),
    /// Route for the `/guilds/:guild_id/prune` path.
    GuildsIdPrune(GuildId),
    /// Route for the `/guilds/:guild_id/regions` path.
    GuildsIdRegions(GuildId),
    /// Route for the `/guilds/:guild_id/roles` path.
    GuildsIdRoles(GuildId),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    GuildsIdRolesId(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events` path.
    GuildsIdScheduledEvents(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id` path.
    GuildsIdScheduledEventsId(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id/users` path.
    GuildsIdScheduledEventsIdUsers(GuildId),
    /// Route for the `/guilds/:guild_id/stickers` path.
    GuildsIdStickers(GuildId),
    /// Route for the `/guilds/:guild_id/stickers/:sticker_id` path.
    GuildsIdStickersId(GuildId),
    /// Route for the `/guilds/:guild_id/vanity-url` path.
    GuildsIdVanityUrl(GuildId),
    /// Route for the `/guilds/:guild_id/voice-states/:user_id` path.
    GuildsIdVoiceStates(GuildId),
    /// Route for the `/guilds/:guild_id/voice-states/@me` path.
    GuildsIdVoiceStatesMe(GuildId),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    GuildsIdWebhooks(GuildId),
    /// Route for the `/guilds/:guild_id/welcome-screen` path.
    GuildsIdWelcomeScreen(GuildId),
    /// Route for the `/guilds/:guild_id/threads/active` path.
    GuildsIdThreadsActive,
    /// Route for the `/invites/:code` path.
    InvitesCode,
    /// Route for the `/sticker-packs` path.
    StickerPacks,
    /// Route for the `/stickers/:sticker_id` path.
    StickersId,
    /// Route for the `/users/:user_id` path.
    UsersId,
    /// Route for the `/users/@me` path.
    UsersMe,
    /// Route for the `/users/@me/channels` path.
    UsersMeChannels,
    /// Route for the `/users/@me/connections` path.
    UsersMeConnections,
    /// Route for the `/users/@me/guilds` path.
    UsersMeGuilds,
    /// Route for the `/users/@me/guilds/:guild_id` path.
    UsersMeGuildsId,
    /// Route for the `/voice/regions` path.
    VoiceRegions,
    /// Route for the `/webhooks/:webhook_id` path.
    WebhooksId(WebhookId),
    /// Route for the `/webhooks/:webhook_id/:token/messages/:message_id` path.
    WebhooksIdMessagesId(WebhookId),
    /// Route for the `/webhooks/:application_id` path.
    WebhooksApplicationId(ApplicationId),
    /// Route for the `/interactions/:interaction_id` path.
    InteractionsId(InteractionId),
    /// Route for the `/applications/:application_id` path.
    ApplicationsIdCommands(ApplicationId),
    /// Route for the `/applications/:application_id/commands/:command_id` path.
    ApplicationsIdCommandsId(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id` path.
    ApplicationsIdGuildsIdCommands(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/permissions` path.
    ApplicationsIdGuildsIdCommandsPermissions(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/:command_id/permissions` path.
    ApplicationsIdGuildsIdCommandIdPermissions(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id` path.
    ApplicationsIdGuildsIdCommandsId(ApplicationId),
    /// Route for the `/stage-instances` path.
    StageInstances,
    /// Route for the `/stage-instances/:channel_id` path.
    StageInstancesChannelId(ChannelId),
    /// Route where no ratelimit headers are in place (i.e. user account-only
    /// routes).
    ///
    /// This is a special case, in that if the route is [`None`] then pre- and
    /// post-hooks are not executed.
    None,
}
