use std::fmt;

/// The available OAuth2 Scopes.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OAuth2Scope {
    /// For oauth2 bots, this puts the bot in the user's selected guild by default.
    Bot,
    /// Allows your app to use Slash Commands in a guild.
    ApplicationsCommands,
    /// Allows your app to update its Slash Commands via this bearer token - client credentials grant only.
    ApplicationsCommandsUpdate,

    /// Allows `/users/@me` without [`Self::Email`].
    Identify,
    /// Enables `/users/@me` to return an `email` field.
    Email,
    /// Allows `/users/@me/connections` to return linked third-party accounts.
    Connections,
    /// Allows `/users/@me/guilds` to return basic information about all of a user's guilds.
    Guilds,
    /// Allows `/guilds/{guild.id}/members/{user.id}` to be used for joining users to a guild.
    GuildsJoin,
    /// Allows your app to join users to a group dm.
    GdmJoin,
    /// For local rpc server access, this allows you to control a user's local Discord client -
    /// requires Discord approval.
    Rpc,
    /// For local rpc server api access, this allows you to receive notifications pushed out to the user - requires Discord approval.
    RpcNotificationsRead,
    RpcVoiceRead,
    RpcVoiceWrite,
    RpcActivitiesWrite,
    /// This generates a webhook that is returned in the oauth token response for authorization code grants.
    WebhookIncomming,
    /// For local rpc server api access, this allows you to read messages from all client channels
    /// (otherwise restricted to channels/guilds your app creates).
    MessagesRead,
    /// Allows your app to upload/update builds for a user's applications - requires Discord approval.
    ApplicationsBuildsUpload,
    /// Allows your app to read build data for a user's applications.
    ApplicationsBuildsRead,
    /// Allows your app to read and update store data (SKUs, store listings, achievements, etc.) for a user's applications.
    ApplicationsStoreUpdate,
    /// Allows your app to read entitlements for a user's applications.
    ApplicationsEntitlements,
    /// Allows your app to fetch data from a user's "Now Playing/Recently Played" list - requires Discord approval.
    ActivitiesRead,
    /// allows your app to update a user's activity - requires Discord approval (Not required for gamesdk activity manager!).
    ActivitiesWrite,
    /// Allows your app to know a user's friends and implicit relationships - requires Discord approval.
    RelactionshipsRead,
}

impl fmt::Display for OAuth2Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            Self::Bot => "bot",
            Self::ApplicationsCommands => "applications.commands",
            Self::ApplicationsCommandsUpdate => "applications.commands.update",
            Self::Identify => "identify",
            Self::Email => "email",
            Self::Connections => "connections",
            Self::Guilds => "guilds",
            Self::GuildsJoin => "guilds.join",
            Self::GdmJoin => "gdm.join",
            Self::Rpc => "rpc",
            Self::RpcNotificationsRead => "rpc.notifications.read",
            Self::RpcVoiceRead => "rpc.voice.read",
            Self::RpcVoiceWrite => "rpc.voice.write",
            Self::RpcActivitiesWrite => "rpc.activities.write",
            Self::WebhookIncomming => "webhook.incoming",
            Self::MessagesRead => "messages.read",
            Self::ApplicationsBuildsUpload => "applications.builds.upload",
            Self::ApplicationsBuildsRead => "applications.builds.read",
            Self::ApplicationsStoreUpdate => "applications.store.update",
            Self::ApplicationsEntitlements => "applications.entitlements",
            Self::ActivitiesRead => "activities.read",
            Self::ActivitiesWrite => "activities.write",
            Self::RelactionshipsRead => "relationships.read",
        };

        write!(f, "{}", val)
    }
}
