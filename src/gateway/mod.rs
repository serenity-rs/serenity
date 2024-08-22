//! Contains the necessary plumping for maintaining a connection with Discord.
//! The primary building blocks are the [`Client`] and the [`Shard`].
//!
//! The [`Client`] is a high-level interface that takes care of communicating with Discord's REST
//! API as well as receiving and dispatching events from the gateway using a WebSocket client.
//!
//! On the other hand, the [`Shard`] is a low-level receiver and sender representing a single
//! connection to Discord. The client will handle shard management automatically for you, so you
//! should only care about using it directly if you really need to. See the [`sharding`] module for
//! details and documentation.
//!
//! [`Client`]: client::Client

pub mod client;
mod error;
pub mod sharding;
#[cfg(feature = "voice")]
mod voice;
mod ws;

#[cfg(feature = "http")]
use reqwest::IntoUrl;
use reqwest::Url;

pub use self::error::Error as GatewayError;
pub use self::sharding::*;
#[cfg(feature = "voice")]
pub use self::voice::VoiceGatewayManager;
pub use self::ws::WsClient;
use crate::internal::prelude::*;
use crate::model::gateway::{Activity, ActivityType};
use crate::model::id::UserId;
use crate::model::user::OnlineStatus;

/// Presence data of the current user.
#[derive(Clone, Debug, Default)]
pub struct PresenceData {
    /// The current activity, if present
    pub activity: Option<ActivityData>,
    /// The current online status
    pub status: OnlineStatus,
}

/// Activity data of the current user.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActivityData {
    /// The name of the activity
    pub name: FixedString,
    /// The type of the activity
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// The state of the activity, if the type is [`ActivityType::Custom`]
    pub state: Option<FixedString>,
    /// The url of the activity, if the type is [`ActivityType::Streaming`]
    pub url: Option<Url>,
}

impl ActivityData {
    /// Creates an activity that appears as `Playing <name>`.
    #[must_use]
    pub fn playing(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trunc_into(),
            kind: ActivityType::Playing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Streaming <name>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL parsing fails.
    #[cfg(feature = "http")]
    pub fn streaming(name: impl Into<String>, url: impl IntoUrl) -> Result<Self> {
        Ok(Self {
            name: name.into().trunc_into(),
            kind: ActivityType::Streaming,
            state: None,
            url: Some(url.into_url()?),
        })
    }

    /// Creates an activity that appears as `Listening to <name>`.
    #[must_use]
    pub fn listening(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trunc_into(),
            kind: ActivityType::Listening,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Watching <name>`.
    #[must_use]
    pub fn watching(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trunc_into(),
            kind: ActivityType::Watching,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Competing in <name>`.
    #[must_use]
    pub fn competing(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trunc_into(),
            kind: ActivityType::Competing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `<state>`.
    #[must_use]
    pub fn custom(state: impl Into<String>) -> Self {
        Self {
            // discord seems to require a name for custom activities
            // even though it's not displayed
            name: FixedString::from_static_trunc("~"),
            kind: ActivityType::Custom,
            state: Some(state.into().trunc_into()),
            url: None,
        }
    }
}

impl From<Activity> for ActivityData {
    fn from(activity: Activity) -> Self {
        Self {
            name: activity.name,
            kind: activity.kind,
            state: activity.state,
            url: activity.url,
        }
    }
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#request-guild-members).
#[derive(Clone, Debug)]
pub enum ChunkGuildFilter {
    /// Returns all members of the guilds specified. Requires GUILD_MEMBERS intent.
    None,
    /// A common username prefix filter for the members returned.
    ///
    /// Will return a maximum of 100 members.
    Query(String),
    /// A set of exact user IDs to query for.
    ///
    /// Will return a maximum of 100 members.
    UserIds(Vec<UserId>),
}
