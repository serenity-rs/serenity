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
pub mod constants;
mod error;
pub mod sharding;
#[cfg(feature = "voice")]
mod voice;
mod ws;

#[cfg(feature = "http")]
use reqwest::IntoUrl;
use reqwest::Url;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::{Serialize, Serializer};
use serde_json::value::RawValue;

use self::constants::Opcode;
pub use self::error::Error as GatewayError;
pub use self::sharding::*;
#[cfg(feature = "voice")]
pub use self::voice::VoiceGatewayManager;
pub use self::ws::WsClient;
use crate::error::CoreError;
use crate::internal::prelude::*;
use crate::model::event::Event;
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
            url: Some(url.into_url().map_err(CoreError::from)?),
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

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#request-guild-members).
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

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#payload-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GatewayEvent {
    Dispatch {
        seq: u64,
        event: DeserializedEvent,
    },
    Heartbeat,
    Reconnect,
    /// Whether the session can be resumed.
    InvalidateSession(bool),
    Hello(u64),
    HeartbeatAck,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum DeserializedEvent {
    Success(Box<Event>),
    Unknown(UnknownEvent),
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct UnknownEvent {
    #[cfg_attr(feature = "typesize", typesize(with = raw_value_len))]
    pub data: Box<RawValue>,
    pub err: String,
}

impl Serialize for UnknownEvent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "typesize")]
fn raw_value_len(val: &RawValue) -> usize {
    val.get().len()
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug, Clone, Deserialize)]
        struct GatewayEventRaw<'a> {
            op: Opcode,
            #[serde(rename = "s")]
            seq: Option<u64>,
            #[serde(rename = "d")]
            data: &'a RawValue,
            #[serde(rename = "t")]
            ty: Option<&'a str>,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;

        let raw = GatewayEventRaw::deserialize(raw_data).map_err(DeError::custom)?;

        Ok(match raw.op {
            Opcode::Dispatch => {
                if raw.ty.is_none() {
                    return Err(DeError::missing_field("t"));
                }

                Self::Dispatch {
                    seq: raw.seq.ok_or_else(|| DeError::missing_field("s"))?,
                    event: match Deserialize::deserialize(raw_data) {
                        Ok(event) => DeserializedEvent::Success(event),
                        Err(e) => DeserializedEvent::Unknown(UnknownEvent {
                            data: Deserialize::deserialize(raw_data).map_err(DeError::custom)?,
                            err: e.to_string(),
                        }),
                    },
                }
            },
            Opcode::Heartbeat => Self::Heartbeat,
            Opcode::InvalidSession => {
                Self::InvalidateSession(bool::deserialize(raw.data).map_err(DeError::custom)?)
            },
            Opcode::Hello => {
                #[derive(Deserialize)]
                struct HelloPayload {
                    heartbeat_interval: u64,
                }

                let inner = HelloPayload::deserialize(raw.data).map_err(DeError::custom)?;

                Self::Hello(inner.heartbeat_interval)
            },
            Opcode::Reconnect => Self::Reconnect,
            Opcode::HeartbeatAck => Self::HeartbeatAck,
            _ => return Err(DeError::custom("invalid opcode")),
        })
    }
}
