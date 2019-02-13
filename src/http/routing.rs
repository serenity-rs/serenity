use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FmtResult, Write};
use super::LightMethod;

/// A representation of all path registered within the library. These are safe
/// and memory-efficient representations of each path that request functions
/// exist for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Path {
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
    // and _not_ a `hyper` `Method` due to `hyper`'s not deriving `Copy`.
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
    /// Route for the `/channels/:channel_id/recipients/:user_id` path.
    ///
    /// The data is the relevant `ChannelId`.
    ChannelsIdRecipientsId(u64),
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
    /// [`GuildId`]: struct.GuildId.html
    GuildsId(u64),
    /// Route for the `/guilds/:guild_id/bans` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdBans(u64),
    /// Route for the `/guilds/:guild_id/audit-logs` path.
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdAuditLogs(u64),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdBansUserId(u64),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdChannels(u64),
    /// Route for the `/guilds/:guild_id/embed` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmbed(u64),
    /// Route for the `/guilds/:guild_id/emojis` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmojis(u64),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmojisId(u64),
    /// Route for the `/guilds/:guild_id/integrations` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrations(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrationsId(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrationsIdSync(u64),
    /// Route for the `/guilds/:guild_id/invites` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdInvites(u64),
    /// Route for the `/guilds/:guild_id/members` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembers(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersId(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersMeNick(u64),
    /// Route for the `/guilds/:guild_id/prune` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdPrune(u64),
    /// Route for the `/guilds/:guild_id/regions` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRegions(u64),
    /// Route for the `/guilds/:guild_id/roles` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRoles(u64),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/vanity-url` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdVanityUrl(u64),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdWebhooks(u64),
    /// Route for the `/invites/:code` path.
    InvitesCode,
    /// Route for the `/incidents/unresolved.json` status API path.
    StatusIncidentsUnresolved,
    /// Route for the `/scheduled-maintenances/active.json` status API path.
    StatusMaintenancesActive,
    /// Route for the `/scheduled-maintenances/upcoming.json` status API path.
    StatusMaintenancesUpcoming,
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
}

impl Path {
    pub fn channel(channel_id: u64) -> String {
        format!("/channels/{}", channel_id)
    }

    pub fn channel_invites(channel_id: u64) -> String {
        format!("/channels/{}/invites", channel_id)
    }

    pub fn channel_message(channel_id: u64, message_id: u64) -> String {
        format!("/channels/{}/messages/{}", channel_id, message_id)
    }

    pub fn channel_message_reaction<D, T>(
        channel_id: u64,
        message_id: u64,
        user_id: D,
        reaction_type: T
    ) -> String where D: Display, T: Display {
        format!(
            "/channels/{}/messages/{}/reactions/{}/{}",
            channel_id,
            message_id,
            reaction_type,
            user_id,
        )
    }

    pub fn channel_message_reactions(
        channel_id: u64,
        message_id: u64,
        reaction: &str,
        limit: u8,
        after: Option<u64>,
    ) -> String {
        let mut uri = format!(
            "/channels/{}/messages/{}/reactions/{}?limit={}",
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

    pub fn channel_messages(channel_id: u64) -> String {
        format!("/channels/{}/messages", channel_id)
    }

    pub fn channel_messages_bulk_delete(channel_id: u64) -> String {
        format!("/channels/{}/messages/bulk-delete", channel_id)
    }

    pub fn channel_permission(channel_id: u64, target_id: u64) -> String {
        format!("/channels/{}/permissions/{}", channel_id, target_id)
    }

    pub fn channel_pin(channel_id: u64, message_id: u64) -> String {
        format!("/channels/{}/pins/{}", channel_id, message_id)
    }

    pub fn channel_pins(channel_id: u64) -> String {
        format!("/channels/{}/pins", channel_id)
    }

    pub fn channel_typing(channel_id: u64) -> String {
        format!("/channels/{}/typing", channel_id)
    }

    pub fn channel_webhooks(channel_id: u64) -> String {
        format!("/channels/{}/webhooks", channel_id)
    }

    pub fn gateway() -> &'static str {
        "/gateway"
    }

    pub fn gateway_bot() -> &'static str {
        "/gateway/bot"
    }

    pub fn group_recipient(group_id: u64, user_id: u64) -> String {
        format!("/channels/{}/recipients/{}", group_id, user_id)
    }

    pub fn guild(guild_id: u64) -> String {
        format!("/guilds/{}", guild_id)
    }

    pub fn guild_audit_logs(
        guild_id: u64,
        action_type: Option<u8>,
        user_id: Option<u64>,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> String {
        let mut s = format!(
            "/guilds/{}/audit-logs?",
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
        format!("/guilds/{}/bans/{}", guild_id, user_id)
    }

    pub fn guild_ban_optioned(
        guild_id: u64,
        user_id: u64,
        delete_message_days: u8,
        reason: &str,
    ) -> String {
        format!(
            "/guilds/{}/bans/{}?delete_message_days={}&reason={}",
            guild_id,
            user_id,
            delete_message_days,
            reason,
        )
    }

    pub fn guild_bans(guild_id: u64) -> String {
        format!("/guilds/{}/bans", guild_id)
    }

    pub fn guild_channels(guild_id: u64) -> String {
        format!("/guilds/{}/channels", guild_id)
    }

    pub fn guild_embed(guild_id: u64) -> String {
        format!("/guilds/{}/embed", guild_id)
    }

    pub fn guild_emojis(guild_id: u64) -> String {
        format!("/guilds/{}/emojis", guild_id)
    }

    pub fn guild_emoji(guild_id: u64, emoji_id: u64) -> String {
        format!("/guilds/{}/emojis/{}", guild_id, emoji_id)
    }

    pub fn guild_integration(
        guild_id: u64,
        integration_id: u64,
    ) -> String {
        format!("/guilds/{}/integrations/{}", guild_id, integration_id)
    }

    pub fn guild_integration_sync(
        guild_id: u64,
        integration_id: u64,
    ) -> String {
        format!(
            "/guilds/{}/integrations/{}/sync",
            guild_id,
            integration_id,
        )
    }

    pub fn guild_integrations(guild_id: u64) -> String {
        format!("/guilds/{}/integrations", guild_id)
    }

    pub fn guild_invites(guild_id: u64) -> String {
        format!("/guilds/{}/invites", guild_id)
    }

    pub fn guild_member(guild_id: u64, user_id: u64) -> String {
        format!("/guilds/{}/members/{}", guild_id, user_id)
    }

    pub fn guild_member_role(
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> String {
        format!(
            "/guilds/{}/members/{}/roles/{}",
            guild_id,
            user_id,
            role_id,
        )
    }

    pub fn guild_members(guild_id: u64) -> String {
        format!("/guilds/{}/members", guild_id)
    }

    pub fn guild_members_optioned(
        guild_id: u64,
        after: Option<u64>,
        limit: Option<u64>,
    ) -> String {
        let mut s = format!("/guilds/{}/members?", guild_id);

        if let Some(after) = after {
            let _ = write!(s, "&after={}", after);
        }

        if let Some(limit) = limit {
            let _ = write!(s, "&limit={}", limit);
        }

        s
    }

    pub fn guild_nickname(guild_id: u64) -> String {
        format!("/guilds/{}/members/@me/nick", guild_id)
    }

    pub fn guild_prune(guild_id: u64, days: u64) -> String {
        format!("/guilds/{}/prune?days={}", guild_id, days)
    }

    pub fn guild_regions(guild_id: u64) -> String {
        format!("/guilds/{}/regions", guild_id)
    }

    pub fn guild_role(guild_id: u64, role_id: u64) -> String {
        format!("/guilds/{}/roles/{}", guild_id, role_id)
    }

    pub fn guild_roles(guild_id: u64) -> String {
        format!("/guilds/{}/roles", guild_id)
    }

    pub fn guild_vanity_url(guild_id: u64) -> String {
        format!("/guilds/{}/vanity-url", guild_id)
    }

    pub fn guild_webhooks(guild_id: u64) -> String {
        format!("/guilds/{}/webhooks", guild_id)
    }

    pub fn guilds() -> &'static str {
        "/guilds"
    }

    pub fn invite(code: &str) -> String {
        format!("/invites/{}", code)
    }

    pub fn invite_optioned(code: &str, stats: bool) -> String {
        format!("/invites/{}?with_counts={}", code, stats)
    }

    pub fn oauth2_application_current() -> &'static str {
        "/oauth2/applications/@me"
    }

    pub fn private_channel() -> &'static str {
        "/users/@me/channels"
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
        format!("/users/{}", target)
    }

    pub fn user_dm_channels<D: Display>(target: D) -> String {
        format!("/users/{}/channels", target)
    }

    pub fn user_guild<D: Display>(target: D, guild_id: u64) -> String {
        format!("/users/{}/guilds/{}", target, guild_id)
    }

    pub fn user_guilds<D: Display>(target: D) -> String {
        format!("/users/{}/guilds", target)
    }

    pub fn user_guilds_optioned<D: Display>(
        target: D,
        after: Option<u64>,
        before: Option<u64>,
        limit: u64,
    ) -> String {
        let mut s = format!("/users/{}/guilds?limit={}&", target, limit);

        if let Some(after) = after {
            let _ = write!(s, "&after={}", after);
        }

        if let Some(before) = before {
            let _ = write!(s, "&before={}", before);
        }

        s
    }

    pub fn voice_regions() -> &'static str {
        "/voice/regions"
    }

    pub fn webhook(webhook_id: u64) -> String {
        format!("/webhooks/{}", webhook_id)
    }

    pub fn webhook_with_token<D>(webhook_id: u64, token: D) -> String
        where D: Display {
        format!("/webhooks/{}/{}", webhook_id, token)
    }

    pub fn webhook_with_token_optioned<D>(webhook_id: u64, token: D, wait: bool)
        -> String where D: Display {
        format!("/webhooks/{}/{}?wait={}", webhook_id, token, wait)
    }

    /// Returns the path's base string without replacements.
    pub fn base(&self) -> &str {
        use self::Path::*;

        match *self {
            ChannelsId(_) => "/channels/{}",
            ChannelsIdInvites(_) => "/channels/{}/invites",
            ChannelsIdMessages(_) => "/channels/{}/messages",
            ChannelsIdMessagesBulkDelete(_) => "/channels/{}/messages/bulk-delete",
            ChannelsIdMessagesId(_, _) => "/channels/{}/messages/{}",
            ChannelsIdMessagesIdAck(_) => "/channels/{}/messages/ack",
            ChannelsIdMessagesIdReactions(_) => "/channels/{}/messages/{}/reactions",
            ChannelsIdMessagesIdReactionsUserIdType(_) => "/channels/{}/messages/{}/reactions/{}/@me",
            ChannelsIdPermissionsOverwriteId(_) => "/channels/{}/permissions/{}",
            ChannelsIdPins(_) => "/channels/{}/pins",
            ChannelsIdPinsMessageId(_) => "/channels/{}/pins/{}",
            ChannelsIdRecipientsId(_) => "/channels/{}/recipients/{}",
            ChannelsIdTyping(_) => "/channels/{}/typing",
            ChannelsIdWebhooks(_) => "/channels/{}/webhooks",
            Gateway => "/gateway",
            GatewayBot => "/gateway/bot",
            Guilds => "/guilds",
            GuildsId(_) => "/guilds/{}",
            GuildsIdBans(_) => "/guilds/{}/bans",
            GuildsIdAuditLogs(_) => "/guilds/{}/audit-logs",
            GuildsIdBansUserId(_) => "/guilds/{}/bans/{}",
            GuildsIdChannels(_) => "/guilds/{}/channels",
            GuildsIdEmbed(_) => "/guilds/{}/embed",
            GuildsIdEmojis(_) => "/guilds/{}/emojis",
            GuildsIdEmojisId(_) => "/guilds/{}/emojis/{}",
            GuildsIdIntegrations(_) => "/guilds/{}/integrations",
            GuildsIdIntegrationsId(_) => "/guilds/{}/integrations/{}",
            GuildsIdIntegrationsIdSync(_) => "/guilds/{}/integrations/{}/sync",
            GuildsIdInvites(_) => "/guilds/{}/invites",
            GuildsIdMembers(_) => "/guilds/{}/membes",
            GuildsIdMembersId(_) => "/guilds/{}/members/{}",
            GuildsIdMembersIdRolesId(_) => "/guilds/{}/members/{}/roles/{}",
            GuildsIdMembersMeNick(_) => "/guilds/{}/members/@me/nick",
            GuildsIdPrune(_) => "/guilds/{}/prune",
            GuildsIdRegions(_) => "/guilds/{}/regions",
            GuildsIdRoles(_) => "/guilds/{}/roles",
            GuildsIdRolesId(_) => "/guilds/{}/roles/{}",
            GuildsIdVanityUrl(_) => "/guilds/{}/vanity-url",
            GuildsIdWebhooks(_) => "/guilds/{}/webhooks",
            InvitesCode => "/invites/{}",
            StatusIncidentsUnresolved => "/incidents/unresolved.json",
            StatusMaintenancesActive => "/scheduled-maintenances/active.json",
            StatusMaintenancesUpcoming => "/scheduled-maintenances/upcoming.json",
            UsersId => "/users/{}",
            UsersMe => "/users/@me",
            UsersMeChannels => "/users/@me/channels",
            UsersMeGuilds => "/users/@me/guilds",
            UsersMeGuildsId => "/users/@me/guilds/{}",
            VoiceRegions => "/voice/regions",
            WebhooksId(_) => "/webhooks/{}",
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.base())
    }
}

#[derive(Clone)]
pub enum Route<'a> {
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
}

impl<'a> Route<'a> {
    pub fn deconstruct(&self) -> (LightMethod, Path, Cow<str>) {
        match *self {
            Route::AddGroupRecipient { group_id, user_id } => (
                LightMethod::Post,
                Path::ChannelsIdRecipientsId(group_id),
                Cow::from(Path::group_recipient(group_id, user_id)),
            ),
            Route::AddMemberRole { guild_id, role_id, user_id } => (
                LightMethod::Delete,
                Path::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Path::guild_member_role(guild_id, user_id, role_id)),
            ),
            Route::GuildBanUser {
                guild_id,
                delete_message_days,
                reason,
                user_id,
            } => (
                // TODO
                LightMethod::Delete,
                Path::GuildsIdBansUserId(guild_id),
                Cow::from(Path::guild_ban_optioned(
                    guild_id,
                    user_id,
                    delete_message_days.unwrap_or(0),
                    reason.unwrap_or(""),
                )),
            ),
            Route::BroadcastTyping { channel_id } => (
                LightMethod::Post,
                Path::ChannelsIdTyping(channel_id),
                Cow::from(Path::channel_typing(channel_id)),
            ),
            Route::CreateChannel { guild_id } => (
                LightMethod::Post,
                Path::GuildsIdChannels(guild_id),
                Cow::from(Path::guild_channels(guild_id)),
            ),
            Route::CreateEmoji { guild_id } => (
                LightMethod::Post,
                Path::GuildsIdEmojis(guild_id),
                Cow::from(Path::guild_emojis(guild_id)),
            ),
            Route::CreateGuild => (
                LightMethod::Post,
                Path::Guilds,
                Cow::from(Path::guilds()),
            ),
            Route::CreateGuildIntegration { guild_id } => (
                LightMethod::Post,
                Path::GuildsIdIntegrationsId(guild_id),
                Cow::from(Path::guild_integrations(guild_id)),
            ),
            Route::CreateInvite { channel_id } => (
                LightMethod::Post,
                Path::ChannelsIdInvites(channel_id),
                Cow::from(Path::channel_invites(channel_id)),
            ),
            Route::CreateMessage { channel_id } => (
                LightMethod::Post,
                Path::ChannelsIdMessages(channel_id),
                Cow::from(Path::channel_messages(channel_id)),
            ),
            Route::CreatePermission { channel_id, target_id } => (
                LightMethod::Post,
                Path::ChannelsIdPermissionsOverwriteId(channel_id),
                Cow::from(Path::channel_permission(channel_id, target_id)),
            ),
            Route::CreatePrivateChannel => (
                LightMethod::Post,
                Path::UsersMeChannels,
                Cow::from(Path::user_dm_channels("@me")),
            ),
            Route::CreateReaction { channel_id, message_id, reaction } => (
                LightMethod::Put,
                Path::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                Cow::from(Path::channel_message_reaction(
                    channel_id,
                    message_id,
                    "@me",
                    reaction,
                )),
            ),
            Route::CreateRole { guild_id } => (
                LightMethod::Delete,
                Path::GuildsIdRoles(guild_id),
                Cow::from(Path::guild_roles(guild_id)),
            ),
            Route::CreateWebhook { channel_id } => (
                LightMethod::Delete,
                Path::ChannelsIdWebhooks(channel_id),
                Cow::from(Path::channel_webhooks(channel_id)),
            ),
            Route::DeleteChannel { channel_id } => (
                LightMethod::Delete,
                Path::ChannelsId(channel_id),
                Cow::from(Path::channel(channel_id)),
            ),
            Route::DeleteEmoji { emoji_id, guild_id } => (
                LightMethod::Delete,
                Path::GuildsIdEmojisId(guild_id),
                Cow::from(Path::guild_emoji(guild_id, emoji_id)),
            ),
            Route::DeleteGuild { guild_id } => (
                LightMethod::Delete,
                Path::GuildsId(guild_id),
                Cow::from(Path::guild(guild_id)),
            ),
            Route::DeleteGuildIntegration { guild_id, integration_id } => (
                LightMethod::Delete,
                Path::GuildsIdIntegrationsId(guild_id),
                Cow::from(Path::guild_integration(guild_id, integration_id)),
            ),
            Route::DeleteInvite { code } => (
                LightMethod::Delete,
                Path::InvitesCode,
                Cow::from(Path::invite(code)),
            ),
            Route::DeleteMessage { channel_id, message_id } => (
                LightMethod::Delete,
                Path::ChannelsIdMessagesId(LightMethod::Delete, message_id),
                Cow::from(Path::channel_message(channel_id, message_id)),
            ),
            Route::DeleteMessages { channel_id } => (
                LightMethod::Delete,
                Path::ChannelsIdMessagesBulkDelete(channel_id),
                Cow::from(Path::channel_messages_bulk_delete(channel_id)),
            ),
            Route::DeletePermission { channel_id, target_id } => (
                LightMethod::Delete,
                Path::ChannelsIdPermissionsOverwriteId(channel_id),
                Cow::from(Path::channel_permission(channel_id, target_id)),
            ),
            Route::DeleteReaction {
                channel_id,
                message_id,
                reaction,
                user,
            } => (
                LightMethod::Delete,
                Path::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                Cow::from(Path::channel_message_reaction(
                    channel_id,
                    message_id,
                    user,
                    reaction,
                ))
            ),
            Route::DeleteRole { guild_id, role_id } => (
                LightMethod::Delete,
                Path::GuildsIdRolesId(guild_id),
                Cow::from(Path::guild_role(guild_id, role_id)),
            ),
            Route::DeleteWebhook { webhook_id } => (
                LightMethod::Delete,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook(webhook_id)),
            ),
            Route::DeleteWebhookWithToken { token, webhook_id } => (
                LightMethod::Delete,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook_with_token(webhook_id, token)),
            ),
            Route::EditChannel { channel_id } => (
                LightMethod::Patch,
                Path::ChannelsId(channel_id),
                Cow::from(Path::channel(channel_id)),
            ),
            Route::EditEmoji { emoji_id, guild_id } => (
                LightMethod::Patch,
                Path::GuildsIdEmojisId(guild_id),
                Cow::from(Path::guild_emoji(guild_id, emoji_id)),
            ),
            Route::EditGuild { guild_id } => (
                LightMethod::Patch,
                Path::GuildsId(guild_id),
                Cow::from(Path::guild(guild_id)),
            ),
            Route::EditGuildChannels { guild_id } => (
                LightMethod::Patch,
                Path::GuildsIdChannels(guild_id),
                Cow::from(Path::guild_channels(guild_id)),
            ),
            Route::EditGuildEmbed { guild_id } => (
                LightMethod::Patch,
                Path::GuildsIdEmbed(guild_id),
                Cow::from(Path::guild_embed(guild_id)),
            ),
            Route::EditMember { guild_id, user_id } => (
                LightMethod::Patch,
                Path::GuildsIdMembersId(guild_id),
                Cow::from(Path::guild_member(guild_id, user_id)),
            ),
            Route::EditMessage { channel_id, message_id } => (
                LightMethod::Patch,
                Path::ChannelsIdMessagesId(LightMethod::Patch, channel_id),
                Cow::from(Path::channel_message(channel_id, message_id)),
            ),
            Route::EditNickname { guild_id } => (
                LightMethod::Patch,
                Path::GuildsIdMembersMeNick(guild_id),
                Cow::from(Path::guild_nickname(guild_id)),
            ),
            Route::EditProfile => (
                LightMethod::Patch,
                Path::UsersMe,
                Cow::from(Path::user("@me")),
            ),
            Route::EditRole { guild_id, role_id } => (
                LightMethod::Patch,
                Path::GuildsIdRolesId(guild_id),
                Cow::from(Path::guild_role(guild_id, role_id)),
            ),
            Route::EditRolePosition { guild_id } => (
                LightMethod::Patch,
                Path::GuildsId(guild_id),
                Cow::from(Path::guild_roles(guild_id)),
            ),
            Route::EditWebhook { webhook_id } => (
                LightMethod::Patch,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook(webhook_id)),
            ),
            Route::EditWebhookWithToken { token, webhook_id } => (
                LightMethod::Patch,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook_with_token(webhook_id, token)),
            ),
            Route::ExecuteWebhook { token, wait, webhook_id } => (
                LightMethod::Post,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook_with_token_optioned(
                    webhook_id,
                    token,
                    wait,
                )),
            ),
            Route::GetActiveMaintenance => (
                LightMethod::Get,
                Path::StatusMaintenancesActive,
                Cow::from(Path::status_maintenances_active()),
            ),
            Route::GetAuditLogs {
                action_type,
                before,
                guild_id,
                limit,
                user_id,
            } => (
                LightMethod::Get,
                Path::GuildsIdAuditLogs(guild_id),
                Cow::from(Path::guild_audit_logs(
                    guild_id,
                    action_type,
                    user_id,
                    before,
                    limit,
                )),
            ),
            Route::GetBans { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdBans(guild_id),
                Cow::from(Path::guild_bans(guild_id)),
            ),
            Route::GetBotGateway => (
                LightMethod::Get,
                Path::GatewayBot,
                Cow::from(Path::gateway_bot()),
            ),
            Route::GetChannel { channel_id } => (
                LightMethod::Get,
                Path::ChannelsId(channel_id),
                Cow::from(Path::channel(channel_id)),
            ),
            Route::GetChannelInvites { channel_id } => (
                LightMethod::Get,
                Path::ChannelsIdInvites(channel_id),
                Cow::from(Path::channel_invites(channel_id)),
            ),
            Route::GetChannelWebhooks { channel_id } => (
                LightMethod::Get,
                Path::ChannelsIdWebhooks(channel_id),
                Cow::from(Path::channel_webhooks(channel_id)),
            ),
            Route::GetChannels { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdChannels(guild_id),
                Cow::from(Path::guild_channels(guild_id)),
            ),
            Route::GetCurrentUser => (
                LightMethod::Get,
                Path::UsersMe,
                Cow::from(Path::user("@me")),
            ),
            Route::GetGateway => (
                LightMethod::Get,
                Path::Gateway,
                Cow::from(Path::gateway()),
            ),
            Route::GetGuild { guild_id } => (
                LightMethod::Get,
                Path::GuildsId(guild_id),
                Cow::from(Path::guild(guild_id)),
            ),
            Route::GetGuildEmbed { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdEmbed(guild_id),
                Cow::from(Path::guild_embed(guild_id)),
            ),
            Route::GetGuildIntegrations { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdIntegrations(guild_id),
                Cow::from(Path::guild_integrations(guild_id)),
            ),
            Route::GetGuildInvites { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdInvites(guild_id),
                Cow::from(Path::guild_invites(guild_id)),
            ),
            Route::GetGuildMembers { after, guild_id, limit } => (
                LightMethod::Get,
                Path::GuildsIdMembers(guild_id),
                Cow::from(Path::guild_members_optioned(guild_id, after, limit)),
            ),
            Route::GetGuildPruneCount { days, guild_id } => (
                LightMethod::Get,
                Path::GuildsIdPrune(guild_id),
                Cow::from(Path::guild_prune(guild_id, days)),
            ),
            Route::GetGuildRegions { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdRegions(guild_id),
                Cow::from(Path::guild_regions(guild_id)),
            ),
            Route::GetGuildRoles { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdRoles(guild_id),
                Cow::from(Path::guild_roles(guild_id)),
            ),
            Route::GetGuildVanityUrl { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdVanityUrl(guild_id),
                Cow::from(Path::guild_vanity_url(guild_id)),
            ),
            Route::GetGuildWebhooks { guild_id } => (
                LightMethod::Get,
                Path::GuildsIdWebhooks(guild_id),
                Cow::from(Path::guild_webhooks(guild_id)),
            ),
            Route::GetGuilds { after, before, limit } => (
                LightMethod::Get,
                Path::UsersMeGuilds,
                Cow::from(Path::user_guilds_optioned(
                    "@me",
                    after,
                    before,
                    limit,
                )),
            ),
            Route::GetInvite { code, stats } => (
                LightMethod::Get,
                Path::InvitesCode,
                Cow::from(Path::invite_optioned(code, stats)),
            ),
            Route::GetMember { guild_id, user_id } => (
                LightMethod::Get,
                Path::GuildsIdMembersId(guild_id),
                Cow::from(Path::guild_member(guild_id, user_id)),
            ),
            Route::GetMessage { channel_id, message_id } => (
                LightMethod::Get,
                Path::ChannelsIdMessagesId(LightMethod::Get, channel_id),
                Cow::from(Path::channel_message(channel_id, message_id)),
            ),
            Route::GetPins { channel_id } => (
                LightMethod::Get,
                Path::ChannelsIdPins(channel_id),
                Cow::from(Path::channel_pins(channel_id)),
            ),
            Route::GetReactionUsers {
                after,
                channel_id,
                limit,
                message_id,
                ref reaction,
            } => (
                LightMethod::Get,
                Path::ChannelsIdMessagesIdReactions(channel_id),
                Cow::from(Path::channel_message_reactions(
                    channel_id,
                    message_id,
                    reaction,
                    limit,
                    after,)),
            ),
            Route::GetUser { user_id } => (
                LightMethod::Get,
                Path::UsersId,
                Cow::from(Path::user(user_id)),
            ),
            Route::GetUserDmChannels => (
                LightMethod::Get,
                Path::UsersMeChannels,
                Cow::from(Path::user_dm_channels("@me")),
            ),
            Route::GetVoiceRegions => (
                LightMethod::Get,
                Path::VoiceRegions,
                Cow::from(Path::voice_regions()),
            ),
            Route::GetWebhook { webhook_id } => (
                LightMethod::Get,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook(webhook_id)),
            ),
            Route::GetWebhookWithToken { token, webhook_id } => (
                LightMethod::Get,
                Path::WebhooksId(webhook_id),
                Cow::from(Path::webhook_with_token(webhook_id, token)),
            ),
            Route::KickMember { guild_id, user_id } => (
                LightMethod::Delete,
                Path::GuildsIdMembersId(guild_id),
                Cow::from(Path::guild_member(guild_id, user_id)),
            ),
            Route::LeaveGroup { group_id } => (
                LightMethod::Delete,
                Path::ChannelsId(group_id),
                Cow::from(Path::channel(group_id)),
            ),
            Route::LeaveGuild { guild_id } => (
                LightMethod::Delete,
                Path::UsersMeGuildsId,
                Cow::from(Path::user_guild("@me", guild_id)),
            ),
            Route::RemoveGroupRecipient { group_id, user_id } => (
                LightMethod::Delete,
                Path::ChannelsIdRecipientsId(group_id),
                Cow::from(Path::group_recipient(group_id, user_id)),
            ),
            Route::PinMessage { channel_id, message_id } => (
                LightMethod::Put,
                Path::ChannelsIdPins(channel_id),
                Cow::from(Path::channel_pin(channel_id, message_id)),
            ),
            Route::RemoveBan { guild_id, user_id } => (
                LightMethod::Delete,
                Path::GuildsIdBansUserId(guild_id),
                Cow::from(Path::guild_ban(guild_id, user_id)),
            ),
            Route::RemoveMemberRole { guild_id, role_id, user_id } => (
                LightMethod::Delete,
                Path::GuildsIdMembersIdRolesId(guild_id),
                Cow::from(Path::guild_member_role(guild_id, user_id, role_id)),
            ),
            Route::StartGuildPrune { days, guild_id } => (
                LightMethod::Post,
                Path::GuildsIdPrune(guild_id),
                Cow::from(Path::guild_prune(guild_id, days)),
            ),
            Route::StartIntegrationSync { guild_id, integration_id } => (
                LightMethod::Post,
                Path::GuildsIdIntegrationsId(guild_id),
                Cow::from(Path::guild_integration_sync(
                    guild_id,
                    integration_id,
                )),
            ),
            Route::StatusIncidentsUnresolved => (
                LightMethod::Get,
                Path::StatusIncidentsUnresolved,
                Cow::from(Path::status_incidents_unresolved()),
            ),
            Route::StatusMaintenancesActive => (
                LightMethod::Get,
                Path::StatusMaintenancesActive,
                Cow::from(Path::status_maintenances_active()),
            ),
            Route::StatusMaintenancesUpcoming => (
                LightMethod::Get,
                Path::StatusMaintenancesUpcoming,
                Cow::from(Path::status_maintenances_upcoming()),
            ),
            Route::UnpinMessage { channel_id, message_id } => (
                LightMethod::Delete,
                Path::ChannelsIdPinsMessageId(channel_id),
                Cow::from(Path::channel_pin(channel_id, message_id)),
            ),
            _ => unimplemented!(), // TODO: finish 5 unconvered variants
        }
    }
}
