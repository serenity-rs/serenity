use std::borrow::Cow;
use std::fmt::{Display, Write};

use super::LightMethod;
use crate::constants;
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
pub enum Route {
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

impl Route {
    #[must_use]
    pub fn channel(channel_id: ChannelId) -> String {
        api!("/channels/{}", channel_id)
    }

    #[must_use]
    pub fn channel_invites(channel_id: ChannelId) -> String {
        api!("/channels/{}/invites", channel_id)
    }

    #[must_use]
    pub fn channel_message(channel_id: ChannelId, message_id: MessageId) -> String {
        api!("/channels/{}/messages/{}", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_crosspost(channel_id: ChannelId, message_id: MessageId) -> String {
        api!("/channels/{}/messages/{}/crosspost", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_reaction<D, T>(
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: D,
        reaction_type: T,
    ) -> String
    where
        D: Display,
        T: Display,
    {
        api!(
            "/channels/{}/messages/{}/reactions/{}/{}",
            channel_id,
            message_id,
            reaction_type,
            user_id,
        )
    }

    #[must_use]
    pub fn channel_message_reaction_emoji<T>(
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: T,
    ) -> String
    where
        T: Display,
    {
        api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction_type)
    }

    #[must_use]
    pub fn channel_message_reactions(channel_id: ChannelId, message_id: MessageId) -> String {
        api!("/channels/{}/messages/{}/reactions", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_reactions_list(
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: &str,
        limit: u8,
        after: Option<u64>,
    ) -> String {
        let mut url = api!(
            "/channels/{}/messages/{}/reactions/{}?limit={}",
            channel_id,
            message_id,
            reaction,
            limit,
        );

        if let Some(after) = after {
            write!(url, "&after={after}").unwrap();
        }

        url
    }

    #[must_use]
    pub fn channel_messages(channel_id: ChannelId, query: Option<&str>) -> String {
        api!("/channels/{}/messages{}", channel_id, query.unwrap_or(""))
    }

    #[must_use]
    pub fn channel_messages_bulk_delete(channel_id: ChannelId) -> String {
        api!("/channels/{}/messages/bulk-delete", channel_id)
    }

    #[must_use]
    pub fn channel_follow_news(channel_id: ChannelId) -> String {
        api!("/channels/{}/followers", channel_id)
    }

    #[must_use]
    pub fn channel_permission(channel_id: ChannelId, target_id: TargetId) -> String {
        api!("/channels/{}/permissions/{}", channel_id, target_id)
    }

    #[must_use]
    pub fn channel_pin(channel_id: ChannelId, message_id: MessageId) -> String {
        api!("/channels/{}/pins/{}", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_pins(channel_id: ChannelId) -> String {
        api!("/channels/{}/pins", channel_id)
    }

    #[must_use]
    pub fn channel_typing(channel_id: ChannelId) -> String {
        api!("/channels/{}/typing", channel_id)
    }

    #[must_use]
    pub fn channel_webhooks(channel_id: ChannelId) -> String {
        api!("/channels/{}/webhooks", channel_id)
    }

    #[must_use]
    pub fn channel_public_threads(channel_id: ChannelId, message_id: MessageId) -> String {
        api!("/channels/{}/messages/{}/threads", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_private_threads(channel_id: ChannelId) -> String {
        api!("/channels/{}/threads", channel_id)
    }

    #[must_use]
    pub fn channel_thread_member(channel_id: ChannelId, user_id: UserId) -> String {
        api!("/channels/{}/thread-members/{}", channel_id, user_id)
    }

    #[must_use]
    pub fn channel_thread_member_me(channel_id: ChannelId) -> String {
        api!("/channels/{}/thread-members/@me", channel_id)
    }

    #[must_use]
    pub fn channel_thread_members(channel_id: ChannelId) -> String {
        api!("/channels/{}/thread-members", channel_id)
    }

    #[must_use]
    pub fn channel_archived_public_threads(
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/threads/archived/public", channel_id);

        if let Some(id) = before {
            write!(s, "&before={id}").unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn channel_archived_private_threads(
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/threads/archived/private", channel_id);

        if let Some(id) = before {
            write!(s, "&before={id}").unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn channel_joined_private_threads(
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/users/@me/threads/archived/private", channel_id);

        if let Some(id) = before {
            write!(s, "&before={id}").unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn gateway() -> &'static str {
        api!("/gateway")
    }

    #[must_use]
    pub fn gateway_bot() -> &'static str {
        api!("/gateway/bot")
    }

    #[must_use]
    pub fn guild(guild_id: GuildId) -> String {
        api!("/guilds/{}", guild_id)
    }

    #[must_use]
    pub fn guild_with_counts(guild_id: GuildId) -> String {
        api!("/guilds/{}?with_counts=true", guild_id)
    }

    #[must_use]
    pub fn guild_audit_logs(
        guild_id: GuildId,
        action_type: Option<u8>,
        user_id: Option<UserId>,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> String {
        let mut s = api!("/guilds/{}/audit-logs?", guild_id);

        if let Some(action_type) = action_type {
            write!(s, "&action_type={action_type}").unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={before}").unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        if let Some(user_id) = user_id {
            write!(s, "&user_id={user_id}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn guild_automod_rule(guild_id: GuildId, rule_id: RuleId) -> String {
        api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id)
    }

    #[must_use]
    pub fn guild_automod_rules(guild_id: GuildId) -> String {
        api!("/guilds/{}/auto-moderation/rules", guild_id)
    }

    #[must_use]
    pub fn guild_ban(guild_id: GuildId, user_id: UserId) -> String {
        api!("/guilds/{}/bans/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_ban_optioned(
        guild_id: GuildId,
        user_id: UserId,
        delete_message_days: u8,
    ) -> String {
        api!("/guilds/{}/bans/{}?delete_message_days={}", guild_id, user_id, delete_message_days)
    }

    #[must_use]
    pub fn guild_kick_optioned(guild_id: GuildId, user_id: UserId) -> String {
        api!("/guilds/{}/members/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_bans(guild_id: GuildId) -> String {
        api!("/guilds/{}/bans", guild_id)
    }

    #[must_use]
    pub fn guild_channels(guild_id: GuildId) -> String {
        api!("/guilds/{}/channels", guild_id)
    }

    #[must_use]
    pub fn guild_widget(guild_id: GuildId) -> String {
        api!("/guilds/{}/widget", guild_id)
    }

    #[must_use]
    pub fn guild_preview(guild_id: GuildId) -> String {
        api!("/guilds/{}/preview", guild_id)
    }

    #[must_use]
    pub fn guild_emojis(guild_id: GuildId) -> String {
        api!("/guilds/{}/emojis", guild_id)
    }

    #[must_use]
    pub fn guild_emoji(guild_id: GuildId, emoji_id: EmojiId) -> String {
        api!("/guilds/{}/emojis/{}", guild_id, emoji_id)
    }

    #[must_use]
    pub fn guild_integration(guild_id: GuildId, integration_id: IntegrationId) -> String {
        api!("/guilds/{}/integrations/{}", guild_id, integration_id)
    }

    #[must_use]
    pub fn guild_integration_sync(guild_id: GuildId, integration_id: IntegrationId) -> String {
        api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id)
    }

    #[must_use]
    pub fn guild_integrations(guild_id: GuildId) -> String {
        api!("/guilds/{}/integrations", guild_id)
    }

    #[must_use]
    pub fn guild_invites(guild_id: GuildId) -> String {
        api!("/guilds/{}/invites", guild_id)
    }

    #[must_use]
    pub fn guild_member(guild_id: GuildId, user_id: UserId) -> String {
        api!("/guilds/{}/members/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_member_role(guild_id: GuildId, user_id: UserId, role_id: RoleId) -> String {
        api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id)
    }

    #[must_use]
    pub fn guild_members(guild_id: GuildId) -> String {
        api!("/guilds/{}/members", guild_id)
    }

    #[must_use]
    pub fn guild_members_search(guild_id: GuildId, query: &str, limit: Option<u64>) -> String {
        let mut s = api!("/guilds/{}/members/search?", guild_id);

        write!(s, "&query={query}&limit={}", limit.unwrap_or(constants::MEMBER_FETCH_LIMIT))
            .unwrap();
        s
    }

    #[must_use]
    pub fn guild_members_optioned(
        guild_id: GuildId,
        after: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/guilds/{}/members?", guild_id);

        if let Some(after) = after {
            write!(s, "&after={after}").unwrap();
        }

        write!(s, "&limit={}", limit.unwrap_or(constants::MEMBER_FETCH_LIMIT)).unwrap();
        s
    }

    #[must_use]
    pub fn guild_member_me(guild_id: GuildId) -> String {
        api!("/guilds/{}/members/@me", guild_id)
    }

    #[must_use]
    pub fn guild_nickname(guild_id: GuildId) -> String {
        api!("/guilds/{}/members/@me/nick", guild_id)
    }

    #[must_use]
    pub fn guild_prune(guild_id: GuildId, days: u8) -> String {
        api!("/guilds/{}/prune?days={}", guild_id, days)
    }

    #[must_use]
    pub fn guild_regions(guild_id: GuildId) -> String {
        api!("/guilds/{}/regions", guild_id)
    }

    #[must_use]
    pub fn guild_role(guild_id: GuildId, role_id: RoleId) -> String {
        api!("/guilds/{}/roles/{}", guild_id, role_id)
    }

    #[must_use]
    pub fn guild_roles(guild_id: GuildId) -> String {
        api!("/guilds/{}/roles", guild_id)
    }

    #[must_use]
    pub fn guild_scheduled_event(
        guild_id: GuildId,
        event_id: ScheduledEventId,
        with_user_count: Option<bool>,
    ) -> String {
        let mut s = api!("/guilds/{}/scheduled-events/{}", guild_id, event_id);
        if let Some(b) = with_user_count {
            write!(s, "?with_user_count={b}").unwrap();
        }
        s
    }

    #[must_use]
    pub fn guild_scheduled_events(guild_id: GuildId, with_user_count: Option<bool>) -> String {
        let mut s = api!("/guilds/{}/scheduled-events", guild_id);
        if let Some(b) = with_user_count {
            write!(s, "?with_user_count={b}").unwrap();
        }
        s
    }

    #[must_use]
    pub fn guild_scheduled_event_users(
        guild_id: GuildId,
        event_id: ScheduledEventId,
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
        with_member: Option<bool>,
    ) -> String {
        let mut s = api!("/guilds/{}/scheduled-events/{}/users?", guild_id, event_id);

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        if let Some(after) = after {
            write!(s, "&after={after}").unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={before}").unwrap();
        }

        if let Some(with_member) = with_member {
            write!(s, "&with_member={with_member}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn guild_sticker(guild_id: GuildId, sticker_id: StickerId) -> String {
        api!("/guilds/{}/stickers/{}", guild_id, sticker_id)
    }

    #[must_use]
    pub fn guild_stickers(guild_id: GuildId) -> String {
        api!("/guilds/{}/stickers", guild_id)
    }

    #[must_use]
    pub fn guild_vanity_url(guild_id: GuildId) -> String {
        api!("/guilds/{}/vanity-url", guild_id)
    }

    #[must_use]
    pub fn guild_voice_states(guild_id: GuildId, user_id: UserId) -> String {
        api!("/guilds/{}/voice-states/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_voice_states_me(guild_id: GuildId) -> String {
        api!("/guilds/{}/voice-states/@me", guild_id)
    }

    #[must_use]
    pub fn guild_webhooks(guild_id: GuildId) -> String {
        api!("/guilds/{}/webhooks", guild_id)
    }

    #[must_use]
    pub fn guild_welcome_screen(guild_id: GuildId) -> String {
        api!("/guilds/{}/welcome-screen", guild_id)
    }

    #[must_use]
    pub fn guild_threads_active(guild_id: GuildId) -> String {
        api!("/guilds/{}/threads/active", guild_id)
    }

    #[must_use]
    pub fn guilds() -> &'static str {
        api!("/guilds")
    }

    #[must_use]
    pub fn invite(code: &str) -> String {
        api!("/invites/{}", code)
    }

    #[must_use]
    pub fn invite_optioned(
        code: &str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    ) -> String {
        api!(
            "/invites/{}?with_counts={}&with_expiration={}{}",
            code,
            member_counts,
            expiration,
            event_id.map(|id| format!("&event_id={id}")).unwrap_or_default(),
        )
    }

    #[must_use]
    pub fn oauth2_application_current() -> &'static str {
        api!("/oauth2/applications/@me")
    }

    #[must_use]
    pub fn private_channel() -> &'static str {
        api!("/users/@me/channels")
    }

    #[must_use]
    pub fn status_incidents_unresolved() -> &'static str {
        status!("/incidents/unresolved.json")
    }

    #[must_use]
    pub fn status_maintenances_active() -> &'static str {
        status!("/scheduled-maintenances/active.json")
    }

    #[must_use]
    pub fn status_maintenances_upcoming() -> &'static str {
        status!("/scheduled-maintenances/upcoming.json")
    }

    #[must_use]
    pub fn sticker(sticker_id: StickerId) -> String {
        api!("/stickers/{}", sticker_id)
    }

    #[must_use]
    pub fn sticker_packs() -> &'static str {
        api!("/sticker-packs")
    }

    #[must_use]
    pub fn user<D: Display>(target: D) -> String {
        api!("/users/{}", target)
    }

    #[must_use]
    pub fn user_me_connections() -> &'static str {
        api!("/users/@me/connections")
    }

    #[must_use]
    pub fn user_dm_channels<D: Display>(target: D) -> String {
        api!("/users/{}/channels", target)
    }

    #[must_use]
    pub fn user_guild<D: Display>(target: D, guild_id: GuildId) -> String {
        api!("/users/{}/guilds/{}", target, guild_id)
    }

    #[must_use]
    pub fn user_guilds<D: Display>(target: D) -> String {
        api!("/users/{}/guilds", target)
    }

    #[must_use]
    pub fn user_guilds_optioned<D: Display>(
        target: D,
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/users/{}/guilds?", target);

        if let Some(limit) = limit {
            write!(s, "&limit={limit}").unwrap();
        }

        if let Some(after) = after {
            write!(s, "&after={after}").unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={before}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn voice_regions() -> &'static str {
        api!("/voice/regions")
    }

    #[must_use]
    pub fn webhook(webhook_id: WebhookId) -> String {
        api!("/webhooks/{}", webhook_id)
    }

    #[must_use]
    pub fn webhook_with_token<D>(webhook_id: WebhookId, token: D) -> String
    where
        D: Display,
    {
        api!("/webhooks/{}/{}", webhook_id, token)
    }

    #[must_use]
    pub fn webhook_with_token_optioned<D>(
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
        token: D,
        wait: bool,
    ) -> String
    where
        D: Display,
    {
        let mut s = api!("/webhooks/{}/{}?wait={}", webhook_id, token, wait);

        if let Some(thread_id) = thread_id {
            write!(s, "&thread_id={thread_id}").unwrap();
        }

        s
    }

    #[must_use]
    pub fn webhook_message<D>(webhook_id: WebhookId, token: D, message_id: MessageId) -> String
    where
        D: Display,
    {
        api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id)
    }

    #[must_use]
    pub fn webhook_original_interaction_response<D: Display>(
        application_id: ApplicationId,
        token: D,
    ) -> String {
        api!("/webhooks/{}/{}/messages/@original", application_id, token)
    }

    #[must_use]
    pub fn webhook_followup_message<D: Display>(
        application_id: ApplicationId,
        token: D,
        message_id: MessageId,
    ) -> String {
        api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id)
    }

    #[must_use]
    pub fn webhook_followup_messages<D: Display>(
        application_id: ApplicationId,
        token: D,
    ) -> String {
        api!("/webhooks/{}/{}", application_id, token)
    }

    #[must_use]
    pub fn interaction_response<D: Display>(interaction_id: InteractionId, token: D) -> String {
        api!("/interactions/{}/{}/callback", interaction_id, token)
    }

    #[must_use]
    pub fn application_command(application_id: ApplicationId, command_id: CommandId) -> String {
        api!("/applications/{}/commands/{}", application_id, command_id)
    }

    #[must_use]
    pub fn application_commands(application_id: ApplicationId) -> String {
        api!("/applications/{}/commands", application_id)
    }

    #[must_use]
    pub fn application_commands_optioned(
        application_id: ApplicationId,
        with_localizations: bool,
    ) -> String {
        let mut s = api!("/applications/{}/commands", application_id);
        if with_localizations {
            write!(s, "?with_localizations={}", with_localizations).unwrap();
        }
        s
    }

    #[must_use]
    pub fn application_guild_command(
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> String {
        api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id)
    }

    #[must_use]
    pub fn application_guild_command_permissions(
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> String {
        api!(
            "/applications/{}/guilds/{}/commands/{}/permissions",
            application_id,
            guild_id,
            command_id,
        )
    }

    #[must_use]
    pub fn application_guild_commands(application_id: ApplicationId, guild_id: GuildId) -> String {
        api!("/applications/{}/guilds/{}/commands", application_id, guild_id)
    }

    #[must_use]
    pub fn application_guild_commands_optioned(
        application_id: ApplicationId,
        guild_id: GuildId,
        with_localizations: bool,
    ) -> String {
        let mut s = api!("/applications/{}/guilds/{}/commands", application_id, guild_id);
        if with_localizations {
            write!(s, "?with_localizations={}", with_localizations).unwrap();
        }
        s
    }

    pub fn application_guild_commands_permissions(
        application_id: ApplicationId,
        guild_id: GuildId,
    ) -> String {
        api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id)
    }

    #[must_use]
    pub fn stage_instances() -> &'static str {
        api!("/stage-instances")
    }

    #[must_use]
    pub fn stage_instance(channel_id: ChannelId) -> String {
        api!("/stage-instances/{}", channel_id)
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum RouteInfo<'a> {
    AddGuildMember {
        guild_id: GuildId,
        user_id: UserId,
    },
    AddMemberRole {
        guild_id: GuildId,
        role_id: RoleId,
        user_id: UserId,
    },
    GuildBanUser {
        guild_id: GuildId,
        user_id: UserId,
        delete_message_days: Option<u8>,
    },
    BroadcastTyping {
        channel_id: ChannelId,
    },
    CreateAutoModRule {
        guild_id: GuildId,
    },
    CreateChannel {
        guild_id: GuildId,
    },
    CreateStageInstance,
    CreatePublicThread {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    CreatePrivateThread {
        channel_id: ChannelId,
    },
    CreateEmoji {
        guild_id: GuildId,
    },
    CreateFollowupMessage {
        application_id: ApplicationId,
        interaction_token: &'a str,
    },
    CreateGlobalApplicationCommand {
        application_id: ApplicationId,
    },
    CreateGlobalApplicationCommands {
        application_id: ApplicationId,
    },
    CreateGuild,
    CreateGuildApplicationCommand {
        application_id: ApplicationId,
        guild_id: GuildId,
    },
    CreateGuildApplicationCommands {
        application_id: ApplicationId,
        guild_id: GuildId,
    },
    CreateGuildIntegration {
        guild_id: GuildId,
        integration_id: IntegrationId,
    },
    CreateInteractionResponse {
        interaction_id: InteractionId,
        interaction_token: &'a str,
    },
    CreateInvite {
        channel_id: ChannelId,
    },
    CreateMessage {
        channel_id: ChannelId,
    },
    CreatePermission {
        channel_id: ChannelId,
        target_id: TargetId,
    },
    CreatePrivateChannel,
    CreateReaction {
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: &'a str,
    },
    CreateRole {
        guild_id: GuildId,
    },
    CreateScheduledEvent {
        guild_id: GuildId,
    },
    CreateSticker {
        guild_id: GuildId,
    },
    CreateWebhook {
        channel_id: ChannelId,
    },
    DeleteAutoModRule {
        guild_id: GuildId,
        rule_id: RuleId,
    },
    DeleteChannel {
        channel_id: ChannelId,
    },
    DeleteStageInstance {
        channel_id: ChannelId,
    },
    DeleteEmoji {
        guild_id: GuildId,
        emoji_id: EmojiId,
    },
    DeleteFollowupMessage {
        application_id: ApplicationId,
        interaction_token: &'a str,
        message_id: MessageId,
    },
    DeleteGlobalApplicationCommand {
        application_id: ApplicationId,
        command_id: CommandId,
    },
    DeleteGuild {
        guild_id: GuildId,
    },
    DeleteGuildApplicationCommand {
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    },
    DeleteGuildIntegration {
        guild_id: GuildId,
        integration_id: IntegrationId,
    },
    DeleteInvite {
        code: &'a str,
    },
    DeleteMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    DeleteMessages {
        channel_id: ChannelId,
    },
    DeleteMessageReactions {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    DeleteMessageReactionEmoji {
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: &'a str,
    },
    DeleteOriginalInteractionResponse {
        application_id: ApplicationId,
        interaction_token: &'a str,
    },
    DeletePermission {
        channel_id: ChannelId,
        target_id: TargetId,
    },
    DeleteReaction {
        channel_id: ChannelId,
        message_id: MessageId,
        user: &'a str,
        reaction: &'a str,
    },
    DeleteRole {
        guild_id: GuildId,
        role_id: RoleId,
    },
    DeleteScheduledEvent {
        guild_id: GuildId,
        event_id: ScheduledEventId,
    },
    DeleteSticker {
        guild_id: GuildId,
        sticker_id: StickerId,
    },
    DeleteWebhook {
        webhook_id: WebhookId,
    },
    DeleteWebhookWithToken {
        token: &'a str,
        webhook_id: WebhookId,
    },
    DeleteWebhookMessage {
        token: &'a str,
        webhook_id: WebhookId,
        message_id: MessageId,
    },
    EditAutoModRule {
        guild_id: GuildId,
        rule_id: RuleId,
    },
    EditChannel {
        channel_id: ChannelId,
    },
    EditStageInstance {
        channel_id: ChannelId,
    },
    EditEmoji {
        guild_id: GuildId,
        emoji_id: EmojiId,
    },
    EditFollowupMessage {
        application_id: ApplicationId,
        interaction_token: &'a str,
        message_id: MessageId,
    },
    EditGlobalApplicationCommand {
        application_id: ApplicationId,
        command_id: CommandId,
    },
    EditGuild {
        guild_id: GuildId,
    },
    EditGuildApplicationCommand {
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    },
    EditGuildApplicationCommandPermission {
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    },
    EditGuildApplicationCommandsPermissions {
        application_id: ApplicationId,
        guild_id: GuildId,
    },
    EditGuildChannels {
        guild_id: GuildId,
    },
    EditGuildWidget {
        guild_id: GuildId,
    },
    EditGuildWelcomeScreen {
        guild_id: GuildId,
    },
    EditMember {
        guild_id: GuildId,
        user_id: UserId,
    },
    EditMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    CrosspostMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    EditMemberMe {
        guild_id: GuildId,
    },
    EditNickname {
        guild_id: GuildId,
    },
    GetOriginalInteractionResponse {
        application_id: ApplicationId,
        interaction_token: &'a str,
    },
    EditOriginalInteractionResponse {
        application_id: ApplicationId,
        interaction_token: &'a str,
    },
    EditProfile,
    EditRole {
        guild_id: GuildId,
        role_id: RoleId,
    },
    EditRolePosition {
        guild_id: GuildId,
    },
    EditScheduledEvent {
        guild_id: GuildId,
        event_id: ScheduledEventId,
    },
    EditSticker {
        guild_id: GuildId,
        sticker_id: StickerId,
    },
    EditThread {
        channel_id: ChannelId,
    },
    EditVoiceState {
        guild_id: GuildId,
        user_id: UserId,
    },
    EditVoiceStateMe {
        guild_id: GuildId,
    },
    EditWebhook {
        webhook_id: WebhookId,
    },
    EditWebhookWithToken {
        token: &'a str,
        webhook_id: WebhookId,
    },
    EditWebhookMessage {
        token: &'a str,
        webhook_id: WebhookId,
        message_id: MessageId,
    },
    ExecuteWebhook {
        token: &'a str,
        wait: bool,
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
    },
    FollowNewsChannel {
        channel_id: ChannelId,
    },
    JoinThread {
        channel_id: ChannelId,
    },
    LeaveThread {
        channel_id: ChannelId,
    },
    AddThreadMember {
        channel_id: ChannelId,
        user_id: UserId,
    },
    RemoveThreadMember {
        channel_id: ChannelId,
        user_id: UserId,
    },
    GetActiveMaintenance,
    GetAuditLogs {
        action_type: Option<u8>,
        before: Option<u64>,
        guild_id: GuildId,
        limit: Option<u8>,
        user_id: Option<UserId>,
    },
    GetAutoModRules {
        guild_id: GuildId,
    },
    GetAutoModRule {
        guild_id: GuildId,
        rule_id: RuleId,
    },
    GetBans {
        guild_id: GuildId,
    },
    GetBotGateway,
    GetChannel {
        channel_id: ChannelId,
    },
    GetChannelInvites {
        channel_id: ChannelId,
    },
    GetChannelWebhooks {
        channel_id: ChannelId,
    },
    GetChannels {
        guild_id: GuildId,
    },
    GetStageInstance {
        channel_id: ChannelId,
    },
    GetChannelThreadMembers {
        channel_id: ChannelId,
    },
    GetChannelArchivedPublicThreads {
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetChannelArchivedPrivateThreads {
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetChannelJoinedPrivateArchivedThreads {
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetCurrentApplicationInfo,
    GetCurrentUser,
    GetEmojis {
        guild_id: GuildId,
    },
    GetEmoji {
        guild_id: GuildId,
        emoji_id: EmojiId,
    },
    GetFollowupMessage {
        application_id: ApplicationId,
        interaction_token: &'a str,
        message_id: MessageId,
    },
    GetGateway,
    GetGlobalApplicationCommands {
        application_id: ApplicationId,
        with_localizations: bool,
    },
    GetGlobalApplicationCommand {
        application_id: ApplicationId,
        command_id: CommandId,
    },
    GetGuild {
        guild_id: GuildId,
    },
    GetGuildWithCounts {
        guild_id: GuildId,
    },
    GetGuildApplicationCommands {
        application_id: ApplicationId,
        guild_id: GuildId,
        with_localizations: bool,
    },
    GetGuildApplicationCommand {
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    },
    GetGuildApplicationCommandsPermissions {
        application_id: ApplicationId,
        guild_id: GuildId,
    },
    GetGuildApplicationCommandPermissions {
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    },
    GetGuildWidget {
        guild_id: GuildId,
    },
    GetGuildActiveThreads {
        guild_id: GuildId,
    },
    GetGuildPreview {
        guild_id: GuildId,
    },
    GetGuildWelcomeScreen {
        guild_id: GuildId,
    },
    GetGuildIntegrations {
        guild_id: GuildId,
    },
    GetGuildInvites {
        guild_id: GuildId,
    },
    GetGuildMembers {
        after: Option<u64>,
        limit: Option<u64>,
        guild_id: GuildId,
    },
    GetGuildPruneCount {
        days: u8,
        guild_id: GuildId,
    },
    GetGuildRegions {
        guild_id: GuildId,
    },
    GetGuildRoles {
        guild_id: GuildId,
    },
    GetScheduledEvent {
        guild_id: GuildId,
        event_id: ScheduledEventId,
        with_user_count: bool,
    },
    GetScheduledEvents {
        guild_id: GuildId,
        with_user_count: bool,
    },
    GetScheduledEventUsers {
        guild_id: GuildId,
        event_id: ScheduledEventId,
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
        with_member: Option<bool>,
    },
    GetGuildStickers {
        guild_id: GuildId,
    },
    GetGuildVanityUrl {
        guild_id: GuildId,
    },
    GetGuildWebhooks {
        guild_id: GuildId,
    },
    GetGuilds {
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetInvite {
        code: &'a str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    },
    GetMember {
        guild_id: GuildId,
        user_id: UserId,
    },
    GetMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    GetMessages {
        channel_id: ChannelId,
        query: &'a str,
    },
    GetPins {
        channel_id: ChannelId,
    },
    GetReactionUsers {
        after: Option<u64>,
        channel_id: ChannelId,
        limit: u8,
        message_id: MessageId,
        reaction: &'a str,
    },
    GetSticker {
        sticker_id: StickerId,
    },
    GetStickerPacks,
    GetGuildSticker {
        guild_id: GuildId,
        sticker_id: StickerId,
    },
    GetUnresolvedIncidents,
    GetUpcomingMaintenances,
    GetUser {
        user_id: UserId,
    },
    GetUserConnections,
    GetUserDmChannels,
    GetVoiceRegions,
    GetWebhook {
        webhook_id: WebhookId,
    },
    GetWebhookWithToken {
        token: &'a str,
        webhook_id: WebhookId,
    },
    GetWebhookMessage {
        token: &'a str,
        webhook_id: WebhookId,
        message_id: MessageId,
    },
    KickMember {
        guild_id: GuildId,
        user_id: UserId,
    },
    LeaveGuild {
        guild_id: GuildId,
    },
    PinMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    RemoveBan {
        guild_id: GuildId,
        user_id: UserId,
    },
    RemoveMemberRole {
        guild_id: GuildId,
        role_id: RoleId,
        user_id: UserId,
    },
    SearchGuildMembers {
        guild_id: GuildId,
        query: &'a str,
        limit: Option<u64>,
    },
    StartGuildPrune {
        days: u8,
        guild_id: GuildId,
    },
    StartIntegrationSync {
        guild_id: GuildId,
        integration_id: IntegrationId,
    },
    StatusIncidentsUnresolved,
    StatusMaintenancesActive,
    StatusMaintenancesUpcoming,
    UnpinMessage {
        channel_id: ChannelId,
        message_id: MessageId,
    },
}

impl<'a> RouteInfo<'a> {
    #[must_use]
    pub fn deconstruct(self) -> (LightMethod, Route, Cow<'static, str>) {
        match self {
            RouteInfo::AddGuildMember {
                guild_id,
                user_id,
            } => (
                LightMethod::Put,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::AddMemberRole {
                guild_id,
                role_id,
                user_id,
            } => (
                LightMethod::Put,
                Route::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Route::guild_member_role(guild_id, user_id, role_id)),
            ),
            RouteInfo::GuildBanUser {
                guild_id,
                delete_message_days,
                user_id,
            } => (
                // TODO
                LightMethod::Put,
                Route::GuildsIdBansUserId(guild_id),
                Cow::from(Route::guild_ban_optioned(
                    guild_id,
                    user_id,
                    delete_message_days.unwrap_or(0),
                )),
            ),
            RouteInfo::BroadcastTyping {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdTyping(channel_id),
                Cow::from(Route::channel_typing(channel_id)),
            ),
            RouteInfo::CreateAutoModRule {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdAutoModRules(guild_id),
                Cow::from(Route::guild_automod_rules(guild_id)),
            ),
            RouteInfo::CreateChannel {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::CreateStageInstance => {
                (LightMethod::Post, Route::StageInstances, Cow::from(Route::stage_instances()))
            },
            RouteInfo::CreatePublicThread {
                channel_id,
                message_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdMessagesIdThreads(channel_id),
                Cow::from(Route::channel_public_threads(channel_id, message_id)),
            ),
            RouteInfo::CreatePrivateThread {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdThreads(channel_id),
                Cow::from(Route::channel_private_threads(channel_id)),
            ),
            RouteInfo::CreateEmoji {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdEmojis(guild_id),
                Cow::from(Route::guild_emojis(guild_id)),
            ),
            RouteInfo::CreateFollowupMessage {
                application_id,
                interaction_token,
            } => (
                LightMethod::Post,
                Route::WebhooksId(WebhookId(application_id.0)),
                Cow::from(Route::webhook_followup_messages(application_id, interaction_token)),
            ),
            RouteInfo::CreateGlobalApplicationCommand {
                application_id,
            } => (
                LightMethod::Post,
                Route::ApplicationsIdCommands(application_id),
                Cow::from(Route::application_commands(application_id)),
            ),
            RouteInfo::CreateGlobalApplicationCommands {
                application_id,
            } => (
                LightMethod::Put,
                Route::ApplicationsIdCommands(application_id),
                Cow::from(Route::application_commands(application_id)),
            ),
            RouteInfo::CreateGuild => {
                (LightMethod::Post, Route::Guilds, Cow::from(Route::guilds()))
            },
            RouteInfo::CreateGuildApplicationCommand {
                application_id,
                guild_id,
            } => (
                LightMethod::Post,
                Route::ApplicationsIdGuildsIdCommands(application_id),
                Cow::from(Route::application_guild_commands(application_id, guild_id)),
            ),
            RouteInfo::CreateGuildApplicationCommands {
                application_id,
                guild_id,
            } => (
                LightMethod::Put,
                Route::ApplicationsIdGuildsIdCommands(application_id),
                Cow::from(Route::application_guild_commands(application_id, guild_id)),
            ),
            RouteInfo::CreateGuildIntegration {
                guild_id,
                integration_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration(guild_id, integration_id)),
            ),
            RouteInfo::CreateInteractionResponse {
                interaction_id,
                interaction_token,
            } => (
                LightMethod::Post,
                Route::InteractionsId(interaction_id),
                Cow::from(Route::interaction_response(interaction_id, interaction_token)),
            ),
            RouteInfo::CreateInvite {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdInvites(channel_id),
                Cow::from(Route::channel_invites(channel_id)),
            ),
            RouteInfo::CreateMessage {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdMessages(channel_id),
                Cow::from(Route::channel_messages(channel_id, None)),
            ),
            RouteInfo::CreatePermission {
                channel_id,
                target_id,
            } => (
                LightMethod::Put,
                Route::ChannelsIdPermissionsOverwriteId(channel_id),
                Cow::from(Route::channel_permission(channel_id, target_id)),
            ),
            RouteInfo::CreatePrivateChannel => (
                LightMethod::Post,
                Route::UsersMeChannels,
                Cow::from(Route::user_dm_channels("@me")),
            ),
            RouteInfo::CreateReaction {
                channel_id,
                message_id,
                reaction,
            } => (
                LightMethod::Put,
                Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                Cow::from(Route::channel_message_reaction(channel_id, message_id, "@me", reaction)),
            ),
            RouteInfo::CreateRole {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdRoles(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::CreateScheduledEvent {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdScheduledEvents(guild_id),
                Cow::from(Route::guild_scheduled_events(guild_id, None)),
            ),
            RouteInfo::CreateSticker {
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdStickers(guild_id),
                Cow::from(Route::guild_stickers(guild_id)),
            ),
            RouteInfo::CrosspostMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdCrosspostsMessageId(channel_id),
                Cow::from(Route::channel_message_crosspost(channel_id, message_id)),
            ),
            RouteInfo::CreateWebhook {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdWebhooks(channel_id),
                Cow::from(Route::channel_webhooks(channel_id)),
            ),
            RouteInfo::DeleteAutoModRule {
                guild_id,
                rule_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdAutoModRulesId(guild_id),
                Cow::from(Route::guild_automod_rule(guild_id, rule_id)),
            ),
            RouteInfo::DeleteChannel {
                channel_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::DeleteStageInstance {
                channel_id,
            } => (
                LightMethod::Delete,
                Route::StageInstancesChannelId(channel_id),
                Cow::from(Route::stage_instance(channel_id)),
            ),
            RouteInfo::DeleteEmoji {
                emoji_id,
                guild_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdEmojisId(guild_id),
                Cow::from(Route::guild_emoji(guild_id, emoji_id)),
            ),
            RouteInfo::DeleteFollowupMessage {
                application_id,
                interaction_token,
                message_id,
            } => (
                LightMethod::Delete,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_followup_message(
                    application_id,
                    interaction_token,
                    message_id,
                )),
            ),
            RouteInfo::DeleteGlobalApplicationCommand {
                application_id,
                command_id,
            } => (
                LightMethod::Delete,
                Route::ApplicationsIdCommandsId(application_id),
                Cow::from(Route::application_command(application_id, command_id)),
            ),
            RouteInfo::DeleteGuild {
                guild_id,
            } => {
                (LightMethod::Delete, Route::GuildsId(guild_id), Cow::from(Route::guild(guild_id)))
            },
            RouteInfo::DeleteGuildApplicationCommand {
                application_id,
                guild_id,
                command_id,
            } => (
                LightMethod::Delete,
                Route::ApplicationsIdGuildsIdCommandsId(application_id),
                Cow::from(Route::application_guild_command(application_id, guild_id, command_id)),
            ),
            RouteInfo::DeleteGuildIntegration {
                guild_id,
                integration_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration(guild_id, integration_id)),
            ),
            RouteInfo::DeleteInvite {
                code,
            } => (LightMethod::Delete, Route::InvitesCode, Cow::from(Route::invite(code))),
            RouteInfo::DeleteMessageReactions {
                channel_id,
                message_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesIdReactions(channel_id),
                Cow::from(Route::channel_message_reactions(channel_id, message_id)),
            ),
            RouteInfo::DeleteMessageReactionEmoji {
                channel_id,
                message_id,
                reaction,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesIdReactions(channel_id),
                Cow::from(Route::channel_message_reaction_emoji(channel_id, message_id, reaction)),
            ),
            RouteInfo::DeleteMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesId(LightMethod::Delete, channel_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::DeleteMessages {
                channel_id,
            } => (
                LightMethod::Post,
                Route::ChannelsIdMessagesBulkDelete(channel_id),
                Cow::from(Route::channel_messages_bulk_delete(channel_id)),
            ),
            RouteInfo::DeleteOriginalInteractionResponse {
                application_id,
                interaction_token,
            } => (
                LightMethod::Delete,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_original_interaction_response(
                    application_id,
                    interaction_token,
                )),
            ),
            RouteInfo::DeletePermission {
                channel_id,
                target_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdPermissionsOverwriteId(channel_id),
                Cow::from(Route::channel_permission(channel_id, target_id)),
            ),
            RouteInfo::DeleteReaction {
                channel_id,
                message_id,
                reaction,
                user,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                Cow::from(Route::channel_message_reaction(channel_id, message_id, user, reaction)),
            ),
            RouteInfo::DeleteRole {
                guild_id,
                role_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_role(guild_id, role_id)),
            ),
            RouteInfo::DeleteScheduledEvent {
                guild_id,
                event_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdScheduledEventsId(guild_id),
                Cow::from(Route::guild_scheduled_event(guild_id, event_id, None)),
            ),
            RouteInfo::DeleteSticker {
                guild_id,
                sticker_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdStickersId(guild_id),
                Cow::from(Route::guild_sticker(guild_id, sticker_id)),
            ),
            RouteInfo::DeleteWebhook {
                webhook_id,
            } => (
                LightMethod::Delete,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::DeleteWebhookWithToken {
                token,
                webhook_id,
            } => (
                LightMethod::Delete,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::DeleteWebhookMessage {
                token,
                webhook_id,
                message_id,
            } => (
                LightMethod::Delete,
                Route::WebhooksIdMessagesId(webhook_id),
                Cow::from(Route::webhook_message(webhook_id, token, message_id)),
            ),
            RouteInfo::EditAutoModRule {
                guild_id,
                rule_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdAutoModRulesId(guild_id),
                Cow::from(Route::guild_automod_rule(guild_id, rule_id)),
            ),
            RouteInfo::EditChannel {
                channel_id,
            }
            | RouteInfo::EditThread {
                channel_id,
            } => (
                LightMethod::Patch,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::EditScheduledEvent {
                guild_id,
                event_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdScheduledEventsId(guild_id),
                Cow::from(Route::guild_scheduled_event(guild_id, event_id, None)),
            ),
            RouteInfo::EditStageInstance {
                channel_id,
            } => (
                LightMethod::Patch,
                Route::StageInstancesChannelId(channel_id),
                Cow::from(Route::stage_instance(channel_id)),
            ),
            RouteInfo::EditEmoji {
                emoji_id,
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdEmojisId(guild_id),
                Cow::from(Route::guild_emoji(guild_id, emoji_id)),
            ),
            RouteInfo::EditFollowupMessage {
                application_id,
                interaction_token,
                message_id,
            } => (
                LightMethod::Patch,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_followup_message(
                    application_id,
                    interaction_token,
                    message_id,
                )),
            ),
            RouteInfo::EditGlobalApplicationCommand {
                application_id,
                command_id,
            } => (
                LightMethod::Patch,
                Route::ApplicationsIdCommandsId(application_id),
                Cow::from(Route::application_command(application_id, command_id)),
            ),
            RouteInfo::EditGuild {
                guild_id,
            } => (LightMethod::Patch, Route::GuildsId(guild_id), Cow::from(Route::guild(guild_id))),
            RouteInfo::EditGuildApplicationCommand {
                application_id,
                guild_id,
                command_id,
            } => (
                LightMethod::Patch,
                Route::ApplicationsIdGuildsIdCommandsId(application_id),
                Cow::from(Route::application_guild_command(application_id, guild_id, command_id)),
            ),
            RouteInfo::EditGuildApplicationCommandPermission {
                application_id,
                guild_id,
                command_id,
            } => (
                LightMethod::Put,
                Route::ApplicationsIdGuildsIdCommandIdPermissions(application_id),
                Cow::from(Route::application_guild_command_permissions(
                    application_id,
                    guild_id,
                    command_id,
                )),
            ),
            RouteInfo::EditGuildApplicationCommandsPermissions {
                application_id,
                guild_id,
            } => (
                LightMethod::Put,
                Route::ApplicationsIdGuildsIdCommandsPermissions(application_id),
                Cow::from(Route::application_guild_commands_permissions(application_id, guild_id)),
            ),
            RouteInfo::EditGuildChannels {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::EditGuildWidget {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdWidget(guild_id),
                Cow::from(Route::guild_widget(guild_id)),
            ),
            RouteInfo::EditGuildWelcomeScreen {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdWelcomeScreen(guild_id),
                Cow::from(Route::guild_welcome_screen(guild_id)),
            ),
            RouteInfo::EditMember {
                guild_id,
                user_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::EditMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Patch,
                Route::ChannelsIdMessagesId(LightMethod::Patch, channel_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::EditMemberMe {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdMembersMe(guild_id),
                Cow::from(Route::guild_member_me(guild_id)),
            ),
            RouteInfo::EditNickname {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdMembersMeNick(guild_id),
                Cow::from(Route::guild_nickname(guild_id)),
            ),
            RouteInfo::GetOriginalInteractionResponse {
                application_id,
                interaction_token,
            } => (
                LightMethod::Get,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_original_interaction_response(
                    application_id,
                    interaction_token,
                )),
            ),
            RouteInfo::EditOriginalInteractionResponse {
                application_id,
                interaction_token,
            } => (
                LightMethod::Patch,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_original_interaction_response(
                    application_id,
                    interaction_token,
                )),
            ),
            RouteInfo::EditProfile => {
                (LightMethod::Patch, Route::UsersMe, Cow::from(Route::user("@me")))
            },
            RouteInfo::EditRole {
                guild_id,
                role_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_role(guild_id, role_id)),
            ),
            RouteInfo::EditRolePosition {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::EditSticker {
                guild_id,
                sticker_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdStickersId(guild_id),
                Cow::from(Route::guild_sticker(guild_id, sticker_id)),
            ),
            RouteInfo::EditVoiceState {
                guild_id,
                user_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdVoiceStates(guild_id),
                Cow::from(Route::guild_voice_states(guild_id, user_id)),
            ),
            RouteInfo::EditVoiceStateMe {
                guild_id,
            } => (
                LightMethod::Patch,
                Route::GuildsIdVoiceStatesMe(guild_id),
                Cow::from(Route::guild_voice_states_me(guild_id)),
            ),
            RouteInfo::EditWebhook {
                webhook_id,
            } => (
                LightMethod::Patch,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::EditWebhookWithToken {
                token,
                webhook_id,
            } => (
                LightMethod::Patch,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::GetWebhookMessage {
                token,
                webhook_id,
                message_id,
            } => (
                LightMethod::Get,
                Route::WebhooksIdMessagesId(webhook_id),
                Cow::from(Route::webhook_message(webhook_id, token, message_id)),
            ),
            RouteInfo::EditWebhookMessage {
                token,
                webhook_id,
                message_id,
            } => (
                LightMethod::Patch,
                Route::WebhooksIdMessagesId(webhook_id),
                Cow::from(Route::webhook_message(webhook_id, token, message_id)),
            ),
            RouteInfo::ExecuteWebhook {
                token,
                wait,
                webhook_id,
                thread_id,
            } => (
                LightMethod::Post,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token_optioned(webhook_id, thread_id, token, wait)),
            ),
            RouteInfo::FollowNewsChannel {
                channel_id,
            } => (
                LightMethod::Post,
                Route::FollowNewsChannel(channel_id),
                Cow::from(Route::channel_follow_news(channel_id)),
            ),
            RouteInfo::GetAuditLogs {
                action_type,
                before,
                guild_id,
                limit,
                user_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdAuditLogs(guild_id),
                Cow::from(Route::guild_audit_logs(guild_id, action_type, user_id, before, limit)),
            ),
            RouteInfo::GetAutoModRules {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdAutoModRules(guild_id),
                Cow::from(Route::guild_automod_rules(guild_id)),
            ),
            RouteInfo::GetAutoModRule {
                guild_id,
                rule_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdAutoModRulesId(guild_id),
                Cow::from(Route::guild_automod_rule(guild_id, rule_id)),
            ),
            RouteInfo::GetBans {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdBans(guild_id),
                Cow::from(Route::guild_bans(guild_id)),
            ),
            RouteInfo::GetBotGateway => {
                (LightMethod::Get, Route::GatewayBot, Cow::from(Route::gateway_bot()))
            },
            RouteInfo::GetChannel {
                channel_id,
            } => (
                LightMethod::Get,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::GetStageInstance {
                channel_id,
            } => (
                LightMethod::Get,
                Route::StageInstancesChannelId(channel_id),
                Cow::from(Route::stage_instance(channel_id)),
            ),
            RouteInfo::GetChannelInvites {
                channel_id,
            } => (
                LightMethod::Get,
                Route::ChannelsIdInvites(channel_id),
                Cow::from(Route::channel_invites(channel_id)),
            ),
            RouteInfo::GetChannelWebhooks {
                channel_id,
            } => (
                LightMethod::Get,
                Route::ChannelsIdWebhooks(channel_id),
                Cow::from(Route::channel_webhooks(channel_id)),
            ),
            RouteInfo::GetChannels {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::GetChannelThreadMembers {
                channel_id,
            } => (
                LightMethod::Get,
                Route::ChannelsIdThreadMembers(channel_id),
                Cow::from(Route::channel_thread_members(channel_id)),
            ),
            RouteInfo::GetChannelArchivedPublicThreads {
                channel_id,
                before,
                limit,
            } => (
                LightMethod::Get,
                Route::ChannelsIdArchivedPublicThreads(channel_id),
                Cow::from(Route::channel_archived_public_threads(channel_id, before, limit)),
            ),
            RouteInfo::GetChannelArchivedPrivateThreads {
                channel_id,
                before,
                limit,
            } => (
                LightMethod::Get,
                Route::ChannelsIdArchivedPrivateThreads(channel_id),
                Cow::from(Route::channel_archived_private_threads(channel_id, before, limit)),
            ),
            RouteInfo::GetChannelJoinedPrivateArchivedThreads {
                channel_id,
                before,
                limit,
            } => (
                LightMethod::Get,
                Route::ChannelsIdMeJoindedArchivedPrivateThreads(channel_id),
                Cow::from(Route::channel_joined_private_threads(channel_id, before, limit)),
            ),
            RouteInfo::GetFollowupMessage {
                application_id,
                interaction_token,
                message_id,
            } => (
                LightMethod::Get,
                Route::WebhooksApplicationId(application_id),
                Cow::from(Route::webhook_followup_message(
                    application_id,
                    interaction_token,
                    message_id,
                )),
            ),
            RouteInfo::GetGuildActiveThreads {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdThreadsActive,
                Cow::from(Route::guild_threads_active(guild_id)),
            ),
            RouteInfo::JoinThread {
                channel_id,
            } => (
                LightMethod::Put,
                Route::ChannelsIdThreadMembersMe(channel_id),
                Cow::from(Route::channel_thread_member_me(channel_id)),
            ),
            RouteInfo::LeaveThread {
                channel_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdThreadMembersMe(channel_id),
                Cow::from(Route::channel_thread_member_me(channel_id)),
            ),
            RouteInfo::AddThreadMember {
                channel_id,
                user_id,
            } => (
                LightMethod::Put,
                Route::ChannelsIdThreadMembersUserId(channel_id),
                Cow::from(Route::channel_thread_member(channel_id, user_id)),
            ),
            RouteInfo::RemoveThreadMember {
                channel_id,
                user_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdThreadMembersUserId(channel_id),
                Cow::from(Route::channel_thread_member(channel_id, user_id)),
            ),
            RouteInfo::GetCurrentApplicationInfo => {
                (LightMethod::Get, Route::None, Cow::from(Route::oauth2_application_current()))
            },
            RouteInfo::GetCurrentUser => {
                (LightMethod::Get, Route::UsersMe, Cow::from(Route::user("@me")))
            },
            RouteInfo::GetEmojis {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdEmojis(guild_id),
                Cow::from(Route::guild_emojis(guild_id)),
            ),
            RouteInfo::GetEmoji {
                guild_id,
                emoji_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdEmojisId(guild_id),
                Cow::from(Route::guild_emoji(guild_id, emoji_id)),
            ),
            RouteInfo::GetGateway => {
                (LightMethod::Get, Route::Gateway, Cow::from(Route::gateway()))
            },
            RouteInfo::GetGlobalApplicationCommands {
                application_id,
                with_localizations,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdCommands(application_id),
                Cow::from(Route::application_commands_optioned(application_id, with_localizations)),
            ),
            RouteInfo::GetGlobalApplicationCommand {
                application_id,
                command_id,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdCommandsId(application_id),
                Cow::from(Route::application_command(application_id, command_id)),
            ),
            RouteInfo::GetGuild {
                guild_id,
            } => (LightMethod::Get, Route::GuildsId(guild_id), Cow::from(Route::guild(guild_id))),
            RouteInfo::GetGuildWithCounts {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsId(guild_id),
                Cow::from(Route::guild_with_counts(guild_id)),
            ),
            RouteInfo::GetGuildApplicationCommands {
                application_id,
                guild_id,
                with_localizations,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdGuildsIdCommands(application_id),
                Cow::from(Route::application_guild_commands_optioned(
                    application_id,
                    guild_id,
                    with_localizations,
                )),
            ),
            RouteInfo::GetGuildApplicationCommand {
                application_id,
                guild_id,
                command_id,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdGuildsIdCommandsId(application_id),
                Cow::from(Route::application_guild_command(application_id, guild_id, command_id)),
            ),
            RouteInfo::GetGuildApplicationCommandsPermissions {
                application_id,
                guild_id,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdGuildsIdCommandsPermissions(application_id),
                Cow::from(Route::application_guild_commands_permissions(application_id, guild_id)),
            ),
            RouteInfo::GetGuildApplicationCommandPermissions {
                application_id,
                guild_id,
                command_id,
            } => (
                LightMethod::Get,
                Route::ApplicationsIdGuildsIdCommandIdPermissions(application_id),
                Cow::from(Route::application_guild_command_permissions(
                    application_id,
                    guild_id,
                    command_id,
                )),
            ),
            RouteInfo::GetGuildWidget {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdWidget(guild_id),
                Cow::from(Route::guild_widget(guild_id)),
            ),
            RouteInfo::GetGuildPreview {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdPreview(guild_id),
                Cow::from(Route::guild_preview(guild_id)),
            ),
            RouteInfo::GetGuildWelcomeScreen {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdWelcomeScreen(guild_id),
                Cow::from(Route::guild_welcome_screen(guild_id)),
            ),
            RouteInfo::GetGuildIntegrations {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdIntegrations(guild_id),
                Cow::from(Route::guild_integrations(guild_id)),
            ),
            RouteInfo::GetGuildInvites {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdInvites(guild_id),
                Cow::from(Route::guild_invites(guild_id)),
            ),
            RouteInfo::GetGuildMembers {
                after,
                guild_id,
                limit,
            } => (
                LightMethod::Get,
                Route::GuildsIdMembers(guild_id),
                Cow::from(Route::guild_members_optioned(guild_id, after, limit)),
            ),
            RouteInfo::GetGuildPruneCount {
                days,
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdPrune(guild_id),
                Cow::from(Route::guild_prune(guild_id, days)),
            ),
            RouteInfo::GetGuildRegions {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdRegions(guild_id),
                Cow::from(Route::guild_regions(guild_id)),
            ),
            RouteInfo::GetGuildRoles {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdRoles(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::GetGuildSticker {
                guild_id,
                sticker_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdStickersId(guild_id),
                Cow::from(Route::guild_sticker(guild_id, sticker_id)),
            ),
            RouteInfo::GetGuildStickers {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdStickers(guild_id),
                Cow::from(Route::guild_stickers(guild_id)),
            ),
            RouteInfo::GetGuildVanityUrl {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdVanityUrl(guild_id),
                Cow::from(Route::guild_vanity_url(guild_id)),
            ),
            RouteInfo::GetGuildWebhooks {
                guild_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdWebhooks(guild_id),
                Cow::from(Route::guild_webhooks(guild_id)),
            ),
            RouteInfo::GetGuilds {
                after,
                before,
                limit,
            } => (
                LightMethod::Get,
                Route::UsersMeGuilds,
                Cow::from(Route::user_guilds_optioned("@me", after, before, limit)),
            ),
            RouteInfo::GetInvite {
                code,
                member_counts,
                expiration,
                event_id,
            } => (
                LightMethod::Get,
                Route::InvitesCode,
                Cow::from(Route::invite_optioned(code, member_counts, expiration, event_id)),
            ),
            RouteInfo::GetMember {
                guild_id,
                user_id,
            } => (
                LightMethod::Get,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::GetMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Get,
                Route::ChannelsIdMessagesId(LightMethod::Get, channel_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::GetMessages {
                channel_id,
                query,
            } => (
                LightMethod::Get,
                Route::ChannelsIdMessages(channel_id),
                Cow::from(Route::channel_messages(channel_id, Some(query))),
            ),
            RouteInfo::GetPins {
                channel_id,
            } => (
                LightMethod::Get,
                Route::ChannelsIdPins(channel_id),
                Cow::from(Route::channel_pins(channel_id)),
            ),
            RouteInfo::GetReactionUsers {
                after,
                channel_id,
                limit,
                message_id,
                reaction,
            } => (
                LightMethod::Get,
                Route::ChannelsIdMessagesIdReactions(channel_id),
                Cow::from(Route::channel_message_reactions_list(
                    channel_id, message_id, reaction, limit, after,
                )),
            ),
            RouteInfo::GetScheduledEvent {
                guild_id,
                event_id,
                with_user_count,
            } => (
                LightMethod::Get,
                Route::GuildsIdScheduledEventsId(guild_id),
                Cow::from(Route::guild_scheduled_event(guild_id, event_id, Some(with_user_count))),
            ),
            RouteInfo::GetScheduledEvents {
                guild_id,
                with_user_count,
            } => (
                LightMethod::Get,
                Route::GuildsIdScheduledEvents(guild_id),
                Cow::from(Route::guild_scheduled_events(guild_id, Some(with_user_count))),
            ),
            RouteInfo::GetScheduledEventUsers {
                guild_id,
                event_id,
                after,
                before,
                limit,
                with_member,
            } => (
                LightMethod::Get,
                Route::GuildsIdScheduledEventsIdUsers(guild_id),
                Cow::from(Route::guild_scheduled_event_users(
                    guild_id,
                    event_id,
                    after,
                    before,
                    limit,
                    with_member,
                )),
            ),
            RouteInfo::GetSticker {
                sticker_id,
            } => (LightMethod::Get, Route::StickersId, Cow::from(Route::sticker(sticker_id))),
            RouteInfo::GetStickerPacks => {
                (LightMethod::Get, Route::StickerPacks, Cow::from(Route::sticker_packs()))
            },
            RouteInfo::GetUser {
                user_id,
            } => (LightMethod::Get, Route::UsersId, Cow::from(Route::user(user_id))),
            RouteInfo::GetUserConnections => (
                LightMethod::Get,
                Route::UsersMeConnections,
                Cow::from(Route::user_me_connections()),
            ),
            RouteInfo::GetUserDmChannels => (
                LightMethod::Get,
                Route::UsersMeChannels,
                Cow::from(Route::user_dm_channels("@me")),
            ),
            RouteInfo::GetVoiceRegions => {
                (LightMethod::Get, Route::VoiceRegions, Cow::from(Route::voice_regions()))
            },
            RouteInfo::GetWebhook {
                webhook_id,
            } => (
                LightMethod::Get,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::GetWebhookWithToken {
                token,
                webhook_id,
            } => (
                LightMethod::Get,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::KickMember {
                guild_id,
                user_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_kick_optioned(guild_id, user_id)),
            ),
            RouteInfo::LeaveGuild {
                guild_id,
            } => (
                LightMethod::Delete,
                Route::UsersMeGuildsId,
                Cow::from(Route::user_guild("@me", guild_id)),
            ),
            RouteInfo::PinMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Put,
                Route::ChannelsIdPins(channel_id),
                Cow::from(Route::channel_pin(channel_id, message_id)),
            ),
            RouteInfo::RemoveBan {
                guild_id,
                user_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdBansUserId(guild_id),
                Cow::from(Route::guild_ban(guild_id, user_id)),
            ),
            RouteInfo::RemoveMemberRole {
                guild_id,
                role_id,
                user_id,
            } => (
                LightMethod::Delete,
                Route::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Route::guild_member_role(guild_id, user_id, role_id)),
            ),
            RouteInfo::SearchGuildMembers {
                guild_id,
                query,
                limit,
            } => (
                LightMethod::Get,
                Route::GuildsIdMembersSearch(guild_id),
                Cow::from(Route::guild_members_search(guild_id, query, limit)),
            ),
            RouteInfo::StartGuildPrune {
                days,
                guild_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdPrune(guild_id),
                Cow::from(Route::guild_prune(guild_id, days)),
            ),
            RouteInfo::StartIntegrationSync {
                guild_id,
                integration_id,
            } => (
                LightMethod::Post,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration_sync(guild_id, integration_id)),
            ),
            RouteInfo::GetUnresolvedIncidents | RouteInfo::StatusIncidentsUnresolved => {
                (LightMethod::Get, Route::None, Cow::from(Route::status_incidents_unresolved()))
            },
            RouteInfo::GetActiveMaintenance | RouteInfo::StatusMaintenancesActive => {
                (LightMethod::Get, Route::None, Cow::from(Route::status_maintenances_active()))
            },
            RouteInfo::GetUpcomingMaintenances | RouteInfo::StatusMaintenancesUpcoming => {
                (LightMethod::Get, Route::None, Cow::from(Route::status_maintenances_upcoming()))
            },
            RouteInfo::UnpinMessage {
                channel_id,
                message_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsIdPinsMessageId(channel_id),
                Cow::from(Route::channel_pin(channel_id, message_id)),
            ),
        }
    }
}
