use std::borrow::Cow;
use std::sync::Arc;

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tracing::{debug, error, info, instrument, trace, warn};
use typemap_rev::TypeMap;

use super::event::{ClientEvent, ShardStageUpdateEvent};
#[cfg(feature = "collector")]
use super::CollectorCallback;
use super::{ShardClientMessage, ShardId, ShardManagerMessage, ShardRunnerMessage};
#[cfg(feature = "voice")]
use crate::client::bridge::voice::VoiceGatewayManager;
use crate::client::dispatch::{dispatch_client, dispatch_model};
use crate::client::{Context, EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::gateway::{GatewayError, InterMessage, ReconnectType, Shard, ShardAction};
use crate::internal::prelude::*;
use crate::model::event::{Event, GatewayEvent};
use crate::CacheAndHttp;

/// A runner for managing a [`Shard`] and its respective WebSocket client.
pub struct ShardRunner {
    data: Arc<RwLock<TypeMap>>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    framework: Option<Arc<dyn Framework>>,
    manager_tx: Sender<ShardManagerMessage>,
    // channel to receive messages from the shard manager and dispatches
    runner_rx: Receiver<InterMessage>,
    // channel to send messages to the shard runner from the shard manager
    runner_tx: Sender<InterMessage>,
    pub(crate) shard: Shard,
    #[cfg(feature = "voice")]
    voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
    cache_and_http: CacheAndHttp,
    #[cfg(feature = "collector")]
    collectors: Vec<CollectorCallback>,
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
            manager_tx: opt.manager_tx,
            shard: opt.shard,
            #[cfg(feature = "voice")]
            voice_manager: opt.voice_manager,
            cache_and_http: opt.cache_and_http,
            #[cfg(feature = "collector")]
            collectors: vec![],
        }
    }

    /// Starts the runner's loop to receive events.
    ///
    /// This runs a loop that performs the following in each iteration:
    ///
    /// 1. checks the receiver for [`ShardRunnerMessage`]s, possibly from the
    /// [`ShardManager`], and if there is one, acts on it.
    ///
    /// 2. checks if a heartbeat should be sent to the discord Gateway, and if
    /// so, sends one.
    ///
    /// 3. attempts to retrieve a message from the WebSocket, processing it into
    /// a [`GatewayEvent`]. This will block for 100ms before assuming there is
    /// no message available.
    ///
    /// 4. Checks with the [`Shard`] to determine if the gateway event is
    /// specifying an action to take (e.g. resuming, reconnecting, heartbeating)
    /// and then performs that action, if any.
    ///
    /// 5. Dispatches the event via the Client.
    ///
    /// 6. Go back to 1.
    ///
    /// [`ShardManager`]: super::ShardManager
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<()> {
        info!("[ShardRunner {:?}] Running", self.shard.shard_info());

        loop {
            trace!("[ShardRunner {:?}] loop iteration started.", self.shard.shard_info());
            if !self.recv().await? {
                return Ok(());
            }

            // check heartbeat
            if !self.shard.check_heartbeat().await {
                warn!("[ShardRunner {:?}] Error heartbeating", self.shard.shard_info(),);

                return self.request_restart().await;
            }

            let pre = self.shard.stage();
            let (event, action, successful) = self.recv_event().await?;
            let post = self.shard.stage();

            if post != pre {
                self.update_manager();

                let e = ClientEvent::ShardStageUpdate(ShardStageUpdateEvent {
                    new: post,
                    old: pre,
                    shard_id: ShardId(self.shard.shard_info().id),
                });

                dispatch_client(e, self.make_context(), self.event_handlers.clone()).await;
            }

            match action {
                Some(ShardAction::Reconnect(ReconnectType::Reidentify)) => {
                    return self.request_restart().await;
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
                            ReconnectType::Reidentify => return self.request_restart().await,
                            ReconnectType::Resume => {
                                if let Err(why) = self.shard.resume().await {
                                    warn!(
                                        "[ShardRunner {:?}] Resume failed, reidentifying: {:?}",
                                        self.shard.shard_info(),
                                        why
                                    );

                                    return self.request_restart().await;
                                }
                            },
                        };
                    }
                },
                None => {},
            }

            if let Some(event) = event {
                #[cfg(feature = "collector")]
                self.collectors.retain_mut(|callback| (callback.0)(&event));

                dispatch_model(
                    event,
                    self.make_context(),
                    #[cfg(feature = "framework")]
                    self.framework.clone(),
                    self.event_handlers.clone(),
                    self.raw_event_handlers.clone(),
                )
                .await;
            }

            if !successful && !self.shard.stage().is_connecting() {
                return self.request_restart().await;
            }
            trace!("[ShardRunner {:?}] loop iteration reached the end.", self.shard.shard_info());
        }
    }

    /// Clones the internal copy of the Sender to the shard runner.
    pub(super) fn runner_tx(&self) -> Sender<InterMessage> {
        self.runner_tx.clone()
    }

    /// Takes an action that a [`Shard`] has determined should happen and then
    /// does it.
    ///
    /// For example, if the shard says that an Identify message needs to be
    /// sent, this will do that.
    ///
    /// # Errors
    ///
    /// Returns
    #[instrument(skip(self, action))]
    async fn action(&mut self, action: &ShardAction) -> Result<()> {
        match *action {
            ShardAction::Reconnect(ReconnectType::Reidentify) => self.request_restart().await,
            ShardAction::Reconnect(ReconnectType::Resume) => self.shard.resume().await,
            ShardAction::Heartbeat => self.shard.heartbeat().await,
            ShardAction::Identify => self.shard.identify().await,
        }
    }

    // Checks if the ID received to shutdown is equivalent to the ID of the
    // shard this runner is responsible. If so, it shuts down the WebSocket
    // client.
    //
    // Returns whether the WebSocket client is still active.
    //
    // If true, the WebSocket client was _not_ shutdown. If false, it was.
    #[instrument(skip(self))]
    async fn checked_shutdown(&mut self, id: ShardId, close_code: u16) -> bool {
        // First verify the ID so we know for certain this runner is
        // to shutdown.
        if id.0 != self.shard.shard_info().id {
            // Not meant for this runner for some reason, don't
            // shutdown.
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

        // In return, we wait for either a Close Frame response, or an error, after which this WS is deemed
        // disconnected from Discord.
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
                _ => continue,
            }
        }

        // Inform the manager that shutdown for this shard has finished.
        if let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::ShutdownFinished(id))
        {
            warn!(
                "[ShardRunner {:?}] Could not send ShutdownFinished: {:#?}",
                self.shard.shard_info(),
                why,
            );
        }
        false
    }

    fn make_context(&self) -> Context {
        Context::new(
            Arc::clone(&self.data),
            self.runner_tx.clone(),
            self.shard.shard_info().id,
            Arc::clone(&self.cache_and_http.http),
            #[cfg(feature = "cache")]
            Arc::clone(&self.cache_and_http.cache),
        )
    }

    // Handles a received value over the shard runner rx channel.
    //
    // Returns a boolean on whether the shard runner can continue.
    //
    // This always returns true, except in the case that the shard manager asked
    // the runner to shutdown.
    #[instrument(skip(self))]
    async fn handle_rx_value(&mut self, value: InterMessage) -> bool {
        match value {
            InterMessage::Client(value) => match value {
                ShardClientMessage::Manager(msg) => match msg {
                    ShardManagerMessage::Restart(id) => self.checked_shutdown(id, 4000).await,
                    ShardManagerMessage::Shutdown(id, code) => {
                        self.checked_shutdown(id, code).await
                    },
                    ShardManagerMessage::ShutdownAll => {
                        // This variant should never be received.
                        warn!(
                            "[ShardRunner {:?}] Received a ShutdownAll?",
                            self.shard.shard_info(),
                        );

                        true
                    },
                    ShardManagerMessage::ShardUpdate {
                        ..
                    }
                    | ShardManagerMessage::ShutdownInitiated
                    | ShardManagerMessage::ShutdownFinished(_) => {
                        // nb: not sent here
                        true
                    },
                    ShardManagerMessage::ShardDisallowedGatewayIntents
                    | ShardManagerMessage::ShardInvalidAuthentication
                    | ShardManagerMessage::ShardInvalidGatewayIntents => {
                        // These variants should never be received.
                        warn!("[ShardRunner {:?}] Received a ShardError?", self.shard.shard_info(),);

                        true
                    },
                },
                ShardClientMessage::Runner(msg) => match *msg {
                    ShardRunnerMessage::ChunkGuild {
                        guild_id,
                        limit,
                        filter,
                        nonce,
                    } => self
                        .shard
                        .chunk_guild(guild_id, limit, filter, nonce.as_deref())
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
                        // To avoid a clone of `activity`, we do a little bit of
                        // trickery here:
                        //
                        // First, we obtain a reference to the current presence of
                        // the shard, and create a new presence tuple of the new
                        // activity we received over the channel as well as the
                        // online status that the shard already had.
                        //
                        // We then (attempt to) send the websocket message with the
                        // status update, expressively returning:
                        //
                        // - whether the message successfully sent
                        // - the original activity we received over the channel
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
                    #[cfg(feature = "collector")]
                    ShardRunnerMessage::AddCollector(collector) => {
                        self.collectors.push(collector);
                        true
                    },
                },
            },
        }
    }

    #[cfg(feature = "voice")]
    #[instrument(skip(self))]
    async fn handle_voice_event(&self, event: &Event) {
        if let Some(voice_manager) = &self.voice_manager {
            match event {
                Event::Ready(_) => {
                    voice_manager
                        .register_shard(self.shard.shard_info().id, self.runner_tx.clone())
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

    // Receives values over the internal shard runner rx channel and handles
    // them.
    //
    // This will loop over values until there is no longer one.
    //
    // Requests a restart if the sending half of the channel disconnects. This
    // should _never_ happen, as the sending half is kept on the runner.

    // Returns whether the shard runner is in a state that can continue.
    #[instrument(skip(self))]
    async fn recv(&mut self) -> Result<bool> {
        loop {
            match self.runner_rx.try_next() {
                Ok(Some(value)) => {
                    if !self.handle_rx_value(value).await {
                        return Ok(false);
                    }
                },
                Ok(None) => {
                    warn!(
                        "[ShardRunner {:?}] Sending half DC; restarting",
                        self.shard.shard_info(),
                    );

                    drop(self.request_restart().await);
                    return Ok(false);
                },
                Err(_) => break,
            }
        }

        // There are no longer any values available.

        Ok(true)
    }

    /// Returns a received event, as well as whether reading the potentially
    /// present event was successful.
    #[instrument(skip(self))]
    async fn recv_event(&mut self) -> Result<(Option<Event>, Option<ShardAction>, bool)> {
        let gw_event = match self.shard.client.recv_json().await {
            Ok(inner) => Ok(inner),
            Err(Error::Tungstenite(TungsteniteError::Io(_))) => {
                debug!("Attempting to auto-reconnect");

                match self.shard.reconnection_type() {
                    ReconnectType::Reidentify => return Ok((None, None, false)),
                    ReconnectType::Resume => {
                        if let Err(why) = self.shard.resume().await {
                            warn!("Failed to resume: {:?}", why);

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

        let action = match self.shard.handle_event(&event) {
            Ok(Some(action)) => Some(action),
            Ok(None) => None,
            Err(why) => {
                error!("Shard handler received err: {:?}", why);

                match why {
                    Error::Gateway(GatewayError::InvalidAuthentication) => {
                        if self
                            .manager_tx
                            .unbounded_send(ShardManagerMessage::ShardInvalidAuthentication)
                            .is_err()
                        {
                            panic!(
                                "Failed sending InvalidAuthentication error to the shard manager."
                            );
                        }

                        return Err(why);
                    },
                    Error::Gateway(GatewayError::InvalidGatewayIntents) => {
                        if self
                            .manager_tx
                            .unbounded_send(ShardManagerMessage::ShardInvalidGatewayIntents)
                            .is_err()
                        {
                            panic!(
                                "Failed sending InvalidGatewayIntents error to the shard manager."
                            );
                        }

                        return Err(why);
                    },
                    Error::Gateway(GatewayError::DisallowedGatewayIntents) => {
                        if self
                            .manager_tx
                            .unbounded_send(ShardManagerMessage::ShardDisallowedGatewayIntents)
                            .is_err()
                        {
                            panic!("Failed sending DisallowedGatewayIntents error to the shard manager.");
                        }

                        return Err(why);
                    },
                    _ => return Ok((None, None, true)),
                }
            },
        };

        if let Ok(GatewayEvent::HeartbeatAck) = event {
            self.update_manager();
        }

        #[cfg(feature = "voice")]
        {
            if let Ok(GatewayEvent::Dispatch(_, ref event)) = event {
                self.handle_voice_event(event).await;
            }
        }

        let event = match event {
            Ok(GatewayEvent::Dispatch(_, event)) => Some(event),
            _ => None,
        };

        Ok((event, action, true))
    }

    #[instrument(skip(self))]
    async fn request_restart(&mut self) -> Result<()> {
        self.update_manager();

        debug!("[ShardRunner {:?}] Requesting restart", self.shard.shard_info(),);
        let shard_id = ShardId(self.shard.shard_info().id);
        let msg = ShardManagerMessage::Restart(shard_id);

        if let Err(error) = self.manager_tx.unbounded_send(msg) {
            warn!("Error sending request restart: {:?}", error);
        }

        #[cfg(feature = "voice")]
        if let Some(voice_manager) = &self.voice_manager {
            voice_manager.deregister_shard(shard_id.0).await;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    fn update_manager(&self) {
        drop(self.manager_tx.unbounded_send(ShardManagerMessage::ShardUpdate {
            id: ShardId(self.shard.shard_info().id),
            latency: self.shard.latency(),
            stage: self.shard.stage(),
        }));
    }
}

/// Options to be passed to [`ShardRunner::new`].
pub struct ShardRunnerOptions {
    pub data: Arc<RwLock<TypeMap>>,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    pub raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Option<Arc<dyn Framework>>,
    pub manager_tx: Sender<ShardManagerMessage>,
    pub shard: Shard,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    pub cache_and_http: CacheAndHttp,
}
