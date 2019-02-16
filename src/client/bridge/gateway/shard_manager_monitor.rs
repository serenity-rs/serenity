use parking_lot::Mutex;
use std::sync::{
    mpsc::Receiver,
    Arc
};
use super::{ShardManager, ShardManagerMessage};
use log::debug;

/// The shard manager monitor does what it says on the tin -- it monitors the
/// shard manager and performs actions on it as received.
///
/// The monitor is essentially responsible for running in its own thread and
/// receiving [`ShardManagerMessage`]s, such as whether to shutdown a shard or
/// shutdown everything entirely.
///
/// [`ShardManagerMessage`]: enum.ShardManagerMessage.html
#[derive(Debug)]
pub struct ShardManagerMonitor {
    /// An clone of the Arc to the manager itself.
    pub manager: Arc<Mutex<ShardManager>>,
    /// The mpsc Receiver channel to receive shard manager messages over.
    pub rx: Receiver<ShardManagerMessage>,
}

impl ShardManagerMonitor {
    /// "Runs" the monitor, waiting for messages over the Receiver.
    ///
    /// This should be called in its own thread due to its blocking, looped
    /// nature.
    ///
    /// This will continue running until either:
    ///
    /// - a [`ShardManagerMessage::ShutdownAll`] has been received
    /// - an error is returned while receiving a message from the
    /// channel (probably indicating that the shard manager should stop anyway)
    ///
    /// [`ShardManagerMessage::ShutdownAll`]: enum.ShardManagerMessage.html#variant.ShutdownAll
    pub fn run(&mut self) {
        debug!("Starting shard manager worker");

        while let Ok(value) = self.rx.recv() {
            match value {
                ShardManagerMessage::Restart(shard_id) => {
                    self.manager.lock().restart(shard_id);
                },
                ShardManagerMessage::ShardUpdate { id, latency, stage } => {
                    let manager = self.manager.lock();
                    let mut runners = manager.runners.lock();

                    if let Some(runner) = runners.get_mut(&id) {
                        runner.latency = latency;
                        runner.stage = stage;
                    }
                }
                ShardManagerMessage::Shutdown(shard_id) => {
                    self.manager.lock().shutdown(shard_id);
                },
                ShardManagerMessage::ShutdownAll => {
                    self.manager.lock().shutdown_all();

                    break;
                },
                ShardManagerMessage::ShutdownInitiated => break,
            }
        }
    }
}
