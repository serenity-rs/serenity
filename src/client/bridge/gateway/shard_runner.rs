use internal::prelude::*;
use internal::ws_impl::ReceiverExt;
use model::event::{Event, GatewayEvent};
use parking_lot::Mutex as ParkingLotMutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use super::super::super::{EventHandler, dispatch};
use super::{LockedShard, ShardId, ShardManagerMessage};
use threadpool::ThreadPool;
use typemap::ShareMap;
use websocket::WebSocketError;

#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "framework")]
use std::sync::Mutex;

pub struct ShardRunner<H: EventHandler + Send + Sync + 'static> {
    data: Arc<ParkingLotMutex<ShareMap>>,
    event_handler: Arc<H>,
    #[cfg(feature = "framework")]
    framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
    manager_tx: Sender<ShardManagerMessage>,
    runner_rx: Receiver<ShardManagerMessage>,
    runner_tx: Sender<ShardManagerMessage>,
    shard: LockedShard,
    shard_info: [u64; 2],
    threadpool: ThreadPool,
}

impl<H: EventHandler + Send + Sync + 'static> ShardRunner<H> {
    #[cfg(feature = "framework")]
    pub fn new(
        shard: LockedShard,
        manager_tx: Sender<ShardManagerMessage>,
        framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
        data: Arc<ParkingLotMutex<ShareMap>>,
        event_handler: Arc<H>,
        threadpool: ThreadPool,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let shard_info = shard.lock().shard_info();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data,
            event_handler,
            framework,
            manager_tx,
            shard,
            shard_info,
            threadpool,
        }
    }

    #[cfg(not(feature = "framework"))]
    pub fn new(
        shard: LockedShard,
        manager_tx: Sender<ShardManagerMessage>,
        data: Arc<ParkingLotMutex<ShareMap>>,
        event_handler: Arc<H>,
        threadpool: ThreadPool,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let shard_info = shard.lock().shard_info();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data,
            event_handler,
            manager_tx,
            shard,
            shard_info,
            threadpool,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        debug!("[ShardRunner {:?}] Running", self.shard_info);

        loop {
            {
                let mut shard = self.shard.lock();
                let incoming = self.runner_rx.try_recv();

                // Check for an incoming message over the runner channel.
                //
                // If the message is to shutdown, first verify the ID so we know
                // for certain this runner is to shutdown.
                if let Ok(ShardManagerMessage::Shutdown(id)) = incoming {
                    if id.0 == self.shard_info[0] {
                        let _ = shard.shutdown_clean();

                        return Ok(());
                    }
                }

                if let Err(why) = shard.check_heartbeat() {
                    error!("Failed to heartbeat and reconnect: {:?}", why);

                    return self.request_restart();
                }

                #[cfg(feature = "voice")]
                {
                    shard.cycle_voice_recv();
                }
            }

            let (event, successful) = self.recv_event();

            if let Some(event) = event {
                let data = Arc::clone(&self.data);
                let event_handler = Arc::clone(&self.event_handler);
                let shard = Arc::clone(&self.shard);

                feature_framework! {{
                    let framework = Arc::clone(&self.framework);

                    self.threadpool.execute(|| {
                        dispatch(
                            event,
                            shard,
                            framework,
                            data,
                            event_handler,
                        );
                    });
                } else {
                    self.threadpool.execute(|| {
                        dispatch(
                            event,
                            shard,
                            data,
                            event_handler,
                        );
                    });
                }}
            }

            {
                let shard = self.shard.lock();

                if !successful && !shard.stage().is_connecting() {
                    return self.request_restart();
                }

                if shard.is_shutdown() {
                    return self.request_shutdown();
                }
            }
        }
    }

    pub(super) fn runner_tx(&self) -> Sender<ShardManagerMessage> {
        self.runner_tx.clone()
    }

    /// Returns a received event, as well as whether reading the potentially
    /// present event was successful.
    fn recv_event(&mut self) -> (Option<Event>, bool) {
        let mut shard = self.shard.lock();

        let gw_event = match shard.client.recv_json(GatewayEvent::decode) {
            Err(Error::WebSocket(WebSocketError::IoError(_))) => {
                // Check that an amount of time at least double the
                // heartbeat_interval has passed.
                //
                // If not, continue on trying to receive messages.
                //
                // If it has, attempt to auto-reconnect.
                let last = shard.last_heartbeat_ack();
                let interval = shard.heartbeat_interval();

                if let (Some(last_heartbeat_ack), Some(interval)) = (last, interval) {
                    let seconds_passed = last_heartbeat_ack.elapsed().as_secs();
                    let interval_in_secs = interval / 1000;

                    if seconds_passed <= interval_in_secs * 2 {
                        return (None, true);
                    }
                } else {
                    return (None, true);
                }

                debug!("Attempting to auto-reconnect");

                if let Err(why) = shard.autoreconnect() {
                    error!("Failed to auto-reconnect: {:?}", why);
                }

                return (None, true);
            },
            Err(Error::WebSocket(WebSocketError::NoDataAvailable)) => {
                // This is hit when the websocket client dies this will be
                // hit every iteration.
                return (None, false);
            },
            other => other,
        };

        let event = match gw_event {
            Ok(Some(event)) => Ok(event),
            Ok(None) => return (None, true),
            Err(why) => Err(why),
        };

        let event = match shard.handle_event(event) {
            Ok(Some(event)) => event,
            Ok(None) => return (None, true),
            Err(why) => {
                error!("Shard handler received err: {:?}", why);

                return (None, true);
            },
         };

        (Some(event), true)
    }

    fn request_restart(&self) -> Result<()> {
        debug!("[ShardRunner {:?}] Requesting restart", self.shard_info);
        let msg = ShardManagerMessage::Restart(ShardId(self.shard_info[0]));
        let _ = self.manager_tx.send(msg);

        Ok(())
    }

    fn request_shutdown(&self) -> Result<()> {
        debug!("[ShardRunner {:?}] Requesting shutdown", self.shard_info);
        let _ = self.manager_tx.send(ShardManagerMessage::ShutdownAll);

        Ok(())
    }
}
