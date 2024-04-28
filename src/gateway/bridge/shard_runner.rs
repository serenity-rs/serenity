use std::borrow::Cow;
use std::sync::Arc;

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tracing::{debug, error, info, trace, warn};

use super::event::ShardStageUpdateEvent;
#[cfg(feature = "collector")]
use super::CollectorCallback;
#[cfg(feature = "voice")]
use super::VoiceGatewayManager;
use super::{ShardId, ShardManager, ShardRunnerMessage};
#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::client::dispatch::dispatch_model;
use crate::client::{Context, InternalEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::{GatewayError, ReconnectType, Shard, ShardAction};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
use crate::model::event::{Event, GatewayEvent};

/// A runner for managing a [`Shard`] and its respective WebSocket client.
pub struct ShardRunner {
    data: Arc<dyn std::any::Any + Send + Sync>,
    event_handler: Option<InternalEventHandler>,
    #[cfg(feature = "framework")]
    framework: Option<Arc<dyn Framework>>,
    manager: Arc<ShardManager>,
    // channel to receive messages from the shard manager and dispatches
    runner_rx: Receiver<ShardRunnerMessage>,
    // channel to send messages to the shard runner from the shard manager
    runner_tx: Sender<ShardRunnerMessage>,
    pub(crate) shard: Shard,
    #[cfg(feature = "voice")]
    voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    #[cfg(feature = "collector")]
    pub(crate) collectors: Arc<parking_lot::RwLock<Vec<CollectorCallback>>>,
}

impl ShardRunner {
    /// Creates a new runner for a Shard.
    pub fn new(opt: ShardRunnerOptions) -> Self {
        let (tx, rx) = mpsc::unbounded();

        Self {
            runner_rx: rx,
            runner_tx: tx,
            data: opt.data,
            event_handler: opt.event_handler,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            manager: opt.manager,
            shard: opt.shard,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            #[cfg(feature = "cache")]
            cache: opt.cache,
            http: opt.http,
            #[cfg(feature = "collector")]
            collectors: Arc::new(parking_lot::RwLock::new(vec![])),
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
    /// 3. attempts to retrieve a message from the WebSocket, processing it into a [`GatewayEvent`].
    ///    This will block for 100ms before assuming there is no message available.
    ///
    /// 4. Checks with the [`Shard`] to determine if the gateway event is specifying an action to
    ///    take (e.g. resuming, reconnecting, heartbeating) and then performs that action, if any.
    ///
    /// 5. Dispatches the event via the Client.
    ///
    /// 6. Go back to 1.
    ///
    /// # Errors
    /// Returns errors if the internal WS connection drops in a non-recoverable way.
    ///
    /// [`ShardManager`]: super::ShardManager
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn run(&mut self) -> Result<()> {
        info!("[ShardRunner {:?}] Running", self.shard.shard_info());

        loop {
            trace!("[ShardRunner {:?}] loop iteration started.", self.shard.shard_info());
            if !self.recv().await {
                return Ok(());
            }

            // check heartbeat
            if !self.shard.do_heartbeat().await {
                warn!("[ShardRunner {:?}] Error heartbeating", self.shard.shard_info(),);

                self.request_restart().await;
                return Ok(());
            }

            let pre = self.shard.stage();
            let (event, action, successful) = self.recv_event().await?;
            let post = self.shard.stage();

            if post != pre {
                self.update_manager().await;

                if let Some(InternalEventHandler::Normal(event_handler)) = &self.event_handler {
                    let event_handler = Arc::clone(event_handler);
                    let context = self.make_context();
                    let event = ShardStageUpdateEvent {
                        new: post,
                        old: pre,
                        shard_id: self.shard.shard_info().id,
                    };

                    spawn_named("dispatch::event_handler::shard_stage_update", async move {
                        event_handler.shard_stage_update(context, event).await;
                    });
                }
            }

            match action {
                Some(ShardAction::Reconnect(ReconnectType::Reidentify)) => {
                    self.request_restart().await;
                    return Ok(());
                },
                Some(other) => {
                    if let Err(e) = self.action(&other).await {
                        debug!(
                            "[ShardRunner {:?}] Reconnecting due to error performing {:?}: {:?}",
                            self.shard.shard_info(),
                            other,
                            e
                        );
                        match self.shard.reconnection_type() {
                            ReconnectType::Reidentify => {
                                self.request_restart().await;
                                return Ok(());
                            },
                            ReconnectType::Resume => {
                                if let Err(why) = self.shard.resume().await {
                                    warn!(
                                        "[ShardRunner {:?}] Resume failed, reidentifying: {:?}",
                                        self.shard.shard_info(),
                                        why
                                    );

                                    self.request_restart().await;
                                    return Ok(());
                                }
                            },
                        }
                    }
                },
                None => {},
            }

            if let Some(event) = event {
                let context = self.make_context();
                let can_dispatch = match &self.event_handler {
                    Some(InternalEventHandler::Normal(handler)) => {
                        handler.filter_event(&context, &event)
                    },
                    Some(InternalEventHandler::Raw(handler)) => {
                        handler.filter_event(&context, &event)
                    },
                    None => true,
                };

                if can_dispatch {
                    #[cfg(feature = "collector")]
                    {
                        let read_lock = self.collectors.read();
                        // search all collectors to be removed and clone the Arcs
                        let to_remove: Vec<_> = read_lock
                            .iter()
                            .filter(|callback| !callback.0(&event))
                            .cloned()
                            .collect();
                        drop(read_lock);
                        // remove all found arcs from the collection
                        // this compares the inner pointer of the Arc
                        if !to_remove.is_empty() {
                            self.collectors.write().retain(|f| !to_remove.contains(f));
                        }
                    }
                    spawn_named(
                        "shard_runner::dispatch",
                        dispatch_model(
                            event,
                            context,
                            #[cfg(feature = "framework")]
                            self.framework.clone(),
                            self.event_handler.clone(),
                        ),
                    );
                }
            }

            if !successful && !self.shard.stage().is_connecting() {
                self.request_restart().await;
                return Ok(());
            }
            trace!("[ShardRunner {:?}] loop iteration reached the end.", self.shard.shard_info());
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
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, action)))]
    async fn action(&mut self, action: &ShardAction) -> Result<()> {
        match *action {
            ShardAction::Reconnect(ReconnectType::Reidentify) => {
                self.request_restart().await;
                Ok(())
            },
            ShardAction::Reconnect(ReconnectType::Resume) => self.shard.resume().await,
            ShardAction::Heartbeat => self.shard.heartbeat().await,
            ShardAction::Identify => self.shard.identify().await,
        }
    }

    // Checks if the ID received to shutdown is equivalent to the ID of the shard this runner is
    // responsible. If so, it shuts down the WebSocket client.
    //
    // Returns whether the WebSocket client is still active.
    //
    // If true, the WebSocket client was _not_ shutdown. If false, it was.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn checked_shutdown(&mut self, id: ShardId, close_code: u16) -> bool {
        // First verify the ID so we know for certain this runner is to shutdown.
        if id != self.shard.shard_info().id {
            // Not meant for this runner for some reason, don't shutdown.
            return true;
        }

        // Send a Close Frame to Discord, which allows a bot to "log off"
        drop(
            self.shard
                .client
                .close(Some(CloseFrame {
                    code: close_code.into(),
                    reason: Cow::from(""),
                }))
                .await,
        );

        // In return, we wait for either a Close Frame response, or an error, after which this WS
        // is deemed disconnected from Discord.
        loop {
            match self.shard.client.next().await {
                Some(Ok(tungstenite::Message::Close(_))) => break,
                Some(Err(_)) => {
                    warn!(
                        "[ShardRunner {:?}] Received an error awaiting close frame",
                        self.shard.shard_info(),
                    );
                    break;
                },
                _ => {},
            }
        }

        // Inform the manager that shutdown for this shard has finished.
        self.manager.shutdown_finished(id);
        false
    }

    fn make_context(&self) -> Context {
        Context::new(
            Arc::clone(&self.data),
            self,
            self.shard.shard_info().id,
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
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn handle_rx_value(&mut self, msg: ShardRunnerMessage) -> bool {
        match msg {
            ShardRunnerMessage::Restart(id) => self.checked_shutdown(id, 4000).await,
            ShardRunnerMessage::Shutdown(id, code) => self.checked_shutdown(id, code).await,
            ShardRunnerMessage::ChunkGuild {
                guild_id,
                limit,
                presences,
                filter,
                nonce,
            } => self
                .shard
                .chunk_guild(guild_id, limit, presences, filter, nonce.as_deref())
                .await
                .is_ok(),
            ShardRunnerMessage::Close(code, reason) => {
                let reason = reason.unwrap_or_default();
                let close = CloseFrame {
                    code: code.into(),
                    reason: Cow::from(reason),
                };
                self.shard.client.close(Some(close)).await.is_ok()
            },
            ShardRunnerMessage::Message(msg) => self.shard.client.send(msg).await.is_ok(),
            ShardRunnerMessage::SetActivity(activity) => {
                self.shard.set_activity(activity);
                self.shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetPresence(activity, status) => {
                self.shard.set_presence(activity, status);
                self.shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetStatus(status) => {
                self.shard.set_status(status);
                self.shard.update_presence().await.is_ok()
            },
        }
    }

    #[cfg(feature = "voice")]
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn handle_voice_event(&self, event: &Event) {
        if let Some(voice_manager) = &self.voice_manager {
            match event {
                Event::Ready(_) => {
                    voice_manager
                        .register_shard(self.shard.shard_info().id.0, self.runner_tx.clone())
                        .await;
                },
                Event::VoiceServerUpdate(event) => {
                    voice_manager
                        .server_update(event.guild_id, event.endpoint.as_deref(), &event.token)
                        .await;
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
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv(&mut self) -> bool {
        loop {
            match self.runner_rx.try_next() {
                Ok(Some(value)) => {
                    if !self.handle_rx_value(value).await {
                        return false;
                    }
                },
                Ok(None) => {
                    warn!(
                        "[ShardRunner {:?}] Sending half DC; restarting",
                        self.shard.shard_info(),
                    );

                    self.request_restart().await;
                    return false;
                },
                Err(_) => break,
            }
        }

        // There are no longer any values available.
        true
    }

    /// Returns a received event, as well as whether reading the potentially present event was
    /// successful.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv_event(&mut self) -> Result<(Option<Event>, Option<ShardAction>, bool)> {
        let gateway_event = match self.shard.client.recv_json().await {
            Ok(Some(inner)) => Ok(inner),
            Ok(None) => {
                return Ok((None, None, true));
            },
            Err(Error::Tungstenite(tung_err)) if matches!(*tung_err, TungsteniteError::Io(_)) => {
                debug!("Attempting to auto-reconnect");

                match self.shard.reconnection_type() {
                    ReconnectType::Reidentify => return Ok((None, None, false)),
                    ReconnectType::Resume => {
                        if let Err(why) = self.shard.resume().await {
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

        let is_ack = matches!(gateway_event, Ok(GatewayEvent::HeartbeatAck));
        let (action, event) = match self.shard.handle_event(gateway_event) {
            Ok((action, event)) => (action, event),
            Err(Error::Gateway(
                why @ (GatewayError::InvalidAuthentication
                | GatewayError::InvalidGatewayIntents
                | GatewayError::DisallowedGatewayIntents),
            )) => {
                error!("Shard handler received fatal err: {why:?}");

                self.manager.return_with_value(Err(why.clone())).await;
                return Err(Error::Gateway(why));
            },
            Err(Error::Json(_)) => return Ok((None, None, true)),
            Err(why) => {
                error!("Shard handler recieved err: {why:?}");
                return Ok((None, None, true));
            },
        };

        if is_ack {
            self.update_manager().await;
        }

        #[cfg(feature = "voice")]
        {
            if let Some(event) = &event {
                self.handle_voice_event(event).await;
            }
        }

        Ok((event, action, true))
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn request_restart(&mut self) {
        debug!("[ShardRunner {:?}] Requesting restart", self.shard.shard_info());

        self.update_manager().await;

        let shard_id = self.shard.shard_info().id;
        self.manager.restart_shard(shard_id).await;

        #[cfg(feature = "voice")]
        if let Some(voice_manager) = &self.voice_manager {
            voice_manager.deregister_shard(shard_id.0).await;
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn update_manager(&self) {
        self.manager
            .update_shard_latency_and_stage(
                self.shard.shard_info().id,
                self.shard.latency(),
                self.shard.stage(),
            )
            .await;
    }
}

/// Options to be passed to [`ShardRunner::new`].
pub struct ShardRunnerOptions {
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    pub event_handler: Option<InternalEventHandler>,
    #[cfg(feature = "framework")]
    pub framework: Option<Arc<dyn Framework>>,
    pub manager: Arc<ShardManager>,
    pub shard: Shard,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
}
