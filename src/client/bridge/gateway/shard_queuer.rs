use gateway::Shard;
use internal::prelude::*;
use parking_lot::Mutex as ParkingLotMutex;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use super::super::super::EventHandler;
use super::{
    ShardId,
    ShardManagerMessage,
    ShardQueuerMessage,
    ShardRunner,
    ShardRunnerInfo,
};
use typemap::ShareMap;

#[cfg(feature = "framework")]
use framework::Framework;

/// The shard queuer is a simple loop that runs indefinitely to manage the
/// startup of shards.
///
/// A shard queuer instance _should_ be run in its own thread, due to the
/// blocking nature of the loop itself as well as a 5 second thread sleep
/// between shard starts.
pub struct ShardQueuer<H: EventHandler + Send + Sync + 'static> {
    pub data: Arc<ParkingLotMutex<ShareMap>>,
    pub event_handler: Arc<H>,
    #[cfg(feature = "framework")]
    pub framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
    pub last_start: Option<Instant>,
    pub manager_tx: Sender<ShardManagerMessage>,
    pub runners: Arc<ParkingLotMutex<HashMap<ShardId, ShardRunnerInfo>>>,
    pub rx: Receiver<ShardQueuerMessage>,
    pub token: Arc<Mutex<String>>,
    pub ws_url: Arc<Mutex<String>>,
}

impl<H: EventHandler + Send + Sync + 'static> ShardQueuer<H> {
    pub fn run(&mut self) {
        loop {
            let msg = match self.rx.recv() {
                Ok(msg) => msg,
                Err(_) => {
                    break;
                }
            };

            match msg {
                ShardQueuerMessage::Shutdown => break,
                ShardQueuerMessage::Start(shard_id, shard_total) => {
                    self.check_last_start();

                    if let Err(why) = self.start(shard_id, shard_total) {
                        warn!("Err starting shard {}: {:?}", shard_id, why);
                    }

                    self.last_start = Some(Instant::now());
                },
            }
        }
    }

    fn check_last_start(&mut self) {
        let instant = match self.last_start {
            Some(instant) => instant,
            None => return,
        };

        // We must wait 5 seconds between IDENTIFYs to avoid session
        // invalidations.
        let duration = Duration::from_secs(5);
        let elapsed = instant.elapsed();

        if elapsed >= duration {
            return;
        }

        let to_sleep = duration - elapsed;

        thread::sleep(to_sleep);
    }

    fn start(&mut self, shard_id: ShardId, shard_total: ShardId) -> Result<()> {
        let shard_info = [shard_id.0, shard_total.0];
        let shard = Shard::new(self.ws_url.clone(), self.token.clone(), shard_info)?;
        let locked = Arc::new(ParkingLotMutex::new(shard));

        let mut runner = feature_framework! {{
            ShardRunner::new(
                locked.clone(),
                self.manager_tx.clone(),
                self.framework.clone(),
                self.data.clone(),
                self.event_handler.clone(),
            )
        } else {
            ShardRunner::new(
                locked.clone(),
                self.manager_tx.clone(),
                self.data.clone(),
                self.event_handler.clone(),
            )
        }};

        let runner_info = ShardRunnerInfo {
            runner_tx: runner.runner_tx(),
        };

        thread::spawn(move || {
            let _ = runner.run();
        });

        self.runners.lock().insert(shard_id, runner_info);

        Ok(())
    }
}
