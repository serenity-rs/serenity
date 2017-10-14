use internal::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use super::super::super::EventHandler;
use super::{
    ShardId,
    ShardManagerMessage,
    ShardQueuer,
    ShardQueuerMessage,
    ShardRunnerInfo,
};
use threadpool::ThreadPool;
use typemap::ShareMap;

#[cfg(feature = "framework")]
use framework::Framework;

pub struct ShardManager {
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// The index of the first shard to initialize, 0-indexed.
    shard_index: u64,
    /// The number of shards to initialize.
    shard_init: u64,
    /// The total shards in use, 1-indexed.
    shard_total: u64,
    shard_queuer: Sender<ShardQueuerMessage>,
    thread_rx: Receiver<ShardManagerMessage>,
}

impl ShardManager {
    #[cfg(feature = "framework")]
    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    pub fn new<H>(
        shard_index: u64,
        shard_init: u64,
        shard_total: u64,
        ws_url: Arc<Mutex<String>>,
        token: Arc<Mutex<String>>,
        data: Arc<Mutex<ShareMap>>,
        event_handler: Arc<H>,
        framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
        threadpool: ThreadPool,
    ) -> Self where H: EventHandler + Send + Sync + 'static {
        let (thread_tx, thread_rx) = mpsc::channel();
        let (shard_queue_tx, shard_queue_rx) = mpsc::channel();

        let runners = Arc::new(Mutex::new(HashMap::new()));

        let mut shard_queuer = ShardQueuer {
            data: Arc::clone(&data),
            event_handler: Arc::clone(&event_handler),
            framework: Arc::clone(&framework),
            last_start: None,
            manager_tx: thread_tx.clone(),
            runners: Arc::clone(&runners),
            rx: shard_queue_rx,
            token: Arc::clone(&token),
            ws_url: Arc::clone(&ws_url),
            threadpool,
        };

        thread::spawn(move || {
            shard_queuer.run();
        });

        Self {
            shard_queuer: shard_queue_tx,
            thread_rx: thread_rx,
            runners,
            shard_index,
            shard_init,
            shard_total,
        }
    }

    #[cfg(not(feature = "framework"))]
    pub fn new<H>(
        shard_index: u64,
        shard_init: u64,
        shard_total: u64,
        ws_url: Arc<Mutex<String>>,
        token: Arc<Mutex<String>>,
        data: Arc<Mutex<ShareMap>>,
        event_handler: Arc<H>,
        threadpool: ThreadPool,
    ) -> Self where H: EventHandler + Send + Sync + 'static {
        let (thread_tx, thread_rx) = mpsc::channel();
        let (shard_queue_tx, shard_queue_rx) = mpsc::channel();

        let runners = Arc::new(Mutex::new(HashMap::new()));

        let mut shard_queuer = ShardQueuer {
            data: data.clone(),
            event_handler: event_handler.clone(),
            last_start: None,
            manager_tx: thread_tx.clone(),
            runners: runners.clone(),
            rx: shard_queue_rx,
            token: token.clone(),
            ws_url: ws_url.clone(),
            threadpool,
        };

        thread::spawn(move || {
            shard_queuer.run();
        });

        Self {
            shard_queuer: shard_queue_tx,
            thread_rx: thread_rx,
            runners,
            shard_index,
            shard_init,
            shard_total,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let shard_to = self.shard_index + self.shard_init;

        debug!("{}, {}", self.shard_index, self.shard_init);

        for shard_id in self.shard_index..shard_to {
            let shard_total = self.shard_total;

            self.boot([ShardId(shard_id), ShardId(shard_total)]);
        }

        Ok(())
    }

    pub fn run(&mut self) {
        while let Ok(value) = self.thread_rx.recv() {
            match value {
                ShardManagerMessage::Restart(shard_id) => self.restart(shard_id),
                ShardManagerMessage::Shutdown(shard_id) => self.shutdown(shard_id),
                ShardManagerMessage::ShutdownAll => {
                    self.shutdown_all();

                    break;
                },
            }
        }
    }

    pub fn shutdown_all(&mut self) {
        info!("Shutting down all shards");
        let keys = {
            self.runners.lock().keys().cloned().collect::<Vec<ShardId>>()
        };

        for shard_id in keys {
            self.shutdown(shard_id);
        }
    }

    fn boot(&mut self, shard_info: [ShardId; 2]) {
        info!("Telling shard queuer to start shard {}", shard_info[0]);

        let msg = ShardQueuerMessage::Start(shard_info[0], shard_info[1]);
        let _ = self.shard_queuer.send(msg);
    }

    fn restart(&mut self, shard_id: ShardId) {
        info!("Restarting shard {}", shard_id);
        self.shutdown(shard_id);

        let shard_total = self.shard_total;

        self.boot([shard_id, ShardId(shard_total)]);
    }

    fn shutdown(&mut self, shard_id: ShardId) {
        info!("Shutting down shard {}", shard_id);

        if let Some(runner) = self.runners.lock().get(&shard_id) {
            let msg = ShardManagerMessage::Shutdown(shard_id);

            if let Err(why) = runner.runner_tx.send(msg) {
                warn!("Failed to cleanly shutdown shard {}: {:?}", shard_id, why);
            }
        }

        self.runners.lock().remove(&shard_id);
    }
}

impl Drop for ShardManager {
    fn drop(&mut self) {
        if let Err(why) = self.shard_queuer.send(ShardQueuerMessage::Shutdown) {
            warn!("Failed to send shutdown to shard queuer: {:?}", why);
        }
    }
}
