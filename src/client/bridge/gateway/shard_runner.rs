use internal::prelude::*;
use internal::ws_impl::ReceiverExt;
use model::event::{Event, GatewayEvent};
use parking_lot::Mutex as ParkingLotMutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use super::super::super::{EventHandler, dispatch};
use super::{LockedShard, ShardId, ShardManagerMessage};
use typemap::ShareMap;
use websocket::WebSocketError;

#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "framework")]
use std::sync::Mutex;

pub struct ShardRunner<H: EventHandler + 'static> {
    data: Arc<ParkingLotMutex<ShareMap>>,
    event_handler: Arc<H>,
    #[cfg(feature = "framework")]
    framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
    manager_tx: Sender<ShardManagerMessage>,
    runner_rx: Receiver<ShardManagerMessage>,
    runner_tx: Sender<ShardManagerMessage>,
    shard: LockedShard,
}

impl<H: EventHandler + 'static> ShardRunner<H> {
    #[cfg(feature = "framework")]
    pub fn new(shard: LockedShard,
               manager_tx: Sender<ShardManagerMessage>,
               framework: Arc<Mutex<Option<Box<Framework + Send>>>>,
               data: Arc<ParkingLotMutex<ShareMap>>,
               event_handler: Arc<H>) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data,
            event_handler,
            framework,
            manager_tx,
            shard,
        }
    }

    #[cfg(not(feature = "framework"))]
    pub fn new(shard: LockedShard,
               manager_tx: Sender<ShardManagerMessage>,
               data: Arc<ParkingLotMutex<ShareMap>>,
               event_handler: Arc<H>) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data,
            event_handler,
            manager_tx,
            shard,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            {
                let mut shard = self.shard.lock();
                let incoming = self.runner_rx.try_recv();

                // Check for an incoming message over the runner channel.
                //
                // If the message is to shutdown, first verify the ID so we know
                // for certain this runner is to shutdown.
                if let Ok(ShardManagerMessage::Shutdown(id)) = incoming {
                    if id.0 == shard.shard_info()[0] {
                        let _ = shard.shutdown_clean();

                        return Ok(());
                    }
                }

                if let Err(why) = shard.check_heartbeat() {
                    error!("Failed to heartbeat and reconnect: {:?}", why);

                    let msg = ShardManagerMessage::Restart(ShardId(shard.shard_info()[0]));
                    let _ = self.manager_tx.send(msg);

                    return Ok(());
                }

                #[cfg(feature = "voice")]
                {
                    shard.cycle_voice_recv();
                }
            }

            let events = self.recv_events();

            for event in events {
                feature_framework! {{
                    dispatch(event,
                            &self.shard,
                            &self.framework,
                            &self.data,
                            &self.event_handler);
                } else {
                    dispatch(event,
                            &self.shard,
                            &self.data,
                            &self.event_handler);
                }}
            }
        }
    }

    pub(super) fn runner_tx(&self) -> Sender<ShardManagerMessage> {
        self.runner_tx.clone()
    }

    fn recv_events(&mut self) -> Vec<Event> {
        let mut shard = self.shard.lock();

        let mut events = vec![];

        loop {
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
                            break;
                        }
                    } else {
                        break;
                    }

                    debug!("Attempting to auto-reconnect");

                    if let Err(why) = shard.autoreconnect() {
                        error!("Failed to auto-reconnect: {:?}", why);
                    }

                    break;
                },
                Err(Error::WebSocket(WebSocketError::NoDataAvailable)) => break,
                other => other,
            };

            let event = match gw_event {
                Ok(Some(event)) => Ok(event),
                Ok(None) => break,
                Err(why) => Err(why),
            };

            let event = match shard.handle_event(event) {
                Ok(Some(event)) => event,
                Ok(None) => continue,
                Err(why) => {
                    error!("Shard handler received err: {:?}", why);

                    continue;
                },
            };

            events.push(event);

            if events.len() > 5 {
                break;
            }
        };

        events
    }
}
