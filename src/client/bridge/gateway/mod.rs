mod shard_manager;
mod shard_queuer;
mod shard_runner;

pub use self::shard_manager::ShardManager;
pub use self::shard_queuer::ShardQueuer;
pub use self::shard_runner::ShardRunner;

use gateway::Shard;
use parking_lot::Mutex;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::mpsc::Sender;
use std::sync::Arc;

type Parked<T> = Arc<Mutex<T>>;
type LockedShard = Parked<Shard>;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ShardManagerMessage {
    Restart(ShardId),
    Shutdown(ShardId),
    #[allow(dead_code)]
    ShutdownAll,
}

pub enum ShardQueuerMessage {
    /// Message to start a shard, where the 0-index element is the ID of the
    /// Shard to start and the 1-index element is the total shards in use.
    Start(ShardId, ShardId),
    /// Message to shutdown the shard queuer.
    Shutdown,
}

// A light tuplestruct wrapper around a u64 to verify type correctness when
// working with the IDs of shards.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ShardId(pub u64);

impl Display for ShardId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

pub struct ShardRunnerInfo {
    runner_tx: Sender<ShardManagerMessage>,
}
