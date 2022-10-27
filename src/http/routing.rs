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
    Channel(ChannelId),
    /// Route for the `/channels/:channel_id/invites` path.
    ChannelInvite(ChannelId),
    /// Route for the `/channels/:channel_id/messages` path.
    ChannelMessages(ChannelId),
    /// Route for the `/channels/:channel_id/messages/bulk-delete` path.
    ChannelMessagesBulkDelete(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id` path.
    ///
    /// This route is a unique case. The ratelimit for message _deletions_ is
    /// different than the overall route ratelimit.
    ///
    /// Refer to the docs on [Rate Limits] in the yellow warning section.
    /// [Rate Limits]: <https://discord.com/developers/docs/topics/rate-limits>
    ChannelMessage(LightMethod, ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/ack` path.
    ChannelMessageAck(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/reactions`
    /// path.
    ChannelMessageReactions(ChannelId),
    /// Route for the
    /// `/channels/:channel_id/messages/:message_id/reactions/:reaction/@me`
    /// path.
    ChannelMessageReactionsMe(ChannelId),
    /// Route for the `/channels/:channel_id/permissions/:target_id` path.
    ChannelPermission(ChannelId),
    /// Route for the `/channels/:channel_id/pins` path.
    ChannelPins(ChannelId),
    /// Route for the `/channels/:channel_id/pins/:message_id` path.
    ChannelPin(ChannelId),
    /// Route for the `/channels/:channel_id/message/:message_id/crosspost` path.
    ChannelMessageCrosspost(ChannelId),
    /// Route for the `/channels/:channel_id/typing` path.
    ChannelTyping(ChannelId),
    /// Route for the `/channels/:channel_id/webhooks` path.
    ChannelWebhooks(ChannelId),
    /// Route for the `/channels/:channel_id/messages/:message_id/threads` path.
    ChannelMessageThreads(ChannelId),
    /// Route for the `/channels/:channel_id/threads` path.
    ChannelsThreads(ChannelId),
    /// Route for the `/channels/:channel_id/thread-members/@me` path.
    ChannelThreadMembersMe(ChannelId),
    /// Route for the `/channels/:channel_id/thread-members/:user_id` path.
    ChannelThreadMember(ChannelId),
    /// Route for the `/channels/:channel_id/thread-members` path.
    ChannelThreadMembers(ChannelId),
    /// Route for the `/channels/:channel_id/threads/archived/public` path.
    ChannelThreadsArchivedPublic(ChannelId),
    /// Route for the `/channels/:channel_id/threads/archived/private` path.
    ChannelThreadsArchivedPrivate(ChannelId),
    /// Route for the `/channels/:channel_id/users/@me/threads/archived/private` path.
    ChannelUsersMeThreadsArchivedPrivate(ChannelId),
    /// Route for the `/channels/:channel_id/followers` path.
    ChannelFollowers(ChannelId),
    /// Route for the `/gateway` path.
    Gateway,
    /// Route for the `/gateway/bot` path.
    GatewayBot,
    /// Route for the `/guilds` path.
    Guilds,
    /// Route for the `/guilds/:guild_id` path.
    Guild(GuildId),
    /// Route for the `/guilds/:guild_id/auto-moderation/rules` path.
    GuildAutoModRules(GuildId),
    /// Route for the `/guilds/:guild_id/auto-moderation/rules/:rule_id` path.
    GuildAutoModRule(GuildId),
    /// Route for the `/guilds/:guild_id/bans` path.
    GuildBans(GuildId),
    /// Route for the `/guilds/:guild_id/audit-logs` path
    GuildAuditLogs(GuildId),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    GuildBan(GuildId),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    GuildChannel(GuildId),
    /// Route for the `/guilds/:guild_id/widget` path.
    GuildWidget(GuildId),
    /// Route for the `/guilds/:guild_id/preview` path.
    ///
    /// [`GuildPreview`]: crate::model::guild::GuildPreview
    GuildPreview(GuildId),
    /// Route for the `/guilds/:guild_id/emojis` path.
    GuildEmojis(GuildId),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    GuildEmoji(GuildId),
    /// Route for the `/guilds/:guild_id/integrations` path.
    GuildIntegrations(GuildId),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    GuildIntegration(GuildId),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    GuildIntegrationSync(GuildId),
    /// Route for the `/guilds/:guild_id/invites` path.
    GuildInvites(GuildId),
    /// Route for the `/guilds/:guild_id/members` path.
    GuildMembers(GuildId),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    GuildMember(GuildId),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    GuildMemberRole(GuildId),
    /// Route for the `/guilds/:guild_id/members/@me` path.
    GuildMembersMe(GuildId),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    GuildMembersMeNick(GuildId),
    /// Route for the `/guilds/:guild_id/members/search` path.
    GuildMembersSearch(GuildId),
    /// Route for the `/guilds/:guild_id/prune` path.
    GuildPrune(GuildId),
    /// Route for the `/guilds/:guild_id/regions` path.
    GuildRegions(GuildId),
    /// Route for the `/guilds/:guild_id/roles` path.
    GuildRoles(GuildId),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    GuildRole(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events` path.
    GuildScheduledEvents(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id` path.
    GuildScheduledEvent(GuildId),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id/users` path.
    GuildScheduledEventUsers(GuildId),
    /// Route for the `/guilds/:guild_id/stickers` path.
    GuildStickers(GuildId),
    /// Route for the `/guilds/:guild_id/stickers/:sticker_id` path.
    GuildSticker(GuildId),
    /// Route for the `/guilds/:guild_id/vanity-url` path.
    GuildVanityUrl(GuildId),
    /// Route for the `/guilds/:guild_id/voice-states/:user_id` path.
    GuildVoiceState(GuildId),
    /// Route for the `/guilds/:guild_id/voice-states/@me` path.
    GuildVoiceStatesMe(GuildId),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    GuildWebhooks(GuildId),
    /// Route for the `/guilds/:guild_id/welcome-screen` path.
    GuildWelcomeScreen(GuildId),
    /// Route for the `/guilds/:guild_id/threads/active` path.
    GuildThreadsActive,
    /// Route for the `/invites/:code` path.
    Invite,
    /// Route for the `/sticker-packs` path.
    StickerPacks,
    /// Route for the `/stickers/:sticker_id` path.
    Sticker,
    /// Route for the `/users/:user_id` path.
    User,
    /// Route for the `/users/@me` path.
    UsersMe,
    /// Route for the `/users/@me/channels` path.
    UsersMeChannels,
    /// Route for the `/users/@me/connections` path.
    UsersMeConnections,
    /// Route for the `/users/@me/guilds` path.
    UsersMeGuilds,
    /// Route for the `/users/@me/guilds/:guild_id` path.
    UsersMeGuild,
    /// Route for the `/voice/regions` path.
    VoiceRegions,
    /// Route for the `/webhooks/:webhook_id` path.
    Webhook(WebhookId),
    /// Route for the `/webhooks/:webhook_id/:token/messages/:message_id` path.
    WebhookMessage(WebhookId),
    /// Route for the `/webhooks/:application_id` path.
    WebhookOfTypeApplication(ApplicationId),
    /// Route for the `/interactions/:interaction_id` path.
    Interaction(InteractionId),
    /// Route for the `/applications/:application_id/commands` path.
    ApplicationCommands(ApplicationId),
    /// Route for the `/applications/:application_id/commands/:command_id` path.
    ApplicationCommand(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands` path.
    ApplicationGuildCommands(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/permissions` path.
    ApplicationGuildCommandsPermissions(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/:command_id/permissions` path.
    ApplicationGuildCommandPermissions(ApplicationId),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/:command_id` path.
    ApplicationGuildCommand(ApplicationId),
    /// Route for the `/stage-instances` path.
    StageInstances,
    /// Route for the `/stage-instances/:channel_id` path.
    StageInstance(ChannelId),
    /// Route where no ratelimit headers are in place (i.e. user account-only
    /// routes).
    ///
    /// This is a special case, in that if the route is [`None`] then pre- and
    /// post-hooks are not executed.
    None,
}
