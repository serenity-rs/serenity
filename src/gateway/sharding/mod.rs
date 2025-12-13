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

use std::fmt;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

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
};
pub use self::shard_queue::ShardQueue;
pub use self::shard_runner::{ShardRunner, ShardRunnerMessage, ShardRunnerOptions};
use super::{ActivityData, ChunkGuildFilter, GatewayError, PresenceData, WsClient};
use crate::constants::{self, CloseCode};
use crate::internal::prelude::*;
use crate::model::event::{DeserializedEvent, Event, GatewayEvent};
use crate::model::gateway::{GatewayIntents, ShardInfo};
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
    last_heartbeat_sent: Option<Instant>,
    last_heartbeat_ack: Option<Instant>,
    heartbeat_interval: Option<std::time::Duration>,
    application_id_callback: Option<Box<dyn FnOnce(ApplicationId) + Send + Sync>>,
    /// This is used by the heartbeater to determine whether the last heartbeat was sent without an
    /// acknowledgement, and whether to reconnect.
    // This must be set to `true` in `Shard::handle_event`'s `Ok(GatewayEvent::HeartbeatAck)` arm.
    last_heartbeat_acknowledged: bool,
    seq: u64,
    info: ShardInfo,
    stage: ConnectionStage,
    /// Instant of when the shard was started.
    // This acts as a timeout to determine if the shard has - for some reason - not started within
    // a decent amount of time.
    pub started: Instant,
    token: Token,
    ws_url: Arc<str>,
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
        let last_heartbeat_sent = None;
        let last_heartbeat_ack = None;
        let heartbeat_interval = None;
        let last_heartbeat_acknowledged = true;
        let seq = 0;
        let stage = ConnectionStage::Handshake;

        Ok(Shard {
            client,
            presence,
            last_heartbeat_sent,
            last_heartbeat_ack,
            heartbeat_interval,
            application_id_callback: None,
            last_heartbeat_acknowledged,
            seq,
            stage,
            started: Instant::now(),
            token,
            info,
            ws_url,
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

    /// Retrieves the value of when the last heartbeat was sent.
    pub fn last_heartbeat_sent(&self) -> Option<Instant> {
        self.last_heartbeat_sent
    }

    /// Retrieves the value of when the last heartbeat ack was received.
    pub fn last_heartbeat_ack(&self) -> Option<Instant> {
        self.last_heartbeat_ack
    }

    /// Sends a heartbeat to the gateway with the current sequence.
    ///
    /// This sets the last heartbeat time to now, and [`Self::last_heartbeat_acknowledged`] to
    /// `false`.
    ///
    /// # Errors
    ///
    /// Returns [`GatewayError::HeartbeatFailed`] if there was an error sending a heartbeat.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn heartbeat(&mut self) -> Result<()> {
        match self.client.send_heartbeat(&self.info, Some(self.seq)).await {
            Ok(()) => {
                self.last_heartbeat_sent = Some(Instant::now());
                self.last_heartbeat_acknowledged = false;

                Ok(())
            },
            Err(why) => {
                if let Error::Tungstenite(err) = &why
                    && let TungsteniteError::Io(err) = &**err
                    && err.raw_os_error() != Some(32)
                {
                    debug!("[{:?}] Err heartbeating: {:?}", self.info, err);
                    return Err(Error::Gateway(GatewayError::HeartbeatFailed));
                }

                warn!("[{:?}] Other err w/ keepalive: {:?}", self.info, why);
                Err(Error::Gateway(GatewayError::HeartbeatFailed))
            },
        }
    }

    /// Returns the heartbeat interval dictated by Discord, if the Hello packet has been received.
    pub fn heartbeat_interval(&self) -> Option<std::time::Duration> {
        self.heartbeat_interval
    }

    pub fn last_heartbeat_acknowledged(&self) -> bool {
        self.last_heartbeat_acknowledged
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
                self.last_heartbeat_acknowledged = true;
                self.last_heartbeat_sent = Some(Instant::now());
                self.last_heartbeat_ack = None;
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
                info!("[{:?}] Received shard heartbeat", self.info);

                Ok(Some(ShardAction::Heartbeat))
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.last_heartbeat_ack = Some(Instant::now());
                self.last_heartbeat_acknowledged = true;

                trace!("[{:?}] Received heartbeat ack", self.info);

                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                debug!("[{:?}] Received a Hello; interval: {}", self.info, interval);

                if self.stage == ConnectionStage::Resuming {
                    Ok(None)
                } else {
                    self.heartbeat_interval = Some(std::time::Duration::from_millis(interval));
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
                self.handle_gateway_closed(data.as_ref())?;
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
        let Some(heartbeat_interval) = self.heartbeat_interval else {
            // No Hello received yet
            return self.started.elapsed() < StdDuration::from_secs(15);
        };

        // If a duration of time less than the heartbeat_interval has passed, then don't perform a
        // keepalive or attempt to reconnect.
        if let Some(last_sent) = self.last_heartbeat_sent
            && last_sent.elapsed() <= heartbeat_interval
        {
            return true;
        }

        // If the last heartbeat didn't receive an acknowledgement, then auto-reconnect.
        if !self.last_heartbeat_acknowledged {
            debug!("[{:?}] Last heartbeat not acknowledged", self.info,);

            return false;
        }

        // Otherwise, we're good to heartbeat.
        if let Err(why) = self.heartbeat().await {
            warn!("[{:?}] Err heartbeating: {:?}", self.info, why);

            false
        } else {
            trace!("[{:?}] Heartbeat", self.info);

            true
        }
    }

    /// Calculates the heartbeat latency between the shard and the gateway.
    // Shamelessly stolen from brayzure's commit in eris:
    // <https://github.com/abalabahaha/eris/commit/0ce296ae9a542bcec0edf1c999ee2d9986bed5a6>
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn latency(&self) -> Option<StdDuration> {
        if let Some(sent) = self.last_heartbeat_sent
            && let Some(received) = self.last_heartbeat_ack
            && received > sent
        {
            return Some(received - sent);
        }

        None
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

        self.last_heartbeat_sent = Some(Instant::now());
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
        self.started = Instant::now();
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
    pub latency: Option<StdDuration>,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}

/// An event denoting that a shard's connection stage was changed.
///
/// # Examples
///
/// This might happen when a shard changes from [`ConnectionStage::Identifying`] to
/// [`ConnectionStage::Connected`].
#[derive(Clone, Debug, Serialize)]
pub struct ShardStageUpdateEvent {
    /// The new connection stage.
    pub new: ConnectionStage,
    /// The old connection stage.
    pub old: ConnectionStage,
    /// The ID of the shard that had its connection stage change.
    pub shard_id: ShardId,
}

/// Indicates the current connection stage of a [`Shard`].
///
/// This can be useful for knowing which shards are currently "down"/"up".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[non_exhaustive]
pub enum ConnectionStage {
    /// Indicator that the [`Shard`] is normally connected and is not in, e.g., a resume phase.
    Connected,
    /// Indicator that the [`Shard`] is connecting and is in, e.g., a resume phase.
    Connecting,
    /// Indicator that the [`Shard`] is fully disconnected and is not in a reconnecting phase.
    Disconnected,
    /// Indicator that the [`Shard`] is currently initiating a handshake.
    Handshake,
    /// Indicator that the [`Shard`] has sent an IDENTIFY packet and is awaiting a READY packet.
    Identifying,
    /// Indicator that the [`Shard`] has sent a RESUME packet and is awaiting a RESUMED packet.
    Resuming,
}

impl ConnectionStage {
    /// Whether the stage is a form of connecting.
    ///
    /// This will return `true` on:
    /// - [`Connecting`][`ConnectionStage::Connecting`]
    /// - [`Handshake`][`ConnectionStage::Handshake`]
    /// - [`Identifying`][`ConnectionStage::Identifying`]
    /// - [`Resuming`][`ConnectionStage::Resuming`]
    ///
    /// All other variants will return `false`.
    ///
    /// # Examples
    ///
    /// Assert that [`ConnectionStage::Identifying`] is a connecting stage:
    ///
    /// ```rust
    /// use serenity::gateway::ConnectionStage;
    ///
    /// assert!(ConnectionStage::Identifying.is_connecting());
    /// ```
    ///
    /// Assert that [`ConnectionStage::Connected`] is _not_ a connecting stage:
    ///
    /// ```rust
    /// use serenity::gateway::ConnectionStage;
    ///
    /// assert!(!ConnectionStage::Connected.is_connecting());
    /// ```
    #[must_use]
    pub fn is_connecting(self) -> bool {
        use self::ConnectionStage::{Connecting, Handshake, Identifying, Resuming};
        matches!(self, Connecting | Handshake | Identifying | Resuming)
    }
}

impl fmt::Display for ConnectionStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::Disconnected => "disconnected",
            Self::Handshake => "handshaking",
            Self::Identifying => "identifying",
            Self::Resuming => "resuming",
        })
    }
}

/// Newtype around a callback that will be called on every incoming request. As long as this
/// collector should still receive events, it should return `true`. Once it returns `false`, it is
/// removed.
#[cfg(feature = "collector")]
#[derive(Clone)]
pub struct CollectorCallback(pub Arc<dyn Fn(&Event) -> bool + Send + Sync>);

#[cfg(feature = "collector")]
impl fmt::Debug for CollectorCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CollectorCallback").finish()
    }
}

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
