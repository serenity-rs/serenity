//! The client gateway bridge is support essential for the [`client`] module.
//!
//! This is made available for user use if one wishes to be lower-level or avoid the higher
//! functionality of the [`Client`].
//!
//! Of interest are three pieces:
//!
//! ### [`ShardManager`]
//!
//! The shard manager is responsible for being a clean interface between the user and the shard
//! runners, providing essential functions such as [`ShardManager::shutdown`] to shutdown a shard
//! and [`ShardManager::restart`] to restart a shard.
//!
//! If you are using the `Client`, this is likely the only piece of interest to you. Refer to [its
//! documentation][`ShardManager`] for more information.
//!
//! ### [`ShardQueuer`]
//!
//! The shard queuer is a light wrapper around an mpsc receiver that receives
//! [`ShardQueuerMessage`]s. It should be run in its own thread so it can receive messages to
//! start shards in a queue.
//!
//! Refer to [its documentation][`ShardQueuer`] for more information.
//!
//! ### [`ShardRunner`]
//!
//! The shard runner is responsible for actually running a shard and communicating with its
//! respective WebSocket client.
//!
//! It is performs all actions such as sending a presence update over the client and, with the help
//! of the [`Shard`], will be able to determine what to do. This is, for example, whether to
//! reconnect, resume, or identify with the gateway.
//!
//! ### In Conclusion
//!
//! For almost every - if not every - use case, you only need to _possibly_ be concerned about the
//! [`ShardManager`] in this module.
//!
//! [`client`]: crate::client
//! [`Client`]: crate::Client
//! [`Shard`]: crate::gateway::Shard

mod event;
mod shard_manager;
mod shard_messenger;
mod shard_queuer;
mod shard_runner;
mod shard_runner_message;
#[cfg(feature = "voice")]
mod voice;

use std::fmt;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::Duration as StdDuration;

pub use self::event::ShardStageUpdateEvent;
pub use self::shard_manager::{ShardManager, ShardManagerOptions};
pub use self::shard_messenger::ShardMessenger;
pub use self::shard_queuer::{ShardQueue, ShardQueuer};
pub use self::shard_runner::{ShardRunner, ShardRunnerOptions};
pub use self::shard_runner_message::ShardRunnerMessage;
#[cfg(feature = "voice")]
pub use self::voice::VoiceGatewayManager;
use super::ChunkGuildFilter;
use crate::gateway::ConnectionStage;
use crate::model::event::Event;
use crate::model::id::ShardId;

/// A message to be sent to the [`ShardQueuer`].
#[derive(Clone, Debug)]
pub enum ShardQueuerMessage {
    /// Message to set the shard total.
    SetShardTotal(NonZeroU16),
    /// Message to start a shard.
    Start { shard_id: ShardId, concurrent: bool },
    /// Message to shutdown the shard queuer.
    Shutdown,
    /// Message to dequeue/shutdown a shard.
    ShutdownShard { shard_id: ShardId, code: u16 },
}

/// Information about a [`ShardRunner`].
///
/// The [`ShardId`] is not included because, as it stands, you probably already know the Id if you
/// obtained this.
#[derive(Debug)]
pub struct ShardRunnerInfo {
    /// The latency between when a heartbeat was sent and when the acknowledgement was received.
    pub latency: Option<StdDuration>,
    /// The channel used to communicate with the shard runner, telling it what to do with regards
    /// to its status.
    pub runner_tx: ShardMessenger,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}

/// Newtype around a callback that will be called on every incoming request. As long as this
/// collector should still receive events, it should return `true`. Once it returns `false`, it is
/// removed.
#[derive(Clone)]
pub struct CollectorCallback(pub Arc<dyn Fn(&Event) -> bool + Send + Sync>);

impl std::fmt::Debug for CollectorCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CollectorCallback").finish()
    }
}

impl PartialEq for CollectorCallback {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
