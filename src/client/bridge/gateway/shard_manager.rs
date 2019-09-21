use crate::gateway::InterMessage;
use crate::internal::prelude::*;
use crate::CacheAndHttp;
use parking_lot::Mutex;
use parking_lot::RwLock;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        mpsc::{self, channel, Sender, Receiver},
        Arc,
    },
    thread,
    time::Duration
};
use super::super::super::{EventHandler, RawEventHandler};
use super::{
    ShardClientMessage,
    ShardId,
    ShardManagerMessage,
    ShardManagerMonitor,
    ShardQueuer,
    ShardQueuerMessage,
    ShardRunnerInfo,
};
use threadpool::ThreadPool;
use typemap::ShareMap;
use log::{info, warn};

#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::client::bridge::voice::ClientVoiceManager;

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
/// # use serenity::client::bridge::voice::ClientVoiceManager;
/// # #[cfg(feature = "voice")]
/// # use serenity::model::id::UserId;
/// # #[cfg(feature = "cache")]
/// # use serenity::cache::Cache;
/// #
/// # #[cfg(feature = "framework")]
/// # fn try_main() -> Result<(), Box<Error>> {
/// #
/// use parking_lot::{Mutex, RwLock};
/// use serenity::client::bridge::gateway::{ShardManager, ShardManagerOptions};
/// use serenity::client::{EventHandler, RawEventHandler};
/// // Of note, this imports `typemap`'s `ShareMap` type.
/// use serenity::prelude::*;
/// use serenity::http::Http;
/// use serenity::CacheAndHttp;
/// // Of note, this imports `typemap`'s `ShareMap` type.
/// use serenity::prelude::*;
/// use std::sync::Arc;
/// use std::env;
/// use threadpool::ThreadPool;
///
/// struct Handler;
///
/// impl EventHandler for Handler { }
/// impl RawEventHandler for Handler { }
///
/// # let cache_and_http = Arc::new(CacheAndHttp::default());
/// # let http = &cache_and_http.http;
/// let gateway_url = Arc::new(Mutex::new(http.get_gateway()?.url));
/// let data = Arc::new(RwLock::new(ShareMap::custom()));
/// let event_handler = Arc::new(Handler) as Arc<dyn EventHandler>;
/// let framework = Arc::new(Mutex::new(None));
/// let threadpool = ThreadPool::with_name("my threadpool".to_owned(), 5);
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
///     threadpool,
///     # #[cfg(feature = "voice")]
///     # voice_manager: &Arc::new(Mutex::new(ClientVoiceManager::new(0, UserId(0)))),
///     ws_url: &gateway_url,
///     # cache_and_http: &cache_and_http,
/// });
/// #     Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "framework"))]
/// # fn try_main() -> Result<(), Box<Error>> {
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
///
/// [`Client`]: ../../struct.Client.html
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
    pub fn new(opt: ShardManagerOptions<'_>) -> (Arc<Mutex<Self>>, ShardManagerMonitor) {
        let (thread_tx, thread_rx) = mpsc::channel();
        let (shard_queue_tx, shard_queue_rx) = mpsc::channel();

        let runners = Arc::new(Mutex::new(HashMap::new()));

        let mut shard_queuer = ShardQueuer {
            data: Arc::clone(opt.data),
            event_handler: opt.event_handler.as_ref().map(|h| Arc::clone(h)),
            raw_event_handler: opt.raw_event_handler.as_ref().map(|rh| Arc::clone(rh)),
            #[cfg(feature = "framework")]
            framework: Arc::clone(opt.framework),
            last_start: None,
            manager_tx: thread_tx.clone(),
            queue: VecDeque::new(),
            runners: Arc::clone(&runners),
            rx: shard_queue_rx,
            threadpool: opt.threadpool,
            #[cfg(feature = "voice")]
            voice_manager: Arc::clone(opt.voice_manager),
            ws_url: Arc::clone(opt.ws_url),
            cache_and_http: Arc::clone(&opt.cache_and_http),
        };

        thread::spawn(move || {
            shard_queuer.run();
        });

        let (shutdown_send, shutdown_recv) = channel();
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
    /// will return `false`. Consider double-checking [`is_responsible_for`] to
    /// determine whether this shard manager is responsible for the given shard.
    ///
    /// [`is_responsible_for`]: #method.is_responsible_for
    pub fn has(&self, shard_id: ShardId) -> bool {
        self.runners.lock().contains_key(&shard_id)
    }

    /// Initializes all shards that the manager is responsible for.
    ///
    /// This will communicate shard boots with the [`ShardQueuer`] so that they
    /// are properly queued.
    ///
    /// [`ShardQueuer`]: struct.ShardQueuer.html
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
    pub fn set_shards(&mut self, index: u64, init: u64, total: u64) {
        self.shutdown_all();

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
    /// let token = env::var("DISCORD_TOKEN").unwrap();
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// // restart shard ID 7
    /// client.shard_manager.lock().restart(ShardId(7));
    /// ```
    ///
    /// [`ShardQueuer`]: struct.ShardQueuer.html
    /// [`ShardRunner`]: struct.ShardRunner.html
    /// [`initialize`]: #method.initialize
    pub fn restart(&mut self, shard_id: ShardId) {
        info!("Restarting shard {}", shard_id);
        self.shutdown(shard_id);

        let shard_total = self.shard_total;

        self.boot([shard_id, ShardId(shard_total)]);
    }

    /// Returns the [`ShardId`]s of the shards that have been instantiated and
    /// currently have a valid [`ShardRunner`].
    ///
    /// [`ShardId`]: struct.ShardId.html
    /// [`ShardRunner`]: struct.ShardRunner.html
    pub fn shards_instantiated(&self) -> Vec<ShardId> {
        self.runners.lock().keys().cloned().collect()
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
    pub fn shutdown(&mut self, shard_id: ShardId) -> bool {
        info!("Shutting down shard {}", shard_id);

        if let Some(runner) = self.runners.lock().get(&shard_id) {
            let shutdown = ShardManagerMessage::Shutdown(shard_id);
            let client_msg = ShardClientMessage::Manager(shutdown);
            let msg = InterMessage::Client(Box::new(client_msg));

            if let Err(why) = runner.runner_tx.send(msg) {
                warn!(
                    "Failed to cleanly shutdown shard {}: {:?}",
                    shard_id,
                    why,
                );
            }
            match self.shard_shutdown.recv_timeout(Duration::from_secs(5)) {
                Ok(shutdown_shard_id) =>
                    if shutdown_shard_id != shard_id {
                        warn!(
                            "Failed to cleanly shutdown shard {}: Shutdown channel sent incorrect ID",
                            shard_id,
                        );
                    },
                Err(why) => warn!(
                    "Failed to cleanly shutdown shard {}: {:?}",
                    shard_id,
                    why,
                )
            }
        }

        self.runners.lock().remove(&shard_id).is_some()
    }

    /// Sends a shutdown message for all shards that the manager is responsible
    /// for that are still known to be running.
    ///
    /// If you only need to shutdown a select number of shards, prefer looping
    /// over the [`shutdown`] method.
    ///
    /// [`shutdown`]: #method.shutdown
    pub fn shutdown_all(&mut self) {
        let keys = {
            let runners = self.runners.lock();

            if runners.is_empty() {
                return;
            }

            runners.keys().cloned().collect::<Vec<_>>()
        };

        info!("Shutting down all shards");

        for shard_id in keys {
            self.shutdown(shard_id);
        }

        let _ = self.shard_queuer.send(ShardQueuerMessage::Shutdown);
        let _ = self.monitor_tx.send(ShardManagerMessage::ShutdownInitiated);
    }

    fn boot(&mut self, shard_info: [ShardId; 2]) {
        info!("Telling shard queuer to start shard {}", shard_info[0]);

        let msg = ShardQueuerMessage::Start(shard_info[0], shard_info[1]);
        let _ = self.shard_queuer.send(msg);
    }
}

impl Drop for ShardManager {
    /// A custom drop implementation to clean up after the manager.
    ///
    /// This shuts down all active [`ShardRunner`]s and attempts to tell the
    /// [`ShardQueuer`] to shutdown.
    ///
    /// [`ShardQueuer`]: struct.ShardQueuer.html
    /// [`ShardRunner`]: struct.ShardRunner.html
    fn drop(&mut self) {
        self.shutdown_all();

        if let Err(why) = self.shard_queuer.send(ShardQueuerMessage::Shutdown) {
            warn!("Failed to send shutdown to shard queuer: {:?}", why);
        }
    }
}

pub struct ShardManagerOptions<'a> {
    pub data: &'a Arc<RwLock<ShareMap>>,
    pub event_handler: &'a Option<Arc<dyn EventHandler>>,
    pub raw_event_handler: &'a Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: &'a Arc<Mutex<Option<Box<dyn Framework + Send>>>>,
    pub shard_index: u64,
    pub shard_init: u64,
    pub shard_total: u64,
    pub threadpool: ThreadPool,
    #[cfg(feature = "voice")]
    pub voice_manager: &'a Arc<Mutex<ClientVoiceManager>>,
    pub ws_url: &'a Arc<Mutex<String>>,
    pub cache_and_http: &'a Arc<CacheAndHttp>,
}
