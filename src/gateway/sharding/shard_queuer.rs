use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU16;
use std::sync::Arc;
#[cfg(feature = "framework")]
use std::sync::OnceLock;

use futures::channel::mpsc::UnboundedReceiver as Receiver;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout, Duration, Instant};
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, info, warn};

use super::{
    ShardId,
    ShardManager,
    ShardMessenger,
    ShardRunner,
    ShardRunnerInfo,
    ShardRunnerOptions,
    TransportCompression,
};
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::client::{EventHandler, RawEventHandler};
#[cfg(feature = "voice")]
use crate::gateway::VoiceGatewayManager;
use crate::gateway::{ConnectionStage, PresenceData, Shard, ShardRunnerMessage};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::gateway::{GatewayIntents, ShardInfo};

/// The shard queuer is a simple loop that runs indefinitely to manage the startup of shards.
///
/// A shard queuer instance _should_ be run in its own thread, due to the blocking nature of the
/// loop itself as well as a 5 second thread sleep between shard starts.
pub struct ShardQueuer {
    pub(super) token: Token,
    /// A copy of [`Client::data`] to be given to runners for contextual dispatching.
    ///
    /// [`Client::data`]: crate::Client::data
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    /// A reference to an [`EventHandler`].
    pub event_handler: Option<Arc<dyn EventHandler>>,
    /// A reference to a [`RawEventHandler`].
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    /// A copy of the framework
    #[cfg(feature = "framework")]
    pub framework: Arc<OnceLock<Arc<dyn Framework>>>,
    /// The instant that a shard was last started.
    ///
    /// This is used to determine how long to wait between shard IDENTIFYs.
    pub last_start: Option<Instant>,
    /// A copy of the [`ShardManager`] to communicate with it.
    pub manager: Arc<ShardManager>,
    /// The shards that are queued for booting.
    pub queue: ShardQueue,
    /// A copy of the map of shard runners.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// A receiver channel for the shard queuer to be told to start shards.
    pub rx: Receiver<ShardQueuerMessage>,
    /// A copy of the client's voice manager.
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    /// A copy of the URL to use to connect to the gateway.
    pub ws_url: Arc<str>,
    /// The compression method to use for the WebSocket connection.
    pub compression: TransportCompression,
    /// The total amount of shards to start.
    pub shard_total: NonZeroU16,
    /// Number of seconds to wait between each start
    pub wait_time_between_shard_start: Duration,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub intents: GatewayIntents,
    pub presence: Option<PresenceData>,
}

impl ShardQueuer {
    /// Begins the shard queuer loop.
    ///
    /// This will loop over the internal [`Self::rx`] for [`ShardQueuerMessage`]s, blocking for
    /// messages on what to do.
    ///
    /// If a [`ShardQueuerMessage::Start`] is received, this will:
    ///
    /// 1. Check how much time has passed since the last shard was started
    /// 2. If the amount of time is less than the ratelimit, it will sleep until that time has
    ///    passed
    /// 3. Start the shard by ID
    ///
    /// If a [`ShardQueuerMessage::Shutdown`] is received, this will return and the loop will be
    /// over.
    ///
    /// **Note**: This should be run in its own thread due to the blocking nature of the loop.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn run(&mut self) {
        // We read from the Rx channel in a loop, and use a timeout of
        // {self.WAIT_TIME_BETWEEN_SHARD_START} (5 seconds normally) seconds so that we don't
        // hang forever. When we receive a command to start a shard, we append it to our queue. The
        // queue is popped in batches of shards, which are started in parallel. A batch is fired
        // every WAIT_TIME_BETWEEN_SHARD_START at minimum in order to avoid being ratelimited.

        loop {
            if let Ok(msg) = timeout(self.wait_time_between_shard_start, self.rx.next()).await {
                match msg {
                    Some(ShardQueuerMessage::SetShardTotal(shard_total)) => {
                        self.shard_total = shard_total;
                    },
                    Some(ShardQueuerMessage::Start {
                        shard_id,
                        concurrent,
                    }) => {
                        if concurrent {
                            // If we're starting multiple shards, we can start them concurrently
                            // according to `max_concurrency`, and want our batches to be of
                            // maximal size.
                            self.queue.push_back(shard_id);
                            if self.queue.buckets_filled() {
                                let batch = self.queue.pop_batch();
                                self.checked_start_batch(batch).await;
                            }
                        } else {
                            // In cases where we're only starting a single shard (e.g. if we're
                            // restarting a shard), we assume the queue will never fill up and skip
                            // using it so that we don't incur a 5 second timeout.
                            self.checked_start(shard_id).await;
                        }
                    },
                    Some(ShardQueuerMessage::ShutdownShard {
                        shard_id,
                        code,
                    }) => {
                        debug!(
                            "[Shard Queuer] Received to shutdown shard {} with code {}",
                            shard_id.0, code
                        );
                        self.shutdown(shard_id, code).await;
                    },
                    Some(ShardQueuerMessage::Shutdown) => {
                        debug!("[Shard Queuer] Received to shutdown all shards");
                        self.shutdown_runners().await;
                        break;
                    },
                    None => break,
                }
            } else {
                // Once we've stopped receiving `Start` commands, we no longer care about the size
                // of our batches being maximal.
                let batch = self.queue.pop_batch();
                self.checked_start_batch(batch).await;
            }
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn check_last_start(&mut self) {
        let Some(instant) = self.last_start else { return };

        // We must wait 5 seconds between IDENTIFYs to avoid session invalidations.
        let elapsed = instant.elapsed();

        if elapsed >= self.wait_time_between_shard_start {
            return;
        }

        let to_sleep = self.wait_time_between_shard_start - elapsed;

        sleep(to_sleep).await;
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn checked_start(&mut self, shard_id: ShardId) {
        debug!("[Shard Queuer] Checked start for shard {shard_id}");

        self.check_last_start().await;
        self.try_start(shard_id).await;

        self.last_start = Some(Instant::now());
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn checked_start_batch(&mut self, shard_ids: Vec<ShardId>) {
        if shard_ids.is_empty() {
            return;
        }

        debug!("[Shard Queuer] Starting batch of {} shards", shard_ids.len());
        self.check_last_start().await;
        for shard_id in shard_ids {
            debug!("[Shard Queuer] Starting shard {shard_id}");
            self.try_start(shard_id).await;
        }
        self.last_start = Some(Instant::now());
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn try_start(&mut self, shard_id: ShardId) {
        if let Err(why) = self.start(shard_id).await {
            warn!("[Shard Queuer] Err starting shard {shard_id}: {why:?}");
            info!("[Shard Queuer] Re-queueing start of shard {shard_id}");

            // Try again in the next batch.
            self.queue.push_front(shard_id);
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn start(&mut self, shard_id: ShardId) -> Result<()> {
        let shard_info = ShardInfo {
            id: shard_id,
            total: self.shard_total,
        };
        let mut shard = Shard::new(
            Arc::clone(&self.ws_url),
            self.token.clone(),
            shard_info,
            self.intents,
            self.presence.clone(),
            self.compression,
        )
        .await?;

        let cloned_http = Arc::clone(&self.http);
        shard.set_application_id_callback(move |id| cloned_http.set_application_id(id));

        let mut runner = ShardRunner::new(ShardRunnerOptions {
            data: Arc::clone(&self.data),
            event_handler: self.event_handler.clone(),
            raw_event_handler: self.raw_event_handler.clone(),
            #[cfg(feature = "framework")]
            framework: self.framework.get().cloned(),
            manager: Arc::clone(&self.manager),
            #[cfg(feature = "voice")]
            voice_manager: self.voice_manager.clone(),
            shard,
            #[cfg(feature = "cache")]
            cache: Arc::clone(&self.cache),
            http: Arc::clone(&self.http),
        });

        let runner_info = ShardRunnerInfo {
            latency: None,
            runner_tx: ShardMessenger::new(&runner),
            stage: ConnectionStage::Disconnected,
        };

        spawn_named("shard_queuer::stop", async move {
            drop(runner.run().await);
            debug!("[ShardRunner {:?}] Stopping", runner.shard.shard_info());
        });

        self.runners.lock().await.insert(shard_id, runner_info);

        Ok(())
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn shutdown_runners(&mut self) {
        let keys = {
            let runners = self.runners.lock().await;

            if runners.is_empty() {
                return;
            }

            runners.keys().copied().collect::<Vec<_>>()
        };

        info!("Shutting down all shards");

        for shard_id in keys {
            self.shutdown(shard_id, 1000).await;
        }
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// **Note**: If the receiving end of an mpsc channel - owned by the shard runner - no longer
    /// exists, then the shard runner will not know it should shut down. This _should never happen_.
    /// It may already be stopped.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn shutdown(&mut self, shard_id: ShardId, code: u16) {
        info!("Shutting down shard {}", shard_id);

        if let Some(runner) = self.runners.lock().await.get(&shard_id) {
            let msg = ShardRunnerMessage::Shutdown(shard_id, code);

            if let Err(why) = runner.runner_tx.tx.unbounded_send(msg) {
                warn!(
                    "Failed to cleanly shutdown shard {} when sending message to shard runner: {:?}",
                    shard_id,
                    why,
                );
            }
        }
    }
}

/// A queue of [`ShardId`]s that is split up into multiple buckets according to the value of
/// [`max_concurrency`](crate::model::gateway::SessionStartLimit::max_concurrency).
#[must_use]
pub struct ShardQueue {
    buckets: FixedArray<VecDeque<ShardId>, u16>,
}

impl ShardQueue {
    pub fn new(max_concurrency: NonZeroU16) -> Self {
        let buckets = vec![VecDeque::new(); max_concurrency.get() as usize].into_boxed_slice();
        let buckets = FixedArray::try_from(buckets).expect("should fit without truncation");

        Self {
            buckets,
        }
    }

    fn calculate_bucket(&self, shard_id: ShardId) -> u16 {
        shard_id.0 % self.buckets.len()
    }

    /// Calculates the corresponding bucket for the given `ShardId` and **appends** to it.
    pub fn push_back(&mut self, shard_id: ShardId) {
        let bucket = self.calculate_bucket(shard_id);
        self.buckets[bucket].push_back(shard_id);
    }

    /// Calculates the corresponding bucket for the given `ShardId` and **prepends** to it.
    pub fn push_front(&mut self, shard_id: ShardId) {
        let bucket = self.calculate_bucket(shard_id);
        self.buckets[bucket].push_front(shard_id);
    }

    /// Pops a `ShardId` from every bucket containing at least one and returns them all as a `Vec`.
    pub fn pop_batch(&mut self) -> Vec<ShardId> {
        self.buckets.iter_mut().filter_map(VecDeque::pop_front).collect()
    }

    /// Returns `true` if every bucket contains at least one `ShardId`.
    #[must_use]
    pub fn buckets_filled(&self) -> bool {
        self.buckets.iter().all(|b| !b.is_empty())
    }
}

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
