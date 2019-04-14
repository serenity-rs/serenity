//! The client gateway bridge is support essential for the [`client`][client] module.
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
//! [client]: ../../index.html
//! [`Client`]: ../../struct.Client.html
//! [`Shard`]: ../../../gateway/struct.Shard.html
//! [`ShardManager`]: struct.ShardManager.html
//! [`ShardManager::restart`]: struct.ShardManager.html#method.restart
//! [`ShardManager::shutdown`]: struct.ShardManager.html#method.shutdown
//! [`ShardManagerMessage`]: enum.ShardManagerMessage.html
//! [`ShardQueue`]: struct.ShardQueuer.html
//! [`ShardRunner`]: struct.ShardRunner.html

pub mod event;

mod shard_manager;
mod shard_manager_monitor;
mod shard_messenger;
mod shard_queuer;
mod shard_runner;
mod shard_runner_message;

pub use self::shard_manager::{ShardManager, ShardManagerOptions};
pub use self::shard_manager_monitor::ShardManagerMonitor;
pub use self::shard_messenger::ShardMessenger;
pub use self::shard_queuer::ShardQueuer;
pub use self::shard_runner::{ShardRunner, ShardRunnerOptions};
pub use self::shard_runner_message::ShardRunnerMessage;

use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    },
    sync::mpsc::Sender,
    time::Duration as StdDuration
};
use crate::gateway::{ConnectionStage, InterMessage};

/// A message either for a [`ShardManager`] or a [`ShardRunner`].
///
/// [`ShardManager`]: struct.ShardManager.html
/// [`ShardRunner`]: struct.ShardRunner.html
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ShardClientMessage {
    /// A message intended to be worked with by a [`ShardManager`].
    ///
    /// [`ShardManager`]: struct.ShardManager.html
    Manager(ShardManagerMessage),
    /// A message intended to be worked with by a [`ShardRunner`].
    ///
    /// [`ShardRunner`]: struct.ShardRunner.html
    Runner(ShardRunnerMessage),
}

/// A message for a [`ShardManager`] relating to an operation with a shard.
///
/// [`ShardManager`]: struct.ShardManager.html
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ShardManagerMessage {
    /// Indicator that a [`ShardManagerMonitor`] should restart a shard.
    ///
    /// [`ShardManagerMonitor`]: struct.ShardManagerMonitor.html
    Restart(ShardId),
    /// An update from a shard runner,
    ShardUpdate {
        id: ShardId,
        latency: Option<StdDuration>,
        stage: ConnectionStage,
    },
    /// Indicator that a [`ShardManagerMonitor`] should fully shutdown a shard
    /// without bringing it back up.
    ///
    /// [`ShardManagerMonitor`]: struct.ShardManagerMonitor.html
    Shutdown(ShardId),
    /// Indicator that a [`ShardManagerMonitor`] should fully shutdown all shards
    /// and end its monitoring process for the [`ShardManager`].
    ///
    /// [`ShardManager`]: struct.ShardManager.html
    /// [`ShardManagerMonitor`]: struct.ShardManagerMonitor.html
    ShutdownAll,
    /// Indicator that a [`ShardManager`] has initiated a shutdown, and for the
    /// component that receives this to also shutdown with no further action
    /// taken.
    ShutdownInitiated,
}

/// A message to be sent to the [`ShardQueuer`].
///
/// This should usually be wrapped in a [`ShardClientMessage`].
///
/// [`ShardClientMessage`]: enum.ShardClientMessage.html
/// [`ShardQueuer`]: struct.ShardQueuer.html
#[derive(Clone, Debug)]
pub enum ShardQueuerMessage {
    /// Message to start a shard, where the 0-index element is the ID of the
    /// Shard to start and the 1-index element is the total shards in use.
    Start(ShardId, ShardId),
    /// Message to shutdown the shard queuer.
    Shutdown,
}

/// A light tuplestruct wrapper around a u64 to verify type correctness when
/// working with the IDs of shards.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ShardId(pub u64);

impl Display for ShardId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

/// Information about a [`ShardRunner`].
///
/// The [`ShardId`] is not included because, as it stands, you probably already
/// know the Id if you obtained this.
///
/// [`ShardId`]: struct.ShardId.html
/// [`ShardRunner`]: struct.ShardRunner.html
#[derive(Debug)]
pub struct ShardRunnerInfo {
    /// The latency between when a heartbeat was sent and when the
    /// acknowledgement was received.
    pub latency: Option<StdDuration>,
    /// The channel used to communicate with the shard runner, telling it
    /// what to do with regards to its status.
    pub runner_tx: Sender<InterMessage>,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}
