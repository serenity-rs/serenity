use std::borrow::Cow;

use crate::model::id::*;

/// Used to group requests together for ratelimiting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RatelimitingBucket(Option<(RouteKind, Option<GenericId>)>);

impl RatelimitingBucket {
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

enum RatelimitingKind {
    /// Requests with the same path and major parameter (usually an Id) should be grouped together
    /// for ratelimiting.
    PathAndId(GenericId),
    /// Requests with the same path should be ratelimited together.
    Path,
}

/// A macro for defining routes as well as the type of ratelimiting they perform. Takes as input a
/// list of route definitions, and generates a definition for the `Route` enum and implements
/// methods on it.
macro_rules! routes {
    ($lt:lifetime, {
        $(
            $name:ident $({ $($field_name_route:ident: $field_type:ty | $field_type_owned:ty),* })?,
            { $($field_name:ident),* },
            $path:expr,
            $ratelimiting_kind:expr;
        )+
    }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        enum RouteKind {
            $($name,)+
        }

        #[derive(Clone, Copy, Debug)]
        pub enum Route<$lt> {
            $(
                $name $({ $($field_name_route: $field_type),* })?,
            )+
        }

        impl<$lt> Route<$lt> {
            fn kind(&self) -> RouteKind {
                match self {
                    $(
                        Self::$name {..} => RouteKind::$name,
                    )+
                }
            }

            #[must_use]
            pub fn path(self) -> Cow<'static, str> {
                match self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => $path.into(),
                    )+
                }
            }

            #[must_use]
            pub fn ratelimiting_bucket(&self) -> RatelimitingBucket {
                #[expect(unused_variables)]
                let ratelimiting_kind = match *self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => $ratelimiting_kind,
                    )+
                };

                RatelimitingBucket(ratelimiting_kind.map(|r| {
                    let id = match r {
                        RatelimitingKind::PathAndId(id) => Some(id),
                        RatelimitingKind::Path => None,
                    };
                    (self.kind(), id)
                }))
            }

            #[must_use]
            pub fn upgrade_to_fully_owned_route(&self) -> OwnedRoute {
                match self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => OwnedRoute::$name $({ $($field_name_route: (*$field_name_route).to_owned()),* })?,
                    )+
                }
            }
        }

        /// This represents the common identifiers, which are used in rate limit buckets.
        #[derive(Default, Clone)]
        pub struct RatelimitCause {
            pub user_id: Option<UserId>,
            pub channel_id: Option<ChannelId>,
            pub guild_id: Option<GuildId>,
            pub message_id: Option<MessageId>
        }

        #[derive(Clone, Debug, Serialize)]
        pub enum OwnedRoute {
            $(
                $name $({ $($field_name_route: $field_type_owned),* })?,
            )+
        }

        impl OwnedRoute {
            fn kind(&self) -> RouteKind {
                match self {
                    $(
                        Self::$name {..} => RouteKind::$name,
                    )+
                }
            }

            #[must_use]
            pub fn path(self) -> Cow<'static, str> {
                match self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => $path.into(),
                    )+
                }
            }

            #[must_use]
            pub fn ratelimiting_bucket(&self) -> RatelimitingBucket {
                #[expect(unused_variables)]
                let ratelimiting_kind = match self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => $ratelimiting_kind,
                    )+
                };

                RatelimitingBucket(ratelimiting_kind.map(|r| {
                    let id = match r {
                        RatelimitingKind::PathAndId(id) => Some(id),
                        RatelimitingKind::Path => None,
                    };
                    (self.kind(), id)
                }))
            }

            #[must_use]
            #[expect(unused_variables)] // prevent compiler from complaining about unused variables
            pub fn get_common_identifiers(&self) -> RatelimitCause {
                match self {
                    $(
                        Self::$name $({ $($field_name_route),* })? => {
                            RatelimitCause {
                                $($field_name: Some(*$field_name),)*
                                ..Default::default()
                            }
                        },
                    )+
                }
            }
        }
    };
}

// This macro takes as input a list of route definitions, represented in the following way:
// 1. The first line defines an enum variant representing an endpoint.
// The type before the "|" represents the type in the "Route" enum,
// the second one the type in the "OwnedRoute" enum.
// 2. The second line provides the names of the parameters, which are used for [`RatelimitCause`].
// This line must not contain any names, which are not present in the line above, any may only
// contain `message_id`, `guild_id`, `user_id`, or ` channel_id`.
// 3. The third line provides the url for that endpoint.
// 4. The fourth line indicates what type of ratelimiting the endpoint employs.
routes! ('a, {
    Channel { channel_id: ChannelId | ChannelId},
    { channel_id },
    api!("/channels/{}", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelInvites { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/invites", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessage { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageCrosspost { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}/crosspost", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageReaction { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId, user_id: UserId | UserId, reaction: &'a str | String },
    { channel_id, message_id, user_id },
    api!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, reaction, user_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageReactionMe { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId, reaction: &'a str | String },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageReactionEmoji { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId, reaction: &'a str | String },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageReactions { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}/reactions", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessages { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/messages", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessagesBulkDelete { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/messages/bulk-delete", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelFollowNews { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/followers", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelPermission { channel_id: ChannelId | ChannelId, target_id: TargetId | TargetId },
    { channel_id },
    api!("/channels/{}/permissions/{}", channel_id, target_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelPin { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/pins/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelPins { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/pins", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelTyping { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/typing", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelWebhooks { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/webhooks", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelMessageThreads { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/messages/{}/threads", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelThreads { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelForumPosts { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelThreadMember { channel_id: ChannelId | ChannelId, user_id: UserId | UserId },
    { channel_id, user_id },
    api!("/channels/{}/thread-members/{}", channel_id, user_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelThreadMemberMe { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/thread-members/@me", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelThreadMembers { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/thread-members", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelArchivedPublicThreads { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/threads/archived/public", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelArchivedPrivateThreads { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelJoinedPrivateThreads { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/users/@me/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelPollGetAnswerVoters { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId, answer_id: AnswerId | AnswerId },
    { channel_id, message_id },
    api!("/channels/{}/polls/{}/answers/{}", channel_id, message_id, answer_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelPollExpire { channel_id: ChannelId | ChannelId, message_id: MessageId | MessageId },
    { channel_id, message_id },
    api!("/channels/{}/polls/{}/expire", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    ChannelVoiceStatus { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/channels/{}/voice-status", channel_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(channel_id.get())));

    Gateway,
    {},
    api!("/gateway"),
    Some(RatelimitingKind::Path);

    GatewayBot,
    {},
    api!("/gateway/bot"),
    Some(RatelimitingKind::Path);

    Guild { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildAuditLogs { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/audit-logs", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildAutomodRule { guild_id: GuildId | GuildId, rule_id: RuleId| RuleId },
    { guild_id },
    api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildAutomodRules { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/auto-moderation/rules", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildBan { guild_id: GuildId | GuildId, user_id: UserId | UserId },
    { guild_id, user_id },
    api!("/guilds/{}/bans/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildBulkBan { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/bulk-ban", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildBans { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/bans", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildChannels { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/channels", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildWidget { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/widget", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildPreview { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/preview", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildEmojis { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/emojis", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildEmoji { guild_id: GuildId | GuildId, emoji_id: EmojiId | EmojiId },
    { guild_id },
    api!("/guilds/{}/emojis/{}", guild_id, emoji_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildIntegration { guild_id: GuildId | GuildId, integration_id: IntegrationId | IntegrationId },
    { guild_id },
    api!("/guilds/{}/integrations/{}", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildIntegrationSync { guild_id: GuildId | GuildId, integration_id: IntegrationId | IntegrationId },
    { guild_id },
    api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildIntegrations { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/integrations", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildInvites { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/invites", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMember { guild_id: GuildId | GuildId, user_id: UserId | UserId },
    { guild_id, user_id },
    api!("/guilds/{}/members/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMemberRole { guild_id: GuildId | GuildId, user_id: UserId | UserId, role_id: RoleId | RoleId },
    { guild_id, user_id },
    api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMembers { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/members", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMembersSearch { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/members/search", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMemberMe { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/members/@me", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildMfa { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/mfa", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildPrune { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/prune", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildRegions { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/regions", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildRole { guild_id: GuildId | GuildId, role_id: RoleId | RoleId },
    { guild_id },
    api!("/guilds/{}/roles/{}", guild_id, role_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildRoles { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/roles", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildScheduledEvent { guild_id: GuildId | GuildId, event_id: ScheduledEventId | ScheduledEventId },
    { guild_id },
    api!("/guilds/{}/scheduled-events/{}", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildScheduledEvents { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/scheduled-events", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildScheduledEventUsers { guild_id: GuildId | GuildId, event_id: ScheduledEventId | ScheduledEventId },
    { guild_id },
    api!("/guilds/{}/scheduled-events/{}/users", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildSticker { guild_id: GuildId | GuildId, sticker_id: StickerId | StickerId },
    { guild_id },
    api!("/guilds/{}/stickers/{}", guild_id, sticker_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildStickers { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/stickers", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildVanityUrl { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/vanity-url", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildVoiceStates { guild_id: GuildId | GuildId, user_id: UserId | UserId },
    { guild_id, user_id },
    api!("/guilds/{}/voice-states/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildVoiceStateMe { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/voice-states/@me", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildWebhooks { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/webhooks", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildWelcomeScreen { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/welcome-screen", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    GuildThreadsActive { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/guilds/{}/threads/active", guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(guild_id.get())));

    Guilds,
    {},
    api!("/guilds"),
    Some(RatelimitingKind::Path);

    Invite { code: &'a str | String },
    {},
    api!("/invites/{}", code),
    Some(RatelimitingKind::Path);

    OAuth2Token,
    {},
    api!("/oauth2/token"),
    None;

    OAuth2TokenRevocation,
    {},
    api!("/oauth2/token/revoke"),
    None;

    OAuth2ApplicationCurrent,
    {},
    api!("/oauth2/applications/@me"),
    None;

    OAuth2AuthorizationCurrent,
    {},
    api!("/oauth2/@me"),
    None;

    StatusIncidentsUnresolved,
    {},
    status!("/incidents/unresolved.json"),
    None;

    StatusMaintenancesActive,
    {},
    status!("/scheduled-maintenances/active.json"),
    None;

    StatusMaintenancesUpcoming,
    {},
    status!("/scheduled-maintenances/upcoming.json"),
    None;

    Sticker { sticker_id: StickerId | StickerId },
    {},
    api!("/stickers/{}", sticker_id),
    Some(RatelimitingKind::Path);

    StickerPacks,
    {},
    api!("/sticker-packs"),
    Some(RatelimitingKind::Path);

    StickerPack { sticker_pack_id: StickerPackId | StickerPackId },
    {},
    api!("/sticker-packs/{}", sticker_pack_id),
    Some(RatelimitingKind::Path);

    User { user_id: UserId | UserId },
    {},
    api!("/users/{}", user_id),
    Some(RatelimitingKind::Path);

    UserMe,
    {},
    api!("/users/@me"),
    Some(RatelimitingKind::Path);

    UserMeConnections,
    {},
    api!("/users/@me/connections"),
    Some(RatelimitingKind::Path);

    UserMeDmChannels,
    {},
    api!("/users/@me/channels"),
    Some(RatelimitingKind::Path);

    UserMeGuild { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/users/@me/guilds/{}", guild_id),
    Some(RatelimitingKind::Path);

    UserMeGuildMember { guild_id: GuildId | GuildId },
    { guild_id },
    api!("/users/@me/guilds/{}/member", guild_id),
    Some(RatelimitingKind::Path);

    UserMeGuilds,
    {},
    api!("/users/@me/guilds"),
    Some(RatelimitingKind::Path);

    VoiceRegions,
    {},
    api!("/voice/regions"),
    Some(RatelimitingKind::Path);

    Webhook { webhook_id: WebhookId | WebhookId },
    {},
    api!("/webhooks/{}", webhook_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(webhook_id.get())));

    WebhookWithToken { webhook_id: WebhookId | WebhookId, token: &'a str | String },
    {},
    api!("/webhooks/{}/{}", webhook_id, token),
    Some(RatelimitingKind::PathAndId(GenericId::new(webhook_id.get())));

    WebhookMessage { webhook_id: WebhookId | WebhookId, token: &'a str | String, message_id: MessageId | MessageId },
    { message_id },
    api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(webhook_id.get())));

    WebhookOriginalInteractionResponse { application_id: ApplicationId | ApplicationId, token: &'a str | String },
    {},
    api!("/webhooks/{}/{}/messages/@original", application_id, token),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    WebhookFollowupMessage { application_id: ApplicationId | ApplicationId, token: &'a str | String, message_id: MessageId | MessageId },
    { message_id },
    api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    WebhookFollowupMessages { application_id: ApplicationId | ApplicationId, token: &'a str | String },
    {},
    api!("/webhooks/{}/{}", application_id, token),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    InteractionResponse { interaction_id: InteractionId | InteractionId, token: &'a str | String },
    {},
    api!("/interactions/{}/{}/callback", interaction_id, token),
    Some(RatelimitingKind::PathAndId(GenericId::new(interaction_id.get())));

    Command { application_id: ApplicationId | ApplicationId, command_id: CommandId | CommandId },
    {},
    api!("/applications/{}/commands/{}", application_id, command_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Commands { application_id: ApplicationId | ApplicationId },
    {},
    api!("/applications/{}/commands", application_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    GuildCommand { application_id: ApplicationId | ApplicationId, guild_id: GuildId | GuildId, command_id: CommandId | CommandId },
    { guild_id },
    api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    GuildCommandPermissions { application_id: ApplicationId | ApplicationId, guild_id: GuildId | GuildId, command_id: CommandId | CommandId },
    { guild_id },
    api!("/applications/{}/guilds/{}/commands/{}/permissions", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    GuildCommands { application_id: ApplicationId | ApplicationId, guild_id: GuildId | GuildId },
    { guild_id },
    api!("/applications/{}/guilds/{}/commands", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    GuildCommandsPermissions { application_id: ApplicationId | ApplicationId, guild_id: GuildId | GuildId },
    { guild_id },
    api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Skus { application_id: ApplicationId | ApplicationId },
    {},
    api!("/applications/{}/skus", application_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Emoji { application_id: ApplicationId | ApplicationId, emoji_id: EmojiId | EmojiId },
    {},
    api!("/applications/{}/emojis/{}", application_id, emoji_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Emojis { application_id: ApplicationId | ApplicationId },
    {},
    api!("/applications/{}/emojis", application_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Entitlement { application_id: ApplicationId | ApplicationId, entitlement_id: EntitlementId | EntitlementId },
    {},
    api!("/applications/{}/entitlements/{}", application_id, entitlement_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    Entitlements { application_id: ApplicationId | ApplicationId },
    {},
    api!("/applications/{}/entitlements", application_id),
    Some(RatelimitingKind::PathAndId(GenericId::new(application_id.get())));

    StageInstances,
    {},
    api!("/stage-instances"),
    Some(RatelimitingKind::Path);

    StageInstance { channel_id: ChannelId | ChannelId },
    { channel_id },
    api!("/stage-instances/{}", channel_id),
    Some(RatelimitingKind::Path);
});
