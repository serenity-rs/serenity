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
    /// The visibility of this connection.
    pub visibility: ConnectionVisibility,
}

/// The visibility of a user connection on a user's profile.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#connection-object-visibility-types).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ConnectionVisibility {
    None = 0,
    Everyone = 1,
    Unknown = !0,
}

enum_number!(ConnectionVisibility {
    None,
    Everyone
});
