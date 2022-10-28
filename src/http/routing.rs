use std::num::NonZeroU64;

use super::LightMethod;
use crate::model::id::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct RatelimitBucket(Option<(std::mem::Discriminant<Route<'static>>, Option<NonZeroU64>)>);

macro_rules! routes {
    ($lt:lifetime $(
        // First param is the ratelimiting policy: `ratelimit_by ChannelId`
        $(ratelimit_by $ratelimit_by_id:ident)?
        $(ratelimit $($ratelimit_dummy:ident)?)?
        $(dont_ratelimit $($dont_ratelimit_dummy:ident)?)?,
        // Second param is the route name and parameters: `Channel { channel_id: ChannelId }`
        $name:ident { $($arg_name:ident: $arg_type:ty),* },
        // Third param is the format string for the path: `api!("/channels/{}/invites", channel_id)`
        $path:expr
    );* $(;)?) => {
        pub enum Route<$lt> { $(
            $name { $($arg_name: $arg_type),* },
        )* }

        impl Route<'_> {
            pub fn path(&self) -> String {
                match self { $(
                    Self::$name( $($arg_name),* ) => $path,
                )* }
            }

            pub fn bucket(&self) -> Option<impl Eq> {
                let discriminant = std::mem::discriminant(self);
                match self { $(
                    Self::$name( $($arg_name),* ) => RatelimitBucket(
                        $( Some((discriminant, Some($ratelimit_by_id.0))) )?
                        $( Some((discriminant, None)) $($ratelimit_dummy)? )?
                        $( None $($dont_ratelimit_dummy)? )?
                    ),
                )* }
            }
        }
    };
}

routes!('a
    ratelimit_by channel_id, Channel { channel_id: ChannelId }, api!("/channels/{}", channel_id);
    ratelimit_by channel_id, ChannelInvites { channel_id: ChannelId }, api!("/channels/{}/invites", channel_id);
    ratelimit_by channel_id, ChannelMessage { channel_id: ChannelId, message_id: MessageId }, api!("/channels/{}/messages/{}", channel_id, message_id);
    ratelimit_by channel_id, ChannelMessageCrosspost { channel_id: ChannelId, message_id: MessageId }, api!("/channels/{}/messages/{}/crosspost", channel_id, message_id);
    ratelimit_by channel_id, ChannelMessageReaction { channel_id: ChannelId, message_id: MessageId, user_id: UserId, reaction: &'a str }, api!("/channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, reaction, user_id);
    ratelimit_by channel_id, ChannelMessageReactionMe { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }, api!("/channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, reaction);
    ratelimit_by channel_id, ChannelMessageReactionEmoji { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }, api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction);
    ratelimit_by channel_id, ChannelMessageReactions { channel_id: ChannelId, message_id: MessageId }, api!("/channels/{}/messages/{}/reactions", channel_id, message_id);
    ratelimit_by channel_id, ChannelMessageReactionsList { channel_id: ChannelId, message_id: MessageId, reaction: &'a str }, api!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, reaction);
    ratelimit_by channel_id, ChannelMessages { channel_id: ChannelId }, api!("/channels/{}/messages", channel_id);
    ratelimit_by channel_id, ChannelMessagesBulkDelete { channel_id: ChannelId }, api!("/channels/{}/messages/bulk-delete", channel_id);
    ratelimit_by channel_id, ChannelFollowNews { channel_id: ChannelId }, api!("/channels/{}/followers", channel_id);
    ratelimit_by channel_id, ChannelPermission { channel_id: ChannelId, target_id: TargetId }, api!("/channels/{}/permissions/{}", channel_id, target_id);
    ratelimit_by channel_id, ChannelPin { channel_id: ChannelId, message_id: MessageId }, api!("/channels/{}/pins/{}", channel_id, message_id);
    ratelimit_by channel_id, ChannelPins { channel_id: ChannelId }, api!("/channels/{}/pins", channel_id);
    ratelimit_by channel_id, ChannelTyping { channel_id: ChannelId }, api!("/channels/{}/typing", channel_id);
    ratelimit_by channel_id, ChannelWebhooks { channel_id: ChannelId }, api!("/channels/{}/webhooks", channel_id);
    ratelimit_by channel_id, ChannelPublicThreads { channel_id: ChannelId, message_id: MessageId }, api!("/channels/{}/messages/{}/threads", channel_id, message_id);
    ratelimit_by channel_id, ChannelPrivateThreads { channel_id: ChannelId }, api!("/channels/{}/threads", channel_id);
    ratelimit_by channel_id, ChannelThreadMember { channel_id: ChannelId, user_id: UserId }, api!("/channels/{}/thread-members/{}", channel_id, user_id);
    ratelimit_by channel_id, ChannelThreadMemberMe { channel_id: ChannelId }, api!("/channels/{}/thread-members/@me", channel_id);
    ratelimit_by channel_id, ChannelThreadMembers { channel_id: ChannelId }, api!("/channels/{}/thread-members", channel_id);
    ratelimit_by channel_id, ChannelArchivedPublicThreads { channel_id: ChannelId }, api!("/channels/{}/threads/archived/public", channel_id);
    ratelimit_by channel_id, ChannelArchivedPrivateThreads { channel_id: ChannelId }, api!("/channels/{}/threads/archived/private", channel_id);
    ratelimit_by channel_id, ChannelJoinedPrivateThreads { channel_id: ChannelId }, api!("/channels/{}/users/@me/threads/archived/private", channel_id);
    ratelimit, Gateway {}, api!("/gateway");
    ratelimit, GatewayBot {}, api!("/gateway/bot");
    ratelimit_by guild_id, Guild { guild_id: GuildId }, api!("/guilds/{}", guild_id);
    ratelimit_by guild_id, GuildAuditLogs { guild_id: GuildId }, api!("/guilds/{}/audit-logs", guild_id);
    ratelimit_by guild_id, GuildAutomodRule { guild_id: GuildId, rule_id: RuleId }, api!("/guilds/{}/auto-moderation/rules/{}", guild_id, rule_id);
    ratelimit_by guild_id, GuildAutomodRules { guild_id: GuildId }, api!("/guilds/{}/auto-moderation/rules", guild_id);
    ratelimit_by guild_id, GuildBan { guild_id: GuildId, user_id: UserId }, api!("/guilds/{}/bans/{}", guild_id, user_id);
    ratelimit_by guild_id, GuildKick { guild_id: GuildId, user_id: UserId }, api!("/guilds/{}/members/{}", guild_id, user_id);
    ratelimit_by guild_id, GuildBans { guild_id: GuildId }, api!("/guilds/{}/bans", guild_id);
    ratelimit_by guild_id, GuildChannels { guild_id: GuildId }, api!("/guilds/{}/channels", guild_id);
    ratelimit_by guild_id, GuildWidget { guild_id: GuildId }, api!("/guilds/{}/widget", guild_id);
    ratelimit_by guild_id, GuildPreview { guild_id: GuildId }, api!("/guilds/{}/preview", guild_id);
    ratelimit_by guild_id, GuildEmojis { guild_id: GuildId }, api!("/guilds/{}/emojis", guild_id);
    ratelimit_by guild_id, GuildEmoji { guild_id: GuildId, emoji_id: EmojiId }, api!("/guilds/{}/emojis/{}", guild_id, emoji_id);
    ratelimit_by guild_id, GuildIntegration { guild_id: GuildId, integration_id: IntegrationId }, api!("/guilds/{}/integrations/{}", guild_id, integration_id);
    ratelimit_by guild_id, GuildIntegrationSync { guild_id: GuildId, integration_id: IntegrationId }, api!("/guilds/{}/integrations/{}/sync", guild_id, integration_id);
    ratelimit_by guild_id, GuildIntegrations { guild_id: GuildId }, api!("/guilds/{}/integrations", guild_id);
    ratelimit_by guild_id, GuildInvites { guild_id: GuildId }, api!("/guilds/{}/invites", guild_id);
    ratelimit_by guild_id, GuildMember { guild_id: GuildId, user_id: UserId }, api!("/guilds/{}/members/{}", guild_id, user_id);
    ratelimit_by guild_id, GuildMemberRole { guild_id: GuildId, user_id: UserId, role_id: RoleId }, api!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id);
    ratelimit_by guild_id, GuildMembers { guild_id: GuildId }, api!("/guilds/{}/members", guild_id);
    ratelimit_by guild_id, GuildMembersSearch { guild_id: GuildId, query: &'a str }, api!("/guilds/{}/members/search", guild_id);
    ratelimit_by guild_id, GuildMemberMe { guild_id: GuildId }, api!("/guilds/{}/members/@me", guild_id);
    ratelimit_by guild_id, GuildNickname { guild_id: GuildId }, api!("/guilds/{}/members/@me/nick", guild_id);
    ratelimit_by guild_id, GuildPrune { guild_id: GuildId, days: u8 }, api!("/guilds/{}/prune", guild_id);
    ratelimit_by guild_id, GuildRegions { guild_id: GuildId }, api!("/guilds/{}/regions", guild_id);
    ratelimit_by guild_id, GuildRole { guild_id: GuildId, role_id: RoleId }, api!("/guilds/{}/roles/{}", guild_id, role_id);
    ratelimit_by guild_id, GuildRoles { guild_id: GuildId }, api!("/guilds/{}/roles", guild_id);
    ratelimit_by guild_id, GuildScheduledEvent { guild_id: GuildId, event_id: ScheduledEventId }, api!("/guilds/{}/scheduled-events/{}", guild_id, event_id);
    ratelimit_by guild_id, GuildScheduledEvents { guild_id: GuildId }, api!("/guilds/{}/scheduled-events", guild_id);
    ratelimit_by guild_id, GuildScheduledEventUsers { guild_id: GuildId, event_id: ScheduledEventId }, api!("/guilds/{}/scheduled-events/{}/users", guild_id, event_id);
    ratelimit_by guild_id, GuildSticker { guild_id: GuildId, sticker_id: StickerId }, api!("/guilds/{}/stickers/{}", guild_id, sticker_id);
    ratelimit_by guild_id, GuildStickers { guild_id: GuildId }, api!("/guilds/{}/stickers", guild_id);
    ratelimit_by guild_id, GuildVanityUrl { guild_id: GuildId }, api!("/guilds/{}/vanity-url", guild_id);
    ratelimit_by guild_id, GuildVoiceStates { guild_id: GuildId, user_id: UserId }, api!("/guilds/{}/voice-states/{}", guild_id, user_id);
    ratelimit_by guild_id, GuildVoiceStateMe { guild_id: GuildId }, api!("/guilds/{}/voice-states/@me", guild_id);
    ratelimit_by guild_id, GuildWebhooks { guild_id: GuildId }, api!("/guilds/{}/webhooks", guild_id);
    ratelimit_by guild_id, GuildWelcomeScreen { guild_id: GuildId }, api!("/guilds/{}/welcome-screen", guild_id);
    ratelimit_by guild_id, GuildThreadsActive { guild_id: GuildId }, api!("/guilds/{}/threads/active", guild_id);
    ratelimit, Guilds {}, api!("/guilds");
    ratelimit, Invite { code: &'a str }, api!("/invites/{}", code);
    dont_ratelimit, Oauth2ApplicationCurrent {}, api!("/oauth2/applications/@me");
    ratelimit, PrivateChannel {}, api!("/users/@me/channels");
    dont_ratelimit, StatusIncidentsUnresolved {}, status!("/incidents/unresolved.json");
    dont_ratelimit, StatusMaintenancesActive {}, status!("/scheduled-maintenances/active.json");
    dont_ratelimit, StatusMaintenancesUpcoming {}, status!("/scheduled-maintenances/upcoming.json");
    ratelimit, Sticker { sticker_id: StickerId }, api!("/stickers/{}", sticker_id);
    ratelimit, StickerPacks {}, api!("/sticker-packs");
    ratelimit, User { user_id: UserId }, api!("/users/{}", user_id);
    ratelimit, UserMe {}, api!("/users/@me");
    ratelimit, UserMeConnections {}, api!("/users/@me/connections");
    ratelimit, UserDmChannels { user_id: UserId }, api!("/users/{}/channels", user_id);
    ratelimit, UserMeDmChannels {}, api!("/users/@me/channels");
    ratelimit, UserGuild { user_id: UserId, guild_id: GuildId }, api!("/users/{}/guilds/{}", user_id, guild_id);
    ratelimit, UserMeGuild { guild_id: GuildId }, api!("/users/@me/guilds/{}", guild_id);
    ratelimit, UserGuilds { user_id: UserId }, api!("/users/{}/guilds", user_id);
    ratelimit, UserMeGuilds {}, api!("/users/@me/guilds");
    ratelimit, VoiceRegions {}, api!("/voice/regions");
    ratelimit_by webhook_id, Webhook { webhook_id: WebhookId }, api!("/webhooks/{}", webhook_id);
    ratelimit_by webhook_id, WebhookWithToken { webhook_id: WebhookId, token: &'a str }, api!("/webhooks/{}/{}", webhook_id, token);
    ratelimit_by webhook_id, WebhookMessage { webhook_id: WebhookId, token: &'a str, message_id: MessageId }, api!("/webhooks/{}/{}/messages/{}", webhook_id, token, message_id);
    ratelimit_by application_id, WebhookOriginalInteractionResponse { application_id: ApplicationId, token: &'a str }, api!("/webhooks/{}/{}/messages/@original", application_id, token);
    ratelimit_by application_id, WebhookFollowupMessage { application_id: ApplicationId, token: &'a str, message_id: MessageId }, api!("/webhooks/{}/{}/messages/{}", application_id, token, message_id);
    ratelimit_by application_id, WebhookFollowupMessages { application_id: ApplicationId, token: &'a str }, api!("/webhooks/{}/{}", application_id, token);
    ratelimit_by interaction_id, InteractionResponse { interaction_id: InteractionId, token: &'a str }, api!("/interactions/{}/{}/callback", interaction_id, token);
    ratelimit_by application_id, ApplicationCommand { application_id: ApplicationId, command_id: CommandId }, api!("/applications/{}/commands/{}", application_id, command_id);
    ratelimit_by application_id, ApplicationCommands { application_id: ApplicationId }, api!("/applications/{}/commands", application_id);
    ratelimit_by application_id, ApplicationGuildCommand { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId }, api!("/applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id);
    ratelimit_by application_id, ApplicationGuildCommandPermissions { application_id: ApplicationId, guild_id: GuildId, command_id: CommandId }, api!("/applications/{}/guilds/{}/commands/{}/permissions", application_id, guild_id, command_id);
    ratelimit_by application_id, ApplicationGuildCommands { application_id: ApplicationId, guild_id: GuildId }, api!("/applications/{}/guilds/{}/commands", application_id, guild_id);
    ratelimit_by application_id, ApplicationGuildCommandsPermissions { application_id: ApplicationId, guild_id: GuildId }, api!("/applications/{}/guilds/{}/commands/permissions", application_id, guild_id);
    ratelimit, StageInstances {}, api!("/stage-instances");
    ratelimit, StageInstance { channel_id: ChannelId }, api!("/stage-instances/{}", channel_id);
);
