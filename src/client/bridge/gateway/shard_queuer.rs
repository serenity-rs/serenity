use crate::gateway::{InterMessage, Shard};
use crate::internal::prelude::*;
use crate::CacheAndHttp;
use tokio::sync::{Mutex, RwLock};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use futures::{
    StreamExt,
    channel::mpsc::{UnboundedSender as Sender, UnboundedReceiver as Receiver},
};
use tokio::time::{delay_for, timeout, Duration, Instant};
use crate::client::{EventHandler, RawEventHandler};
use super::{
    GatewayIntents,
    ShardId,
    ShardClientMessage,
    ShardManagerMessage,
    ShardMessenger,
    ShardQueuerMessage,
    ShardRunner,
    ShardRunnerInfo,
    ShardRunnerOptions,
};
use crate::gateway::ConnectionStage;
use tracing::{debug, info, warn, instrument};

use typemap_rev::TypeMap;
#[cfg(feature = "voice")]
use crate::client::bridge::voice::ClientVoiceManager;
#[cfg(feature = "framework")]
use crate::framework::Framework;

const WAIT_BETWEEN_BOOTS_IN_SECONDS: u64 = 5;

/// The shard queuer is a simple loop that runs indefinitely to manage the
/// startup of shards.
///
/// A shard queuer instance _should_ be run in its own thread, due to the
/// blocking nature of the loop itself as well as a 5 second thread sleep
/// between shard starts.
pub struct ShardQueuer {
    /// A copy of [`Client::data`] to be given to runners for contextual
    /// dispatching.
    ///
    /// [`Client::data`]: ../../struct.Client.html#structfield.data
    pub data: Arc<RwLock<TypeMap>>,
    /// A reference to an `EventHandler`, such as the one given to the
    /// [`Client`].
    ///
    /// [`Client`]: ../../struct.Client.html
    pub event_handler: Option<Arc<dyn EventHandler>>,
    /// A reference to an `RawEventHandler`, such as the one given to the
    /// [`Client`].
    ///
    /// [`Client`]: ../../struct.Client.html
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    /// A copy of the framework
    #[cfg(feature = "framework")]
    pub framework: Arc<Box<dyn Framework + Send + Sync>>,
    /// The instant that a shard was last started.
    ///
    /// This is used to determine how long to wait between shard IDENTIFYs.
    pub last_start: Option<Instant>,
    /// A copy of the sender channel to communicate with the
    /// [`ShardManagerMonitor`].
    ///
    /// [`ShardManagerMonitor`]: struct.ShardManagerMonitor.html
    pub manager_tx: Sender<ShardManagerMessage>,
    /// The shards that are queued for booting.
    ///
    /// This will typically be filled with previously failed boots.
    pub queue: VecDeque<(u64, u64)>,
    /// A copy of the map of shard runners.
    pub runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// A receiver channel for the shard queuer to be told to start shards.
    pub rx: Receiver<ShardQueuerMessage>,
    /// A copy of the client's voice manager.
    #[cfg(feature = "voice")]
    pub voice_manager: Arc<Mutex<ClientVoiceManager>>,
    /// A copy of the URI to use to connect to the gateway.
    pub ws_url: Arc<Mutex<String>>,
    pub cache_and_http: Arc<CacheAndHttp>,
    pub guild_subscriptions: bool,
    pub intents: Option<GatewayIntents>,
}

impl ShardQueuer {
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
    #[instrument(skip(self))]
    pub async fn run(&mut self) {
        // The duration to timeout from reads over the Rx channel. This can be
        // done in a loop, and if the read times out then a shard can be
        // started if one is presently waiting in the queue.
        const TIMEOUT: Duration = Duration::from_secs(WAIT_BETWEEN_BOOTS_IN_SECONDS);

        loop {
            match timeout(TIMEOUT, self.rx.next()).await {
                Ok(Some(ShardQueuerMessage::Shutdown)) => {
                    debug!("[Shard Queuer] Received to shutdown.");
                    self.shutdown_runners().await;

                    break
                },
                Ok(Some(ShardQueuerMessage::ShutdownShard(shard, code))) => {
                    debug!("[Shard Queuer] Received to shutdown shard {} with {}.", shard.0, code);
                    self.shutdown(shard, code).await;
                },
                Ok(Some(ShardQueuerMessage::Start(id, total))) => {
                    debug!("[Shard Queuer] Received to start shard {} of {}.", id.0, total.0);
                    self.checked_start(id.0, total.0).await;
                },
                Ok(None) => break,
                Err(_) => {
                    if let Some((id, total)) = self.queue.pop_front() {
                        self.checked_start(id, total).await;
                    }
                },
            }
        }
    }

    #[instrument(skip(self))]
    async fn check_last_start(&mut self) {
        let instant = match self.last_start {
            Some(instant) => instant,
            None => return,
        };

        // We must wait 5 seconds between IDENTIFYs to avoid session
        // invalidations.
        let duration = Duration::from_secs(WAIT_BETWEEN_BOOTS_IN_SECONDS);
        let elapsed = instant.elapsed();

        if elapsed >= duration {
            return;
        }

        let to_sleep = duration - elapsed;

        delay_for(to_sleep).await;
    }

    #[instrument(skip(self))]
    async fn checked_start(&mut self, id: u64, total: u64) {
        debug!("[Shard Queuer] Checked start for shard {} out of {}", id, total);
        self.check_last_start().await;

        if let Err(why) = self.start(id, total).await {
            warn!("[Shard Queuer] Err starting shard {}: {:?}", id, why);
            info!("[Shard Queuer] Re-queueing start of shard {}", id);

            self.queue.push_back((id, total));
        }

        self.last_start = Some(Instant::now());
    }

    #[instrument(skip(self))]
    async fn start(&mut self, shard_id: u64, shard_total: u64) -> Result<()> {
        let shard_info = [shard_id, shard_total];

        let shard = Shard::new(
            Arc::clone(&self.ws_url),
            &self.cache_and_http.http.token,
            shard_info,
            self.guild_subscriptions,
            self.intents,
        ).await?;

        let mut runner = ShardRunner::new(ShardRunnerOptions {
            data: Arc::clone(&self.data),
            event_handler: self.event_handler.as_ref().map(|eh| Arc::clone(eh)),
            raw_event_handler: self.raw_event_handler.as_ref().map(|rh| Arc::clone(rh)),
            #[cfg(feature = "framework")]
            framework: Arc::clone(&self.framework),
            manager_tx: self.manager_tx.clone(),
            #[cfg(feature = "voice")]
            voice_manager: Arc::clone(&self.voice_manager),
            shard,
            cache_and_http: Arc::clone(&self.cache_and_http),
        });

        let runner_info = ShardRunnerInfo {
            latency: None,
            runner_tx: ShardMessenger::new(runner.runner_tx()),
            stage: ConnectionStage::Disconnected,
        };

        tokio::spawn(async move {
            let _ = runner.run().await;
            debug!("[ShardRunner {:?}] Stopping", runner.shard.shard_info());
        });

        self.runners.lock().await.insert(ShardId(shard_id), runner_info);

        Ok(())
    }

    #[instrument(skip(self))]
    async fn shutdown_runners(&mut self) {
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
    }

    /// Attempts to shut down the shard runner by Id.
    ///
    /// **Note**: If the receiving end of an mpsc channel - theoretically owned
    /// by the shard runner - no longer exists, then the shard runner will not
    /// know it should shut down. This _should never happen_. It may already be
    /// stopped.
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self, shard_id: ShardId, code: u16) {
        info!("Shutting down shard {}", shard_id);

        if let Some(runner) = self.runners.lock().await.get(&shard_id) {
            let shutdown = ShardManagerMessage::Shutdown(shard_id, code);
            let client_msg = ShardClientMessage::Manager(shutdown);
            let msg = InterMessage::Client(Box::new(client_msg));

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
