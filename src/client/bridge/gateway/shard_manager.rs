use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use futures::StreamExt;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{info, instrument, warn};
use typemap_rev::TypeMap;

use super::{
    GatewayIntents,
    ShardId,
    ShardManagerMessage,
    ShardManagerMonitor,
    ShardQueuer,
    ShardQueuerMessage,
    ShardRunnerInfo,
};
#[cfg(feature = "voice")]
use crate::client::bridge::voice::VoiceGatewayManager;
use crate::client::{EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::prelude::*;
use crate::CacheAndHttp;

/// A manager for handling the status of shards by starting them, restarting
/// them, and stopping them when required.
///
/// **Note**: The [`Client`] internally uses a shard manager. If you are using a
/// Client, then you do not need to make one of these.
///
/// # Examples
///
/// Initialize a shard manager with a framework responsible for shards 0 through
/// 2, of 5 total shards:
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
/// use tokio::sync::{Mutex, RwLock};
/// use serenity::client::bridge::gateway::{ShardManager, ShardManagerOptions, GatewayIntents};
/// use serenity::client::{EventHandler, RawEventHandler};
/// use serenity::http::Http;
/// use serenity::CacheAndHttp;
/// use serenity::prelude::*;
/// use serenity::framework::{Framework, StandardFramework};
/// use std::sync::Arc;
/// use std::env;
///
/// struct Handler;
///
/// impl EventHandler for Handler { }
/// impl RawEventHandler for Handler { }
///
/// # let cache_and_http = Arc::new(CacheAndHttp::default());
/// # let http = &cache_and_http.http;
/// let gateway_url = Arc::new(Mutex::new(http.get_gateway().await?.url));
/// let data = Arc::new(RwLock::new(TypeMap::new()));
/// let event_handler = Arc::new(Handler) as Arc<dyn EventHandler>;
/// let framework = Arc::new(Box::new(StandardFramework::new()) as Box<dyn Framework + 'static + Send + Sync>);
///
/// ShardManager::new(ShardManagerOptions {
///     data: &data,
///     event_handler: &Some(event_handler),
///     raw_event_handler: &None,
///     framework: &framework,
///     // the shard index to start initiating from
///     shard_index: 0,
///     // the number of shards to initiate (this initiates 0, 1, and 2)
///     shard_init: 3,
///     // the total number of shards in use
///     shard_total: 5,
///     # #[cfg(feature = "voice")]
///     # voice_manager: &None,
///     ws_url: &gateway_url,
///     # cache_and_http: &cache_and_http,
///     intents: GatewayIntents::non_privileged(),
/// });
/// #     Ok(())
/// # }
/// ```
///
/// [`Client`]: crate::Client
#[derive(Debug)]
pub struct ShardManager {
    monitor_tx: Sender<ShardManagerMessage>,
    /// The shard runners currently managed.
    ///
    /// **Note**: It is highly unrecommended to mutate this yourself unless you
    /// need to. Instead prefer to use methods on this struct that are provided
    /// where possible.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// The index of the first shard to initialize, 0-indexed.
    shard_index: u64,
    /// The number of shards to initialize.
    shard_init: u64,
    /// The total shards in use, 1-indexed.
    shard_total: u64,
    shard_queuer: Sender<ShardQueuerMessage>,
    shard_shutdown: Receiver<ShardId>,
}

impl ShardManager {
    /// Creates a new shard manager, returning both the manager and a monitor
    /// for usage in a separate thread.
    pub async fn new(opt: ShardManagerOptions<'_>) -> (Arc<Mutex<Self>>, ShardManagerMonitor) {
        let (thread_tx, thread_rx) = mpsc::unbounded();
        let (shard_queue_tx, shard_queue_rx) = mpsc::unbounded();

        let runners = Arc::new(Mutex::new(HashMap::new()));
        let (shutdown_send, shutdown_recv) = mpsc::unbounded();

        let mut shard_queuer = ShardQueuer {
            data: Arc::clone(opt.data),
            event_handler: opt.event_handler.as_ref().map(|h| Arc::clone(h)),
            raw_event_handler: opt.raw_event_handler.as_ref().map(|rh| Arc::clone(rh)),
            #[cfg(feature = "framework")]
            framework: Arc::clone(&opt.framework),
            last_start: None,
            manager_tx: thread_tx.clone(),
            queue: VecDeque::new(),
            runners: Arc::clone(&runners),
            rx: shard_queue_rx,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager.clone(),
            ws_url: Arc::clone(opt.ws_url),
            cache_and_http: Arc::clone(&opt.cache_and_http),
            intents: opt.intents,
        };

        tokio::spawn(async move {
            shard_queuer.run().await;
        });

        let manager = Arc::new(Mutex::new(Self {
            monitor_tx: thread_tx,
            shard_index: opt.shard_index,
            shard_init: opt.shard_init,
            shard_queuer: shard_queue_tx,
            shard_total: opt.shard_total,
            shard_shutdown: shutdown_recv,
            runners,
        }));

        (Arc::clone(&manager), ShardManagerMonitor {
            rx: thread_rx,
            manager,
            shutdown: shutdown_send,
        })
    }

    /// Returns whether the shard manager contains either an active instance of
    /// a shard runner responsible for the given ID.
    ///
    /// If a shard has been queued but has not yet been initiated, then this
    /// will return `false`.
    pub async fn has(&self, shard_id: ShardId) -> bool {
        self.runners.lock().await.contains_key(&shard_id)
    }

    /// Initializes all shards that the manager is responsible for.
    ///
    /// This will communicate shard boots with the [`ShardQueuer`] so that they
    /// are properly queued.
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
    pub async fn set_shards(&mut self, index: u64, init: u64, total: u64) {
        self.shutdown_all().await;

        self.shard_index = index;
        self.shard_init = init;
        self.shard_total = total;
    }

    /// Restarts a shard runner.
    ///
    /// This sends a shutdown signal to a shard's associated [`ShardRunner`],
    /// and then queues a initialization of a shard runner for the same shard
    /// via the [`ShardQueuer`].
    ///
    /// # Examples
    ///
    /// Creating a client and then restarting a shard by ID:
    ///
    /// _(note: in reality this precise code doesn't have an effect since the
    /// shard would not yet have been initialized via [`initialize`], but the
    /// concept is the same)_
    ///
    /// ```rust,no_run
    /// use serenity::client::bridge::gateway::ShardId;
    /// use serenity::client::{Client, EventHandler};
    /// use std::env;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler { }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let mut client = Client::builder(&token).event_handler(Handler).await?;
    ///
    /// // restart shard ID 7
    /// client.shard_manager.lock().await.restart(ShardId(7)).await;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ShardRunner`]: super::ShardRunner
    /// [`initialize`]: Self::initialize
    #[instrument(skip(self))]
    pub async fn restart(&mut self, shard_id: ShardId) {
        info!("Restarting shard {}", shard_id);
        self.shutdown(shard_id, 4000).await;

        let shard_total = self.shard_total;

        self.boot([shard_id, ShardId(shard_total)]);
    }

    /// Returns the [`ShardId`]s of the shards that have been instantiated and
    /// currently have a valid [`ShardRunner`].
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[instrument(skip(self))]
    pub async fn shards_instantiated(&self) -> Vec<ShardId> {
        self.runners.lock().await.keys().cloned().collect()
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// Returns a boolean indicating whether a shard runner was present. This is
    /// _not_ necessary an indicator of whether the shard runner was
    /// successfully shut down.
    ///
    /// **Note**: If the receiving end of an mpsc channel - theoretically owned
    /// by the shard runner - no longer exists, then the shard runner will not
    /// know it should shut down. This _should never happen_. It may already be
    /// stopped.
    #[instrument(skip(self))]
    #[allow(clippy::let_underscore_must_use)]
    pub async fn shutdown(&mut self, shard_id: ShardId, code: u16) {
        info!("Shutting down shard {}", shard_id);

        let _ = self.shard_queuer.unbounded_send(ShardQueuerMessage::ShutdownShard(shard_id, code));

        const TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(5);
        match timeout(TIMEOUT, self.shard_shutdown.next()).await {
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
                warn!("Failed to cleanly shutdown shard {}, reached timeout: {:?}", shard_id, why,)
            },
        }

        self.runners.lock().await.remove(&shard_id);
    }

    /// Sends a shutdown message for all shards that the manager is responsible
    /// for that are still known to be running.
    ///
    /// If you only need to shutdown a select number of shards, prefer looping
    /// over the [`shutdown`] method.
    ///
    /// [`shutdown`]: Self::shutdown
    #[instrument(skip(self))]
    #[allow(clippy::let_underscore_must_use)]
    pub async fn shutdown_all(&mut self) {
        let keys = {
            let runners = self.runners.lock().await;

            if runners.is_empty() {
                return;
            }

            runners.keys().cloned().collect::<Vec<_>>()
        };

        info!("Shutting down all shards");

        for shard_id in keys {
            self.shutdown(shard_id, 1000).await;
        }

        let _ = self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown);
        let _ = self.monitor_tx.unbounded_send(ShardManagerMessage::ShutdownInitiated);
    }

    #[instrument(skip(self))]
    fn boot(&mut self, shard_info: [ShardId; 2]) {
        info!("Telling shard queuer to start shard {}", shard_info[0]);

        let msg = ShardQueuerMessage::Start(shard_info[0], shard_info[1]);

        #[allow(clippy::let_underscore_must_use)]
        let _ = self.shard_queuer.unbounded_send(msg);
    }
}

impl Drop for ShardManager {
    /// A custom drop implementation to clean up after the manager.
    ///
    /// This shuts down all active [`ShardRunner`]s and attempts to tell the
    /// [`ShardQueuer`] to shutdown.
    ///
    /// [`ShardRunner`]: super::ShardRunner
    #[allow(clippy::let_underscore_must_use)]
    fn drop(&mut self) {
        let _ = self.shard_queuer.unbounded_send(ShardQueuerMessage::Shutdown);
        let _ = self.monitor_tx.unbounded_send(ShardManagerMessage::ShutdownInitiated);
    }
}

pub struct ShardManagerOptions<'a> {
    pub data: &'a Arc<RwLock<TypeMap>>,
    pub event_handler: &'a Option<Arc<dyn EventHandler>>,
    pub raw_event_handler: &'a Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: &'a Arc<Box<dyn Framework + Send + Sync>>,
    pub shard_index: u64,
    pub shard_init: u64,
    pub shard_total: u64,
    #[cfg(feature = "voice")]
    pub voice_manager: &'a Option<Arc<dyn VoiceGatewayManager + Send + Sync + 'static>>,
    pub ws_url: &'a Arc<Mutex<String>>,
    pub cache_and_http: &'a Arc<CacheAndHttp>,
    pub intents: GatewayIntents,
}
