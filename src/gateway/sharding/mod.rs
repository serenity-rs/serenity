//! Mechanisms for configuring and managing sharded gateway connections.
//!
//! Sharding is a method for load-balancing bots across separate threads or processes. Sharding is
//! enforced on bots by Discord once they reach a certain number of guilds (2500). Once this
//! threshold is reached, a but must be sharded such that at most 2500 guilds are allocated per
//! shard.
//!
//! The "recommended" number of guilds per shard is _around_ 1000. Sharding allows for bots to be
//! distributed by handing shards off to separate processes or even separate machines in a
//! distributed network (e.g. cloud workers). However, sometimes you may wish for all shards to
//! share some global state. Serenity accomodates both of these usecases.
//!
//! See [Discord's documentation][docs] for more information.
//!
//! This module also provides some lower-level facilities for performing sharding manually:
//!
//! ### [`ShardManager`]
//!
//! The shard manager provides an interface for managing the starting, restarting, and shutdown of
//! shards by spawning tasks, each containing a [`ShardRunner`], and communicating with each task
//! using message passing.
//!
//! ### [`ShardRunner`]
//!
//! The shard runner is responsible for directly running a single shard and communicating with the
//! gateway through its respective WebSocket client. It performs actions such as identifying,
//! reconnecting, resuming, and sending presence updates to the gateway.
//!
//! [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding

mod shard_manager;
mod shard_queue;
mod shard_runner;

use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(any(feature = "transport_compression_zlib", feature = "transport_compression_zstd"))]
use aformat::aformat_into;
use aformat::{ArrayString, CapStr, aformat};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, error, info, trace, warn};
use url::Url;

pub use self::shard_manager::{
    DEFAULT_WAIT_BETWEEN_SHARD_START,
    ShardManager,
    ShardManagerMessage,
    ShardManagerOptions,
    ShardRunnerMetadata,
};
pub use self::shard_queue::ShardQueue;
pub use self::shard_runner::{ShardRunner, ShardRunnerMessage};
use super::{ActivityData, ChunkGuildFilter, GatewayError, PresenceData, WsClient};
use crate::constants::{self, CloseCode};
use crate::internal::prelude::*;
use crate::model::event::{DeserializedEvent, Event, GatewayEvent};
use crate::model::gateway::{ConnectionStage, GatewayIntents, ShardInfo};
#[cfg(feature = "voice")]
use crate::model::id::ChannelId;
use crate::model::id::{ApplicationId, GuildId, ShardId};
use crate::model::user::OnlineStatus;

/// An abstract handler for a websocket connection to Discord's gateway.
///
/// Allows a user to send and receive messages over said websocket, including:
///   * setting the current activity
///   * setting the current online status
///   * receiving gateway events
///   * connection management via heartbeating
///
/// Shard management (including starting, restarting, heartbeating), is performed by the [`Client`]
/// automatically on the user's behalf.
///
/// # Stand-alone shards
///
/// You may instantiate a shard yourself - decoupled from the [`Client`] - by calling
/// [`Shard::new`]. Most use cases will not necessitate this, and unless you're doing something
/// really weird you can just let the client do it for you.
///
/// **Note**: You _really_ do not need to do this, especially if using multiple shards. Just call
/// one of the appropriate methods on the [`Client`].
///
/// # Examples
///
/// See the documentation for [`Self::new`] on how to use this.
///
/// [`Client`]: crate::Client
pub struct Shard {
    pub client: WsClient,
    presence: PresenceData,
    application_id_callback: Option<Box<dyn FnOnce(ApplicationId) + Send + Sync>>,
    seq: u64,
    info: ShardInfo,
    stage: ConnectionStage,
    token: Token,
    ws_url: Arc<str>,
    heartbeater: HeartbeatMetadata,
    resume_metadata: Option<ResumeMetadata>,
    compression: TransportCompression,
    pub intents: GatewayIntents,
}

impl Shard {
    /// Instantiates a new instance of a Shard, bypassing the client.
    ///
    /// **Note**: You should likely never need to do this yourself.
    ///
    /// # Examples
    ///
    /// Instantiating a new Shard manually for a bot with no shards, and then listening for events:
    ///
    /// ```rust,no_run
    /// use std::num::NonZeroU16;
    /// use std::sync::Arc;
    ///
    /// use serenity::gateway::{Shard, TransportCompression};
    /// use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// use serenity::model::id::ShardId;
    /// use serenity::secrets::Token;
    /// use tokio::sync::Mutex;
    /// #
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Arc<Http> = unimplemented!();
    /// let token = Token::from_env("DISCORD_TOKEN")?;
    /// let shard_info = ShardInfo {
    ///     id: ShardId(0),
    ///     total: NonZeroU16::MIN,
    /// };
    ///
    /// // retrieve the gateway response, which contains the URL to connect to
    /// let gateway = Arc::from(http.get_gateway().await?.url);
    /// let shard = Shard::new(
    ///     gateway,
    ///     token,
    ///     shard_info,
    ///     GatewayIntents::all(),
    ///     None,
    ///     TransportCompression::None,
    /// )
    /// .await?;
    ///
    /// // at this point, you can create a `loop`, and receive events and match
    /// // their variants
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// On Error, will return either [`Error::Gateway`], [`Error::Tungstenite`] or a Rustls/native
    /// TLS error.
    pub async fn new(
        ws_url: Arc<str>,
        token: Token,
        info: ShardInfo,
        intents: GatewayIntents,
        presence: Option<PresenceData>,
        compression: TransportCompression,
    ) -> Result<Shard> {
        let client = connect(&ws_url, compression).await?;

        let presence = presence.unwrap_or_default();
        let seq = 0;
        let stage = ConnectionStage::Handshake;

        Ok(Shard {
            client,
            presence,
            application_id_callback: None,
            seq,
            stage,
            token,
            info,
            ws_url,
            heartbeater: HeartbeatMetadata::default(),
            resume_metadata: None,
            compression,
            intents,
        })
    }

    /// Sets a callback to be called when the gateway receives the application's ID from Discord.
    ///
    /// Used internally by serenity to set the Http's internal application ID automatically.
    pub fn set_application_id_callback(
        &mut self,
        callback: impl FnOnce(ApplicationId) + Send + Sync + 'static,
    ) {
        self.application_id_callback = Some(Box::new(callback));
    }

    /// Retrieves the current presence of the shard.
    pub fn presence(&self) -> &PresenceData {
        &self.presence
    }

    /// Sends a heartbeat to the gateway with the current sequence.
    ///
    /// This sets the last heartbeat time to now.
    ///
    /// # Errors
    ///
    /// Returns [`GatewayError::HeartbeatFailed`] if there was an error sending a heartbeat.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn heartbeat(&mut self) -> Result<()> {
        match self.client.send_heartbeat(&self.info, Some(self.seq)).await {
            Ok(()) => self.heartbeater.send(),
            Err(why) => {
                if let Error::Tungstenite(err) = &why
                    && let TungsteniteError::Io(err) = &**err
                    && err.raw_os_error() != Some(32)
                {
                    debug!("[{:?}] Err heartbeating: {:?}", self.info, err);
                } else {
                    warn!("[{:?}] Other err w/ keepalive: {:?}", self.info, why);
                }
                return Err(Error::Gateway(GatewayError::HeartbeatFailed));
            },
        }

        Ok(())
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn session_id(&self) -> Option<&str> {
        self.resume_metadata.as_ref().map(|m| &*m.session_id)
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn set_activity(&mut self, activity: Option<ActivityData>) {
        self.presence.activity = activity;
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn set_status(&mut self, mut status: OnlineStatus) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        self.presence.status = status;
    }

    /// Retrieves a copy of the current shard information.
    ///
    /// For example, if using 3 shards in total, and if this is shard 1, then it can be read as
    /// "the second of three shards".
    pub fn shard_info(&self) -> ShardInfo {
        self.info
    }

    /// Returns the current connection stage of the shard.
    pub fn stage(&self) -> ConnectionStage {
        self.stage
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn handle_gateway_dispatch(
        &mut self,
        seq: u64,
        event: DeserializedEvent,
    ) -> Option<Box<Event>> {
        if seq > self.seq + 1 {
            warn!("[{:?}] Sequence off; them: {}, us: {}", self.info, seq, self.seq);
        }

        self.seq = seq;

        let event = match event {
            DeserializedEvent::Success(event) => event,
            DeserializedEvent::Unknown(event) => {
                debug!("Failed to deserialize event data: {}", event.err);
                debug!("Raw event data: {}", event.data);
                return None;
            },
        };

        match &*event {
            Event::Ready(ready) => {
                debug!("[{:?}] Received Ready", self.info);

                self.resume_metadata = Some(ResumeMetadata {
                    session_id: ready.ready.session_id.clone(),
                    resume_ws_url: ready.ready.resume_gateway_url.clone(),
                });
                self.stage = ConnectionStage::Connected;

                if let Some(callback) = self.application_id_callback.take() {
                    callback(ready.ready.application.id);
                }
            },
            Event::Resumed(_) => {
                info!("[{:?}] Resumed", self.info);

                self.stage = ConnectionStage::Connected;
                self.heartbeater.resume();
            },
            _ => {},
        }

        Some(event)
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn handle_gateway_closed(&mut self, data: Option<&CloseFrame>) -> Result<()> {
        if let Some(code) = data.map(|d| d.code) {
            match CloseCode(code.into()) {
                CloseCode::UnknownError => warn!("[{:?}] Unknown gateway error.", self.info),
                CloseCode::UnknownOpcode => warn!("[{:?}] Sent invalid opcode.", self.info),
                CloseCode::DecodeError => warn!("[{:?}] Sent invalid message.", self.info),
                CloseCode::NotAuthenticated => {
                    warn!("[{:?}] Sent no authentication, or session invalidated.", self.info);
                    return Err(Error::Gateway(GatewayError::NoAuthentication));
                },
                CloseCode::AuthenticationFailed => {
                    error!(
                        "[{:?}] Sent invalid authentication, please check the token.",
                        self.info
                    );

                    return Err(Error::Gateway(GatewayError::InvalidAuthentication));
                },
                CloseCode::AlreadyAuthenticated => {
                    warn!("[{:?}] Already authenticated.", self.info);
                },
                CloseCode::InvalidSequence => {
                    warn!("[{:?}] Sent invalid seq: {}.", self.info, self.seq);
                    self.seq = 0;
                },
                CloseCode::RateLimited => warn!("[{:?}] Gateway ratelimited.", self.info),
                CloseCode::SessionTimeout => {
                    info!("[{:?}] Invalid session.", self.info);
                    self.resume_metadata = None;
                },
                CloseCode::InvalidShard => {
                    warn!("[{:?}] Sent invalid shard data.", self.info);
                    return Err(Error::Gateway(GatewayError::InvalidShardData));
                },
                CloseCode::ShardingRequired => {
                    error!("[{:?}] Shard has too many guilds.", self.info);
                    return Err(Error::Gateway(GatewayError::OverloadedShard));
                },
                CloseCode::InvalidApiVersion => {
                    error!("[{:?}] Invalid gateway API version provided.", self.info);
                    return Err(Error::Gateway(GatewayError::InvalidApiVersion));
                },
                CloseCode::InvalidGatewayIntents => {
                    error!("[{:?}] Invalid gateway intents have been provided.", self.info);
                    return Err(Error::Gateway(GatewayError::InvalidGatewayIntents));
                },
                CloseCode::DisallowedGatewayIntents => {
                    error!("[{:?}] Disallowed gateway intents have been provided.", self.info);
                    return Err(Error::Gateway(GatewayError::DisallowedGatewayIntents));
                },
                _ => warn!(
                    "[{:?}] Unknown close code {}: {:?}",
                    self.info,
                    code,
                    data.map(|d| &d.reason)
                ),
            }
        }
        Ok(())
    }

    /// Handles an event from the gateway over the receiver, requiring the receiver to be passed if
    /// a reconnect needs to occur.
    ///
    /// The best case scenario is that one of two values is returned:
    /// - `Ok(None)`: a heartbeat, late hello, or session invalidation was received;
    /// - `Ok(Some((event, None)))`: an op0 dispatch was received, and the shard's voice state will
    ///   be updated, _if_ the `voice` feature is enabled.
    ///
    /// # Errors
    ///
    /// Returns a [`GatewayError::InvalidAuthentication`] if invalid authentication was sent in the
    /// IDENTIFY.
    ///
    /// Returns a [`GatewayError::InvalidShardData`] if invalid shard data was sent in the
    /// IDENTIFY.
    ///
    /// Returns a [`GatewayError::NoAuthentication`] if no authentication was sent in the IDENTIFY.
    ///
    /// Returns a [`GatewayError::OverloadedShard`] if the shard would have too many guilds
    /// assigned to it.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn handle_event(&mut self, event: Result<GatewayEvent>) -> Result<Option<ShardAction>> {
        match event {
            Ok(GatewayEvent::Dispatch {
                seq,
                event,
            }) => Ok(self.handle_gateway_dispatch(seq, event).map(ShardAction::Dispatch)),
            Ok(GatewayEvent::Heartbeat) => {
                info!("[{:?}] Received a request to heartbeat", self.info);
                Ok(Some(ShardAction::Heartbeat))
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.heartbeater.ack();
                trace!("[{:?}] Received heartbeat ack", self.info);
                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                debug!("[{:?}] Received a Hello; interval: {}", self.info, interval);

                if self.stage == ConnectionStage::Resuming {
                    Ok(None)
                } else {
                    self.heartbeater.set_interval(interval);
                    let action = if self.stage == ConnectionStage::Handshake {
                        ShardAction::Identify
                    } else {
                        debug!("[{:?}] Received late Hello; autoreconnecting", self.info);
                        ShardAction::Reconnect
                    };

                    Ok(Some(action))
                }
            },
            Ok(GatewayEvent::InvalidateSession(resumable)) => {
                info!("[{:?}] Received session invalidation", self.info);
                if !resumable {
                    self.resume_metadata = None;
                }

                Ok(Some(ShardAction::Reconnect))
            },
            Ok(GatewayEvent::Reconnect) => Ok(Some(ShardAction::Reconnect)),
            Err(Error::Gateway(GatewayError::Closed(data))) => {
                self.handle_gateway_closed(data.as_deref())?;
                Ok(Some(ShardAction::Reconnect))
            },
            Err(Error::Tungstenite(why)) => {
                info!("[{:?}] Websocket error: {:?}", self.info, why);
                info!("[{:?}] Will attempt to auto-reconnect", self.info);

                Ok(Some(ShardAction::Reconnect))
            },
            Err(why) => {
                warn!("[{:?}] Unhandled error: {:?}", self.info, why);
                Ok(None)
            },
        }
    }

    /// Does a heartbeat if needed. Returns false if something went wrong and the shard should be
    /// restarted.
    ///
    /// `true` is returned under one of the following conditions:
    /// - the heartbeat interval has not elapsed
    /// - a heartbeat was successfully sent
    /// - there is no known heartbeat interval yet
    ///
    /// `false` is returned under one of the following conditions:
    /// - a heartbeat acknowledgement was not received in time
    /// - an error occurred while heartbeating
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn do_heartbeat(&mut self) -> bool {
        // Check if we already acked the last heartbeat, or didn't receive a heartbeat ACK in time.
        // Otherwise, we're good to heartbeat.
        if let Some(acked) = self.heartbeater.last_heartbeat_acked() {
            if !acked {
                debug!("[{:?}] Last heartbeat not acknowledged", self.info);
            }
            acked
        } else if let Err(why) = self.heartbeat().await {
            warn!("[{:?}] Err heartbeating: {:?}", self.info, why);
            false
        } else {
            trace!("[{:?}] Heartbeat", self.info);
            true
        }
    }

    /// Returns the heartbeat latency between the shard and the gateway.
    pub fn heartbeat_latency(&self) -> Option<Duration> {
        self.heartbeater.latency()
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large guilds. If a guild is
    /// large enough, then a full member list will not be provided upon connection, and must
    /// instead be requested directly. The full list will be sent in "chunks" until all members
    /// matching the request have been sent.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each chunk only contains
    /// a partial amount of the total members.
    ///
    /// # Errors
    /// Errors if there is a problem with the WS connection.
    ///
    /// [`Guild`]: crate::model::guild::Guild
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn chunk_guild(
        &mut self,
        guild_id: GuildId,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[{:?}] Requesting member chunks", self.info);

        self.client.send_chunk_guild(guild_id, &self.info, limit, presences, filter, nonce).await
    }

    /// Requests [Soundboard sounds][soundboard] to be fetched from one or multiple [`Guild`]s.
    ///
    /// This will ask the gateway to start sending soundboard sounds.
    ///
    /// Soundboard sounds are sent as the [`Event::SoundboardSounds`] event.
    ///
    /// # Errors
    /// Errors if there is a problem with the WS connection.
    ///
    /// [`Event::SoundboardSounds`]: crate::model::event::Event::SoundboardSounds
    /// [`Guild`]: crate::model::guild::Guild
    /// [soundboard]: crate::model::soundboard::Soundboard
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn request_soundboard_sounds(&mut self, guild_ids: &[GuildId]) -> Result<()> {
        debug!("[{:?}] Requesting soundboard sounds", self.info);

        self.client.request_soundboard_sounds(guild_ids, &self.info).await
    }

    /// Requests ephemeral channel data for channels in a [`Guild`].
    ///
    /// # Errors
    /// Errors if there is a problem with the WS connection.
    ///
    /// [`Guild`]: crate::model::guild::Guild
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn request_channel_info(&mut self, guild_id: GuildId, fields: &[&str]) -> Result<()> {
        debug!("[{:?}] Requesting channel info", self.info);

        self.client.request_channel_info(&self.info, guild_id, fields).await
    }

    /// Indicates to the gateway that the client wants to join, move, or disconnect from a voice
    /// channel.
    ///
    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg(feature = "voice")]
    pub async fn update_voice_state(
        &mut self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    ) -> Result<()> {
        self.client
            .send_voice_state_update(&self.info, guild_id, channel_id, self_mute, self_deaf)
            .await
    }

    /// Sets the shard as going into identifying stage, which sets:
    /// - the time that the last heartbeat sent as being now
    /// - the `stage` to [`ConnectionStage::Identifying`]
    ///
    /// # Errors
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn identify(&mut self) -> Result<()> {
        self.client
            .send_identify(&self.info, self.token.expose_secret(), self.intents, &self.presence)
            .await?;

        self.heartbeater.send();
        self.stage = ConnectionStage::Identifying;

        Ok(())
    }

    /// Reinitializes an existing WebSocket client, replacing it.
    ///
    /// This will set the stage of the shard before and after instantiation of the client.
    ///
    /// # Errors
    ///
    /// Errors if unable to establish a websocket connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn reinitialize(&mut self) -> Result<WsClient> {
        debug!("[{:?}] Initializing.", self.info);

        // Reconnect to the resume URL if possible, otherwise use the generic URL.
        let ws_url = self
            .resume_metadata
            .as_ref()
            .map_or(self.ws_url.as_ref(), |m| m.resume_ws_url.as_ref());

        // We need to do two, sort of three things here:
        // - set the stage of the shard as opening the websocket connection
        // - open the websocket connection
        // - if successful, set the current stage as Handshaking
        //
        // This is used to accurately assess whether the state of the shard is accurate when a
        // Hello is received.
        self.stage = ConnectionStage::Connecting;
        self.heartbeater.start();
        let client = connect(ws_url, self.compression).await?;
        self.stage = ConnectionStage::Handshake;

        Ok(client)
    }

    /// # Errors
    ///
    /// Errors if unable to re-establish a websocket connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn resume(&mut self) -> Result<()> {
        debug!("[{:?}] Attempting to resume", self.info);

        self.client = self.reinitialize().await?;
        self.stage = ConnectionStage::Resuming;

        if let Some(m) = &self.resume_metadata {
            self.client
                .send_resume(&self.info, &m.session_id, self.seq, self.token.expose_secret())
                .await
        } else {
            Err(Error::Gateway(GatewayError::NoSessionId))
        }
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn update_presence(&mut self) -> Result<()> {
        self.client.send_presence_update(&self.info, &self.presence).await
    }
}

async fn connect(base_url: &str, compression: TransportCompression) -> Result<WsClient> {
    let url = Url::parse(&aformat!(
        "{}?v={}{}",
        CapStr::<64>(base_url),
        constants::GATEWAY_VERSION,
        compression.query_param()
    ))
    .map_err(|why| {
        warn!("Error building gateway URL with base `{base_url}`: {why:?}");
        Error::Gateway(GatewayError::BuildingUrl)
    })?;

    WsClient::connect(url, compression).await
}

struct HeartbeatMetadata {
    /// Instant when the shard was started.
    started: Instant,
    /// When the last heartbeat was sent by the client.
    last_sent: Option<Instant>,
    /// When the last heartbeat was acknowledged by the server.
    last_ack: Option<Instant>,
    /// The heartbeat interval dictated by Discord, if the Hello packet has been received.
    interval: Option<Duration>,
}

impl HeartbeatMetadata {
    fn start(&mut self) {
        self.started = Instant::now();
    }

    fn resume(&mut self) {
        self.last_sent = Some(Instant::now());
        self.last_ack = None;
    }

    fn send(&mut self) {
        self.last_sent = Some(Instant::now());
    }

    fn ack(&mut self) {
        self.last_ack = Some(Instant::now());
    }

    fn set_interval(&mut self, millis: u64) {
        self.interval = Some(Duration::from_millis(millis));
    }

    fn last_heartbeat_acked(&self) -> Option<bool> {
        let Some(interval) = self.interval else {
            // No Hello received yet, wait a bit longer before reconnecting.
            return Some(self.started.elapsed() < Duration::from_secs(15));
        };

        // If the heartbeat interval has not yet passed, then we don't need to perform a keepalive
        // or attempt to reconnect. Similarly, if we've just resumed then simply keep waiting for
        // the first heartbeat ACK.
        if let Some(last_sent) = self.last_sent {
            if last_sent.elapsed() <= interval {
                return Some(true);
            } else if let Some(last_ack) = self.last_ack
                && last_ack <= last_sent
            {
                return Some(false);
            }
        }

        None
    }

    fn latency(&self) -> Option<Duration> {
        if let Some(sent) = self.last_sent
            && let Some(ack) = self.last_ack
            && ack > sent
        {
            return Some(ack - sent);
        }

        None
    }
}

impl Default for HeartbeatMetadata {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            last_sent: None,
            last_ack: None,
            interval: None,
        }
    }
}

struct ResumeMetadata {
    session_id: FixedString,
    resume_ws_url: FixedString,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ShardAction {
    Heartbeat,
    Identify,
    Reconnect,
    Dispatch(Box<Event>),
}

/// Information about a [`ShardRunner`].
///
/// The [`ShardId`] is not included because, as it stands, you probably already know the Id if you
/// obtained this.
#[derive(Debug)]
pub struct ShardRunnerInfo {
    /// The latency between when a heartbeat was sent and when the acknowledgement was received.
    pub latency: Option<Duration>,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}

/// Newtype around a callback that will be called on every incoming request. As long as this
/// collector should still receive events, it should return `true`. Once it returns `false`, it is
/// removed.
#[cfg(feature = "collector")]
#[derive(Clone)]
pub struct CollectorCallback(pub Arc<dyn Fn(&Event) -> bool + Send + Sync>);

/// The transport compression method to use.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportCompression {
    /// No transport compression. Payload compression will be used instead.
    None,

    #[cfg(feature = "transport_compression_zlib")]
    /// Use zlib-stream transport compression.
    Zlib,

    #[cfg(feature = "transport_compression_zstd")]
    /// Use zstd-stream transport compression.
    Zstd,
}

impl TransportCompression {
    fn query_param(self) -> ArrayString<21> {
        #[cfg_attr(
            not(any(
                feature = "transport_compression_zlib",
                feature = "transport_compression_zstd"
            )),
            expect(unused_mut)
        )]
        let mut res = ArrayString::new();
        match self {
            Self::None => {},
            #[cfg(feature = "transport_compression_zlib")]
            Self::Zlib => aformat_into!(res, "&compress=zlib-stream"),
            #[cfg(feature = "transport_compression_zstd")]
            Self::Zstd => aformat_into!(res, "&compress=zstd-stream"),
        }

        res
    }
}
