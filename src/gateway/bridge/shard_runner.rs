use std::borrow::Cow;
use std::sync::Arc;

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tracing::{debug, error, info, instrument, trace, warn};
use typemap_rev::TypeMap;

use super::event::ShardStageUpdateEvent;
#[cfg(feature = "collector")]
use super::CollectorCallback;
#[cfg(feature = "voice")]
use super::VoiceGatewayManager;
use super::{ShardId, ShardManager, ShardRunnerMessage};
#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::client::dispatch::dispatch_model;
use crate::client::{Context, EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::{ConnectionStage, GatewayError, ReconnectType, Shard, ShardAction};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::event::{Event, GatewayEvent};

/// A runner for managing a [`Shard`] and its respective WebSocket client.
#[must_use]
pub struct ShardRunner {
    data: Arc<RwLock<TypeMap>>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    framework: Option<Arc<dyn Framework>>,
    manager: Arc<Mutex<ShardManager>>,
    // channel to receive messages from the shard manager and dispatches
    runner_rx: Receiver<ShardRunnerMessage>,
    // channel to send messages to the shard runner from the shard manager
    runner_tx: Sender<ShardRunnerMessage>,
    #[cfg(feature = "voice")]
    voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    #[cfg(feature = "collector")]
    pub(crate) collectors: Arc<std::sync::Mutex<Vec<CollectorCallback>>>,
}

impl ShardRunner {
    /// Creates a new runner for a Shard.
    pub fn new(opt: ShardRunnerOptions) -> Self {
        let (tx, rx) = mpsc::unbounded();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data: opt.data,
            event_handlers: opt.event_handlers,
            raw_event_handlers: opt.raw_event_handlers,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            manager: opt.manager,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            #[cfg(feature = "cache")]
            cache: opt.cache,
            http: opt.http,
            #[cfg(feature = "collector")]
            collectors: Arc::new(std::sync::Mutex::new(vec![])),
        }
    }

    /// Starts the runner's loop to receive events.
    ///
    /// This runs a loop that performs the following in each iteration:
    ///
    /// 1. checks the receiver for [`ShardRunnerMessage`]s, possibly from the [`ShardManager`], and
    ///    if there is one, acts on it.
    ///
    /// 2. checks if a heartbeat should be sent to the discord Gateway, and if so, sends one.
    ///
    /// 3. attempts to retrieve a message from the WebSocket, processing it into a
    ///    [`GatewayEvent`]. This will block for 100ms before assuming there is no message
    ///    available.
    ///
    /// 4. Checks with the [`Shard`] to determine if the gateway event is specifying an action to
    ///    take (e.g. resuming, reconnecting, heartbeating) and then performs that action, if any.
    ///
    /// 5. Dispatches the event via the Client.
    ///
    /// 6. Go back to 1.
    ///
    /// [`ShardManager`]: super::ShardManager
    #[instrument(skip(self))]
    pub async fn run(&mut self, shard: &Mutex<Shard>) -> Result<()> {
        info!("[ShardRunner {:?}] Running", shard.lock().await.shard_info());

        loop {
            let shard = &mut shard.lock().await;

            trace!("[ShardRunner {:?}] loop iteration started.", shard.shard_info());
            if shard.stage() == ConnectionStage::Disconnected || !self.recv(shard).await? {
                return Ok(());
            }

            // check heartbeat
            if !shard.do_heartbeat().await {
                warn!("[ShardRunner {:?}] Error heartbeating", shard.shard_info(),);

                return self.request_restart(shard).await;
            }

            let pre = shard.stage();
            let (event, action, successful) = self.recv_event(shard).await?;
            let post = shard.stage();

            if post != pre {
                self.update_manager(shard).await;

                for event_handler in self.event_handlers.clone() {
                    let context = self.make_context(shard);
                    let event = ShardStageUpdateEvent {
                        new: post,
                        old: pre,
                        shard_id: ShardId(shard.shard_info().id),
                    };
                    spawn_named("dispatch::event_handler::shard_stage_update", async move {
                        event_handler.shard_stage_update(context, event).await;
                    });
                }
            }

            match action {
                Some(ShardAction::Reconnect(ReconnectType::Reidentify)) => {
                    return self.request_restart(shard).await;
                },
                Some(other) => {
                    if let Err(e) = self.action(shard, &other).await {
                        debug!(
                            "[ShardRunner {:?}] Reconnecting due to error performing {:?}: {:?}",
                            shard.shard_info(),
                            other,
                            e
                        );
                        match shard.reconnection_type() {
                            ReconnectType::Reidentify => return self.request_restart(shard).await,
                            ReconnectType::Resume => {
                                if let Err(why) = shard.resume().await {
                                    warn!(
                                        "[ShardRunner {:?}] Resume failed, reidentifying: {:?}",
                                        shard.shard_info(),
                                        why
                                    );

                                    return self.request_restart(shard).await;
                                }
                            },
                        };
                    }
                },
                None => {},
            }

            if let Some(event) = event {
                #[cfg(feature = "collector")]
                self.collectors.lock().expect("poison").retain_mut(|callback| (callback.0)(&event));

                dispatch_model(
                    event,
                    self.make_context(shard),
                    #[cfg(feature = "framework")]
                    self.framework.clone(),
                    self.event_handlers.clone(),
                    self.raw_event_handlers.clone(),
                )
                .await;
            }

            if !successful && !shard.stage().is_connecting() {
                return self.request_restart(shard).await;
            }
            trace!("[ShardRunner {:?}] loop iteration reached the end.", shard.shard_info());
        }
    }

    /// Clones the internal copy of the Sender to the shard runner.
    pub(super) fn runner_tx(&self) -> Sender<ShardRunnerMessage> {
        self.runner_tx.clone()
    }

    /// Takes an action that a [`Shard`] has determined should happen and then does it.
    ///
    /// For example, if the shard says that an Identify message needs to be sent, this will do
    /// that.
    ///
    /// # Errors
    ///
    /// Returns
    #[instrument(skip(self, action))]
    async fn action(&mut self, shard: &mut Shard, action: &ShardAction) -> Result<()> {
        match *action {
            ShardAction::Reconnect(ReconnectType::Reidentify) => self.request_restart(shard).await,
            ShardAction::Reconnect(ReconnectType::Resume) => shard.resume().await,
            ShardAction::Heartbeat => shard.heartbeat().await,
            ShardAction::Identify => shard.identify().await,
        }
    }

    fn make_context(&self, shard: &Shard) -> Context {
        Context::new(
            Arc::clone(&self.data),
            self,
            shard.shard_info().id,
            Arc::clone(&self.http),
            #[cfg(feature = "cache")]
            Arc::clone(&self.cache),
        )
    }

    // Handles a received value over the shard runner rx channel.
    //
    // Returns a boolean on whether the shard runner can continue.
    //
    // This always returns true, except in the case that the shard manager asked the runner to
    // shutdown.
    #[instrument(skip(self))]
    async fn handle_rx_value(&mut self, shard: &mut Shard, msg: ShardRunnerMessage) -> bool {
        match msg {
            ShardRunnerMessage::Restart => {
                shard.shutdown(4000).await;
                false
            },
            ShardRunnerMessage::ChunkGuild {
                guild_id,
                limit,
                presences,
                filter,
                nonce,
            } => shard
                .chunk_guild(guild_id, limit, presences, filter, nonce.as_deref())
                .await
                .is_ok(),
            ShardRunnerMessage::Close(code, reason) => {
                let reason = reason.unwrap_or_default();
                let close = CloseFrame {
                    code: code.into(),
                    reason: Cow::from(reason),
                };
                shard.client.close(Some(close)).await.is_ok()
            },
            ShardRunnerMessage::Message(msg) => shard.client.send(msg).await.is_ok(),
            ShardRunnerMessage::SetActivity(activity) => {
                // To avoid a clone of `activity`, we do a little bit of trickery here:
                //
                // First, we obtain a reference to the current presence of the shard, and
                // create a new presence tuple of the new activity we received over the
                // channel as well as the online status that the shard already had.
                //
                // We then (attempt to) send the websocket message with the status update,
                // expressively returning:
                // - whether the message successfully sent
                // - the original activity we received over the channel
                shard.set_activity(activity);
                shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetPresence(activity, status) => {
                shard.set_presence(activity, status);
                shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetStatus(status) => {
                shard.set_status(status);
                shard.update_presence().await.is_ok()
            },
        }
    }

    #[cfg(feature = "voice")]
    #[instrument(skip(self))]
    async fn handle_voice_event(&self, shard: &Shard, event: &Event) {
        if let Some(voice_manager) = &self.voice_manager {
            match event {
                Event::Ready(_) => {
                    voice_manager
                        .register_shard(shard.shard_info().id, self.runner_tx.clone())
                        .await;
                },
                Event::VoiceServerUpdate(event) => {
                    if let Some(guild_id) = event.guild_id {
                        voice_manager.server_update(guild_id, &event.endpoint, &event.token).await;
                    }
                },
                Event::VoiceStateUpdate(event) => {
                    if let Some(guild_id) = event.voice_state.guild_id {
                        voice_manager.state_update(guild_id, &event.voice_state).await;
                    }
                },
                _ => {},
            }
        }
    }

    // Receives values over the internal shard runner rx channel and handles them.
    //
    // This will loop over values until there is no longer one.
    //
    // Requests a restart if the sending half of the channel disconnects. This should _never_
    // happen, as the sending half is kept on the runner.
    // Returns whether the shard runner is in a state that can continue.
    #[instrument(skip(self))]
    async fn recv(&mut self, shard: &mut Shard) -> Result<bool> {
        loop {
            match self.runner_rx.try_next() {
                Ok(Some(value)) => {
                    if !self.handle_rx_value(shard, value).await {
                        return Ok(false);
                    }
                },
                Ok(None) => {
                    warn!("[ShardRunner {:?}] Sending half DC; restarting", shard.shard_info());

                    drop(self.request_restart(shard).await);
                    return Ok(false);
                },
                Err(_) => break,
            }
        }

        // There are no longer any values available.

        Ok(true)
    }

    /// Returns a received event, as well as whether reading the potentially present event was
    /// successful.
    #[instrument(skip(self))]
    async fn recv_event(
        &mut self,
        shard: &mut Shard,
    ) -> Result<(Option<Event>, Option<ShardAction>, bool)> {
        let gw_event = match shard.client.recv_json().await {
            Ok(inner) => Ok(inner),
            Err(Error::Tungstenite(TungsteniteError::Io(_))) => {
                debug!("Attempting to auto-reconnect");

                match shard.reconnection_type() {
                    ReconnectType::Reidentify => return Ok((None, None, false)),
                    ReconnectType::Resume => {
                        if let Err(why) = shard.resume().await {
                            warn!("Failed to resume: {:?}", why);

                            // Don't spam reattempts on internet connection loss
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                            return Ok((None, None, false));
                        }
                    },
                }

                return Ok((None, None, true));
            },
            Err(why) => Err(why),
        };

        let event = match gw_event {
            Ok(Some(event)) => Ok(event),
            Ok(None) => return Ok((None, None, true)),
            Err(why) => Err(why),
        };

        let action = match shard.handle_event(&event) {
            Ok(Some(action)) => Some(action),
            Ok(None) => None,
            Err(why) => {
                error!("Shard handler received err: {:?}", why);

                match &why {
                    Error::Gateway(
                        error @ (GatewayError::InvalidAuthentication
                        | GatewayError::InvalidGatewayIntents
                        | GatewayError::DisallowedGatewayIntents),
                    ) => {
                        self.manager.lock().await.return_with_value(Err(error.clone())).await;

                        return Err(why);
                    },
                    _ => return Ok((None, None, true)),
                }
            },
        };

        if let Ok(GatewayEvent::HeartbeatAck) = event {
            self.update_manager(shard).await;
        }

        #[cfg(feature = "voice")]
        {
            if let Ok(GatewayEvent::Dispatch(_, ref event)) = event {
                self.handle_voice_event(shard, event).await;
            }
        }

        let event = match event {
            Ok(GatewayEvent::Dispatch(_, event)) => Some(event),
            _ => None,
        };

        Ok((event, action, true))
    }

    #[instrument(skip(self))]
    async fn request_restart(&mut self, shard: &Shard) -> Result<()> {
        self.update_manager(shard).await;

        debug!("[ShardRunner {:?}] Requesting restart", shard.shard_info());
        let shard_id = ShardId(shard.shard_info().id);
        self.manager.lock().await.restart_shard(shard_id).await;

        #[cfg(feature = "voice")]
        if let Some(voice_manager) = &self.voice_manager {
            voice_manager.deregister_shard(shard_id.0).await;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update_manager(&self, shard: &Shard) {
        self.manager
            .lock()
            .await
            .update_shard_latency_and_stage(
                ShardId(shard.shard_info().id),
                shard.latency(),
                shard.stage(),
            )
            .await;
    }
}

/// Options to be passed to [`ShardRunner::new`].
pub struct ShardRunnerOptions {
    pub data: Arc<RwLock<TypeMap>>,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    pub raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Option<Arc<dyn Framework>>,
    pub manager: Arc<Mutex<ShardManager>>,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
}
