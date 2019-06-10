use std::{
    borrow::Cow,
    fmt::{Display, Write},
};
use super::LightMethod;

/// A representation of all routes registered within the library. These are safe
/// and memory-efficient representations of each path that functions exist for
/// in the [`http`] module.
///
/// [`http`]: ../index.html
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Route {
    /// Route for the `/channels/:channel_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsId(u64),
    /// Route for the `/channels/:channel_id/invites` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdInvites(u64),
    /// Route for the `/channels/:channel_id/messages` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdMessages(u64),
    /// Route for the `/channels/:channel_id/messages/bulk-delete` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdMessagesBulkDelete(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    // This route is a unique case. The ratelimit for message _deletions_ is
    // different than the overall route ratelimit.
    //
    // Refer to the docs on [Rate Limits] in the yellow warning section.
    //
    // Additionally, this needs to be a `LightMethod` from the parent module
    // and _not_ a `reqwest` `Method` due to `reqwest`'s not deriving `Copy`.
    //
    // [Rate Limits]: https://discordapp.com/developers/docs/topics/rate-limits
    ChannelsIdMessagesId(LightMethod, u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/ack` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdMessagesIdAck(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/reactions`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdMessagesIdReactions(u64),
    /// Route for the
    /// `/channels/:channel_id/messages/:message_id/reactions/:reaction/@me`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdMessagesIdReactionsUserIdType(u64),
    /// Route for the `/channels/:channel_id/permissions/:target_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdPermissionsOverwriteId(u64),
    /// Route for the `/channels/:channel_id/pins` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdPins(u64),
    /// Route for the `/channels/:channel_id/pins/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdPinsMessageId(u64),
    /// Route for the `/channels/:channel_id/typing` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdTyping(u64),
    /// Route for the `/channels/:channel_id/webhooks` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/id/struct.ChannelId.html
    ChannelsIdWebhooks(u64),
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
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsId(u64),
    /// Route for the `/guilds/:guild_id/bans` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdBans(u64),
    /// Route for the `/guilds/:guild_id/audit-logs` path.
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdAuditLogs(u64),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdBansUserId(u64),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdChannels(u64),
    /// Route for the `/guilds/:guild_id/embed` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdEmbed(u64),
    /// Route for the `/guilds/:guild_id/emojis` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdEmojis(u64),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdEmojisId(u64),
    /// Route for the `/guilds/:guild_id/integrations` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdIntegrations(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdIntegrationsId(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdIntegrationsIdSync(u64),
    /// Route for the `/guilds/:guild_id/invites` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdInvites(u64),
    /// Route for the `/guilds/:guild_id/members` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdMembers(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdMembersId(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdMembersIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdMembersMeNick(u64),
    /// Route for the `/guilds/:guild_id/prune` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdPrune(u64),
    /// Route for the `/guilds/:guild_id/regions` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdRegions(u64),
    /// Route for the `/guilds/:guild_id/roles` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdRoles(u64),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/vanity-url` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdVanityUrl(u64),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: ../../model/id/struct.GuildId.html
    GuildsIdWebhooks(u64),
    /// Route for the `/invites/:code` path.
    InvitesCode,
    /// Route for the `/users/:user_id` path.
    UsersId,
    /// Route for the `/users/@me` path.
    UsersMe,
    /// Route for the `/users/@me/channels` path.
    UsersMeChannels,
    /// Route for the `/users/@me/guilds` path.
    UsersMeGuilds,
    /// Route for the `/users/@me/guilds/:guild_id` path.
    UsersMeGuildsId,
    /// Route for the `/voice/regions` path.
    VoiceRegions,
    /// Route for the `/webhooks/:webhook_id` path.
    WebhooksId(u64),
    /// Route where no ratelimit headers are in place (i.e. user account-only
    /// routes).
    ///
    /// This is a special case, in that if the route is `None` then pre- and
    /// post-hooks are not executed.
    None,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Route {
    pub fn channel(channel_id: u64) -> String {
        format!(api!("/channels/{}"), channel_id)
    }

    pub fn channel_invites(channel_id: u64) -> String {
        format!(api!("/channels/{}/invites"), channel_id)
    }

    pub fn channel_message(channel_id: u64, message_id: u64) -> String {
        format!(api!("/channels/{}/messages/{}"), channel_id, message_id)
    }

    pub fn channel_message_reaction<D, T>(
        channel_id: u64,
        message_id: u64,
        user_id: D,
        reaction_type: T
    ) -> String where D: Display, T: Display {
        format!(
            api!("/channels/{}/messages/{}/reactions/{}/{}"),
            channel_id,
            message_id,
            reaction_type,
            user_id,
        )
    }

    pub fn channel_message_reactions(
        channel_id: u64,
        message_id: u64,
    ) -> String {
        api!("/channels/{}/messages/{}/reactions", channel_id, message_id)
    }

    pub fn channel_message_reactions_list(
        channel_id: u64,
        message_id: u64,
        reaction: &str,
        limit: u8,
        after: Option<u64>,
    ) -> String {
        let mut uri = format!(
            api!("/channels/{}/messages/{}/reactions/{}?limit={}"),
            channel_id,
            message_id,
            reaction,
            limit,
        );

        if let Some(after) = after {
            let _ = write!(uri, "&after={}", after);
        }

        uri
    }

    pub fn channel_messages(channel_id: u64, query: Option<&str>) -> String {
        format!(
            api!("/channels/{}/messages{}"),
            channel_id,
            query.unwrap_or(""),
        )
    }

    pub fn channel_messages_bulk_delete(channel_id: u64) -> String {
        format!(api!("/channels/{}/messages/bulk-delete"), channel_id)
    }

    pub fn channel_permission(channel_id: u64, target_id: u64) -> String {
        format!(api!("/channels/{}/permissions/{}"), channel_id, target_id)
    }

    pub fn channel_pin(channel_id: u64, message_id: u64) -> String {
        format!(api!("/channels/{}/pins/{}"), channel_id, message_id)
    }

    pub fn channel_pins(channel_id: u64) -> String {
        format!(api!("/channels/{}/pins"), channel_id)
    }

    pub fn channel_typing(channel_id: u64) -> String {
        format!(api!("/channels/{}/typing"), channel_id)
    }

    pub fn channel_webhooks(channel_id: u64) -> String {
        format!(api!("/channels/{}/webhooks"), channel_id)
    }

    pub fn gateway() -> &'static str {
        api!("/gateway")
    }

    pub fn gateway_bot() -> &'static str {
        api!("/gateway/bot")
    }

    pub fn group_recipient(group_id: u64, user_id: u64) -> String {
        format!(api!("/channels/{}/recipients/{}"), group_id, user_id)
    }

    pub fn guild(guild_id: u64) -> String {
        format!(api!("/guilds/{}"), guild_id)
    }

    pub fn guild_audit_logs(
        guild_id: u64,
        action_type: Option<u8>,
        user_id: Option<u64>,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> String {
        let mut s = format!(
            api!("/guilds/{}/audit-logs?"),
            guild_id,
        );

        if let Some(action_type) = action_type {
            let _ = write!(s, "&action_type={}", action_type);
        }

        if let Some(before) = before {
            let _ = write!(s, "&before={}", before);
        }

        if let Some(limit) = limit {
            let _ = write!(s, "&limit={}", limit);
        }

        if let Some(user_id) = user_id {
            let _ = write!(s, "&user_id={}", user_id);
        }

        s
    }

    pub fn guild_ban(guild_id: u64, user_id: u64) -> String {
        format!(api!("/guilds/{}/bans/{}"), guild_id, user_id)
    }

    pub fn guild_ban_optioned(
        guild_id: u64,
        user_id: u64,
        delete_message_days: u8,
        reason: &str,
    ) -> String {
        format!(
            api!("/guilds/{}/bans/{}?delete-message-days={}&reason={}"),
            guild_id,
            user_id,
            delete_message_days,
            reason,
        )
    }

    pub fn guild_bans(guild_id: u64) -> String {
        format!(api!("/guilds/{}/bans"), guild_id)
    }

    pub fn guild_channels(guild_id: u64) -> String {
        format!(api!("/guilds/{}/channels"), guild_id)
    }

    pub fn guild_embed(guild_id: u64) -> String {
        format!(api!("/guilds/{}/embed"), guild_id)
    }

    pub fn guild_emojis(guild_id: u64) -> String {
        format!(api!("/guilds/{}/emojis"), guild_id)
    }

    pub fn guild_emoji(guild_id: u64, emoji_id: u64) -> String {
        format!(api!("/guilds/{}/emojis/{}"), guild_id, emoji_id)
    }

    pub fn guild_integration(
        guild_id: u64,
        integration_id: u64,
    ) -> String {
        format!(api!("/guilds/{}/integrations/{}"), guild_id, integration_id)
    }

    pub fn guild_integration_sync(
        guild_id: u64,
        integration_id: u64,
    ) -> String {
        format!(
            api!("/guilds/{}/integrations/{}/sync"),
            guild_id,
            integration_id,
        )
    }

    pub fn guild_integrations(guild_id: u64) -> String {
        format!(api!("/guilds/{}/integrations"), guild_id)
    }

    pub fn guild_invites(guild_id: u64) -> String {
        format!(api!("/guilds/{}/invites"), guild_id)
    }

    pub fn guild_member(guild_id: u64, user_id: u64) -> String {
        format!(api!("/guilds/{}/members/{}"), guild_id, user_id)
    }

    pub fn guild_member_role(
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> String {
        format!(
            api!("/guilds/{}/members/{}/roles/{}"),
            guild_id,
            user_id,
            role_id,
        )
    }

    pub fn guild_members(guild_id: u64) -> String {
        format!(api!("/guilds/{}/members"), guild_id)
    }

    pub fn guild_members_optioned(
        guild_id: u64,
        after: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = format!(api!("/guilds/{}/members?"), guild_id);

        if let Some(after) = after {
            let _ = write!(s, "&after={}", after);
        }

        if let Some(limit) = limit {
            let _ = write!(s, "&limit={}", limit);
        }

        s
    }

    pub fn guild_nickname(guild_id: u64) -> String {
        format!(api!("/guilds/{}/members/@me/nick"), guild_id)
    }

    pub fn guild_prune(guild_id: u64, days: u64) -> String {
        format!(api!("/guilds/{}/prune?days={}"), guild_id, days)
    }

    pub fn guild_regions(guild_id: u64) -> String {
        format!(api!("/guilds/{}/regions"), guild_id)
    }

    pub fn guild_role(guild_id: u64, role_id: u64) -> String {
        format!(api!("/guilds/{}/roles/{}"), guild_id, role_id)
    }

    pub fn guild_roles(guild_id: u64) -> String {
        format!(api!("/guilds/{}/roles"), guild_id)
    }

    pub fn guild_vanity_url(guild_id: u64) -> String {
        format!(api!("/guilds/{}/vanity-url"), guild_id)
    }

    pub fn guild_webhooks(guild_id: u64) -> String {
        format!(api!("/guilds/{}/webhooks"), guild_id)
    }

    pub fn guilds() -> &'static str {
        api!("/guilds")
    }

    pub fn invite(code: &str) -> String {
        format!(api!("/invites/{}"), code)
    }

    pub fn invite_optioned(code: &str, stats: bool) -> String {
        format!(api!("/invites/{}?with_counts={}"), code, stats)
    }

    pub fn oauth2_application_current() -> &'static str {
        api!("/oauth2/applications/@me")
    }

    pub fn private_channel() -> &'static str {
        api!("/users/@me/channels")
    }

    pub fn status_incidents_unresolved() -> &'static str {
        status!("/incidents/unresolved.json")
    }

    pub fn status_maintenances_active() -> &'static str {
        status!("/scheduled-maintenances/active.json")
    }

    pub fn status_maintenances_upcoming() -> &'static str {
        status!("/scheduled-maintenances/upcoming.json")
    }

    pub fn user<D: Display>(target: D) -> String {
        format!(api!("/users/{}"), target)
    }

    pub fn user_dm_channels<D: Display>(target: D) -> String {
        format!(api!("/users/{}/channels"), target)
    }

    pub fn user_guild<D: Display>(target: D, guild_id: u64) -> String {
        format!(api!("/users/{}/guilds/{}"), target, guild_id)
    }

    pub fn user_guilds<D: Display>(target: D) -> String {
        format!(api!("/users/{}/guilds"), target)
    }

    pub fn user_guilds_optioned<D: Display>(
        target: D,
        after: Option<u64>,
        before: Option<u64>,
        limit: u64,
    ) -> String {
        let mut s = format!(api!("/users/{}/guilds?limit={}&"), target, limit);

        if let Some(after) = after {
            let _ = write!(s, "&after={}", after);
        }

        if let Some(before) = before {
            let _ = write!(s, "&before={}", before);
        }

        s
    }

    pub fn voice_regions() -> &'static str {
        api!("/voice/regions")
    }

    pub fn webhook(webhook_id: u64) -> String {
        format!(api!("/webhooks/{}"), webhook_id)
    }

    pub fn webhook_with_token<D>(webhook_id: u64, token: D) -> String
        where D: Display {
        format!(api!("/webhooks/{}/{}"), webhook_id, token)
    }

    pub fn webhook_with_token_optioned<D>(webhook_id: u64, token: D, wait: bool)
        -> String where D: Display {
        format!(api!("/webhooks/{}/{}?wait={}"), webhook_id, token, wait)
    }
}

#[derive(Clone, Debug)]
pub enum RouteInfo<'a> {
    AddGroupRecipient {
        group_id: u64,
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
        reason: Option<&'a str>,
    },
    BroadcastTyping {
        channel_id: u64,
    },
    CreateChannel {
        guild_id: u64,
    },
    CreateEmoji {
        guild_id: u64,
    },
    CreateGuild,
    CreateGuildIntegration {
        guild_id: u64,
        integration_id: u64,
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
    CreateWebhook {
        channel_id: u64,
    },
    DeleteChannel {
        channel_id: u64,
    },
    DeleteEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    DeleteGuild {
        guild_id: u64,
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
    DeleteWebhook {
        webhook_id: u64,
    },
    DeleteWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
    },
    EditChannel {
        channel_id: u64,
    },
    EditEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    EditGuild {
        guild_id: u64,
    },
    EditGuildChannels {
        guild_id: u64,
    },
    EditGuildEmbed {
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
    EditNickname {
        guild_id: u64,
    },
    EditProfile,
    EditRole {
        guild_id: u64,
        role_id: u64,
    },
    EditRolePosition {
        guild_id: u64,
    },
    EditWebhook {
        webhook_id: u64,
    },
    EditWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
    },
    ExecuteWebhook {
        token: &'a str,
        wait: bool,
        webhook_id: u64,
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
    GetCurrentApplicationInfo,
    GetCurrentUser,
    GetGateway,
    GetGuild {
        guild_id: u64,
    },
    GetGuildEmbed {
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
    GetGuildVanityUrl {
        guild_id: u64,
    },
    GetGuildWebhooks {
        guild_id: u64,
    },
    GetGuilds {
        after: Option<u64>,
        before: Option<u64>,
        limit: u64,
    },
    GetInvite {
        code: &'a str,
        stats: bool,
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
    GetUnresolvedIncidents,
    GetUpcomingMaintenances,
    GetUser {
        user_id: u64,
    },
    GetUserDmChannels,
    GetVoiceRegions,
    GetWebhook {
        webhook_id: u64,
    },
    GetWebhookWithToken {
        token: &'a str,
        webhook_id: u64,
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
    RemoveGroupRecipient {
        group_id: u64,
        user_id: u64,
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
    #[doc(hidden)]
    __Nonexhaustive,
}

impl<'a> RouteInfo<'a> {
    pub fn deconstruct(&self) -> (LightMethod, Route, Cow<'_, str>) {
        match *self {
            RouteInfo::AddGroupRecipient { group_id, user_id } => (
                LightMethod::Put,
                Route::None,
                Cow::from(Route::group_recipient(group_id, user_id)),
            ),
            RouteInfo::AddMemberRole { guild_id, role_id, user_id } => (
                LightMethod::Put,
                Route::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Route::guild_member_role(guild_id, user_id, role_id)),
            ),
            RouteInfo::GuildBanUser {
                guild_id,
                delete_message_days,
                reason,
                user_id,
            } => (
                // TODO
                LightMethod::Put,
                Route::GuildsIdBansUserId(guild_id),
                Cow::from(Route::guild_ban_optioned(
                    guild_id,
                    user_id,
                    delete_message_days.unwrap_or(0),
                    reason.unwrap_or(""),
                )),
            ),
            RouteInfo::BroadcastTyping { channel_id } => (
                LightMethod::Post,
                Route::ChannelsIdTyping(channel_id),
                Cow::from(Route::channel_typing(channel_id)),
            ),
            RouteInfo::CreateChannel { guild_id } => (
                LightMethod::Post,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::CreateEmoji { guild_id } => (
                LightMethod::Post,
                Route::GuildsIdEmojis(guild_id),
                Cow::from(Route::guild_emojis(guild_id)),
            ),
            RouteInfo::CreateGuild => (
                LightMethod::Post,
                Route::Guilds,
                Cow::from(Route::guilds()),
            ),
            RouteInfo::CreateGuildIntegration { guild_id, integration_id } => (
                LightMethod::Post,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration(guild_id, integration_id)),
            ),
            RouteInfo::CreateInvite { channel_id } => (
                LightMethod::Post,
                Route::ChannelsIdInvites(channel_id),
                Cow::from(Route::channel_invites(channel_id)),
            ),
            RouteInfo::CreateMessage { channel_id } => (
                LightMethod::Post,
                Route::ChannelsIdMessages(channel_id),
                Cow::from(Route::channel_messages(channel_id, None)),
            ),
            RouteInfo::CreatePermission { channel_id, target_id } => (
                LightMethod::Put,
                Route::ChannelsIdPermissionsOverwriteId(channel_id),
                Cow::from(Route::channel_permission(channel_id, target_id)),
            ),
            RouteInfo::CreatePrivateChannel => (
                LightMethod::Post,
                Route::UsersMeChannels,
                Cow::from(Route::user_dm_channels("@me")),
            ),
            RouteInfo::CreateReaction { channel_id, message_id, reaction } => (
                LightMethod::Put,
                Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                Cow::from(Route::channel_message_reaction(
                    channel_id,
                    message_id,
                    "@me",
                    reaction,
                )),
            ),
            RouteInfo::CreateRole { guild_id } => (
                LightMethod::Post,
                Route::GuildsIdRoles(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::CreateWebhook { channel_id } => (
                LightMethod::Post,
                Route::ChannelsIdWebhooks(channel_id),
                Cow::from(Route::channel_webhooks(channel_id)),
            ),
            RouteInfo::DeleteChannel { channel_id } => (
                LightMethod::Delete,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::DeleteEmoji { emoji_id, guild_id } => (
                LightMethod::Delete,
                Route::GuildsIdEmojisId(guild_id),
                Cow::from(Route::guild_emoji(guild_id, emoji_id)),
            ),
            RouteInfo::DeleteGuild { guild_id } => (
                LightMethod::Delete,
                Route::GuildsId(guild_id),
                Cow::from(Route::guild(guild_id)),
            ),
            RouteInfo::DeleteGuildIntegration { guild_id, integration_id } => (
                LightMethod::Delete,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration(guild_id, integration_id)),
            ),
            RouteInfo::DeleteInvite { code } => (
                LightMethod::Delete,
                Route::InvitesCode,
                Cow::from(Route::invite(code)),
            ),
            RouteInfo::DeleteMessageReactions { channel_id, message_id } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesIdReactions(channel_id),
                Cow::from(Route::channel_message_reactions(
                    channel_id,
                    message_id,
                )),
            ),
            RouteInfo::DeleteMessage { channel_id, message_id } => (
                LightMethod::Delete,
                Route::ChannelsIdMessagesId(LightMethod::Delete, message_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::DeleteMessages { channel_id } => (
                LightMethod::Post,
                Route::ChannelsIdMessagesBulkDelete(channel_id),
                Cow::from(Route::channel_messages_bulk_delete(channel_id)),
            ),
            RouteInfo::DeletePermission { channel_id, target_id } => (
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
                Cow::from(Route::channel_message_reaction(
                    channel_id,
                    message_id,
                    user,
                    reaction,
                ))
            ),
            RouteInfo::DeleteRole { guild_id, role_id } => (
                LightMethod::Delete,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_role(guild_id, role_id)),
            ),
            RouteInfo::DeleteWebhook { webhook_id } => (
                LightMethod::Delete,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::DeleteWebhookWithToken { token, webhook_id } => (
                LightMethod::Delete,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::EditChannel { channel_id } => (
                LightMethod::Patch,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::EditEmoji { emoji_id, guild_id } => (
                LightMethod::Patch,
                Route::GuildsIdEmojisId(guild_id),
                Cow::from(Route::guild_emoji(guild_id, emoji_id)),
            ),
            RouteInfo::EditGuild { guild_id } => (
                LightMethod::Patch,
                Route::GuildsId(guild_id),
                Cow::from(Route::guild(guild_id)),
            ),
            RouteInfo::EditGuildChannels { guild_id } => (
                LightMethod::Patch,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::EditGuildEmbed { guild_id } => (
                LightMethod::Patch,
                Route::GuildsIdEmbed(guild_id),
                Cow::from(Route::guild_embed(guild_id)),
            ),
            RouteInfo::EditMember { guild_id, user_id } => (
                LightMethod::Patch,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::EditMessage { channel_id, message_id } => (
                LightMethod::Patch,
                Route::ChannelsIdMessagesId(LightMethod::Patch, channel_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::EditNickname { guild_id } => (
                LightMethod::Patch,
                Route::GuildsIdMembersMeNick(guild_id),
                Cow::from(Route::guild_nickname(guild_id)),
            ),
            RouteInfo::EditProfile => (
                LightMethod::Patch,
                Route::UsersMe,
                Cow::from(Route::user("@me")),
            ),
            RouteInfo::EditRole { guild_id, role_id } => (
                LightMethod::Patch,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_role(guild_id, role_id)),
            ),
            RouteInfo::EditRolePosition { guild_id } => (
                LightMethod::Patch,
                Route::GuildsIdRolesId(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::EditWebhook { webhook_id } => (
                LightMethod::Patch,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::EditWebhookWithToken { token, webhook_id } => (
                LightMethod::Patch,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::ExecuteWebhook { token, wait, webhook_id } => (
                LightMethod::Post,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token_optioned(
                    webhook_id,
                    token,
                    wait,
                )),
            ),
            RouteInfo::GetActiveMaintenance => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_maintenances_active()),
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
                Cow::from(Route::guild_audit_logs(
                    guild_id,
                    action_type,
                    user_id,
                    before,
                    limit,
                )),
            ),
            RouteInfo::GetBans { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdBans(guild_id),
                Cow::from(Route::guild_bans(guild_id)),
            ),
            RouteInfo::GetBotGateway => (
                LightMethod::Get,
                Route::GatewayBot,
                Cow::from(Route::gateway_bot()),
            ),
            RouteInfo::GetChannel { channel_id } => (
                LightMethod::Get,
                Route::ChannelsId(channel_id),
                Cow::from(Route::channel(channel_id)),
            ),
            RouteInfo::GetChannelInvites { channel_id } => (
                LightMethod::Get,
                Route::ChannelsIdInvites(channel_id),
                Cow::from(Route::channel_invites(channel_id)),
            ),
            RouteInfo::GetChannelWebhooks { channel_id } => (
                LightMethod::Get,
                Route::ChannelsIdWebhooks(channel_id),
                Cow::from(Route::channel_webhooks(channel_id)),
            ),
            RouteInfo::GetChannels { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdChannels(guild_id),
                Cow::from(Route::guild_channels(guild_id)),
            ),
            RouteInfo::GetCurrentApplicationInfo => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::oauth2_application_current()),
            ),
            RouteInfo::GetCurrentUser => (
                LightMethod::Get,
                Route::UsersMe,
                Cow::from(Route::user("@me")),
            ),
            RouteInfo::GetGateway => (
                LightMethod::Get,
                Route::Gateway,
                Cow::from(Route::gateway()),
            ),
            RouteInfo::GetGuild { guild_id } => (
                LightMethod::Get,
                Route::GuildsId(guild_id),
                Cow::from(Route::guild(guild_id)),
            ),
            RouteInfo::GetGuildEmbed { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdEmbed(guild_id),
                Cow::from(Route::guild_embed(guild_id)),
            ),
            RouteInfo::GetGuildIntegrations { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdIntegrations(guild_id),
                Cow::from(Route::guild_integrations(guild_id)),
            ),
            RouteInfo::GetGuildInvites { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdInvites(guild_id),
                Cow::from(Route::guild_invites(guild_id)),
            ),
            RouteInfo::GetGuildMembers { after, guild_id, limit } => (
                LightMethod::Get,
                Route::GuildsIdMembers(guild_id),
                Cow::from(Route::guild_members_optioned(guild_id, after, limit)),
            ),
            RouteInfo::GetGuildPruneCount { days, guild_id } => (
                LightMethod::Get,
                Route::GuildsIdPrune(guild_id),
                Cow::from(Route::guild_prune(guild_id, days)),
            ),
            RouteInfo::GetGuildRegions { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdRegions(guild_id),
                Cow::from(Route::guild_regions(guild_id)),
            ),
            RouteInfo::GetGuildRoles { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdRoles(guild_id),
                Cow::from(Route::guild_roles(guild_id)),
            ),
            RouteInfo::GetGuildVanityUrl { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdVanityUrl(guild_id),
                Cow::from(Route::guild_vanity_url(guild_id)),
            ),
            RouteInfo::GetGuildWebhooks { guild_id } => (
                LightMethod::Get,
                Route::GuildsIdWebhooks(guild_id),
                Cow::from(Route::guild_webhooks(guild_id)),
            ),
            RouteInfo::GetGuilds { after, before, limit } => (
                LightMethod::Get,
                Route::UsersMeGuilds,
                Cow::from(Route::user_guilds_optioned(
                    "@me",
                    after,
                    before,
                    limit,
                )),
            ),
            RouteInfo::GetInvite { code, stats } => (
                LightMethod::Get,
                Route::InvitesCode,
                Cow::from(Route::invite_optioned(code, stats)),
            ),
            RouteInfo::GetMember { guild_id, user_id } => (
                LightMethod::Get,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::GetMessage { channel_id, message_id } => (
                LightMethod::Get,
                Route::ChannelsIdMessagesId(LightMethod::Get, channel_id),
                Cow::from(Route::channel_message(channel_id, message_id)),
            ),
            RouteInfo::GetMessages { channel_id, ref query } => (
                LightMethod::Get,
                Route::ChannelsIdMessages(channel_id),
                Cow::from(Route::channel_messages(
                    channel_id,
                    Some(query.as_ref()),
                )),
            ),
            RouteInfo::GetPins { channel_id } => (
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
                    channel_id,
                    message_id,
                    reaction,
                    limit,
                    after,
                )),
            ),
            RouteInfo::GetUnresolvedIncidents => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_incidents_unresolved()),
            ),
            RouteInfo::GetUpcomingMaintenances => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_maintenances_upcoming()),
            ),
            RouteInfo::GetUser { user_id } => (
                LightMethod::Get,
                Route::UsersId,
                Cow::from(Route::user(user_id)),
            ),
            RouteInfo::GetUserDmChannels => (
                LightMethod::Get,
                Route::UsersMeChannels,
                Cow::from(Route::user_dm_channels("@me")),
            ),
            RouteInfo::GetVoiceRegions => (
                LightMethod::Get,
                Route::VoiceRegions,
                Cow::from(Route::voice_regions()),
            ),
            RouteInfo::GetWebhook { webhook_id } => (
                LightMethod::Get,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook(webhook_id)),
            ),
            RouteInfo::GetWebhookWithToken { token, webhook_id } => (
                LightMethod::Get,
                Route::WebhooksId(webhook_id),
                Cow::from(Route::webhook_with_token(webhook_id, token)),
            ),
            RouteInfo::KickMember { guild_id, user_id } => (
                LightMethod::Delete,
                Route::GuildsIdMembersId(guild_id),
                Cow::from(Route::guild_member(guild_id, user_id)),
            ),
            RouteInfo::LeaveGroup { group_id } => (
                LightMethod::Delete,
                Route::ChannelsId(group_id),
                Cow::from(Route::channel(group_id)),
            ),
            RouteInfo::LeaveGuild { guild_id } => (
                LightMethod::Delete,
                Route::UsersMeGuildsId,
                Cow::from(Route::user_guild("@me", guild_id)),
            ),
            RouteInfo::RemoveGroupRecipient { group_id, user_id } => (
                LightMethod::Delete,
                Route::None,
                Cow::from(Route::group_recipient(group_id, user_id)),
            ),
            RouteInfo::PinMessage { channel_id, message_id } => (
                LightMethod::Put,
                Route::ChannelsIdPins(channel_id),
                Cow::from(Route::channel_pin(channel_id, message_id)),
            ),
            RouteInfo::RemoveBan { guild_id, user_id } => (
                LightMethod::Delete,
                Route::GuildsIdBansUserId(guild_id),
                Cow::from(Route::guild_ban(guild_id, user_id)),
            ),
            RouteInfo::RemoveMemberRole { guild_id, role_id, user_id } => (
                LightMethod::Delete,
                Route::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Route::guild_member_role(guild_id, user_id, role_id)),
            ),
            RouteInfo::StartGuildPrune { days, guild_id } => (
                LightMethod::Post,
                Route::GuildsIdPrune(guild_id),
                Cow::from(Route::guild_prune(guild_id, days)),
            ),
            RouteInfo::StartIntegrationSync { guild_id, integration_id } => (
                LightMethod::Post,
                Route::GuildsIdIntegrationsId(guild_id),
                Cow::from(Route::guild_integration_sync(
                    guild_id,
                    integration_id,
                )),
            ),
            RouteInfo::StatusIncidentsUnresolved => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_incidents_unresolved()),
            ),
            RouteInfo::StatusMaintenancesActive => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_maintenances_active()),
            ),
            RouteInfo::StatusMaintenancesUpcoming => (
                LightMethod::Get,
                Route::None,
                Cow::from(Route::status_maintenances_upcoming()),
            ),
            RouteInfo::UnpinMessage { channel_id, message_id } => (
                LightMethod::Delete,
                Route::ChannelsIdPinsMessageId(channel_id),
                Cow::from(Route::channel_pin(channel_id, message_id)),
            ),
            RouteInfo::__Nonexhaustive => unreachable!(),
        }
    }
}
