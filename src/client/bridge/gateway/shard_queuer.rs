use gateway::Shard;
use internal::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
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
use threadpool::ThreadPool;
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
    /// A copy of [`Client::data`] to be given to runners for contextual
    /// dispatching.
    ///
    /// [`Client::data`]: ../../struct.Client.html#structfield.data
    pub data: Arc<Mutex<ShareMap>>,
    /// A reference to an `EventHandler`, such as the one given to the
    /// [`Client`].
    ///
    /// [`Client`]: ../../struct.Client.html
    pub event_handler: Arc<H>,
    /// A copy of the framework
    #[cfg(feature = "framework")]
    pub framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
    /// The instant that a shard was last started.
    ///
    /// This is used to determine how long to wait between shard IDENTIFYs.
    pub last_start: Option<Instant>,
    /// A copy of the sender channel to communicate with the
    /// [`ShardManagerMonitor`].
    ///
    /// [`ShardManagerMonitor`]: struct.ShardManagerMonitor.html
    pub manager_tx: Sender<ShardManagerMessage>,
    /// A copy of the map of shard runners.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// A receiver channel for the shard queuer to be told to start shards.
    pub rx: Receiver<ShardQueuerMessage>,
    /// A copy of a threadpool to give shard runners.
    ///
    /// For example, when using the [`Client`], this will be a copy of
    /// [`Client::threadpool`].
    ///
    /// [`Client`]: ../../struct.Client.html
    /// [`Client::threadpool`]: ../../struct.Client.html#structfield.threadpool
    pub threadpool: ThreadPool,
    /// A copy of the token to connect with.
    pub token: Arc<Mutex<String>>,
    /// A copy of the URI to use to connect to the gateway.
    pub ws_url: Arc<Mutex<String>>,
}

impl<H: EventHandler + Send + Sync + 'static> ShardQueuer<H> {
    /// Begins the shard queuer loop.
    ///
    /// This will loop over the internal [`rx`] for [`ShardQueuerMessage`]s,
    /// blocking for messages on what to do.
    ///
    /// If a [`ShardQueuerMessage::Start`] is received, this will:
    ///
    /// 1. Check how much time has passed since the last shard was started
    /// 2. If the amount of time is less than the ratelimit, it will sleep until
    /// that time has passed
    /// 3. Start the shard by ID
    ///
    /// If a [`ShardQueuerMessage::Shutdown`] is received, this will return and
    /// the loop will be over.
    ///
    /// **Note**: This should be run in its own thread due to the blocking
    /// nature of the loop.
    ///
    /// [`ShardQueuerMessage`]: enum.ShardQueuerMessage.html
    /// [`ShardQueuerMessage::Shutdown`]: enum.ShardQueuerMessage.html#variant.Shutdown
    /// [`ShardQueuerMessage::Start`]: enum.ShardQueuerMessage.html#variant.Start
    /// [`rx`]: #structfield.rx
    pub fn run(&mut self) {
        while let Ok(msg) = self.rx.recv() {
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

        let shard = Shard::new(
            Arc::clone(&self.ws_url),
            Arc::clone(&self.token),
            shard_info,
        )?;

        let mut runner = feature_framework! {{
            ShardRunner::new(
                shard,
                self.manager_tx.clone(),
                Arc::clone(&self.framework),
                Arc::clone(&self.data),
                Arc::clone(&self.event_handler),
                self.threadpool.clone(),
            )
        } else {
            ShardRunner::new(
                shard,
                self.manager_tx.clone(),
                self.data.clone(),
                self.event_handler.clone(),
                self.threadpool.clone(),
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
