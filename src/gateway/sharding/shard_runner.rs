use futures::channel::mpsc::{
    TryRecvError,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, error, trace, warn};

use super::{Shard, ShardAction, ShardManagerMessage};
use crate::gateway::client::dispatch::EventDispatcher;
use crate::gateway::{ActivityData, ChunkGuildFilter, GatewayError};
use crate::internal::prelude::*;
use crate::model::event::{Event, GatewayEvent, ShardStageUpdateEvent};
#[cfg(feature = "voice")]
use crate::model::id::ChannelId;
use crate::model::id::GuildId;
use crate::model::user::OnlineStatus;

/// A runner for managing a [`Shard`] and its respective WebSocket client.
pub struct ShardRunner {
    // Channel to send messages back to the shard manager
    pub(crate) manager_tx: Sender<ShardManagerMessage>,
    // Channel to receive messages from both the shard manager and dispatches
    pub(crate) runner_rx: Receiver<ShardRunnerMessage>,
    pub(crate) shard: Shard,
    pub(crate) dispatcher: EventDispatcher,
}

impl ShardRunner {
    /// A wrapper around [`ShardRunner::start_loop`] that starts the runner's main loop and
    /// monitors for errors.
    ///
    /// If a fatal error occurs, this will send a message to the shard manager to close the
    /// connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn run(&mut self) {
        if let Err(Error::Gateway(e)) = self.start_loop().await
            && let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::Quit(Err(e)))
        {
            warn!("Failed to send return value: {why}");
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
                self.dispatcher
                    .dispatch(Box::new(Event::ShardStageUpdate(ShardStageUpdateEvent {
                        new: post,
                        old: pre,
                        shard_id: self.shard.shard_info().id,
                    })))
                    .await;
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
                    ShardAction::Dispatch(event) => self.dispatcher.dispatch(event).await,
                }
            }

            trace!("[ShardRunner {:?}] loop iteration reached the end.", self.shard.shard_info());
        }
    }

    /// Shuts down the WebSocket client.
    ///
    /// The Shard will be in an indeterminate state after this call, especially if called after
    /// error.
    ///
    /// Therefore, the only correct code path is to exit out of the ShardRunner loop and discard the
    /// WebSocket client entirely.
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
    }

    // Receives messages over the internal `runner_rx` channel and handles them. Will loop over all
    // queued messages until the channel is empty. Requests a restart if handling a message fails.
    //
    // Returns whether the shard runner can continue executing its main loop.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv(&mut self) -> bool {
        loop {
            // Do not block if the channel is empty
            let msg = match self.runner_rx.try_recv() {
                Ok(msg) => msg,
                Err(TryRecvError::Empty) => return true,
                Err(TryRecvError::Closed) => {
                    // This should never happen, because `ShardManager::runners` always holds a
                    // copy of the other end of the channel, and it's only dropped if the shard is
                    // stopped/restarted (which drops the corresponding `ShardRunner`).
                    warn!(
                        "[ShardRunner {:?}] Internal channel closed; restarting",
                        self.shard.shard_info(),
                    );

                    self.restart().await;
                    return false;
                },
            };

            let res = match msg {
                ShardRunnerMessage::Restart => {
                    self.restart().await;
                    return false;
                },
                ShardRunnerMessage::Shutdown(code) => {
                    self.shutdown(code).await;
                    return false;
                },
                ShardRunnerMessage::ChunkGuild {
                    guild_id,
                    limit,
                    presences,
                    filter,
                    nonce,
                } => {
                    self.shard
                        .chunk_guild(guild_id, limit, presences, filter, nonce.as_deref())
                        .await
                },
                ShardRunnerMessage::SoundboardSounds {
                    guild_ids,
                } => self.shard.request_soundboard_sounds(&guild_ids).await,
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
                    self.shard.update_presence().await
                },
                #[cfg(feature = "voice")]
                ShardRunnerMessage::UpdateVoiceState {
                    guild_id,
                    channel_id,
                    self_mute,
                    self_deaf,
                } => {
                    self.shard.update_voice_state(guild_id, channel_id, self_mute, self_deaf).await
                },
            };

            if let Err(why) = res {
                warn!("[ShardRunner {:?}] Websocket error: {:?}", self.shard.shard_info(), why);

                self.restart().await;
                return false;
            }
        }
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
        let shard_info = self.shard.shard_info();

        self.shutdown(4000).await;

        if let Err(why) =
            self.manager_tx.unbounded_send(ShardManagerMessage::RestartShard(shard_info.id))
        {
            warn!(
                "[ShardRunner {shard_info:?}] Failed to send boot request back to shard manager: {why:?}",
            );
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn update_runner_info(&self) {
        let mut runner_info = self.dispatcher.context.runner_info.write();
        runner_info.latency = self.shard.heartbeat_latency();
        runner_info.stage = self.shard.stage();
    }
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
    /// [Discord docs]: https://docs.discord.com/developers/events/gateway#initiating-a-disconnect
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
    /// Indicates that the client is to send a request soundboard sounds message.
    SoundboardSounds {
        /// The IDs of the [`Guild`] to request soundboard sounds from.
        ///
        /// [`Guild`]: crate::model::guild::Guild
        guild_ids: Vec<GuildId>,
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
