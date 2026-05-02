use std::borrow::Cow;

use crate::model::id::Snowflake;

/// Used to group requests together for ratelimiting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RatelimitingBucket(Option<(RouteKind, Option<Snowflake>)>);

impl RatelimitingBucket {
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

enum RatelimitingKind {
    /// Requests with the same path and major parameter (usually an Id) should be grouped together
    /// for ratelimiting.
    PathAndId(Snowflake),
    /// Requests with the same path should be ratelimited together.
    Path,
}

/// A macro for defining routes as well as the type of ratelimiting they perform. Takes as input a
/// list of route definitions, and generates a definition for the `Route` enum and implements
/// methods on it.
macro_rules! routes {
    ($lt:lifetime, {
        $(
            $name:ident $({ $($field_name:ident: $field_type:ty),* })?,
            $path:expr,
            $ratelimiting_kind:expr;
        )+
    }) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Route<$lt> {
            $(
                $name $({ $($field_name: $field_type),* })?,
            )+
        }

        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        enum RouteKind {
            $($name,)+
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
                        Self::$name $({ $($field_name),* })? => $path.into(),
                    )+
                }
            }

            #[must_use]
            pub fn ratelimiting_bucket(&self) -> RatelimitingBucket {
                #[expect(unused_variables)]
                let ratelimiting_kind = match *self {
                    $(
                        Self::$name $({ $($field_name),* })? => $ratelimiting_kind,
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

        }
    };
}

// This macro takes as input a list of route definitions, represented in the following way:
// 1. The first line defines an enum variant representing an endpoint.
// 2. The second line provides the url for that endpoint.
// 3. The third line indicates what type of ratelimiting the endpoint employs.
routes! ('a, {
    Channel { channel_id: Snowflake },
    api!("/channels/{}", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelInvites { channel_id: Snowflake },
    api!("/channels/{}/invites", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessage { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/messages/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageCrosspost { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/messages/{}/crosspost", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageReaction { channel_id: Snowflake, message_id: Snowflake, user_id: Snowflake, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, reaction, user_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageReactionMe { channel_id: Snowflake, message_id: Snowflake, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageReactionEmoji { channel_id: Snowflake, message_id: Snowflake, reaction: &'a str },
    api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageReactions { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/messages/{}/reactions", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessages { channel_id: Snowflake },
    api!("/channels/{}/messages", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessagesBulkDelete { channel_id: Snowflake },
    api!("/channels/{}/messages/bulk-delete", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelFollowNews { channel_id: Snowflake },
    api!("/channels/{}/followers", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelPermission { channel_id: Snowflake, target_id: Snowflake },
    api!("/channels/{}/permissions/{}", channel_id, target_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelPin { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/messages/pins/{}", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelPins { channel_id: Snowflake },
    api!("/channels/{}/messages/pins", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelTyping { channel_id: Snowflake },
    api!("/channels/{}/typing", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelWebhooks { channel_id: Snowflake },
    api!("/channels/{}/webhooks", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelMessageThreads { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/messages/{}/threads", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelThreads { channel_id: Snowflake },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelForumPosts { channel_id: Snowflake },
    api!("/channels/{}/threads", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelThreadMember { thread_id: Snowflake, user_id: Snowflake },
    api!("/channels/{}/thread-members/{}", thread_id, user_id),
    Some(RatelimitingKind::PathAndId(thread_id));

    ChannelThreadMemberMe { thread_id: Snowflake },
    api!("/channels/{}/thread-members/@me", thread_id),
    Some(RatelimitingKind::PathAndId(thread_id));

    ChannelThreadMembers { thread_id: Snowflake },
    api!("/channels/{}/thread-members", thread_id),
    Some(RatelimitingKind::PathAndId(thread_id));

    ChannelArchivedPublicThreads { channel_id: Snowflake },
    api!("/channels/{}/threads/archived/public", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelArchivedPrivateThreads { channel_id: Snowflake },
    api!("/channels/{}/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelJoinedPrivateThreads { channel_id: Snowflake },
    api!("/channels/{}/users/@me/threads/archived/private", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelPollGetAnswerVoters { channel_id: Snowflake, message_id: Snowflake, answer_id: u8 },
    api!("/channels/{}/polls/{}/answers/{}", channel_id, message_id, answer_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelPollExpire { channel_id: Snowflake, message_id: Snowflake },
    api!("/channels/{}/polls/{}/expire", channel_id, message_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    ChannelVoiceStatus { channel_id: Snowflake },
    api!("/channels/{}/voice-status", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    Gateway,
    api!("/gateway"),
    Some(RatelimitingKind::Path);

    GatewayBot,
    api!("/gateway/bot"),
    Some(RatelimitingKind::Path);

    Guild { guild_id: Snowflake },
    api!("/guilds/{}", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildAuditLogs { guild_id: Snowflake },
    api!("/guilds/{}/audit-logs", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildAutomodRule { guild_id: Snowflake, rule_id: Snowflake },
    api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildAutomodRules { guild_id: Snowflake },
    api!("/guilds/{}/auto-moderation/rules", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildBan { guild_id: Snowflake, user_id: Snowflake },
    api!("/guilds/{}/bans/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildBulkBan { guild_id: Snowflake },
    api!("/guilds/{}/bulk-ban", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildBans { guild_id: Snowflake },
    api!("/guilds/{}/bans", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildChannels { guild_id: Snowflake },
    api!("/guilds/{}/channels", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildWidget { guild_id: Snowflake },
    api!("/guilds/{}/widget", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildPreview { guild_id: Snowflake },
    api!("/guilds/{}/preview", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildEmojis { guild_id: Snowflake },
    api!("/guilds/{}/emojis", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildEmoji { guild_id: Snowflake, emoji_id: Snowflake },
    api!("/guilds/{}/emojis/{}", guild_id, emoji_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildIntegration { guild_id: Snowflake, integration_id: Snowflake },
    api!("/guilds/{}/integrations/{}", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildIntegrationSync { guild_id: Snowflake, integration_id: Snowflake },
    api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildIntegrations { guild_id: Snowflake },
    api!("/guilds/{}/integrations", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildInvites { guild_id: Snowflake },
    api!("/guilds/{}/invites", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMember { guild_id: Snowflake, user_id: Snowflake },
    api!("/guilds/{}/members/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMemberRole { guild_id: Snowflake, user_id: Snowflake, role_id: Snowflake },
    api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMembers { guild_id: Snowflake },
    api!("/guilds/{}/members", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMembersSearch { guild_id: Snowflake },
    api!("/guilds/{}/members/search", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMemberMe { guild_id: Snowflake },
    api!("/guilds/{}/members/@me", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildMfa { guild_id: Snowflake },
    api!("/guilds/{}/mfa", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildPrune { guild_id: Snowflake },
    api!("/guilds/{}/prune", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildRegions { guild_id: Snowflake },
    api!("/guilds/{}/regions", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildRole { guild_id: Snowflake, role_id: Snowflake },
    api!("/guilds/{}/roles/{}", guild_id, role_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildRoles { guild_id: Snowflake },
    api!("/guilds/{}/roles", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildRoleMemberCounts { guild_id: Snowflake },
    api!("/guilds/{}/roles/member-counts", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildScheduledEvent { guild_id: Snowflake, event_id: Snowflake },
    api!("/guilds/{}/scheduled-events/{}", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildScheduledEvents { guild_id: Snowflake },
    api!("/guilds/{}/scheduled-events", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildScheduledEventUsers { guild_id: Snowflake, event_id: Snowflake },
    api!("/guilds/{}/scheduled-events/{}/users", guild_id, event_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildSticker { guild_id: Snowflake, sticker_id: Snowflake },
    api!("/guilds/{}/stickers/{}", guild_id, sticker_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildStickers { guild_id: Snowflake },
    api!("/guilds/{}/stickers", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildVanityUrl { guild_id: Snowflake },
    api!("/guilds/{}/vanity-url", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildVoiceStates { guild_id: Snowflake, user_id: Snowflake },
    api!("/guilds/{}/voice-states/{}", guild_id, user_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildVoiceStateMe { guild_id: Snowflake },
    api!("/guilds/{}/voice-states/@me", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildWebhooks { guild_id: Snowflake },
    api!("/guilds/{}/webhooks", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildWelcomeScreen { guild_id: Snowflake },
    api!("/guilds/{}/welcome-screen", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildThreadsActive { guild_id: Snowflake },
    api!("/guilds/{}/threads/active", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildIncidentActions { guild_id: Snowflake },
    api!("/guilds/{}/incident-actions", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    Guilds,
    api!("/guilds"),
    Some(RatelimitingKind::Path);

    Invite { code: &'a str },
    api!("/invites/{}", code),
    Some(RatelimitingKind::Path);

    OAuth2Token,
    api!("/oauth2/token"),
    None;

    OAuth2TokenRevocation,
    api!("/oauth2/token/revoke"),
    None;

    OAuth2ApplicationCurrent,
    api!("/oauth2/applications/@me"),
    None;

    OAuth2AuthorizationCurrent,
    api!("/oauth2/@me"),
    None;

    SoundboardSend { channel_id: Snowflake },
    api!("/channels/{}/send-soundboard-sound", channel_id),
    Some(RatelimitingKind::PathAndId(channel_id));

    SoundboardDefaultSounds,
    api!("/soundboard-default-sounds"),
    Some(RatelimitingKind::Path);

    GuildSoundboards { guild_id: Snowflake },
    api!("/guilds/{}/soundboard-sounds", guild_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    GuildSoundboard { guild_id: Snowflake, sound_id: Snowflake },
    api!("/guilds/{}/soundboard-sounds/{}", guild_id, sound_id),
    Some(RatelimitingKind::PathAndId(guild_id));

    StatusIncidentsUnresolved,
    status!("/incidents/unresolved.json"),
    None;

    StatusMaintenancesActive,
    status!("/scheduled-maintenances/active.json"),
    None;

    StatusMaintenancesUpcoming,
    status!("/scheduled-maintenances/upcoming.json"),
    None;

    Sticker { sticker_id: Snowflake },
    api!("/stickers/{}", sticker_id),
    Some(RatelimitingKind::Path);

    StickerPacks,
    api!("/sticker-packs"),
    Some(RatelimitingKind::Path);

    StickerPack { sticker_pack_id: Snowflake },
    api!("/sticker-packs/{}", sticker_pack_id),
    Some(RatelimitingKind::Path);

    User { user_id: Snowflake },
    api!("/users/{}", user_id),
    Some(RatelimitingKind::Path);

    UserMe,
    api!("/users/@me"),
    Some(RatelimitingKind::Path);

    UserMeConnections,
    api!("/users/@me/connections"),
    Some(RatelimitingKind::Path);

    UserMeDmChannels,
    api!("/users/@me/channels"),
    Some(RatelimitingKind::Path);

    UserMeGuild { guild_id: Snowflake },
    api!("/users/@me/guilds/{}", guild_id),
    Some(RatelimitingKind::Path);

    UserMeGuildMember { guild_id: Snowflake },
    api!("/users/@me/guilds/{}/member", guild_id),
    Some(RatelimitingKind::Path);

    UserMeGuilds,
    api!("/users/@me/guilds"),
    Some(RatelimitingKind::Path);

    VoiceRegions,
    api!("/voice/regions"),
    Some(RatelimitingKind::Path);

    Webhook { webhook_id: Snowflake },
    api!("/webhooks/{}", webhook_id),
    Some(RatelimitingKind::PathAndId(webhook_id));

    WebhookWithToken { webhook_id: Snowflake, token: &'a str },
    api!("/webhooks/{}/{}", webhook_id, token),
    Some(RatelimitingKind::PathAndId(webhook_id));

    WebhookMessage { webhook_id: Snowflake, token: &'a str, message_id: Snowflake },
    api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id),
    Some(RatelimitingKind::PathAndId(webhook_id));

    WebhookOriginalInteractionResponse { application_id: Snowflake, token: &'a str },
    api!("/webhooks/{}/{}/messages/@original", application_id, token),
    Some(RatelimitingKind::PathAndId(application_id));

    WebhookFollowupMessage { application_id: Snowflake, token: &'a str, message_id: Snowflake },
    api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id),
    Some(RatelimitingKind::PathAndId(application_id));

    WebhookFollowupMessages { application_id: Snowflake, token: &'a str },
    api!("/webhooks/{}/{}", application_id, token),
    Some(RatelimitingKind::PathAndId(application_id));

    InteractionResponse { interaction_id: Snowflake, token: &'a str },
    api!("/interactions/{}/{}/callback", interaction_id, token),
    Some(RatelimitingKind::PathAndId(interaction_id));

    Command { application_id: Snowflake, command_id: Snowflake },
    api!("/applications/{}/commands/{}", application_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Commands { application_id: Snowflake },
    api!("/applications/{}/commands", application_id),
    Some(RatelimitingKind::PathAndId(application_id));

    GuildCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
    api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id));

    GuildCommandPermissions { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
    api!("/applications/{}/guilds/{}/commands/{}/permissions", application_id, guild_id, command_id),
    Some(RatelimitingKind::PathAndId(application_id));

    GuildCommands { application_id: Snowflake, guild_id: Snowflake },
    api!("/applications/{}/guilds/{}/commands", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(application_id));

    GuildCommandsPermissions { application_id: Snowflake, guild_id: Snowflake },
    api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Skus { application_id: Snowflake },
    api!("/applications/{}/skus", application_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Emoji { application_id: Snowflake, emoji_id: Snowflake },
    api!("/applications/{}/emojis/{}", application_id, emoji_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Emojis { application_id: Snowflake },
    api!("/applications/{}/emojis", application_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Entitlement { application_id: Snowflake, entitlement_id: Snowflake },
    api!("/applications/{}/entitlements/{}", application_id, entitlement_id),
    Some(RatelimitingKind::PathAndId(application_id));

    ConsumeEntitlement { application_id: Snowflake, entitlement_id: Snowflake },
    api!("/applications/{}/entitlements/{}/consume", application_id, entitlement_id),
    Some(RatelimitingKind::PathAndId(application_id));

    Entitlements { application_id: Snowflake },
    api!("/applications/{}/entitlements", application_id),
    Some(RatelimitingKind::PathAndId(application_id));

    StageInstances,
    api!("/stage-instances"),
    Some(RatelimitingKind::Path);

    StageInstance { channel_id: Snowflake },
    api!("/stage-instances/{}", channel_id),
    Some(RatelimitingKind::Path);
});
