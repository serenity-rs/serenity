use std::fmt;

use serde::{Deserialize, Serialize};

/// The available OAuth2 Scopes.
///
/// [Discord docs](https://discord.com/developers/docs/topics/oauth2#shared-resources-oauth2-scopes).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Scope {
    /// For oauth2 bots, this puts the bot in the user's selected guild by default.
    #[serde(rename = "bot")]
    Bot,
    /// Allows your app to use Slash Commands in a guild.
    #[serde(rename = "applications.commands")]
    ApplicationsCommands,
    /// Allows your app to update its Slash Commands via this bearer token - client credentials
    /// grant only.
    #[serde(rename = "applications.commands.update")]
    ApplicationsCommandsUpdate,
    /// Allows your app to update permissions for its commands in a guild a user has permissions
    /// to.
    #[serde(rename = "applications.commands.permissions.update")]
    ApplicationsCommandsPermissionsUpdate,
    /// Allows `/users/@me` without [`Self::Email`].
    #[serde(rename = "identify")]
    Identify,
    /// Enables `/users/@me` to return an `email` field.
    #[serde(rename = "email")]
    Email,
    /// Allows `/users/@me/connections` to return linked third-party accounts.
    #[serde(rename = "connections")]
    Connections,
    /// Allows `/users/@me/guilds` to return basic information about all of a user's guilds.
    #[serde(rename = "guilds")]
    Guilds,
    /// Allows `/guilds/{guild.id}/members/{user.id}` to be used for joining users to a guild.
    #[serde(rename = "guilds.join")]
    GuildsJoin,
    /// Allows `/users/@me/guilds/{guild.id}/member` to return a user's member information in a
    /// guild.
    #[serde(rename = "guilds.members.read")]
    GuildsMembersRead,
    /// Allows your app to join users to a group dm.
    #[serde(rename = "gdm.join")]
    GdmJoin,
    /// For local rpc server access, this allows you to control a user's local Discord client -
    /// requires Discord approval.
    #[serde(rename = "rpc")]
    Rpc,
    /// For local rpc server api access, this allows you to receive notifications pushed out to the
    /// user - requires Discord approval.
    #[serde(rename = "rpc.notifications.read")]
    RpcNotificationsRead,
    #[serde(rename = "rpc.voice.read")]
    RpcVoiceRead,
    #[serde(rename = "rpc.voice.write")]
    RpcVoiceWrite,
    #[serde(rename = "rpc.activities.write")]
    RpcActivitiesWrite,
    /// This generates a webhook that is returned in the oauth token response for authorization
    /// code grants.
    #[serde(rename = "webhook.incoming")]
    WebhookIncomming, // TODO: fix misspelling
    /// For local rpc server api access, this allows you to read messages from all client channels
    /// (otherwise restricted to channels/guilds your app creates).
    #[serde(rename = "messages.read")]
    MessagesRead,
    /// Allows your app to upload/update builds for a user's applications - requires Discord
    /// approval.
    #[serde(rename = "applications.builds.upload")]
    ApplicationsBuildsUpload,
    /// Allows your app to read build data for a user's applications.
    #[serde(rename = "applications.builds.read")]
    ApplicationsBuildsRead,
    /// Allows your app to read and update store data (SKUs, store listings, achievements, etc.)
    /// for a user's applications.
    #[serde(rename = "applications.store.update")]
    ApplicationsStoreUpdate,
    /// Allows your app to read entitlements for a user's applications.
    #[serde(rename = "applications.entitlements")]
    ApplicationsEntitlements,
    /// Allows your app to fetch data from a user's "Now Playing/Recently Played" list - requires
    /// Discord approval.
    #[serde(rename = "activities.read")]
    ActivitiesRead,
    /// Allows your app to update a user's activity - requires Discord approval (Not required for
    /// gamesdk activity manager!).
    #[serde(rename = "activities.write")]
    ActivitiesWrite,
    /// Allows your app to know a user's friends and implicit relationships - requires Discord
    /// approval.
    #[serde(rename = "relationships.read")]
    RelationshipsRead,
    /// Allows your app to see information about the user's DMs and group DMs - requires Discord
    /// approval.
    #[serde(rename = "dm_channels.read")]
    DmChannelsRead,
    /// Allows your app to connect to voice on user's behalf and see all the voice members -
    /// requires Discord approval.
    #[serde(rename = "voice")]
    Voice,
    /// Allows your app to update a user's connection and metadata for the app.
    #[serde(rename = "role_connections.write")]
    RoleConnectionsWrite,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}
