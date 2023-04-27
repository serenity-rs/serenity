//! Models for user connections.

use super::prelude::*;

/// Information about a connection between the current user and a third party service.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#connection-object-connection-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Connection {
    /// The ID of the account on the other side of this connection.
    pub id: String,
    /// The username of the account on the other side of this connection.
    pub name: String,
    /// The service that this connection represents (e.g. twitch, youtube)
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/user#connection-object-services).
    #[serde(rename = "type")]
    pub kind: String,
    /// Whether this connection has been revoked and is no longer valid.
    #[serde(default)]
    pub revoked: bool,
    /// A list of partial guild [`Integration`]s that use this connection.
    #[serde(default)]
    pub integrations: Vec<Integration>,
    /// Whether this connection has been verified and the user has proven they own the account.
    pub verified: bool,
    /// Whether friend sync is enabled for this connection.
    pub friend_sync: bool,
    /// Whether activities related to this connection will be shown in presence updates.
    pub show_activity: bool,
    /// Whether this connection has a corresponding third party OAuth2 token.
    pub two_way_link: bool,
    /// The visibility of this connection.
    pub visibility: ConnectionVisibility,
}

enum_number! {
    /// The visibility of a user connection on a user's profile.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/user#connection-object-visibility-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ConnectionVisibility {
        /// Invisible to everyone except the user themselves
        None = 0,
        /// Visible to everyone
        Everyone = 1,
        _ => Unknown(u8),
    }
}
