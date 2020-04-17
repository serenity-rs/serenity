use tokio::sync::Mutex;
use std::{
    sync::{
        Arc,
    }
};
use super::{ShardManager, ShardManagerMessage};
use super::super::gateway::ShardId;
use log::{debug, warn};
use futures::{
    channel::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
    StreamExt,
};

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
    /// The mpsc Sender channel to inform the manager that a shard has just properly shut down
    pub shutdown: Sender<ShardId>,
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
    pub async fn run(&mut self) {
        debug!("Starting shard manager worker");

        while let Some(value) = self.rx.next().await {

            match value {
                ShardManagerMessage::Restart(shard_id) => {
                    self.manager.lock().await.restart(shard_id).await;
                    let _  = self.shutdown.unbounded_send(shard_id);
                },
                ShardManagerMessage::ShardUpdate { id, latency, stage } => {
                    let manager = self.manager.lock().await;
                    let mut runners = manager.runners.lock().await;

                    if let Some(runner) = runners.get_mut(&id) {
                        runner.latency = latency;
                        runner.stage = stage;
                    }
                }
                ShardManagerMessage::Shutdown(shard_id, code) => {
                    self.manager.lock().await.shutdown(shard_id, code).await;
                    let _ = self.shutdown.unbounded_send(shard_id);
                },
                ShardManagerMessage::ShutdownAll => {
                    self.manager.lock().await.shutdown_all().await;

                    break;
                },
                ShardManagerMessage::ShutdownInitiated => break,
                ShardManagerMessage::ShutdownFinished(shard_id) => {
                    if let Err(why) = self.shutdown.unbounded_send(shard_id) {
                        warn!(
                            "[ShardMonitor] Could not forward Shutdown signal to ShardManager for shard {}: {:#?}",
                            shard_id,
                            why
                        );
                    }
                }
            }
        }
        warn!("deads");
    }
}
