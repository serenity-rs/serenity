use std::num::NonZeroU64;

use super::LightMethod;
use crate::model::id::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct RatelimitBucket(Option<(std::mem::Discriminant<Route<'static>>, Option<NonZeroU64>)>);
enum Ratelimit {
    ByRouteAndId(NonZeroU64),
    ByRoute,
    Dont,
}

macro_rules! routes {
    ($lt:lifetime $(
        $name:ident { $($arg_name:ident: $arg_type:ty),* }
        => $path:expr,
        $ratelimit:expr;
    )*) => {
        pub enum Route<$lt> { $(
            $name { $($arg_name: $arg_type),* },
        )* }

        impl Route<'_> {
            pub fn path(&self) -> String {
                match self { $(
                    Self::$name { $($arg_name),* } => $path,
                )* }
            }

            pub fn bucket(&self) -> RatelimitBucket {
                let ratelimit = match self { $(
                    Self::$name { $($arg_name),* } => $ratelimit,
                )* };
                match ratelimit {
                    Ratelimit::ByRouteAndId(id) => RatelimitBucket(Some(std::mem::discriminant(self), Some(id))),
                    Ratelimit::ByRoute => RatelimitBucket(Some(std::mem::discriminant(self), None)),
                    Ratelimit::Dont => RatelimitBucket(None),
                }
            }
        }
    };
}

// This macro creates the Route enum, which contains all API routes of the Discord API.
routes!('a
    // SYNTAX:
    // - First line is the definition of the enum variant
    // - Second line maps it to the URL
    // - Third line gives the ratelimiting policy:
    //   - `Ratelimit::ByRouteAndId`: requests with the same route and FIELD_NAME value are
    //     ratelimited together
    //   - `Ratelimit::ByRoute`: requests with the same route are ratelimited together, independent
    //      of their fields
    //   - `Ratelimit::Dont`: requests to this route aren't ratelimited at all

    Channel { channel_id: ChannelId }
    => api!("/channels/{}", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelInvites { channel_id: ChannelId }
    => api!("/channels/{}/invites", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessage { channel_id: ChannelId, message_id: MessageId }
    => api!("/channels/{}/messages/{}", channel_id, message_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageCrosspost { channel_id: ChannelId, message_id: MessageId }
    => api!("/channels/{}/messages/{}/crosspost", channel_id, message_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageReaction { channel_id: ChannelId, message_id: MessageId, user_id: UserId, reaction: &'a str }
    => api!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, reaction, user_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageReactionMe { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }
    => api!("/channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, reaction),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageReactionEmoji { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }
    => api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageReactions { channel_id: ChannelId, message_id: MessageId }
    => api!("/channels/{}/messages/{}/reactions", channel_id, message_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessageReactionsList { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }
    => api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessages { channel_id: ChannelId }
    => api!("/channels/{}/messages", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelMessagesBulkDelete { channel_id: ChannelId }
    => api!("/channels/{}/messages/bulk-delete", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelFollowNews { channel_id: ChannelId }
    => api!("/channels/{}/followers", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelPermission { channel_id: ChannelId, target_id: TargetId }
    => api!("/channels/{}/permissions/{}", channel_id, target_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelPin { channel_id: ChannelId, message_id: MessageId }
    => api!("/channels/{}/pins/{}", channel_id, message_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelPins { channel_id: ChannelId }
    => api!("/channels/{}/pins", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelTyping { channel_id: ChannelId }
    => api!("/channels/{}/typing", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelWebhooks { channel_id: ChannelId }
    => api!("/channels/{}/webhooks", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelPublicThreads { channel_id: ChannelId, message_id: MessageId }
    => api!("/channels/{}/messages/{}/threads", channel_id, message_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelPrivateThreads { channel_id: ChannelId }
    => api!("/channels/{}/threads", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelThreadMember { channel_id: ChannelId, user_id: UserId }
    => api!("/channels/{}/thread-members/{}", channel_id, user_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelThreadMemberMe { channel_id: ChannelId }
    => api!("/channels/{}/thread-members/@me", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelThreadMembers { channel_id: ChannelId }
    => api!("/channels/{}/thread-members", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelArchivedPublicThreads { channel_id: ChannelId }
    => api!("/channels/{}/threads/archived/public", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelArchivedPrivateThreads { channel_id: ChannelId }
    => api!("/channels/{}/threads/archived/private", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    ChannelJoinedPrivateThreads { channel_id: ChannelId }
    => api!("/channels/{}/users/@me/threads/archived/private", channel_id),
    Ratelimit::ByRouteAndId(channel_id.0);

    Gateway {}
    => api!("/gateway"),
    Ratelimit::ByRoute;

    GatewayBot {}
    => api!("/gateway/bot"),
    Ratelimit::ByRoute;

    Guild { guild_id: GuildId }
    => api!("/guilds/{}", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildAuditLogs { guild_id: GuildId }
    => api!("/guilds/{}/audit-logs", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildAutomodRule { guild_id: GuildId, rule_id: RuleId }
    => api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildAutomodRules { guild_id: GuildId }
    => api!("/guilds/{}/auto-moderation/rules", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildBan { guild_id: GuildId, user_id: UserId }
    => api!("/guilds/{}/bans/{}", guild_id, user_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildKick { guild_id: GuildId, user_id: UserId }
    => api!("/guilds/{}/members/{}", guild_id, user_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildBans { guild_id: GuildId }
    => api!("/guilds/{}/bans", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildChannels { guild_id: GuildId }
    => api!("/guilds/{}/channels", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildWidget { guild_id: GuildId }
    => api!("/guilds/{}/widget", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildPreview { guild_id: GuildId }
    => api!("/guilds/{}/preview", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildEmojis { guild_id: GuildId }
    => api!("/guilds/{}/emojis", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildEmoji { guild_id: GuildId, emoji_id: EmojiId }
    => api!("/guilds/{}/emojis/{}", guild_id, emoji_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildIntegration { guild_id: GuildId, integration_id: IntegrationId }
    => api!("/guilds/{}/integrations/{}", guild_id, integration_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildIntegrationSync { guild_id: GuildId, integration_id: IntegrationId }
    => api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildIntegrations { guild_id: GuildId }
    => api!("/guilds/{}/integrations", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildInvites { guild_id: GuildId }
    => api!("/guilds/{}/invites", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildMember { guild_id: GuildId, user_id: UserId }
    => api!("/guilds/{}/members/{}", guild_id, user_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildMemberRole { guild_id: GuildId, user_id: UserId, role_id: RoleId }
    => api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildMembers { guild_id: GuildId }
    => api!("/guilds/{}/members", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildMembersSearch { guild_id: GuildId, query: &'a str }
    => api!("/guilds/{}/members/search", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildMemberMe { guild_id: GuildId }
    => api!("/guilds/{}/members/@me", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildNickname { guild_id: GuildId }
    => api!("/guilds/{}/members/@me/nick", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildPrune { guild_id: GuildId, days: u8 }
    => api!("/guilds/{}/prune", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildRegions { guild_id: GuildId }
    => api!("/guilds/{}/regions", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildRole { guild_id: GuildId, role_id: RoleId }
    => api!("/guilds/{}/roles/{}", guild_id, role_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildRoles { guild_id: GuildId }
    => api!("/guilds/{}/roles", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildScheduledEvent { guild_id: GuildId, event_id: ScheduledEventId }
    => api!("/guilds/{}/scheduled-events/{}", guild_id, event_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildScheduledEvents { guild_id: GuildId }
    => api!("/guilds/{}/scheduled-events", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildScheduledEventUsers { guild_id: GuildId, event_id: ScheduledEventId }
    => api!("/guilds/{}/scheduled-events/{}/users", guild_id, event_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildSticker { guild_id: GuildId, sticker_id: StickerId }
    => api!("/guilds/{}/stickers/{}", guild_id, sticker_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildStickers { guild_id: GuildId }
    => api!("/guilds/{}/stickers", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildVanityUrl { guild_id: GuildId }
    => api!("/guilds/{}/vanity-url", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildVoiceStates { guild_id: GuildId, user_id: UserId }
    => api!("/guilds/{}/voice-states/{}", guild_id, user_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildVoiceStateMe { guild_id: GuildId }
    => api!("/guilds/{}/voice-states/@me", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildWebhooks { guild_id: GuildId }
    => api!("/guilds/{}/webhooks", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildWelcomeScreen { guild_id: GuildId }
    => api!("/guilds/{}/welcome-screen", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    GuildThreadsActive { guild_id: GuildId }
    => api!("/guilds/{}/threads/active", guild_id),
    Ratelimit::ByRouteAndId(guild_id.0);

    Guilds {}
    => api!("/guilds"),
    Ratelimit::ByRoute;

    Invite { code: &'a str }
    => api!("/invites/{}", code),
    Ratelimit::ByRoute;

    Oauth2ApplicationCurrent {}
    => api!("/oauth2/applications/@me"),
    Ratelimit::Dont;

    PrivateChannel {}
    => api!("/users/@me/channels"),
    Ratelimit::ByRoute;

    StatusIncidentsUnresolved {}
    => status!("/incidents/unresolved.json"),
    Ratelimit::Dont;

    StatusMaintenancesActive {}
    => status!("/scheduled-maintenances/active.json"),
    Ratelimit::Dont;

    StatusMaintenancesUpcoming {}
    => status!("/scheduled-maintenances/upcoming.json"),
    Ratelimit::Dont;

    Sticker { sticker_id: StickerId }
    => api!("/stickers/{}", sticker_id),
    Ratelimit::ByRoute;

    StickerPacks {}
    => api!("/sticker-packs"),
    Ratelimit::ByRoute;

    User { user_id: UserId }
    => api!("/users/{}", user_id),
    Ratelimit::ByRoute;

    UserMe {}
    => api!("/users/@me"),
    Ratelimit::ByRoute;

    UserMeConnections {}
    => api!("/users/@me/connections"),
    Ratelimit::ByRoute;

    UserDmChannels { user_id: UserId }
    => api!("/users/{}/channels", user_id),
    Ratelimit::ByRoute;

    UserMeDmChannels {}
    => api!("/users/@me/channels"),
    Ratelimit::ByRoute;

    UserGuild { user_id: UserId, guild_id: GuildId }
    => api!("/users/{}/guilds/{}", user_id, guild_id),
    Ratelimit::ByRoute;

    UserMeGuild { guild_id: GuildId }
    => api!("/users/@me/guilds/{}", guild_id),
    Ratelimit::ByRoute;

    UserGuilds { user_id: UserId }
    => api!("/users/{}/guilds", user_id),
    Ratelimit::ByRoute;

    UserMeGuilds {}
    => api!("/users/@me/guilds"),
    Ratelimit::ByRoute;

    VoiceRegions {}
    => api!("/voice/regions"),
    Ratelimit::ByRoute;

    Webhook { webhook_id: WebhookId }
    => api!("/webhooks/{}", webhook_id),
    Ratelimit::ByRouteAndId(webhook_id.0);

    WebhookWithToken { webhook_id: WebhookId, token: &'a str }
    => api!("/webhooks/{}/{}", webhook_id, token),
    Ratelimit::ByRouteAndId(webhook_id.0);

    WebhookMessage { webhook_id: WebhookId, token: &'a str, message_id: MessageId }
    => api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id),
    Ratelimit::ByRouteAndId(webhook_id.0);

    WebhookOriginalInteractionResponse { application_id: ApplicationId, token: &'a str }
    => api!("/webhooks/{}/{}/messages/@original", application_id, token),
    Ratelimit::ByRouteAndId(application_id.0);

    WebhookFollowupMessage { application_id: ApplicationId, token: &'a str, message_id: MessageId }
    => api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id),
    Ratelimit::ByRouteAndId(application_id.0);

    WebhookFollowupMessages { application_id: ApplicationId, token: &'a str }
    => api!("/webhooks/{}/{}", application_id, token),
    Ratelimit::ByRouteAndId(application_id.0);

    InteractionResponse { interaction_id: InteractionId, token: &'a str }
    => api!("/interactions/{}/{}/callback", interaction_id, token),
    Ratelimit::ByRouteAndId(interaction_id.0);

    ApplicationCommand { application_id: ApplicationId, command_id: CommandId }
    => api!("/applications/{}/commands/{}", application_id, command_id),
    Ratelimit::ByRouteAndId(application_id.0);

    ApplicationCommands { application_id: ApplicationId }
    => api!("/applications/{}/commands", application_id),
    Ratelimit::ByRouteAndId(application_id.0);

    ApplicationGuildCommand { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId }
    => api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id),
    Ratelimit::ByRouteAndId(application_id.0);

    ApplicationGuildCommandPermissions { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId }
    => api!("/applications/{}/guilds/{}/commands/{}/permissions", application_id, guild_id, command_id),
    Ratelimit::ByRouteAndId(application_id.0);

    ApplicationGuildCommands { application_id: ApplicationId, guild_id: GuildId }
    => api!("/applications/{}/guilds/{}/commands", application_id, guild_id),
    Ratelimit::ByRouteAndId(application_id.0);

    ApplicationGuildCommandsPermissions { application_id: ApplicationId, guild_id: GuildId }
    => api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id),
    Ratelimit::ByRouteAndId(application_id.0);

    StageInstances {}
    => api!("/stage-instances"),
    Ratelimit::ByRoute;

    StageInstance { channel_id: ChannelId }
    => api!("/stage-instances/{}", channel_id),
    Ratelimit::ByRoute;
);
