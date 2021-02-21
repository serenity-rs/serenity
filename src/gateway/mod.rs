//! The gateway module contains the pieces - primarily the `Shard` -
//! responsible for maintaing a WebSocket connection with Discord.
//!
//! A shard is an interface for the lower-level receiver and sender. It provides
//! what can otherwise be thought of as "sugar methods". A shard represents a
//! single connection to Discord. You can make use of a method named "sharding"
//! to have multiple shards, potentially offloading some server load to another
//! server(s).
//!
//! # Sharding
//!
//! Sharding is a method to split portions of bots into separate processes. This
//! is an enforced strategy by Discord once a bot reaches a certain number of
//! guilds (2500). Once this number is reached, a bot must be sharded in a way
//! that only 2500 guilds maximum may be allocated per shard.
//!
//! The "recommended" number of guilds per shard is _around_ 1000. Sharding can
//! be useful for splitting processes across separate servers. Often you may
//! want some or all shards to be in the same process, allowing for a shared
//! State. This is possible through this library.
//!
//! See [Discord's documentation][docs] for more information.
//!
//! If you are not using a bot account or do not require sharding - such as for
//! a small bot - then use [`Client::start`].
//!
//! There are a few methods of sharding available:
//!
//! - [`Client::start_autosharded`]: retrieves the number of shards Discord
//! recommends using from the API, and then automatically starts that number of
//! shards.
//! - [`Client::start_shard`]: starts a single shard for use in the instance,
//! handled by the instance of the Client. Use this if you only want 1 shard
//! handled by this instance.
//! - [`Client::start_shards`]: starts all shards in this instance. This is best
//! for when you want a completely shared State.
//! - [`Client::start_shard_range`]: start a range of shards within this
//! instance. This should be used when you, for example, want to split 10 shards
//! across 3 instances.
//!
//! [`Client`]: crate::Client
//! [`Client::start`]: crate::Client::start
//! [`Client::start_autosharded`]: crate::Client::start_autosharded
//! [`Client::start_shard`]: crate::Client::start_shard
//! [`Client::start_shard_range`]: crate::Client::start_shard_range
//! [`Client::start_shards`]: crate::Client::start_shards
//! [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding

mod error;
mod shard;
mod ws_client_ext;

use std::fmt::{Display, Formatter, Result as FmtResult};

pub use self::{
    error::Error as GatewayError,
    shard::Shard,
    ws_client_ext::WebSocketGatewayClientExt,
};
#[cfg(feature = "client")]
use crate::client::bridge::gateway::ShardClientMessage;
use crate::json::Value;
use crate::model::{gateway::Activity, user::OnlineStatus};

pub type CurrentPresence = (Option<Activity>, OnlineStatus);

use async_tungstenite::{tokio::ConnectStream, WebSocketStream};

pub type WsStream = WebSocketStream<ConnectStream>;

/// Indicates the current connection stage of a [`Shard`].
///
/// This can be useful for knowing which shards are currently "down"/"up".
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ConnectionStage {
    /// Indicator that the [`Shard`] is normally connected and is not in, e.g.,
    /// a resume phase.
    Connected,
    /// Indicator that the [`Shard`] is connecting and is in, e.g., a resume
    /// phase.
    Connecting,
    /// Indicator that the [`Shard`] is fully disconnected and is not in a
    /// reconnecting phase.
    Disconnected,
    /// Indicator that the [`Shard`] is currently initiating a handshake.
    Handshake,
    /// Indicator that the [`Shard`] has sent an IDENTIFY packet and is awaiting
    /// a READY packet.
    Identifying,
    /// Indicator that the [`Shard`] has sent a RESUME packet and is awaiting a
    /// RESUMED packet.
    Resuming,
}

impl ConnectionStage {
    /// Whether the stage is a form of connecting.
    ///
    /// This will return `true` on:
    ///
    /// - [`Connecting`][`ConnectionStage::Connecting`]
    /// - [`Handshake`][`ConnectionStage::Handshake`]
    /// - [`Identifying`][`ConnectionStage::Identifying`]
    /// - [`Resuming`][`ConnectionStage::Resuming`]
    ///
    /// All other variants will return `false`.
    ///
    /// # Examples
    ///
    /// Assert that [`ConnectionStage::Identifying`] is a connecting stage:
    ///
    /// ```rust
    /// use serenity::gateway::ConnectionStage;
    ///
    /// assert!(ConnectionStage::Identifying.is_connecting());
    /// ```
    ///
    /// Assert that [`ConnectionStage::Connected`] is _not_ a connecting stage:
    ///
    /// ```rust
    /// use serenity::gateway::ConnectionStage;
    ///
    /// assert!(!ConnectionStage::Connected.is_connecting());
    /// ```
    pub fn is_connecting(self) -> bool {
        use self::ConnectionStage::*;

        match self {
            Connecting | Handshake | Identifying | Resuming => true,
            Connected | Disconnected => false,
        }
    }
}

impl Display for ConnectionStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use self::ConnectionStage::*;

        f.write_str(match *self {
            Connected => "connected",
            Connecting => "connecting",
            Disconnected => "disconnected",
            Handshake => "handshaking",
            Identifying => "identifying",
            Resuming => "resuming",
        })
    }
}

/// A message to be passed around within the library.
///
/// As a user you usually don't need to worry about this, but when working with
/// the lower-level internals of the `client`, `gateway, and `voice` modules it
/// may be necessary.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum InterMessage {
    #[cfg(feature = "client")]
    Client(Box<ShardClientMessage>),
    Json(Value),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ShardAction {
    Heartbeat,
    Identify,
    Reconnect(ReconnectType),
}

/// The type of reconnection that should be performed.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReconnectType {
    /// Indicator that a new connection should be made by sending an IDENTIFY.
    Reidentify,
    /// Indicator that a new connection should be made by sending a RESUME.
    Resume,
}
