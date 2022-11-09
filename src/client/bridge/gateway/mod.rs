//! The client gateway bridge is support essential for the [`client`] module.
//!
//! This is made available for user use if one wishes to be lower-level or avoid
//! the higher functionality of the [`Client`].
//!
//! Of interest are three pieces:
//!
//! ### [`ShardManager`]
//!
//! The shard manager is responsible for being a clean interface between the
//! user and the shard runners, providing essential functions such as
//! [`ShardManager::shutdown`] to shutdown a shard and [`ShardManager::restart`]
//! to restart a shard.
//!
//! If you are using the `Client`, this is likely the only piece of interest to
//! you. Refer to [its documentation][`ShardManager`] for more information.
//!
//! ### [`ShardQueuer`]
//!
//! The shard queuer is a light wrapper around an mpsc receiver that receives
//! [`ShardManagerMessage`]s. It should be run in its own thread so it can
//! receive messages to start shards in a queue.
//!
//! Refer to [its documentation][`ShardQueuer`] for more information.
//!
//! ### [`ShardRunner`]
//!
//! The shard runner is responsible for actually running a shard and
//! communicating with its respective WebSocket client.
//!
//! It is performs all actions such as sending a presence update over the client
//! and, with the help of the [`Shard`], will be able to determine what to do.
//! This is, for example, whether to reconnect, resume, or identify with the
//! gateway.
//!
//! ### In Conclusion
//!
//! For almost every - if not every - use case, you only need to _possibly_ be
//! concerned about the [`ShardManager`] in this module.
//!
//! [`client`]: crate::client
//! [`Client`]: crate::Client
//! [`Shard`]: crate::gateway::Shard

pub mod event;

mod shard_manager;
mod shard_manager_monitor;
mod shard_messenger;
mod shard_queuer;
mod shard_runner;
mod shard_runner_message;

use std::fmt;
use std::time::Duration as StdDuration;

pub use self::shard_manager::{ShardManager, ShardManagerOptions};
pub use self::shard_manager_monitor::{ShardManagerError, ShardManagerMonitor};
pub use self::shard_messenger::ShardMessenger;
pub use self::shard_queuer::ShardQueuer;
pub use self::shard_runner::{ShardRunner, ShardRunnerOptions};
pub use self::shard_runner_message::{ChunkGuildFilter, ShardRunnerMessage};
use crate::gateway::ConnectionStage;
use crate::model::event::Event;

/// A message either for a [`ShardManager`] or a [`ShardRunner`].
#[derive(Debug)]
pub enum ShardClientMessage {
    /// A message intended to be worked with by a [`ShardManager`].
    Manager(ShardManagerMessage),
    /// A message intended to be worked with by a [`ShardRunner`].
    Runner(Box<ShardRunnerMessage>),
}

/// A message for a [`ShardManager`] relating to an operation with a shard.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ShardManagerMessage {
    /// Indicator that a [`ShardManagerMonitor`] should restart a shard.
    Restart(ShardId),
    /// An update from a shard runner,
    ShardUpdate { id: ShardId, latency: Option<StdDuration>, stage: ConnectionStage },
    /// Indicator that a [`ShardManagerMonitor`] should fully shutdown a shard
    /// without bringing it back up.
    Shutdown(ShardId, u16),
    /// Indicator that a [`ShardManagerMonitor`] should fully shutdown all shards
    /// and end its monitoring process for the [`ShardManager`].
    ShutdownAll,
    /// Indicator that a [`ShardManager`] has initiated a shutdown, and for the
    /// component that receives this to also shutdown with no further action
    /// taken.
    ShutdownInitiated,
    /// Indicator that a [`ShardRunner`] has finished the shutdown of a shard, allowing it to
    /// move toward the next one.
    ShutdownFinished(ShardId),
    /// Indicator that a shard sent invalid authentication (a bad token) when identifying with the gateway.
    /// Emitted when a shard receives an [`InvalidAuthentication`] Error
    ///
    /// [`InvalidAuthentication`]: crate::gateway::GatewayError::InvalidAuthentication
    ShardInvalidAuthentication,
    /// Indicator that a shard provided undocumented gateway intents.
    /// Emitted when a shard received an [`InvalidGatewayIntents`] error.
    ///
    /// [`InvalidGatewayIntents`]: crate::gateway::GatewayError::InvalidGatewayIntents
    ShardInvalidGatewayIntents,
    /// If a connection has been established but privileged gateway intents
    /// were provided without enabling them prior.
    /// Emitted when a shard received a [`DisallowedGatewayIntents`] error.
    ///
    /// [`DisallowedGatewayIntents`]: crate::gateway::GatewayError::DisallowedGatewayIntents
    ShardDisallowedGatewayIntents,
}

/// A message to be sent to the [`ShardQueuer`].
///
/// This should usually be wrapped in a [`ShardClientMessage`].
#[derive(Clone, Debug)]
pub enum ShardQueuerMessage {
    /// Message to start a shard, where the 0-index element is the ID of the
    /// Shard to start and the 1-index element is the total shards in use.
    Start(ShardId, ShardId),
    /// Message to shutdown the shard queuer.
    Shutdown,
    /// Message to dequeue/shutdown a shard.
    ShutdownShard(ShardId, u16),
}

/// A light tuplestruct wrapper around a u32 to verify type correctness when
/// working with the IDs of shards.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShardId(pub u32);

impl fmt::Display for ShardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a [`ShardRunner`].
///
/// The [`ShardId`] is not included because, as it stands, you probably already
/// know the Id if you obtained this.
#[derive(Debug)]
pub struct ShardRunnerInfo {
    /// The latency between when a heartbeat was sent and when the
    /// acknowledgement was received.
    pub latency: Option<StdDuration>,
    /// The channel used to communicate with the shard runner, telling it
    /// what to do with regards to its status.
    pub runner_tx: ShardMessenger,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}

impl AsRef<ShardMessenger> for ShardRunnerInfo {
    fn as_ref(&self) -> &ShardMessenger {
        &self.runner_tx
    }
}

/// Newtype around a callback that will be called on every incoming request. As long as this
/// collector should still receive events, it should return `true`. Once it returns `false`, it is
/// removed.
pub struct CollectorCallback(pub Box<dyn Fn(&Event) -> bool + Send + Sync>);
impl std::fmt::Debug for CollectorCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CollectorCallback").finish()
    }
}
