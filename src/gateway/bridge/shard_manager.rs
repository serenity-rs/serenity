use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
#[cfg(feature = "framework")]
use std::sync::OnceLock;
use std::time::Duration;

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{info, instrument, warn};
use typemap_rev::TypeMap;

#[cfg(feature = "voice")]
use super::VoiceGatewayManager;
use super::{ShardId, ShardQueuer, ShardQueuerMessage, ShardRunnerInfo};
#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::client::{EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::{ConnectionStage, GatewayError, PresenceData};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::gateway::GatewayIntents;

/// A manager for handling the status of shards by starting them, restarting them, and stopping
/// them when required.
///
/// **Note**: The [`Client`] internally uses a shard manager. If you are using a Client, then you
/// do not need to make one of these.
///
/// # Examples
///
/// Initialize a shard manager with a framework responsible for shards 0 through 2, of 5 total
/// shards:
///
/// ```rust,no_run
/// # use std::error::Error;
/// #
/// # #[cfg(feature = "voice")]
/// # use serenity::model::id::UserId;
/// # #[cfg(feature = "cache")]
/// # use serenity::cache::Cache;
/// #
/// # #[cfg(feature = "framework")]
/// # async fn run() -> Result<(), Box<dyn Error>> {
/// #
/// use std::env;
/// use std::sync::{Arc, OnceLock};
///
/// use serenity::client::{EventHandler, RawEventHandler};
/// use serenity::framework::{Framework, StandardFramework};
/// use serenity::gateway::{ShardManager, ShardManagerOptions};
/// use serenity::http::Http;
/// use serenity::model::gateway::GatewayIntents;
/// use serenity::prelude::*;
/// use tokio::sync::{Mutex, RwLock};
///
/// struct Handler;
///
/// impl EventHandler for Handler {}
/// impl RawEventHandler for Handler {}
///
/// # let http: Arc<Http> = unimplemented!();
/// let ws_url = Arc::new(Mutex::new(http.get_gateway().await?.url));
/// let data = Arc::new(RwLock::new(TypeMap::new()));
/// let event_handler = Arc::new(Handler) as Arc<dyn EventHandler>;
/// let framework = Arc::new(StandardFramework::new()) as Arc<dyn Framework + 'static>;
///
/// ShardManager::new(ShardManagerOptions {
///     data,
///     event_handlers: vec![event_handler],
///     raw_event_handlers: vec![],
///     framework: Arc::new(OnceLock::from(framework)),
///     // the shard index to start initiating from
///     shard_index: 0,
///     // the number of shards to initiate (this initiates 0, 1, and 2)
///     shard_init: 3,
///     // the total number of shards in use
///     shard_total: 5,
///     # #[cfg(feature = "voice")]
///     # voice_manager: None,
///     ws_url,
///     # #[cfg(feature = "cache")]
///     # cache: unimplemented!(),
///     # http,
///     intents: GatewayIntents::non_privileged(),
///     presence: None,
/// });
/// # Ok(())
/// # }
/// ```
///
/// [`Client`]: crate::Client
#[derive(Debug)]
pub struct ShardManager {
    return_value_tx: Mutex<Sender<Result<(), GatewayError>>>,
    /// The shard runners currently managed.
    ///
    /// **Note**: It is highly unrecommended to mutate this yourself unless you need to. Instead
    /// prefer to use methods on this struct that are provided where possible.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// The index of the first shard to initialize, 0-indexed.
    // Atomics are used here to allow for mutation without requiring a mutable reference to self.
    shard_index: AtomicU32,
    /// The number of shards to initialize.
    shard_init: AtomicU32,
    /// The total shards in use, 1-indexed.
    shard_total: AtomicU32,
    shard_queuer: Sender<ShardQueuerMessage>,
    // We can safely use a Mutex for this field, as it is only ever used in one single place
    // and only is ever used to receive a single message
    shard_shutdown: Mutex<Receiver<ShardId>>,
    shard_shutdown_send: Sender<ShardId>,
    gateway_intents: GatewayIntents,
}

impl ShardManager {
    /// Creates a new shard manager, returning both the manager and a monitor for usage in a
    /// separate thread.
    #[must_use]
    pub fn new(opt: ShardManagerOptions) -> (Arc<Self>, Receiver<Result<(), GatewayError>>) {
        let (return_value_tx, return_value_rx) = mpsc::unbounded();
        let (shard_queue_tx, shard_queue_rx) = mpsc::unbounded();

        let runners = Arc::new(Mutex::new(HashMap::new()));
        let (shutdown_send, shutdown_recv) = mpsc::unbounded();

        let manager = Arc::new(Self {
            return_value_tx: Mutex::new(return_value_tx),
            shard_index: AtomicU32::new(opt.shard_index),
            shard_init: AtomicU32::new(opt.shard_init),
            shard_queuer: shard_queue_tx,
            shard_total: AtomicU32::new(opt.shard_total),
            shard_shutdown: Mutex::new(shutdown_recv),
            shard_shutdown_send: shutdown_send,
            runners: Arc::clone(&runners),
            gateway_intents: opt.intents,
        });

        let mut shard_queuer = ShardQueuer {
            data: opt.data,
            event_handlers: opt.event_handlers,
            raw_event_handlers: opt.raw_event_handlers,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            last_start: None,
            manager: Arc::clone(&manager),
            queue: VecDeque::new(),
            runners,
            rx: shard_queue_rx,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            ws_url: opt.ws_url,
            #[cfg(feature = "cache")]
            cache: opt.cache,
            http: opt.http,
            intents: opt.intents,
            presence: opt.presence,
        };

        spawn_named("shard_queuer::run", async move {
            shard_queuer.run().await;
        });

        (Arc::clone(&manager), return_value_rx)
    }

    /// Returns whether the shard manager contains either an active instance of a shard runner
    /// responsible for the given ID.
    ///
    /// If a shard has been queued but has not yet been initiated, then this will return `false`.
    pub async fn has(&self, shard_id: ShardId) -> bool {
        self.runners.lock().await.contains_key(&shard_id)
    }

    /// Initializes all shards that the manager is responsible for.
    ///
    /// This will communicate shard boots with the [`ShardQueuer`] so that they are properly
    /// queued.
    #[instrument(skip(self))]
    pub fn initialize(&self) -> Result<()> {
        let shard_index = self.shard_index.load(Ordering::Relaxed);
        let shard_init = self.shard_init.load(Ordering::Relaxed);
        let shard_total = self.shard_total.load(Ordering::Relaxed);

        let shard_to = shard_index + shard_init;

        for shard_id in shard_index..shard_to {
            self.boot([ShardId(shard_id), ShardId(shard_total)]);
        }

        Ok(())
    }

    /// Sets the new sharding information for the manager.
    ///
    /// This will shutdown all existing shards.
    ///
    /// This will _not_ instantiate the new shards.
    #[instrument(skip(self))]
    pub async fn set_shards(&self, index: u32, init: u32, total: u32) {
        self.shutdown_all().await;

        self.shard_index.store(index, Ordering::Relaxed);
        self.shard_init.store(init, Ordering::Relaxed);
        self.shard_total.store(total, Ordering::Relaxed);
    }

    /// Restarts a shard runner.
    ///
    /// This sends a shutdown signal to a shard's associated [`ShardRunner`], and then queues a
    /// initialization of a shard runner for the same shard via the [`ShardQueuer`].
    ///
    /// # Examples
    ///
    /// Restarting a shard by ID:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ShardId;
    /// use serenity::prelude::*;
    ///
    /// # async fn run(client: Client) {
    /// // restart shard ID 7
    /// client.shard_manager.restart(ShardId(7)).await;
    /// # }
    /// ```
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[instrument(skip(self))]
    pub async fn restart(&self, shard_id: ShardId) {
        info!("Restarting shard {}", shard_id);
        self.shutdown(shard_id, 4000).await;

        let shard_total = self.shard_total.load(Ordering::Relaxed);

        self.boot([shard_id, ShardId(shard_total)]);
    }

    /// Returns the [`ShardId`]s of the shards that have been instantiated and currently have a
    /// valid [`ShardRunner`].
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[instrument(skip(self))]
    pub async fn shards_instantiated(&self) -> Vec<ShardId> {
        self.runners.lock().await.keys().copied().collect()
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// Returns a boolean indicating whether a shard runner was present. This is _not_ necessary an
    /// indicator of whether the shard runner was successfully shut down.
    ///
    /// **Note**: If the receiving end of an mpsc channel - owned by the shard runner - no longer
    /// exists, then the shard runner will not know it should shut down. This _should never happen_.
    /// It may already be stopped.
    #[instrument(skip(self))]
    pub async fn shutdown(&self, shard_id: ShardId, code: u16) {
        const TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(5);

        info!("Shutting down shard {}", shard_id);

        {
            let mut shard_shutdown = self.shard_shutdown.lock().await;

            drop(
                self.shard_queuer.unbounded_send(ShardQueuerMessage::ShutdownShard(shard_id, code)),
            );
            match timeout(TIMEOUT, shard_shutdown.next()).await {
                Ok(Some(shutdown_shard_id)) => {
                    if shutdown_shard_id != shard_id {
                        warn!(
                        "Failed to cleanly shutdown shard {}: Shutdown channel sent incorrect ID",
                        shard_id,
                    );
                    }
                },
                Ok(None) => (),
                Err(why) => {
                    warn!(
                        "Failed to cleanly shutdown shard {}, reached timeout: {:?}",
                        shard_id, why
                    );
                },
            }
            // shard_shutdown is dropped here and releases the lock
            // in theory we should never have two calls to shutdown()
            // at the same time but this is a safety measure just in case:tm:
        }

        self.runners.lock().await.remove(&shard_id);
    }

    /// Sends a shutdown message for all shards that the manager is responsible for that are still
    /// known to be running.
    ///
    /// If you only need to shutdown a select number of shards, prefer looping over the
    /// [`Self::shutdown`] method.
    #[instrument(skip(self))]
    pub async fn shutdown_all(&self) {
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

        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown));

        // this message is received by Client::start_connection, which lets the main thread know
        // and finally return from Client::start
        drop(self.return_value_tx.lock().await.unbounded_send(Ok(())));
    }

    #[instrument(skip(self))]
    fn boot(&self, shard_info: [ShardId; 2]) {
        info!("Telling shard queuer to start shard {}", shard_info[0]);

        let msg = ShardQueuerMessage::Start(shard_info[0], shard_info[1]);

        drop(self.shard_queuer.unbounded_send(msg));
    }

    /// Returns the gateway intents used for this gateway connection.
    #[must_use]
    pub fn intents(&self) -> GatewayIntents {
        self.gateway_intents
    }

    pub async fn return_with_value(&self, ret: Result<(), GatewayError>) {
        if let Err(e) = self.return_value_tx.lock().await.send(ret).await {
            tracing::warn!("failed to send return value: {}", e);
        }
    }

    pub fn shutdown_finished(&self, id: ShardId) {
        if let Err(e) = self.shard_shutdown_send.unbounded_send(id) {
            tracing::warn!("failed to notify about finished shutdown: {}", e);
        }
    }

    pub async fn restart_shard(&self, id: ShardId) {
        self.restart(id).await;
        if let Err(e) = self.shard_shutdown_send.unbounded_send(id) {
            tracing::warn!("failed to notify about finished shutdown: {}", e);
        }
    }

    pub async fn update_shard_latency_and_stage(
        &self,
        id: ShardId,
        latency: Option<Duration>,
        stage: ConnectionStage,
    ) {
        if let Some(runner) = self.runners.lock().await.get_mut(&id) {
            runner.latency = latency;
            runner.stage = stage;
        }
    }
}

impl Drop for ShardManager {
    /// A custom drop implementation to clean up after the manager.
    ///
    /// This shuts down all active [`ShardRunner`]s and attempts to tell the [`ShardQueuer`] to
    /// shutdown.
    ///
    /// [`ShardRunner`]: super::ShardRunner
    fn drop(&mut self) {
        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown));
    }
}

pub struct ShardManagerOptions {
    pub data: Arc<RwLock<TypeMap>>,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    pub raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Arc<OnceLock<Arc<dyn Framework>>>,
    pub shard_index: u32,
    pub shard_init: u32,
    pub shard_total: u32,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    pub ws_url: Arc<Mutex<String>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub intents: GatewayIntents,
    pub presence: Option<PresenceData>,
}
