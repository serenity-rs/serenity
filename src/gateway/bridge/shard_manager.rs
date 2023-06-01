use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
#[cfg(feature = "framework")]
use std::sync::OnceLock;
use std::time::Duration;

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use futures::SinkExt;
use tokio::sync::{Mutex, RwLock};
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
///     framework: Arc::new(OnceLock::with_value(framework)),
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
    return_value_tx: Sender<Result<(), GatewayError>>,
    /// The shard runners currently managed.
    ///
    /// **Note**: It is highly unrecommended to mutate this yourself unless you need to. Instead
    /// prefer to use methods on this struct that are provided where possible.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// The index of the first shard to initialize, 0-indexed.
    shard_index: u32,
    /// The number of shards to initialize.
    shard_init: u32,
    /// The total shards in use, 1-indexed.
    shard_total: u32,
    shard_queuer: Sender<ShardQueuerMessage>,
    gateway_intents: GatewayIntents,
}

impl ShardManager {
    /// Creates a new shard manager, returning both the manager and a monitor for usage in a
    /// separate thread.
    #[must_use]
    pub fn new(opt: ShardManagerOptions) -> (Arc<Mutex<Self>>, Receiver<Result<(), GatewayError>>) {
        let (return_value_tx, return_value_rx) = mpsc::unbounded();
        let (shard_queue_tx, shard_queue_rx) = mpsc::unbounded();

        let runners = Arc::new(Mutex::new(HashMap::new()));

        let manager = Arc::new(Mutex::new(Self {
            return_value_tx,
            shard_index: opt.shard_index,
            shard_init: opt.shard_init,
            shard_queuer: shard_queue_tx,
            shard_total: opt.shard_total,
            runners: Arc::clone(&runners),
            gateway_intents: opt.intents,
        }));

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
    pub fn initialize(&mut self) -> Result<()> {
        let shard_to = self.shard_index + self.shard_init;

        for shard_id in self.shard_index..shard_to {
            let shard_total = self.shard_total;

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
    pub async fn set_shards(&mut self, index: u32, init: u32, total: u32) {
        // Don't use shutdown_all here because shutdown_all also returns from Client::start
        let shard_ids = self.runners.lock().await.keys().copied().collect::<Vec<_>>();
        for shard_id in shard_ids {
            self.shutdown(shard_id, 1000).await;
        }

        self.shard_index = index;
        self.shard_init = init;
        self.shard_total = total;
    }

    /// Restarts a shard runner.
    ///
    /// This sends a shutdown signal to a shard's associated [`ShardRunner`], and then queues a
    /// initialization of a shard runner for the same shard via the [`ShardQueuer`].
    ///
    /// # Examples
    ///
    /// Creating a client and then restarting a shard by ID:
    ///
    /// _(note: in reality this precise code doesn't have an effect since the shard would not yet
    /// have been initialized via [`Self::initialize`], but the concept is the same)_
    ///
    /// ```rust,no_run
    /// use std::env;
    ///
    /// use serenity::gateway::ShardId;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {}
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client =
    ///     Client::builder(&token, GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// // restart shard ID 7
    /// client.shard_manager.lock().await.restart(ShardId(7)).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[instrument(skip(self))]
    pub async fn restart(&mut self, shard_id: ShardId) {
        info!("Restarting shard {}", shard_id);
        self.shutdown(shard_id, 4000).await;

        let shard_total = self.shard_total;

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
    /// **Note**: If the receiving end of an mpsc channel - theoretically owned by the shard runner
    /// - no longer exists, then the shard runner will not know it should shut down. This _should
    /// never happen_. It may already be stopped.
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self, shard_id: ShardId, code: u16) {
        info!("Shutting down shard {}", shard_id);

        let Some(shard) = self.runners.lock().await.get(&shard_id).map(|r| Arc::clone(&r.shard)) else {
            warn!("Shard ID {} doesn't exist", shard_id);
            return;
        };
        shard.lock().await.shutdown(code).await;
        self.runners.lock().await.remove(&shard_id);
    }

    /// Shuts down all shards and returns from [`crate::client::Client::start`].
    ///
    /// If you only need to shutdown a select number of shards, prefer looping over the
    /// [`Self::shutdown`] method.
    #[instrument(skip(self))]
    pub async fn shutdown_all(&mut self) {
        info!("Shutting down all shards");

        let shard_ids = self.runners.lock().await.keys().copied().collect::<Vec<_>>();
        for shard_id in shard_ids {
            self.shutdown(shard_id, 1000).await;
        }

        drop(self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown));
        self.return_with_value(Ok(())).await;
    }

    #[instrument(skip(self))]
    fn boot(&mut self, shard_info: [ShardId; 2]) {
        info!("Telling shard queuer to start shard {}", shard_info[0]);

        let msg = ShardQueuerMessage::Start(shard_info[0], shard_info[1]);

        drop(self.shard_queuer.unbounded_send(msg));
    }

    /// Returns the gateway intents used for this gateway connection.
    #[must_use]
    pub fn intents(&self) -> GatewayIntents {
        self.gateway_intents
    }

    pub async fn return_with_value(&mut self, ret: Result<(), GatewayError>) {
        if let Err(e) = self.return_value_tx.send(ret).await {
            tracing::warn!("failed to send return value: {}", e);
        }
    }

    pub async fn restart_shard(&mut self, id: ShardId) {
        self.restart(id).await;
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
        let runners = Arc::clone(&self.runners);
        tokio::spawn(async move {
            for runner in runners.lock().await.values_mut() {
                runner.shard.lock().await.shutdown(1000).await;
            }
        });
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
