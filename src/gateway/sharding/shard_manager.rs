use std::collections::HashMap;
use std::num::NonZeroU16;
#[cfg(feature = "framework")]
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use futures::StreamExt;
use tokio::time::{sleep, timeout};
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, info, warn};

use super::{
    Shard,
    ShardId,
    ShardInfo,
    ShardMessenger,
    ShardQueue,
    ShardRunner,
    ShardRunnerInfo,
    ShardRunnerMessage,
    ShardRunnerOptions,
};
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::client::{EventHandler, RawEventHandler};
#[cfg(feature = "voice")]
use crate::gateway::VoiceGatewayManager;
use crate::gateway::{ConnectionStage, GatewayError, PresenceData, TransportCompression};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::gateway::GatewayIntents;

/// The default time to wait between starting each shard or set of shards.
pub const DEFAULT_WAIT_BETWEEN_SHARD_START: Duration = Duration::from_secs(5);

/// A manager for handling the status of shards by starting them, restarting them, and stopping
/// them when required.
pub struct ShardManager {
    token: Token,
    /// A sender that is cloned and given out to each ShardRunner as it is created
    manager_tx: Sender<ShardManagerMessage>,
    manager_rx: Receiver<ShardManagerMessage>,

    /// A copy of [`Client::data`] to be given to runners for contextual dispatching.
    ///
    /// [`Client::data`]: crate::Client::data
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    /// A reference to an [`EventHandler`].
    pub event_handler: Option<Arc<dyn EventHandler>>,
    /// A reference to a [`RawEventHandler`].
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    /// A copy of the framework.
    #[cfg(feature = "framework")]
    pub framework: Arc<OnceLock<Arc<dyn Framework>>>,
    /// The instant that a shard was last started.
    ///
    /// This is used to determine how long to wait between shard IDENTIFYs.
    pub last_start: Option<Instant>,
    /// The shards that are queued for booting.
    pub queue: ShardQueue,
    /// The shard runners currently managed.
    ///
    /// **Note**: It is highly recommended to not mutate this yourself unless you need to. Instead
    /// prefer to use methods on this struct that are provided where possible.
    pub runners: HashMap<ShardId, (Arc<Mutex<ShardRunnerInfo>>, ShardMessenger)>,
    /// A copy of the client's voice manager.
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    /// A copy of the URL to use to connect to the gateway.
    pub ws_url: Arc<str>,
    /// The compression method to use for the WebSocket connection.
    pub compression: TransportCompression,
    /// The total amount of shards to start.
    pub shard_total: NonZeroU16,
    /// Number of seconds to wait between each start.
    pub wait_time_between_shard_start: Duration,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub intents: GatewayIntents,
    pub presence: Option<PresenceData>,
}

impl ShardManager {
    #[must_use]
    pub fn new(opt: ShardManagerOptions) -> Self {
        let (manager_tx, manager_rx) = mpsc::unbounded();

        Self {
            token: opt.token,
            manager_tx,
            manager_rx,

            data: opt.data,
            event_handler: opt.event_handler,
            raw_event_handler: opt.raw_event_handler,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            last_start: None,
            queue: ShardQueue::new(opt.max_concurrency),
            runners: HashMap::new(),
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            ws_url: opt.ws_url,
            compression: opt.compression,
            shard_total: opt.shard_total,
            #[cfg(feature = "cache")]
            cache: opt.cache,
            http: opt.http,
            intents: opt.intents,
            presence: opt.presence,
            wait_time_between_shard_start: opt.wait_time_between_shard_start,
        }
    }

    /// The main interface for starting the management of shards. Initializes the shards by
    /// queueing them for starting, and then listens for [`ShardManagerMessage`]s in a loop.
    ///
    /// This function takes care of:
    ///   1. Starting [`ShardRunner`]s and spawning tasks.
    ///   2. Restarting shards (by shutting them down and spawning a new task) when requested.
    ///   3. Quitting if a shard runner encounters a fatal gateway error.
    ///
    /// # Errors
    ///
    /// Returns a [`GatewayError`] if an unrecoverable error is received from the gateway. These
    /// include the following:
    ///   * [`GatewayError::InvalidAuthentication`]
    ///   * [`GatewayError::InvalidApiVersion`]
    ///   * [`GatewayError::InvalidGatewayIntents`]
    ///   * [`GatewayError::DisallowedGatewayIntents`]
    pub async fn run(
        &mut self,
        shard_index: u16,
        shard_init: u16,
        shard_total: NonZeroU16,
    ) -> Result<(), GatewayError> {
        self.initialize(shard_index, shard_init, shard_total);
        loop {
            if let Ok(Some(msg)) =
                timeout(self.wait_time_between_shard_start, self.manager_rx.next()).await
            {
                match msg {
                    ShardManagerMessage::Boot(shard_id) => self.queue_for_start(shard_id),
                    ShardManagerMessage::Quit(err) => return Err(err),
                }
            }
            let batch = self.queue.pop_batch();
            self.checked_start(batch).await;
        }
    }

    /// Initializes all shards that the manager is responsible for.
    ///
    /// Note that this queues all shards but does not actually start them. To start the manager's
    /// event loop and dispatch [`ShardRunner`]s as they get queued, call [`Self::run`] instead.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn initialize(&mut self, shard_index: u16, shard_init: u16, shard_total: NonZeroU16) {
        let shard_to = shard_index + shard_init;

        self.shard_total = shard_total;
        for shard_id in shard_index..shard_to {
            self.queue_for_start(ShardId(shard_id));
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn queue_for_start(&mut self, shard_id: ShardId) {
        info!("Queueing shard {shard_id} for starting");

        self.queue.push_back(shard_id);
    }

    /// Restarts a shard runner.
    ///
    /// Sends a shutdown signal to a shard's associated [`ShardRunner`], and then queues an
    /// initialization of a new shard runner for the same shard.
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn restart(&mut self, shard_id: ShardId) {
        info!("Restarting shard {shard_id}");

        if let Some((_, messenger)) = self.runners.remove(&shard_id) {
            if let Err(why) = messenger.tx.unbounded_send(ShardRunnerMessage::Restart) {
                warn!("Failed to send restart signal to shard {shard_id}: {why:?}");
            }
        }
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// **Note**: If the receiving end of an mpsc channel - owned by the shard runner - no longer
    /// exists, then the shard runner will not know it should shut down. This _should never happen_.
    /// It may already be stopped.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn shutdown(&mut self, shard_id: ShardId, code: u16) {
        info!("Shutting down shard {}", shard_id);

        if let Some((_, messenger)) = self.runners.remove(&shard_id) {
            if let Err(why) = messenger.tx.unbounded_send(ShardRunnerMessage::Shutdown(code)) {
                warn!("Failed to send shutdown signal to shard {shard_id}: {why:?}");
            }
        }
    }

    // This function assumes that each of the shard ids are bucketed separately according to
    // `max_concurrency`. If this assumption is violated, you will likely get ratelimited.
    //
    // See: https://discord.com/developers/docs/events/gateway#sharding-max-concurrency
    async fn checked_start(&mut self, shard_ids: Vec<ShardId>) {
        if shard_ids.is_empty() {
            return;
        }

        if shard_ids.len() > 1 {
            debug!("[Shard Manager] Starting batch of {} shards", shard_ids.len());
        }

        // We must wait 5 seconds between IDENTIFYs to avoid session invalidations.
        if let Some(instant) = self.last_start {
            let elapsed = instant.elapsed();
            if elapsed < self.wait_time_between_shard_start {
                sleep(self.wait_time_between_shard_start - elapsed).await;
            }
        }

        for shard_id in shard_ids {
            debug!("[Shard Manager] Starting shard {shard_id}");
            if let Err(why) = self.start_runner(shard_id).await {
                warn!("[Shard Manager] Err starting shard {shard_id}: {why:?}");
                info!("[Shard Manager] Re-queueing shard {shard_id}");

                // Re-queue at the *front* of the bucket in order to keep decent ordering
                self.queue.push_front(shard_id);
            }
            self.last_start = Some(Instant::now());
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn start_runner(&mut self, shard_id: ShardId) -> Result<()> {
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

        let runner_info = Arc::new(Mutex::new(ShardRunnerInfo {
            latency: None,
            stage: ConnectionStage::Disconnected,
        }));

        let mut runner = ShardRunner::new(ShardRunnerOptions {
            data: Arc::clone(&self.data),
            event_handler: self.event_handler.clone(),
            raw_event_handler: self.raw_event_handler.clone(),
            #[cfg(feature = "framework")]
            framework: self.framework.get().cloned(),
            runner_info: Arc::clone(&runner_info),
            manager_tx: self.manager_tx.clone(),
            #[cfg(feature = "voice")]
            voice_manager: self.voice_manager.clone(),
            shard,
            #[cfg(feature = "cache")]
            cache: Arc::clone(&self.cache),
            http: Arc::clone(&self.http),
        });

        self.runners.insert(shard_id, (runner_info, runner.messenger()));

        let manager_tx = self.manager_tx.clone();
        spawn_named("shard_runner::run", async move {
            if let Err(Error::Gateway(e)) = runner.run().await {
                if let Err(why) = manager_tx.unbounded_send(ShardManagerMessage::Quit(e)) {
                    warn!("Failed to send return value: {why}");
                }
            }
            debug!("[ShardRunner {:?}] Stopping", runner.shard.shard_info());
        });

        Ok(())
    }

    /// Returns whether the shard manager contains an active instance of a shard runner responsible
    /// for the given ID.
    ///
    /// If a shard has been queued but has not yet been initiated, then this will return `false`.
    #[must_use]
    pub fn has(&self, shard_id: ShardId) -> bool {
        self.runners.contains_key(&shard_id)
    }

    /// Returns the [`ShardId`]s of the shards that have been instantiated and currently have a
    /// valid [`ShardRunner`].
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    #[must_use]
    pub fn shards_instantiated(&self) -> Vec<ShardId> {
        self.runners.keys().copied().collect()
    }

    /// Returns the [`ShardRunnerInfo`] corresponding to each running shard.
    ///
    /// Note that the shard runner also holds a copy of its info, which is why each entry is
    /// wrapped in `Arc<Mutex<T>>`.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    #[must_use]
    pub fn runner_info(&self) -> HashMap<ShardId, Arc<Mutex<ShardRunnerInfo>>> {
        self.runners.iter().map(|(&id, (runner, _))| (id, Arc::clone(runner))).collect()
    }

    /// Returns the gateway intents used for this gateway connection.
    #[must_use]
    pub fn intents(&self) -> GatewayIntents {
        self.intents
    }
}

impl Drop for ShardManager {
    /// A custom drop implementation to clean up after the manager.
    ///
    /// This shuts down all active [`ShardRunner`]s.
    ///
    /// [`ShardRunner`]: super::ShardRunner
    fn drop(&mut self) {
        info!("Shutting down all shards");

        for (shard_id, (_, messenger)) in self.runners.drain() {
            info!("Shutting down shard {}", shard_id);
            if let Err(why) = messenger.tx.unbounded_send(ShardRunnerMessage::Shutdown(1000)) {
                warn!("Failed to send shutdown signal to shard {shard_id}: {why:?}");
            }
        }
    }
}

pub struct ShardManagerOptions {
    pub token: Token,
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    pub event_handler: Option<Arc<dyn EventHandler>>,
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Arc<OnceLock<Arc<dyn Framework>>>,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    pub ws_url: Arc<str>,
    pub compression: TransportCompression,
    pub shard_total: NonZeroU16,
    pub max_concurrency: NonZeroU16,
    pub wait_time_between_shard_start: Duration,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub intents: GatewayIntents,
    pub presence: Option<PresenceData>,
}

/// A message indicating what action the [`ShardManager`] should take.
pub enum ShardManagerMessage {
    /// Indicates that the manager should attempt to start a shard.
    ///
    /// Note that this makes use of the manager's shard queue for batching shards together, and if
    /// the queue is operating in concurrent mode (in order to start multiple shards at once),
    /// the shard is not guaranteed to immediately start, until more shards are queued.
    Boot(ShardId),
    /// Indicates that a shard runner encountered a fatal error and the shard manager should quit.
    Quit(GatewayError),
}
