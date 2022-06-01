use std::borrow::Cow;
use std::fmt::{Display, Write};

use super::LightMethod;
use crate::constants;

/// A representation of all routes registered within the library. These are safe
/// and memory-efficient representations of each path that functions exist for
/// in the [`http`] module.
///
/// [`http`]: crate::http
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Route {
    /// Route for the `/channels/:channel_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsId(u64),
    /// Route for the `/channels/:channel_id/invites` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdInvites(u64),
    /// Route for the `/channels/:channel_id/messages` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessages(u64),
    /// Route for the `/channels/:channel_id/messages/bulk-delete` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessagesBulkDelete(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// This route is a unique case. The ratelimit for message _deletions_ is
    /// different than the overall route ratelimit.
    ///
    /// Refer to the docs on [Rate Limits] in the yellow warning section.
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    /// [Rate Limits]: https://discord.com/developers/docs/topics/rate-limits
    ChannelsIdMessagesId(LightMethod, u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/ack` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessagesIdAck(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/reactions`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessagesIdReactions(u64),
    /// Route for the
    /// `/channels/:channel_id/messages/:message_id/reactions/:reaction/@me`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessagesIdReactionsUserIdType(u64),
    /// Route for the `/channels/:channel_id/permissions/:target_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdPermissionsOverwriteId(u64),
    /// Route for the `/channels/:channel_id/pins` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdPins(u64),
    /// Route for the `/channels/:channel_id/pins/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdPinsMessageId(u64),
    /// Route for the `/channels/:channel_id/message/:message_id/crosspost` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdCrosspostsMessageId(u64),
    /// Route for the `/channels/:channel_id/typing` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdTyping(u64),
    /// Route for the `/channels/:channel_id/webhooks` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdWebhooks(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/threads` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMessagesIdThreads(u64),
    /// Route for the `/channels/:channel_id/threads` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdThreads(u64),
    /// Route for the `/channels/:channel_id/thread-members/@me` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdThreadMembersMe(u64),
    /// Route for the `/channels/:channel_id/thread-members/:user_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdThreadMembersUserId(u64),
    /// Route for the `/channels/channel_id/thread-members` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdThreadMembers(u64),
    /// Route for the `/channels/:channel_id/threads/archived/public` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdArchivedPublicThreads(u64),
    /// Route for the `/channels/:channel_id/threads/archived/private` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdArchivedPrivateThreads(u64),
    /// Route for the `/channels/:channel_id/users/@me/threads/archived/private` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    ChannelsIdMeJoindedArchivedPrivateThreads(u64),
    /// Route for the `/gateway` path.
    Gateway,
    /// Route for the `/gateway/bot` path.
    GatewayBot,
    /// Route for the `/guilds` path.
    Guilds,
    /// Route for the `/guilds/:guild_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsId(u64),
    /// Route for the `/guilds/:guild_id/bans` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdBans(u64),
    /// Route for the `/guilds/:guild_id/audit-logs` path.
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdAuditLogs(u64),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdBansUserId(u64),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdChannels(u64),
    /// Route for the `/guilds/:guild_id/widget` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdWidget(u64),
    /// Route for the `/guilds/:guild_id/preview` path.
    ///
    /// The data is the relevant [`GuildPreview`].
    ///
    /// [`GuildPreview`]: crate::model::guild::GuildPreview
    GuildsIdPreview(u64),
    /// Route for the `/guilds/:guild_id/emojis` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdEmojis(u64),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdEmojisId(u64),
    /// Route for the `/guilds/:guild_id/integrations` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdIntegrations(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdIntegrationsId(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdIntegrationsIdSync(u64),
    /// Route for the `/guilds/:guild_id/invites` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdInvites(u64),
    /// Route for the `/guilds/:guild_id/members` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembers(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembersId(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembersIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/members/@me` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembersMe(u64),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembersMeNick(u64),
    /// Route for the `/guilds/:guild_id/members/search` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdMembersSearch(u64),
    /// Route for the `/guilds/:guild_id/prune` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdPrune(u64),
    /// Route for the `/guilds/:guild_id/regions` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdRegions(u64),
    /// Route for the `/guilds/:guild_id/roles` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdRoles(u64),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/scheduled-events` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdScheduledEvents(u64),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdScheduledEventsId(u64),
    /// Route for the `/guilds/:guild_id/scheduled-events/:event_id/users` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdScheduledEventsIdUsers(u64),
    /// Route for the `/guilds/:guild_id/stickers` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdStickers(u64),
    /// Route for the `/guilds/:guild_id/stickers/:sticker_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdStickersId(u64),
    /// Route for the `/guilds/:guild_id/vanity-url` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdVanityUrl(u64),
    /// Route for the `/guilds/:guild_id/voice-states/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdVoiceStates(u64),
    /// Route for the `/guilds/:guild_id/voice-states/@me` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdVoiceStatesMe(u64),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdWebhooks(u64),
    /// Route for the `/guilds/:guild_id/welcome-screen` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
    GuildsIdWelcomeScreen(u64),
    /// Route for the `/guilds/:guild_id/threads/active` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: crate::model::id::GuildId
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
    WebhooksId(u64),
    /// Route for the `/webhooks/:webhook_id/:token/messages/:message_id` path.
    ///
    /// The data is the relevant [`WebhookId`].
    ///
    /// [`WebhookId`]: crate::model::id::WebhookId
    WebhooksIdMessagesId(u64),
    /// Route for the `/webhooks/:application_id` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    WebhooksApplicationId(u64),
    /// Route for the `/interactions/:interaction_id` path.
    ///
    /// The data is the relevant [`InteractionId`].
    ///
    /// [`InteractionId`]: crate::model::id::InteractionId
    InteractionsId(u64),
    /// Route for the `/applications/:application_id` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdCommands(u64),
    /// Route for the `/applications/:application_id/commands/:command_id` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdCommandsId(u64),
    /// Route for the `/applications/:application_id/guilds/:guild_id` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdGuildsIdCommands(u64),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/permissions` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdGuildsIdCommandsPermissions(u64),
    /// Route for the `/applications/:application_id/guilds/:guild_id/commands/:command_id/permissions` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdGuildsIdCommandIdPermissions(u64),
    /// Route for the `/applications/:application_id/guilds/:guild_id` path.
    ///
    /// The data is the relevant [`ApplicationId`].
    ///
    /// [`ApplicationId`]: crate::model::id::ApplicationId
    ApplicationsIdGuildsIdCommandsId(u64),
    /// Route for the `/stage-instances` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    StageInstances,
    /// Route for the `/stage-instances/:channel_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: crate::model::id::ChannelId
    StageInstancesChannelId(u64),
    /// Route where no ratelimit headers are in place (i.e. user account-only
    /// routes).
    ///
    /// This is a special case, in that if the route is [`None`] then pre- and
    /// post-hooks are not executed.
    None,
}

impl Route {
    #[must_use]
    pub fn channel(channel_id: u64) -> String {
        api!("/channels/{}", channel_id)
    }

    #[must_use]
    pub fn channel_invites(channel_id: u64) -> String {
        api!("/channels/{}/invites", channel_id)
    }

    #[must_use]
    pub fn channel_message(channel_id: u64, message_id: u64) -> String {
        api!("/channels/{}/messages/{}", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_crosspost(channel_id: u64, message_id: u64) -> String {
        api!("/channels/{}/messages/{}/crosspost", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_reaction<D, T>(
        channel_id: u64,
        message_id: u64,
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
        channel_id: u64,
        message_id: u64,
        reaction_type: T,
    ) -> String
    where
        T: Display,
    {
        api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction_type)
    }

    #[must_use]
    pub fn channel_message_reactions(channel_id: u64, message_id: u64) -> String {
        api!("/channels/{}/messages/{}/reactions", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_message_reactions_list(
        channel_id: u64,
        message_id: u64,
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
            write!(url, "&after={}", after).unwrap();
        }

        url
    }

    #[must_use]
    pub fn channel_messages(channel_id: u64, query: Option<&str>) -> String {
        api!("/channels/{}/messages{}", channel_id, query.unwrap_or(""))
    }

    #[must_use]
    pub fn channel_messages_bulk_delete(channel_id: u64) -> String {
        api!("/channels/{}/messages/bulk-delete", channel_id)
    }

    #[must_use]
    pub fn channel_permission(channel_id: u64, target_id: u64) -> String {
        api!("/channels/{}/permissions/{}", channel_id, target_id)
    }

    #[must_use]
    pub fn channel_pin(channel_id: u64, message_id: u64) -> String {
        api!("/channels/{}/pins/{}", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_pins(channel_id: u64) -> String {
        api!("/channels/{}/pins", channel_id)
    }

    #[must_use]
    pub fn channel_typing(channel_id: u64) -> String {
        api!("/channels/{}/typing", channel_id)
    }

    #[must_use]
    pub fn channel_webhooks(channel_id: u64) -> String {
        api!("/channels/{}/webhooks", channel_id)
    }

    #[must_use]
    pub fn channel_public_threads(channel_id: u64, message_id: u64) -> String {
        api!("/channels/{}/messages/{}/threads", channel_id, message_id)
    }

    #[must_use]
    pub fn channel_private_threads(channel_id: u64) -> String {
        api!("/channels/{}/threads", channel_id)
    }

    #[must_use]
    pub fn channel_thread_member(channel_id: u64, user_id: u64) -> String {
        api!("/channels/{}/thread-members/{}", channel_id, user_id)
    }

    #[must_use]
    pub fn channel_thread_member_me(channel_id: u64) -> String {
        api!("/channels/{}/thread-members/@me", channel_id)
    }

    #[must_use]
    pub fn channel_thread_members(channel_id: u64) -> String {
        api!("/channels/{}/thread-members", channel_id)
    }

    #[must_use]
    pub fn channel_archived_public_threads(
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/threads/archived/public", channel_id);

        if let Some(id) = before {
            write!(s, "&before={}", id).unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={}", limit).unwrap();
        }

        s
    }

    #[must_use]
    pub fn channel_archived_private_threads(
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/threads/archived/private", channel_id);

        if let Some(id) = before {
            write!(s, "&before={}", id).unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={}", limit).unwrap();
        }

        s
    }

    #[must_use]
    pub fn channel_joined_private_threads(
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = api!("/channels/{}/users/@me/threads/archived/private", channel_id);

        if let Some(id) = before {
            write!(s, "&before={}", id).unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={}", limit).unwrap();
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
    pub fn guild(guild_id: u64) -> String {
        api!("/guilds/{}", guild_id)
    }

    #[must_use]
    pub fn guild_with_counts(guild_id: u64) -> String {
        api!("/guilds/{}?with_counts=true", guild_id)
    }

    #[must_use]
    pub fn guild_audit_logs(
        guild_id: u64,
        action_type: Option<u8>,
        user_id: Option<u64>,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> String {
        let mut s = api!("/guilds/{}/audit-logs?", guild_id);

        if let Some(action_type) = action_type {
            write!(s, "&action_type={}", action_type).unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={}", before).unwrap();
        }

        if let Some(limit) = limit {
            write!(s, "&limit={}", limit).unwrap();
        }

        if let Some(user_id) = user_id {
            write!(s, "&user_id={}", user_id).unwrap();
        }

        s
    }

    #[must_use]
    pub fn guild_ban(guild_id: u64, user_id: u64) -> String {
        api!("/guilds/{}/bans/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_ban_optioned(guild_id: u64, user_id: u64, delete_message_days: u8) -> String {
        api!("/guilds/{}/bans/{}?delete_message_days={}", guild_id, user_id, delete_message_days)
    }

    #[must_use]
    pub fn guild_kick_optioned(guild_id: u64, user_id: u64) -> String {
        api!("/guilds/{}/members/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_bans(guild_id: u64) -> String {
        api!("/guilds/{}/bans", guild_id)
    }

    #[must_use]
    pub fn guild_channels(guild_id: u64) -> String {
        api!("/guilds/{}/channels", guild_id)
    }

    #[must_use]
    pub fn guild_widget(guild_id: u64) -> String {
        api!("/guilds/{}/widget", guild_id)
    }

    #[must_use]
    pub fn guild_preview(guild_id: u64) -> String {
        api!("/guilds/{}/preview", guild_id)
    }

    #[must_use]
    pub fn guild_emojis(guild_id: u64) -> String {
        api!("/guilds/{}/emojis", guild_id)
    }

    #[must_use]
    pub fn guild_emoji(guild_id: u64, emoji_id: u64) -> String {
        api!("/guilds/{}/emojis/{}", guild_id, emoji_id)
    }

    #[must_use]
    pub fn guild_integration(guild_id: u64, integration_id: u64) -> String {
        api!("/guilds/{}/integrations/{}", guild_id, integration_id)
    }

    #[must_use]
    pub fn guild_integration_sync(guild_id: u64, integration_id: u64) -> String {
        api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id)
    }

    #[must_use]
    pub fn guild_integrations(guild_id: u64) -> String {
        api!("/guilds/{}/integrations", guild_id)
    }

    #[must_use]
    pub fn guild_invites(guild_id: u64) -> String {
        api!("/guilds/{}/invites", guild_id)
    }

    #[must_use]
    pub fn guild_member(guild_id: u64, user_id: u64) -> String {
        api!("/guilds/{}/members/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_member_role(guild_id: u64, user_id: u64, role_id: u64) -> String {
        api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id)
    }

    #[must_use]
    pub fn guild_members(guild_id: u64) -> String {
        api!("/guilds/{}/members", guild_id)
    }

    #[must_use]
    pub fn guild_members_search(guild_id: u64, query: &str, limit: Option<u64>) -> String {
        let mut s = api!("/guilds/{}/members/search?", guild_id);

        write!(s, "&query={}&limit={}", query, limit.unwrap_or(constants::MEMBER_FETCH_LIMIT))
            .unwrap();
        s
    }

    #[must_use]
    pub fn guild_members_optioned(guild_id: u64, after: Option<u64>, limit: Option<u64>) -> String {
        let mut s = api!("/guilds/{}/members?", guild_id);

        if let Some(after) = after {
            write!(s, "&after={}", after).unwrap();
        }

        write!(s, "&limit={}", limit.unwrap_or(constants::MEMBER_FETCH_LIMIT)).unwrap();
        s
    }

    #[must_use]
    pub fn guild_member_me(guild_id: u64) -> String {
        api!("/guilds/{}/members/@me", guild_id)
    }

    #[must_use]
    pub fn guild_nickname(guild_id: u64) -> String {
        api!("/guilds/{}/members/@me/nick", guild_id)
    }

    #[must_use]
    pub fn guild_prune(guild_id: u64, days: u64) -> String {
        api!("/guilds/{}/prune?days={}", guild_id, days)
    }

    #[must_use]
    pub fn guild_regions(guild_id: u64) -> String {
        api!("/guilds/{}/regions", guild_id)
    }

    #[must_use]
    pub fn guild_role(guild_id: u64, role_id: u64) -> String {
        api!("/guilds/{}/roles/{}", guild_id, role_id)
    }

    #[must_use]
    pub fn guild_roles(guild_id: u64) -> String {
        api!("/guilds/{}/roles", guild_id)
    }

    #[must_use]
    pub fn guild_scheduled_event(
        guild_id: u64,
        event_id: u64,
        with_user_count: Option<bool>,
    ) -> String {
        let mut s = api!("/guilds/{}/scheduled-events/{}", guild_id, event_id);
        if let Some(b) = with_user_count {
            write!(s, "?with_user_count={}", b).unwrap();
        }
        s
    }

    #[must_use]
    pub fn guild_scheduled_events(guild_id: u64, with_user_count: Option<bool>) -> String {
        let mut s = api!("/guilds/{}/scheduled-events", guild_id);
        if let Some(b) = with_user_count {
            write!(s, "?with_user_count={}", b).unwrap();
        }
        s
    }

    #[must_use]
    pub fn guild_scheduled_event_users(
        guild_id: u64,
        event_id: u64,
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
        with_member: Option<bool>,
    ) -> String {
        let mut s = api!("/guilds/{}/scheduled-events/{}/users?", guild_id, event_id);

        if let Some(limit) = limit {
            write!(s, "&limit={}", limit).unwrap();
        }

        if let Some(after) = after {
            write!(s, "&after={}", after).unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={}", before).unwrap();
        }

        if let Some(with_member) = with_member {
            write!(s, "&with_member={}", with_member).unwrap();
        }

        s
    }

    #[must_use]
    pub fn guild_sticker(guild_id: u64, sticker_id: u64) -> String {
        api!("/guilds/{}/stickers/{}", guild_id, sticker_id)
    }

    #[must_use]
    pub fn guild_stickers(guild_id: u64) -> String {
        api!("/guilds/{}/stickers", guild_id)
    }

    #[must_use]
    pub fn guild_vanity_url(guild_id: u64) -> String {
        api!("/guilds/{}/vanity-url", guild_id)
    }

    #[must_use]
    pub fn guild_voice_states(guild_id: u64, user_id: u64) -> String {
        api!("/guilds/{}/voice-states/{}", guild_id, user_id)
    }

    #[must_use]
    pub fn guild_voice_states_me(guild_id: u64) -> String {
        api!("/guilds/{}/voice-states/@me", guild_id)
    }

    #[must_use]
    pub fn guild_webhooks(guild_id: u64) -> String {
        api!("/guilds/{}/webhooks", guild_id)
    }

    #[must_use]
    pub fn guild_welcome_screen(guild_id: u64) -> String {
        api!("/guilds/{}/welcome-screen", guild_id)
    }

    #[must_use]
    pub fn guild_threads_active(guild_id: u64) -> String {
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
        event_id: Option<u64>,
    ) -> String {
        api!(
            "/invites/{}?with_counts={}&with_expiration={}{}",
            code,
            member_counts,
            expiration,
            match event_id {
                Some(id) => format!("&event_id={}", id),
                None => "".to_string(),
            }
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
    pub fn sticker(sticker_id: u64) -> String {
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
    pub fn user_guild<D: Display>(target: D, guild_id: u64) -> String {
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
            write!(s, "&limit={}", limit).unwrap();
        }

        if let Some(after) = after {
            write!(s, "&after={}", after).unwrap();
        }

        if let Some(before) = before {
            write!(s, "&before={}", before).unwrap();
        }

        s
    }

    #[must_use]
    pub fn voice_regions() -> &'static str {
        api!("/voice/regions")
    }

    #[must_use]
    pub fn webhook(webhook_id: u64) -> String {
        api!("/webhooks/{}", webhook_id)
    }

    #[must_use]
    pub fn webhook_with_token<D>(webhook_id: u64, token: D) -> String
    where
        D: Display,
    {
        api!("/webhooks/{}/{}", webhook_id, token)
    }

    #[must_use]
    pub fn webhook_with_token_optioned<D>(webhook_id: u64, token: D, wait: bool) -> String
    where
        D: Display,
    {
        api!("/webhooks/{}/{}?wait={}", webhook_id, token, wait)
    }

    #[must_use]
    pub fn webhook_message<D>(webhook_id: u64, token: D, message_id: u64) -> String
    where
        D: Display,
    {
        api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id)
    }

    #[must_use]
    pub fn webhook_original_interaction_response<D: Display>(
        application_id: u64,
        token: D,
    ) -> String {
        api!("/webhooks/{}/{}/messages/@original", application_id, token)
    }

    #[must_use]
    pub fn webhook_followup_message<D: Display>(
        application_id: u64,
        token: D,
        message_id: u64,
    ) -> String {
        api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id)
    }

    #[must_use]
    pub fn webhook_followup_messages<D: Display>(application_id: u64, token: D) -> String {
        api!("/webhooks/{}/{}", application_id, token)
    }

    #[must_use]
    pub fn interaction_response<D: Display>(application_id: u64, token: D) -> String {
        api!("/interactions/{}/{}/callback", application_id, token)
    }

    #[must_use]
    pub fn application_command(application_id: u64, command_id: u64) -> String {
        api!("/applications/{}/commands/{}", application_id, command_id)
    }

    #[must_use]
    pub fn application_commands(application_id: u64) -> String {
        api!("/applications/{}/commands", application_id)
    }

    #[must_use]
    pub fn application_guild_command(
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    ) -> String {
        api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id)
    }

    #[must_use]
    pub fn application_guild_command_permissions(
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    ) -> String {
        api!(
            "/applications/{}/guilds/{}/commands/{}/permissions",
            application_id,
            guild_id,
            command_id,
        )
    }

    #[must_use]
    pub fn application_guild_commands(application_id: u64, guild_id: u64) -> String {
        api!("/applications/{}/guilds/{}/commands", application_id, guild_id)
    }

    #[must_use]
    pub fn application_guild_commands_permissions(application_id: u64, guild_id: u64) -> String {
        api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id)
    }

    #[must_use]
    pub fn stage_instances() -> &'static str {
        api!("/stage-instances")
    }

    #[must_use]
    pub fn stage_instance(channel_id: u64) -> String {
        api!("/stage-instances/{}", channel_id)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RouteInfo<'a> {
    AddGuildMember {
        guild_id: u64,
        user_id: u64,
    },
    AddMemberRole {
        guild_id: u64,
        role_id: u64,
        user_id: u64,
    },
    GuildBanUser {
        guild_id: u64,
        user_id: u64,
        delete_message_days: Option<u8>,
    },
    BroadcastTyping {
        channel_id: u64,
    },
    CreateChannel {
        guild_id: u64,
    },
    CreateStageInstance,
    CreatePublicThread {
        channel_id: u64,
        message_id: u64,
    },
    CreatePrivateThread {
        channel_id: u64,
    },
    CreateEmoji {
        guild_id: u64,
    },
    CreateFollowupMessage {
        application_id: u64,
        interaction_token: &'a str,
    },
    CreateGlobalApplicationCommand {
        application_id: u64,
    },
    CreateGlobalApplicationCommands {
        application_id: u64,
    },
    CreateGuild,
    CreateGuildApplicationCommand {
        application_id: u64,
        guild_id: u64,
    },
    CreateGuildApplicationCommands {
        application_id: u64,
        guild_id: u64,
    },
    CreateGuildIntegration {
        guild_id: u64,
        integration_id: u64,
    },
    CreateInteractionResponse {
        interaction_id: u64,
        interaction_token: &'a str,
    },
    CreateInvite {
        channel_id: u64,
    },
    CreateMessage {
        channel_id: u64,
    },
    CreatePermission {
        channel_id: u64,
        target_id: u64,
    },
    CreatePrivateChannel,
    CreateReaction {
        channel_id: u64,
        message_id: u64,
        reaction: &'a str,
    },
    CreateRole {
        guild_id: u64,
    },
    CreateScheduledEvent {
        guild_id: u64,
    },
    CreateSticker {
        guild_id: u64,
    },
    CreateWebhook {
        channel_id: u64,
    },
    DeleteChannel {
        channel_id: u64,
    },
    DeleteStageInstance {
        channel_id: u64,
    },
    DeleteEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    DeleteFollowupMessage {
        application_id: u64,
        interaction_token: &'a str,
        message_id: u64,
    },
    DeleteGlobalApplicationCommand {
        application_id: u64,
        command_id: u64,
    },
    DeleteGuild {
        guild_id: u64,
    },
    DeleteGuildApplicationCommand {
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    },
    DeleteGuildIntegration {
        guild_id: u64,
        integration_id: u64,
    },
    DeleteInvite {
        code: &'a str,
    },
    DeleteMessage {
        channel_id: u64,
        message_id: u64,
    },
    DeleteMessages {
        channel_id: u64,
    },
    DeleteMessageReactions {
        channel_id: u64,
        message_id: u64,
    },
    DeleteMessageReactionEmoji {
        channel_id: u64,
        message_id: u64,
        reaction: &'a str,
    },
    DeleteOriginalInteractionResponse {
        application_id: u64,
        interaction_token: &'a str,
    },
    DeletePermission {
        channel_id: u64,
        target_id: u64,
    },
    DeleteReaction {
        channel_id: u64,
        message_id: u64,
        user: &'a str,
        reaction: &'a str,
    },
    DeleteRole {
        guild_id: u64,
        role_id: u64,
    },
    DeleteScheduledEvent {
        guild_id: u64,
        event_id: u64,
    },
    DeleteSticker {
        guild_id: u64,
        sticker_id: u64,
    },
    DeleteWebhook {
        webhook_id: u64,
    },
    DeleteWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
    },
    DeleteWebhookMessage {
        token: &'a str,
        webhook_id: u64,
        message_id: u64,
    },
    EditChannel {
        channel_id: u64,
    },
    EditStageInstance {
        channel_id: u64,
    },
    EditEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    EditFollowupMessage {
        application_id: u64,
        interaction_token: &'a str,
        message_id: u64,
    },
    EditGlobalApplicationCommand {
        application_id: u64,
        command_id: u64,
    },
    EditGuild {
        guild_id: u64,
    },
    EditGuildApplicationCommand {
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    },
    EditGuildApplicationCommandPermission {
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    },
    EditGuildApplicationCommandsPermissions {
        application_id: u64,
        guild_id: u64,
    },
    EditGuildChannels {
        guild_id: u64,
    },
    EditGuildWidget {
        guild_id: u64,
    },
    EditGuildWelcomeScreen {
        guild_id: u64,
    },
    EditMember {
        guild_id: u64,
        user_id: u64,
    },
    EditMessage {
        channel_id: u64,
        message_id: u64,
    },
    CrosspostMessage {
        channel_id: u64,
        message_id: u64,
    },
    EditMemberMe {
        guild_id: u64,
    },
    EditNickname {
        guild_id: u64,
    },
    GetOriginalInteractionResponse {
        application_id: u64,
        interaction_token: &'a str,
    },
    EditOriginalInteractionResponse {
        application_id: u64,
        interaction_token: &'a str,
    },
    EditProfile,
    EditRole {
        guild_id: u64,
        role_id: u64,
    },
    EditRolePosition {
        guild_id: u64,
    },
    EditScheduledEvent {
        guild_id: u64,
        event_id: u64,
    },
    EditSticker {
        guild_id: u64,
        sticker_id: u64,
    },
    EditThread {
        channel_id: u64,
    },
    EditVoiceState {
        guild_id: u64,
        user_id: u64,
    },
    EditVoiceStateMe {
        guild_id: u64,
    },
    EditWebhook {
        webhook_id: u64,
    },
    EditWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
    },
    EditWebhookMessage {
        token: &'a str,
        webhook_id: u64,
        message_id: u64,
    },
    ExecuteWebhook {
        token: &'a str,
        wait: bool,
        webhook_id: u64,
    },
    JoinThread {
        channel_id: u64,
    },
    LeaveThread {
        channel_id: u64,
    },
    AddThreadMember {
        channel_id: u64,
        user_id: u64,
    },
    RemoveThreadMember {
        channel_id: u64,
        user_id: u64,
    },
    GetActiveMaintenance,
    GetAuditLogs {
        action_type: Option<u8>,
        before: Option<u64>,
        guild_id: u64,
        limit: Option<u8>,
        user_id: Option<u64>,
    },
    GetBans {
        guild_id: u64,
    },
    GetBotGateway,
    GetChannel {
        channel_id: u64,
    },
    GetChannelInvites {
        channel_id: u64,
    },
    GetChannelWebhooks {
        channel_id: u64,
    },
    GetChannels {
        guild_id: u64,
    },
    GetStageInstance {
        channel_id: u64,
    },
    GetChannelThreadMembers {
        channel_id: u64,
    },
    GetChannelArchivedPublicThreads {
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetChannelArchivedPrivateThreads {
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetChannelJoinedPrivateArchivedThreads {
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u64>,
    },
    GetCurrentApplicationInfo,
    GetCurrentUser,
    GetEmojis {
        guild_id: u64,
    },
    GetEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    GetFollowupMessage {
        application_id: u64,
        interaction_token: &'a str,
        message_id: u64,
    },
    GetGateway,
    GetGlobalApplicationCommands {
        application_id: u64,
    },
    GetGlobalApplicationCommand {
        application_id: u64,
        command_id: u64,
    },
    GetGuild {
        guild_id: u64,
    },
    GetGuildWithCounts {
        guild_id: u64,
    },
    GetGuildApplicationCommands {
        application_id: u64,
        guild_id: u64,
    },
    GetGuildApplicationCommand {
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    },
    GetGuildApplicationCommandsPermissions {
        application_id: u64,
        guild_id: u64,
    },
    GetGuildApplicationCommandPermissions {
        application_id: u64,
        guild_id: u64,
        command_id: u64,
    },
    GetGuildWidget {
        guild_id: u64,
    },
    GetGuildActiveThreads {
        guild_id: u64,
    },
    GetGuildPreview {
        guild_id: u64,
    },
    GetGuildWelcomeScreen {
        guild_id: u64,
    },
    GetGuildIntegrations {
        guild_id: u64,
    },
    GetGuildInvites {
        guild_id: u64,
    },
    GetGuildMembers {
        after: Option<u64>,
        limit: Option<u64>,
        guild_id: u64,
    },
    GetGuildPruneCount {
        days: u64,
        guild_id: u64,
    },
    GetGuildRegions {
        guild_id: u64,
    },
    GetGuildRoles {
        guild_id: u64,
    },
    GetScheduledEvent {
        guild_id: u64,
        event_id: u64,
        with_user_count: bool,
    },
    GetScheduledEvents {
        guild_id: u64,
        with_user_count: bool,
    },
    GetScheduledEventUsers {
        guild_id: u64,
        event_id: u64,
        after: Option<u64>,
        before: Option<u64>,
        limit: Option<u64>,
        with_member: Option<bool>,
    },
    GetGuildStickers {
        guild_id: u64,
    },
    GetGuildVanityUrl {
        guild_id: u64,
    },
    GetGuildWebhooks {
        guild_id: u64,
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
        event_id: Option<u64>,
    },
    GetMember {
        guild_id: u64,
        user_id: u64,
    },
    GetMessage {
        channel_id: u64,
        message_id: u64,
    },
    GetMessages {
        channel_id: u64,
        query: String,
    },
    GetPins {
        channel_id: u64,
    },
    GetReactionUsers {
        after: Option<u64>,
        channel_id: u64,
        limit: u8,
        message_id: u64,
        reaction: String,
    },
    GetSticker {
        sticker_id: u64,
    },
    GetStickerPacks,
    GetGuildSticker {
        guild_id: u64,
        sticker_id: u64,
    },
    GetUnresolvedIncidents,
    GetUpcomingMaintenances,
    GetUser {
        user_id: u64,
    },
    GetUserConnections,
    GetUserDmChannels,
    GetVoiceRegions,
    GetWebhook {
        webhook_id: u64,
    },
    GetWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
    },
    GetWebhookMessage {
        token: &'a str,
        webhook_id: u64,
        message_id: u64,
    },
    KickMember {
        guild_id: u64,
        user_id: u64,
    },
    LeaveGroup {
        group_id: u64,
    },
    LeaveGuild {
        guild_id: u64,
    },
    PinMessage {
        channel_id: u64,
        message_id: u64,
    },
    RemoveBan {
        guild_id: u64,
        user_id: u64,
    },
    RemoveMemberRole {
        guild_id: u64,
        role_id: u64,
        user_id: u64,
    },
    SearchGuildMembers {
        guild_id: u64,
        query: &'a str,
        limit: Option<u64>,
    },
    StartGuildPrune {
        days: u64,
        guild_id: u64,
    },
    StartIntegrationSync {
        guild_id: u64,
        integration_id: u64,
    },
    StatusIncidentsUnresolved,
    StatusMaintenancesActive,
    StatusMaintenancesUpcoming,
    UnpinMessage {
        channel_id: u64,
        message_id: u64,
    },
}

impl<'a> RouteInfo<'a> {
    #[must_use]
    pub fn deconstruct(&self) -> (LightMethod, Route, Cow<'_, str>) {
        match *self {
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
                Route::WebhooksId(application_id),
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
                Route::ChannelsIdMessagesId(LightMethod::Delete, message_id),
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
            } => (
                LightMethod::Post,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token_optioned(webhook_id, token, wait)),
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
            } => (
                LightMethod::Get,
                Route::ApplicationsIdCommands(application_id),
                Cow::from(Route::application_commands(application_id)),
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
            } => (
                LightMethod::Get,
                Route::ApplicationsIdGuildsIdCommands(application_id),
                Cow::from(Route::application_guild_commands(application_id, guild_id)),
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
                ref query,
            } => (
                LightMethod::Get,
                Route::ChannelsIdMessages(channel_id),
                Cow::from(Route::channel_messages(channel_id, Some(query.as_ref()))),
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
                ref reaction,
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
            RouteInfo::LeaveGroup {
                group_id,
            } => (
                LightMethod::Delete,
                Route::ChannelsId(group_id),
                Cow::from(Route::channel(group_id)),
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
