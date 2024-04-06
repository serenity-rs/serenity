use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use secrecy::{ExposeSecret as _, Secret};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tracing::{debug, error, info, trace, warn};
use url::Url;

use super::{
    ActivityData,
    ChunkGuildFilter,
    ConnectionStage,
    GatewayError,
    PresenceData,
    ReconnectType,
    ShardAction,
    WsClient,
};
use crate::constants::{self, close_codes};
use crate::http::Token;
use crate::internal::prelude::*;
use crate::model::event::{Event, GatewayEvent};
use crate::model::gateway::{GatewayIntents, ShardInfo};
use crate::model::id::{ApplicationId, GuildId};
use crate::model::user::OnlineStatus;

/// A Shard is a higher-level handler for a websocket connection to Discord's gateway.
///
/// The shard allows for sending and receiving messages over the websocket, such as setting the
/// active activity, reconnecting, syncing guilds, and more.
///
/// Refer to the [module-level documentation][module docs] for information on effectively using
/// multiple shards, if you need to.
///
/// Note that there are additional methods available if you are manually managing a shard yourself,
/// although they are hidden from the documentation since there are few use cases for doing such.
///
/// # Stand-alone shards
///
/// You may instantiate a shard yourself - decoupled from the [`Client`] - if you need to. For most
/// use cases, you will not need to do this, and you can leave the client to do it.
///
/// This can be done by passing in the required parameters to [`Self::new`]. You can then manually
/// handle the shard yourself.
///
/// **Note**: You _really_ do not need to do this. Just call one of the appropriate methods on the
/// [`Client`].
///
/// # Examples
///
/// See the documentation for [`Self::new`] on how to use this.
///
/// [`Client`]: crate::Client
/// [`receive`]: #method.receive
/// [docs]: https://discord.com/developers/docs/topics/gateway#sharding
/// [module docs]: crate::gateway#sharding
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
    session_id: Option<FixedString>,
    shard_info: ShardInfo,
    stage: ConnectionStage,
    /// Instant of when the shard was started.
    // This acts as a timeout to determine if the shard has - for some reason - not started within
    // a decent amount of time.
    pub started: Instant,
    token: Secret<Token>,
    ws_url: Arc<str>,
    resume_ws_url: Option<FixedString>,
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
    /// use serenity::gateway::Shard;
    /// use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// use serenity::model::id::ShardId;
    /// use tokio::sync::Mutex;
    /// #
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Arc<Http> = unimplemented!();
    /// let token = Arc::from(std::env::var("DISCORD_BOT_TOKEN")?);
    /// let shard_info = ShardInfo {
    ///     id: ShardId(0),
    ///     total: NonZeroU16::MIN,
    /// };
    ///
    /// // retrieve the gateway response, which contains the URL to connect to
    /// let gateway = Arc::from(http.get_gateway().await?.url);
    /// let shard = Shard::new(gateway, token, shard_info, GatewayIntents::all(), None).await?;
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
        token: Arc<str>,
        shard_info: ShardInfo,
        intents: GatewayIntents,
        presence: Option<PresenceData>,
    ) -> Result<Shard> {
        let client = connect(&ws_url).await?;

        let presence = presence.unwrap_or_default();
        let last_heartbeat_sent = None;
        let last_heartbeat_ack = None;
        let heartbeat_interval = None;
        let last_heartbeat_acknowledged = true;
        let seq = 0;
        let stage = ConnectionStage::Handshake;
        let session_id = None;

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
            token: Token::new(token),
            session_id,
            shard_info,
            ws_url,
            resume_ws_url: None,
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
        match self.client.send_heartbeat(&self.shard_info, Some(self.seq)).await {
            Ok(()) => {
                self.last_heartbeat_sent = Some(Instant::now());
                self.last_heartbeat_acknowledged = false;

                Ok(())
            },
            Err(why) => {
                if let Error::Tungstenite(err) = &why {
                    if let TungsteniteError::Io(err) = &**err {
                        if err.raw_os_error() != Some(32) {
                            debug!("[{:?}] Err heartbeating: {:?}", self.shard_info, err);
                            return Err(Error::Gateway(GatewayError::HeartbeatFailed));
                        }
                    }
                }

                warn!("[{:?}] Other err w/ keepalive: {:?}", self.shard_info, why);
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
        self.session_id.as_deref()
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn set_activity(&mut self, activity: Option<ActivityData>) {
        self.presence.activity = activity;
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn set_presence(&mut self, activity: Option<ActivityData>, status: OnlineStatus) {
        self.set_activity(activity);
        self.set_status(status);
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
        self.shard_info
    }

    /// Returns the current connection stage of the shard.
    pub fn stage(&self) -> ConnectionStage {
        self.stage
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn handle_gateway_dispatch(
        &mut self,
        seq: u64,
        event: JsonMap,
        original_str: &str,
    ) -> Result<(Option<ShardAction>, Option<Event>)> {
        if seq > self.seq + 1 {
            warn!("[{:?}] Sequence off; them: {}, us: {}", self.shard_info, seq, self.seq);
        }

        self.seq = seq;
        let event = Event::deserialize_and_log(event, original_str)?;

        match &event {
            Event::Ready(ready) => {
                debug!("[{:?}] Received Ready", self.shard_info);

                self.resume_ws_url = Some(ready.ready.resume_gateway_url.clone());
                self.session_id = Some(ready.ready.session_id.clone());
                self.stage = ConnectionStage::Connected;

                if let Some(callback) = self.application_id_callback.take() {
                    callback(ready.ready.application.id);
                }
            },
            Event::Resumed(_) => {
                info!("[{:?}] Resumed", self.shard_info);

                self.stage = ConnectionStage::Connected;
                self.last_heartbeat_acknowledged = true;
                self.last_heartbeat_sent = Some(Instant::now());
                self.last_heartbeat_ack = None;
            },
            _ => {},
        }

        Ok((None, Some(event)))
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn handle_gateway_closed(
        &mut self,
        data: Option<&CloseFrame<'static>>,
    ) -> Result<Option<ShardAction>> {
        let num = data.map(|d| d.code.into());
        let clean = num == Some(1000);

        match num {
            Some(close_codes::UNKNOWN_OPCODE) => {
                warn!("[{:?}] Sent invalid opcode.", self.shard_info);
            },
            Some(close_codes::DECODE_ERROR) => {
                warn!("[{:?}] Sent invalid message.", self.shard_info);
            },
            Some(close_codes::NOT_AUTHENTICATED) => {
                warn!("[{:?}] Sent no authentication.", self.shard_info);

                return Err(Error::Gateway(GatewayError::NoAuthentication));
            },
            Some(close_codes::AUTHENTICATION_FAILED) => {
                error!(
                    "[{:?}] Sent invalid authentication, please check the token.",
                    self.shard_info
                );

                return Err(Error::Gateway(GatewayError::InvalidAuthentication));
            },
            Some(close_codes::ALREADY_AUTHENTICATED) => {
                warn!("[{:?}] Already authenticated.", self.shard_info);
            },
            Some(close_codes::INVALID_SEQUENCE) => {
                warn!("[{:?}] Sent invalid seq: {}.", self.shard_info, self.seq);

                self.seq = 0;
            },
            Some(close_codes::RATE_LIMITED) => {
                warn!("[{:?}] Gateway ratelimited.", self.shard_info);
            },
            Some(close_codes::INVALID_SHARD) => {
                warn!("[{:?}] Sent invalid shard data.", self.shard_info);

                return Err(Error::Gateway(GatewayError::InvalidShardData));
            },
            Some(close_codes::SHARDING_REQUIRED) => {
                error!("[{:?}] Shard has too many guilds.", self.shard_info);

                return Err(Error::Gateway(GatewayError::OverloadedShard));
            },
            Some(4006 | close_codes::SESSION_TIMEOUT) => {
                info!("[{:?}] Invalid session.", self.shard_info);

                self.session_id = None;
            },
            Some(close_codes::INVALID_GATEWAY_INTENTS) => {
                error!("[{:?}] Invalid gateway intents have been provided.", self.shard_info);

                return Err(Error::Gateway(GatewayError::InvalidGatewayIntents));
            },
            Some(close_codes::DISALLOWED_GATEWAY_INTENTS) => {
                error!("[{:?}] Disallowed gateway intents have been provided.", self.shard_info);

                return Err(Error::Gateway(GatewayError::DisallowedGatewayIntents));
            },
            Some(other) if !clean => {
                warn!(
                    "[{:?}] Unknown unclean close {}: {:?}",
                    self.shard_info,
                    other,
                    data.map(|d| &d.reason),
                );
            },
            _ => {},
        }

        let resume = num
            .map_or(true, |x| x != close_codes::AUTHENTICATION_FAILED && self.session_id.is_some());

        Ok(Some(if resume {
            ShardAction::Reconnect(ReconnectType::Resume)
        } else {
            ShardAction::Reconnect(ReconnectType::Reidentify)
        }))
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
    pub fn handle_event(
        &mut self,
        event: Result<GatewayEvent>,
    ) -> Result<(Option<ShardAction>, Option<Event>)> {
        let action = match event {
            Ok(GatewayEvent::Dispatch {
                seq,
                data,
                original_str,
            }) => {
                return self.handle_gateway_dispatch(seq, data, &original_str);
            },
            Ok(GatewayEvent::Heartbeat(..)) => {
                info!("[{:?}] Received shard heartbeat", self.shard_info);

                Ok(Some(ShardAction::Heartbeat))
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.last_heartbeat_ack = Some(Instant::now());
                self.last_heartbeat_acknowledged = true;

                trace!("[{:?}] Received heartbeat ack", self.shard_info);

                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                debug!("[{:?}] Received a Hello; interval: {}", self.shard_info, interval);

                if self.stage == ConnectionStage::Resuming {
                    return Ok((None, None));
                }

                self.heartbeat_interval = Some(std::time::Duration::from_millis(interval));

                Ok(Some(if self.stage == ConnectionStage::Handshake {
                    ShardAction::Identify
                } else {
                    debug!("[{:?}] Received late Hello; autoreconnecting", self.shard_info);

                    ShardAction::Reconnect(self.reconnection_type())
                }))
            },
            Ok(GatewayEvent::InvalidateSession(resumable)) => {
                info!("[{:?}] Received session invalidation", self.shard_info);

                Ok(Some(if resumable {
                    ShardAction::Reconnect(ReconnectType::Resume)
                } else {
                    ShardAction::Reconnect(ReconnectType::Reidentify)
                }))
            },
            Ok(GatewayEvent::Reconnect) => Ok(Some(ShardAction::Reconnect(ReconnectType::Resume))),
            Err(Error::Gateway(GatewayError::Closed(data))) => {
                self.handle_gateway_closed(data.as_ref())
            },
            Err(Error::Tungstenite(why)) => {
                info!("[{:?}] Websocket error: {:?}", self.shard_info, why);
                info!("[{:?}] Will attempt to auto-reconnect", self.shard_info);

                Ok(Some(ShardAction::Reconnect(self.reconnection_type())))
            },
            Err(why) => {
                warn!("[{:?}] Unhandled error: {:?}", self.shard_info, why);
                Ok(None)
            },
        };

        action.map(|a| (a, None))
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
        if let Some(last_sent) = self.last_heartbeat_sent {
            if last_sent.elapsed() <= heartbeat_interval {
                return true;
            }
        }

        // If the last heartbeat didn't receive an acknowledgement, then auto-reconnect.
        if !self.last_heartbeat_acknowledged {
            debug!("[{:?}] Last heartbeat not acknowledged", self.shard_info,);

            return false;
        }

        // Otherwise, we're good to heartbeat.
        if let Err(why) = self.heartbeat().await {
            warn!("[{:?}] Err heartbeating: {:?}", self.shard_info, why);

            false
        } else {
            trace!("[{:?}] Heartbeat", self.shard_info);

            true
        }
    }

    /// Calculates the heartbeat latency between the shard and the gateway.
    // Shamelessly stolen from brayzure's commit in eris:
    // <https://github.com/abalabahaha/eris/commit/0ce296ae9a542bcec0edf1c999ee2d9986bed5a6>
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn latency(&self) -> Option<StdDuration> {
        if let (Some(sent), Some(received)) = (self.last_heartbeat_sent, self.last_heartbeat_ack) {
            if received > sent {
                return Some(received - sent);
            }
        }

        None
    }

    /// Performs a deterministic reconnect.
    ///
    /// The type of reconnect is deterministic on whether a [`Self::session_id`].
    ///
    /// If the `session_id` still exists, then a RESUME is sent. If not, then an IDENTIFY is sent.
    ///
    /// Note that, if the shard is already in a stage of [`ConnectionStage::Connecting`], then no
    /// action will be performed.
    pub fn should_reconnect(&mut self) -> Option<ReconnectType> {
        if self.stage == ConnectionStage::Connecting {
            return None;
        }

        Some(self.reconnection_type())
    }

    pub fn reconnection_type(&self) -> ReconnectType {
        if self.session_id().is_some() {
            ReconnectType::Resume
        } else {
            ReconnectType::Reidentify
        }
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large guilds (250 members+).
    /// If a guild is over 250 members, then a full member list will not be downloaded, and must
    /// instead be requested to be sent in "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each chunk only contains
    /// a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be updated with member
    /// chunks.
    ///
    /// # Examples
    ///
    /// Chunk a single guild by Id, limiting to 2000 [`Member`]s, and not
    /// specifying a query parameter:
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::id::GuildId;
    ///
    /// shard
    ///     .chunk_guild(
    ///         GuildId::new(81384788765712384),
    ///         Some(2000),
    ///         false,
    ///         ChunkGuildFilter::None,
    ///         None,
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, and specifying a query parameter of
    /// `"do"` and a nonce of `"request"`:
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::id::GuildId;
    ///
    /// shard
    ///     .chunk_guild(
    ///         GuildId::new(81384788765712384),
    ///         Some(20),
    ///         false,
    ///         ChunkGuildFilter::Query("do".to_owned()),
    ///         Some("request"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Errors if there is a problem with the WS connection.
    ///
    /// [`Event::GuildMembersChunk`]: crate::model::event::Event::GuildMembersChunk
    /// [`Guild`]: crate::model::guild::Guild
    /// [`Member`]: crate::model::guild::Member
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn chunk_guild(
        &mut self,
        guild_id: GuildId,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[{:?}] Requesting member chunks", self.shard_info);

        self.client
            .send_chunk_guild(guild_id, &self.shard_info, limit, presences, filter, nonce)
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
            .send_identify(
                &self.shard_info,
                self.token.expose_secret(),
                self.intents,
                &self.presence,
            )
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
        debug!("[{:?}] Initializing.", self.shard_info);

        // Reconnect to the resume URL if possible, otherwise use the generic URL.
        let ws_url = self.resume_ws_url.as_deref().unwrap_or(&self.ws_url);

        // We need to do two, sort of three things here:
        // - set the stage of the shard as opening the websocket connection
        // - open the websocket connection
        // - if successful, set the current stage as Handshaking
        //
        // This is used to accurately assess whether the state of the shard is accurate when a
        // Hello is received.
        self.stage = ConnectionStage::Connecting;
        self.started = Instant::now();
        let client = connect(ws_url).await?;
        self.stage = ConnectionStage::Handshake;

        Ok(client)
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub fn reset(&mut self) {
        self.last_heartbeat_sent = Some(Instant::now());
        self.last_heartbeat_ack = None;
        self.heartbeat_interval = None;
        self.last_heartbeat_acknowledged = true;
        self.session_id = None;
        self.stage = ConnectionStage::Disconnected;
        self.seq = 0;
    }

    /// # Errors
    ///
    /// Errors if unable to re-establish a websocket connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn resume(&mut self) -> Result<()> {
        debug!("[{:?}] Attempting to resume", self.shard_info);

        self.client = self.reinitialize().await?;
        self.stage = ConnectionStage::Resuming;

        match &self.session_id {
            Some(session_id) => {
                self.client
                    .send_resume(&self.shard_info, session_id, self.seq, self.token.expose_secret())
                    .await
            },
            None => Err(Error::Gateway(GatewayError::NoSessionId)),
        }
    }

    /// # Errors
    ///
    /// Errors if unable to re-establish a websocket connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn reconnect(&mut self) -> Result<()> {
        info!("[{:?}] Attempting to reconnect", self.shard_info());

        self.reset();
        self.client = self.reinitialize().await?;

        Ok(())
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn update_presence(&mut self) -> Result<()> {
        self.client.send_presence_update(&self.shard_info, &self.presence).await
    }
}

async fn connect(base_url: &str) -> Result<WsClient> {
    let url =
        Url::parse(&format!("{base_url}?v={}", constants::GATEWAY_VERSION)).map_err(|why| {
            warn!("Error building gateway URL with base `{}`: {:?}", base_url, why);

            Error::Gateway(GatewayError::BuildingUrl)
        })?;

    WsClient::connect(url).await
}
