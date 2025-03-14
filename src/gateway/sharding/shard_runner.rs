use std::sync::{Arc, Mutex};

use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, error, trace, warn};

#[cfg(feature = "collector")]
use super::CollectorCallback;
use super::{Shard, ShardAction, ShardManagerMessage, ShardRunnerInfo, ShardStageUpdateEvent};
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::gateway::VoiceGatewayManager;
use crate::gateway::client::dispatch::dispatch_model;
use crate::gateway::client::{Context, EventHandler, RawEventHandler};
use crate::gateway::{ActivityData, ChunkGuildFilter, GatewayError};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::internal::tokio::spawn_named;
#[cfg(feature = "voice")]
use crate::model::event::Event;
use crate::model::event::GatewayEvent;
#[cfg(feature = "voice")]
use crate::model::id::ChannelId;
use crate::model::id::GuildId;
use crate::model::user::OnlineStatus;

/// A runner for managing a [`Shard`] and its respective WebSocket client.
pub struct ShardRunner {
    data: Arc<dyn std::any::Any + Send + Sync>,
    event_handler: Option<Arc<dyn EventHandler>>,
    raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    framework: Option<Arc<dyn Framework>>,
    runner_info: Arc<Mutex<ShardRunnerInfo>>,
    // channel to send messages back to the shard manager
    manager_tx: Sender<ShardManagerMessage>,
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
            data: opt.data,
            event_handler: opt.event_handler,
            raw_event_handler: opt.raw_event_handler,
            #[cfg(feature = "framework")]
            framework: opt.framework,
            runner_info: opt.runner_info,
            manager_tx: opt.manager_tx,
            runner_rx: rx,
            runner_tx: tx,
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

    /// A wrapper around [`ShardRunner::start_loop`] that starts the runner's main loop and
    /// monitors for errors.
    ///
    /// If a fatal error occurs, this will send a message to the shard manager to close the
    /// connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn run(&mut self) {
        if let Err(Error::Gateway(e)) = self.start_loop().await {
            if let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::Quit(Err(e))) {
                warn!("Failed to send return value: {why}");
            }
        }
        debug!("[ShardRunner {:?}] Stopping", self.shard.shard_info());
    }

    /// Starts the runner's loop to receive events.
    ///
    /// This runs a loop that performs the following in each iteration:
    ///
    /// 1. Checks the receiver for [`ShardRunnerMessage`]s from the [`ShardManager`] and acts on any
    ///    that are received.
    ///
    /// 2. Checks if a heartbeat should be sent to the discord Gateway and sends one if needed.
    ///
    /// 3. Attempts to retrieve a message from the WebSocket, processing it into a [`GatewayEvent`].
    ///    This will block for at most 500ms before assuming there is no message available.
    ///
    /// 4. Checks to determine if the received gateway response is specifying an action to take
    ///    (e.g. resuming, reconnecting, heartbeating), or if it contains an [`Event`] to dispatch
    ///    via the client, then performs said action.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or the intents contained disallowed values.
    ///
    /// [`ShardManager`]: super::ShardManager
    /// [`Event`]: crate::model::event::Event
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_loop(&mut self) -> Result<()> {
        debug!("[ShardRunner {:?}] Running", self.shard.shard_info());

        loop {
            trace!("[ShardRunner {:?}] loop iteration started.", self.shard.shard_info());
            if !self.recv().await {
                return Ok(());
            }

            // check heartbeat
            if !self.shard.do_heartbeat().await {
                warn!("[ShardRunner {:?}] Error heartbeating", self.shard.shard_info(),);

                self.restart().await;
                return Ok(());
            }

            let pre = self.shard.stage();
            let action = self.recv_event().await?;
            let post = self.shard.stage();

            if post != pre {
                self.update_runner_info();

                if let Some(event_handler) = &self.event_handler {
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

            if let Some(action) = action {
                match action {
                    ShardAction::Reconnect => {
                        if !self.reconnect().await {
                            return Ok(());
                        }
                    },
                    ShardAction::Heartbeat => {
                        if let Err(e) = self.shard.heartbeat().await {
                            debug!(
                                "[ShardRunner {:?}] Reconnecting due to error while heartbeating: {:?}",
                                self.shard.shard_info(),
                                e
                            );
                            if !self.reconnect().await {
                                return Ok(());
                            }
                        }
                    },
                    ShardAction::Identify => {
                        if let Err(e) = self.shard.identify().await {
                            debug!(
                                "[ShardRunner {:?}] Reconnecting due to error while identifying: {:?}",
                                self.shard.shard_info(),
                                e
                            );
                            if !self.reconnect().await {
                                return Ok(());
                            }
                        }
                    },
                    ShardAction::Dispatch(event) => {
                        #[cfg(feature = "voice")]
                        {
                            self.handle_voice_event(&event).await;
                        }

                        let context = self.make_context();
                        if self
                            .event_handler
                            .as_ref()
                            .is_none_or(|handler| handler.filter_event(&context, &event))
                            && self
                                .raw_event_handler
                                .as_ref()
                                .is_none_or(|handler| handler.filter_event(&context, &event))
                        {
                            #[cfg(feature = "collector")]
                            self.collectors.write().retain(|callback| (callback.0)(&event));

                            spawn_named(
                                "shard_runner::dispatch",
                                dispatch_model(
                                    *event,
                                    context,
                                    #[cfg(feature = "framework")]
                                    self.framework.clone(),
                                    self.event_handler.clone(),
                                    self.raw_event_handler.clone(),
                                ),
                            );
                        }
                    },
                }
            }

            trace!("[ShardRunner {:?}] loop iteration reached the end.", self.shard.shard_info());
        }
    }

    // Shuts down the WebSocket client.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn shutdown(&mut self, close_code: u16) {
        debug!("[ShardRunner {:?}] Shutting down.", self.shard.shard_info());
        // Send a Close Frame to Discord, which allows a bot to "log off"
        drop(
            self.shard
                .client
                .close(Some(CloseFrame {
                    code: close_code.into(),
                    reason: "".into(),
                }))
                .await,
        );

        // In return, we wait for either a Close Frame response, or an error, after which this WS
        // is deemed disconnected from Discord.
        loop {
            match self.shard.client.next().await {
                Some(Ok(tungstenite::Message::Close(_))) => return,
                Some(Err(_)) => {
                    warn!(
                        "[ShardRunner {:?}] Received an error awaiting close frame",
                        self.shard.shard_info(),
                    );
                    return;
                },
                _ => {},
            }
        }
    }

    // Handles a received value over the shard runner rx channel.
    //
    // Returns a boolean on whether the shard runner can continue.
    //
    // This always returns true, except in the case that the shard manager asked the runner to
    // shutdown or restart.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn handle_rx_value(&mut self, msg: ShardRunnerMessage) -> bool {
        match msg {
            ShardRunnerMessage::Restart => {
                self.restart().await;
                false
            },
            ShardRunnerMessage::Shutdown(code) => {
                self.shutdown(code).await;
                false
            },
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
            ShardRunnerMessage::SoundboardSounds {
                guild_ids,
            } => self.shard.request_soundboard_sounds(&guild_ids).await.is_ok(),
            ShardRunnerMessage::Close(code, reason) => {
                let reason = reason.unwrap_or_default();
                let close = CloseFrame {
                    code: code.into(),
                    reason: reason.into(),
                };
                self.shard.client.close(Some(close)).await.is_ok()
            },
            ShardRunnerMessage::Message(msg) => self.shard.client.send(msg).await.is_ok(),
            ShardRunnerMessage::SetActivity(activity) => {
                self.shard.set_activity(activity);
                self.shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetStatus(status) => {
                self.shard.set_status(status);
                self.shard.update_presence().await.is_ok()
            },
            ShardRunnerMessage::SetPresence {
                activity,
                status,
            } => {
                if let Some(activity) = activity {
                    self.shard.set_activity(activity);
                }
                if let Some(status) = status {
                    self.shard.set_status(status);
                }
                self.shard.update_presence().await.is_ok()
            },
            #[cfg(feature = "voice")]
            ShardRunnerMessage::UpdateVoiceState {
                guild_id,
                channel_id,
                self_mute,
                self_deaf,
            } => self
                .shard
                .update_voice_state(guild_id, channel_id, self_mute, self_deaf)
                .await
                .is_ok(),
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

    // Receives values over the internal shard runner rx channel and handles them. This will loop
    // over values until there is no longer one.
    //
    // Requests a restart if the sending half of the channel disconnects. This should _never_
    // happen, as the sending half is kept on the runner.
    //
    // Returns whether the shard runner is in a state that can continue.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv(&mut self) -> bool {
        while let Ok(msg) = self.runner_rx.try_next() {
            if let Some(value) = msg {
                if !self.handle_rx_value(value).await {
                    return false;
                }
            } else {
                warn!("[ShardRunner {:?}] Sending half DC; restarting", self.shard.shard_info(),);

                self.restart().await;
                return false;
            }
        }

        // There are no longer any values available.
        true
    }

    /// Returns a received event, as well as whether reading the potentially present event was
    /// successful.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv_event(&mut self) -> Result<Option<ShardAction>> {
        let gateway_event = match self.shard.client.recv_json().await {
            Ok(Some(inner)) => Ok(inner),
            Ok(None) => {
                return Ok(None);
            },
            Err(Error::Tungstenite(tung_err)) if matches!(*tung_err, TungsteniteError::Io(_)) => {
                debug!("Attempting to auto-reconnect");

                return Ok(Some(ShardAction::Reconnect));
            },
            Err(why) => Err(why),
        };

        let is_ack = matches!(gateway_event, Ok(GatewayEvent::HeartbeatAck));
        let action = match self.shard.handle_event(gateway_event) {
            Ok(action) => action,
            Err(Error::Gateway(
                why @ (GatewayError::InvalidAuthentication
                | GatewayError::InvalidApiVersion
                | GatewayError::InvalidGatewayIntents
                | GatewayError::DisallowedGatewayIntents),
            )) => {
                error!("Shard handler received fatal err: {why:?}");

                return Err(Error::Gateway(why));
            },
            Err(why) => {
                error!("Shard handler recieved err: {why:?}");
                return Ok(None);
            },
        };

        if is_ack {
            self.update_runner_info();
        }

        Ok(action)
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn reconnect(&mut self) -> bool {
        if self.shard.session_id().is_some() {
            match self.shard.resume().await {
                Ok(()) => true,
                Err(why) => {
                    warn!(
                        "[ShardRunner {:?}] Resume failed, reidentifying: {:?}",
                        self.shard.shard_info(),
                        why,
                    );

                    // Don't spam reattempts on internet connection loss
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    self.restart().await;
                    false
                },
            }
        } else {
            self.restart().await;
            false
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn restart(&mut self) {
        let shard_id = self.shard.shard_info().id;

        #[cfg(feature = "voice")]
        if let Some(voice_manager) = &self.voice_manager {
            voice_manager.deregister_shard(shard_id.0).await;
        }

        self.shutdown(4000).await;

        if let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::Boot(shard_id)) {
            warn!(
                "[ShardRunner {:?}] Failed to send boot request back to shard manager: {why:?}",
                self.shard.shard_info(),
            );
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn update_runner_info(&self) {
        if let Ok(mut runner_info) = self.runner_info.try_lock() {
            runner_info.latency = self.shard.latency();
            runner_info.stage = self.shard.stage();
        }
    }

    fn make_context(&self) -> Context {
        Context {
            data: Arc::clone(&self.data),
            shard: self.runner_tx.clone(),
            manager: self.manager_tx.clone(),
            shard_id: self.shard.shard_info().id,
            http: Arc::clone(&self.http),
            #[cfg(feature = "cache")]
            cache: Arc::clone(&self.cache),
            runner_info: Arc::clone(&self.runner_info),
            #[cfg(feature = "collector")]
            collectors: Arc::clone(&self.collectors),
        }
    }

    pub(super) fn runner_tx(&self) -> Sender<ShardRunnerMessage> {
        self.runner_tx.clone()
    }
}

/// Options to be passed to [`ShardRunner::new`].
pub struct ShardRunnerOptions {
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    pub event_handler: Option<Arc<dyn EventHandler>>,
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Option<Arc<dyn Framework>>,
    pub runner_info: Arc<Mutex<ShardRunnerInfo>>,
    pub manager_tx: Sender<ShardManagerMessage>,
    pub shard: Shard,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager>>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
}

/// A message to send from a shard over a WebSocket.
#[derive(Debug)]
pub enum ShardRunnerMessage {
    /// Indicator that a shard should be restarted.
    Restart,
    /// Indicator that a shard should be shutdown with a specific WebSocket close code. Sending a
    /// code of 1000 or 1001 will invalidate the session and show the current user as logged off.
    /// Any other code will keep the session active until it times out.
    ///
    /// See the [Discord docs].
    ///
    /// [Discord docs]: https://discord.com/developers/docs/events/gateway#initiating-a-disconnect
    Shutdown(u16),
    /// Indicates that the client is to send a member chunk message.
    ChunkGuild {
        /// The IDs of the [`Guild`] to chunk.
        ///
        /// [`Guild`]: crate::model::guild::Guild
        guild_id: GuildId,
        /// The maximum number of members to receive [`GuildMembersChunkEvent`]s for.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        limit: Option<u16>,
        /// Used to specify if we want the presences of the matched members.
        ///
        /// Requires [`crate::model::gateway::GatewayIntents::GUILD_PRESENCES`].
        presences: bool,
        /// A filter to apply to the returned members.
        filter: ChunkGuildFilter,
        /// Optional nonce to identify [`GuildMembersChunkEvent`] responses.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        nonce: Option<String>,
    },
    /// Indicates that the client is to update the shard's presence.
    ///
    /// Pass `None` to keep a value unmodified. The `activity` field is nullable, in other words
    /// passing `Some(None)` will clear the current activity.
    #[expect(clippy::option_option)]
    SetPresence { activity: Option<Option<ActivityData>>, status: Option<OnlineStatus> },
    /// Indicates that the client wants to join, move, or disconnect from a voice channel.
    #[cfg(feature = "voice")]
    UpdateVoiceState {
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    },
}
