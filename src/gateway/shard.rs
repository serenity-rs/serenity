use chrono::Utc;
use serde_json::Value;
use std::env::consts;
use std::io::Write;
use std::net::Shutdown;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};
use super::{ConnectionStage, GatewayError};
use websocket::client::Url;
use websocket::message::{CloseData, OwnedMessage};
use websocket::stream::sync::AsTcpStream;
use websocket::sync::client::{Client, ClientBuilder};
use websocket::sync::stream::{TcpStream, TlsStream};
use websocket::WebSocketError;
use constants::{self, OpCode, close_codes};
use internal::prelude::*;
use internal::ws_impl::SenderExt;
use model::event::{Event, GatewayEvent};
use model::{Game, GuildId, OnlineStatus};

#[cfg(feature = "voice")]
use std::sync::mpsc::{self, Receiver as MpscReceiver};
#[cfg(feature = "cache")]
use client::CACHE;
#[cfg(feature = "voice")]
use voice::Manager as VoiceManager;
#[cfg(feature = "voice")]
use http;
#[cfg(feature = "cache")]
use utils;

pub type WsClient = Client<TlsStream<TcpStream>>;

type CurrentPresence = (Option<Game>, OnlineStatus, bool);

/// A Shard is a higher-level handler for a websocket connection to Discord's
/// gateway. The shard allows for sending and receiving messages over the
/// websocket, such as setting the active game, reconnecting, syncing guilds,
/// and more.
///
/// Refer to the [module-level documentation][module docs] for information on
/// effectively using multiple shards, if you need to.
///
/// Note that there are additional methods available if you are manually
/// managing a shard yourself, although they are hidden from the documentation
/// since there are few use cases for doing such.
///
/// # Stand-alone shards
///
/// You may instantiate a shard yourself - decoupled from the [`Client`] - if
/// you need to. For most use cases, you will not need to do this, and you can
/// leave the client to do it.
///
/// This can be done by passing in the required parameters to [`new`]. You can
/// then manually handle the shard yourself and receive events via
/// [`receive`].
///
/// **Note**: You _really_ do not need to do this. Just call one of the
/// appropriate methods on the [`Client`].
///
/// # Examples
///
/// See the documentation for [`new`] on how to use this.
///
/// [`Client`]: ../struct.Client.html
/// [`new`]: #method.new
/// [`receive`]: #method.receive
/// [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding
/// [module docs]: index.html#sharding
pub struct Shard {
    pub client: WsClient,
    current_presence: CurrentPresence,
    /// A tuple of:
    ///
    /// - the last instant that a heartbeat was sent
    /// - the last instant that an acknowledgement was received
    ///
    /// This can be used to calculate [`latency`].
    ///
    /// [`latency`]: fn.latency.html
    heartbeat_instants: (Option<Instant>, Option<Instant>),
    heartbeat_interval: Option<u64>,
    /// This is used by the heartbeater to determine whether the last
    /// heartbeat was sent without an acknowledgement, and whether to reconnect.
    // This _must_ be set to `true` in `Shard::handle_event`'s
    // `Ok(GatewayEvent::HeartbeatAck)` arm.
    last_heartbeat_acknowledged: bool,
    seq: u64,
    session_id: Option<String>,
    shard_info: [u64; 2],
    stage: ConnectionStage,
    token: Arc<Mutex<String>>,
    ws_url: Arc<Mutex<String>>,
    /// The voice connections that this Shard is responsible for. The Shard will
    /// update the voice connections' states.
    #[cfg(feature = "voice")]
    pub manager: VoiceManager,
    #[cfg(feature = "voice")]
    manager_rx: MpscReceiver<Value>,
}

impl Shard {
    /// Instantiates a new instance of a Shard, bypassing the client.
    ///
    /// **Note**: You should likely never need to do this yourself.
    ///
    /// # Examples
    ///
    /// Instantiating a new Shard manually for a bot with no shards, and
    /// then listening for events:
    ///
    /// ```rust,ignore
    /// use serenity::gateway::Shard;
    /// use serenity::http;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_BOT_TOKEN").expect("Token in environment");
    /// // retrieve the gateway response, which contains the URL to connect to
    /// let gateway = http::get_gateway().expect("Valid gateway response").url;
    /// let shard = Shard::new(&gateway, &token, None)
    ///     .expect("Working shard");
    ///
    /// // at this point, you can create a `loop`, and receive events and match
    /// // their variants
    /// ```
    pub fn new(ws_url: Arc<Mutex<String>>,
               token: Arc<Mutex<String>>,
               shard_info: [u64; 2])
               -> Result<Shard> {
        let client = connect(&*ws_url.lock().unwrap())?;

        let current_presence = (None, OnlineStatus::Online, false);
        let heartbeat_instants = (None, None);
        let heartbeat_interval = None;
        let last_heartbeat_acknowledged = true;
        let seq = 0;
        let stage = ConnectionStage::Handshake;
        let session_id = None;

        let mut shard =
            feature_voice! {{
                                            let (tx, rx) = mpsc::channel();
        
                                            let user = http::get_current_user()?;
        
                                            Shard {
                                                client,
                                                current_presence,
                                                heartbeat_instants,
                                                heartbeat_interval,
                                                last_heartbeat_acknowledged,
                                                seq,
                                                stage,
                                                token,
                                                session_id,
                                                shard_info,
                                                ws_url,
                                                manager: VoiceManager::new(tx, user.id),
                                                manager_rx: rx,
                                            }
                                        } else {
                                            Shard {
                                                client,
                                                current_presence,
                                                heartbeat_instants,
                                                heartbeat_interval,
                                                last_heartbeat_acknowledged,
                                                seq,
                                                stage,
                                                token,
                                                session_id,
                                                shard_info,
                                                ws_url,
                                            }
                                        }};

        shard.identify()?;

        Ok(shard)
    }

    /// Retrieves a copy of the current shard information.
    ///
    /// The first element is the _current_ shard - 0-indexed - while the second
    /// element is the _total number_ of shards -- 1-indexed.
    ///
    /// For example, if using 3 shards in total, and if this is shard 1, then it
    /// can be read as "the second of three shards".
    ///
    /// # Examples
    ///
    /// Retrieving the shard info for the second shard, out of two shards total:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let shard = Shard::new(mutex.clone(), mutex, [1, 2]).unwrap();
    /// #
    /// assert_eq!(shard.shard_info(), [1, 2]);
    /// ```
    pub fn shard_info(&self) -> [u64; 2] { self.shard_info }

    /// Sets whether the current user is afk. This helps Discord determine where
    /// to send notifications.
    ///
    /// Other presence settings are maintained.
    pub fn set_afk(&mut self, afk: bool) {
        self.current_presence.2 = afk;

        self.update_presence();
    }

    /// Sets the user's current game, if any.
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current game to playing `"Heroes of the Storm"`:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let mut shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// use serenity::model::Game;
    ///
    /// shard.set_game(Some(Game::playing("Heroes of the Storm")));
    /// ```
    pub fn set_game(&mut self, game: Option<Game>) {
        self.current_presence.0 = game;

        self.update_presence();
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid online status, so it is
    /// automatically converted to [`Invisible`].
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current online status for the shard to [`DoNotDisturb`].
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let mut shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// use serenity::model::OnlineStatus;
    ///
    /// shard.set_status(OnlineStatus::DoNotDisturb);
    /// ```
    ///
    /// [`DoNotDisturb`]: ../../model/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Invisible`]: ../../model/enum.OnlineStatus.html#variant.Invisible
    /// [`Offline`]: ../../model/enum.OnlineStatus.html#variant.Offline
    pub fn set_status(&mut self, online_status: OnlineStatus) {
        self.current_presence.1 = match online_status {
            OnlineStatus::Offline => OnlineStatus::Invisible,
            other => other,
        };

        self.update_presence();
    }

    /// Sets the user's full presence information.
    ///
    /// Consider using the individual setters if you only need to modify one of
    /// these.
    ///
    /// # Examples
    ///
    /// Set the current user as playing `"Heroes of the Storm"`, being online,
    /// and not being afk:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let mut shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// shard.set_presence(Some(Game::playing("Heroes of the Storm")), OnlineStatus::Online,
    /// false);
    /// ```
    pub fn set_presence(&mut self, game: Option<Game>, mut status: OnlineStatus, afk: bool) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        self.current_presence = (game, status, afk);

        self.update_presence();
    }

    /// Handles an event from the gateway over the receiver, requiring the
    /// receiver to be passed if a reconnect needs to occur.
    ///
    /// The best case scenario is that one of two values is returned:
    ///
    /// - `Ok(None)`: a heartbeat, late hello, or session invalidation was
    ///   received;
    /// - `Ok(Some((event, None)))`: an op0 dispatch was received, and the
    ///   shard's voice state will be updated, _if_ the `voice` feature is
    ///   enabled.
    ///
    /// # Errors
    ///
    /// Returns a `GatewayError::InvalidAuthentication` if invalid
    /// authentication was sent in the IDENTIFY.
    ///
    /// Returns a `GatewayError::InvalidShardData` if invalid shard data was
    /// sent in the IDENTIFY.
    ///
    /// Returns a `GatewayError::NoAuthentication` if no authentication was sent
    /// in the IDENTIFY.
    ///
    /// Returns a `GatewayError::OverloadedShard` if the shard would have too
    /// many guilds assigned to it.
    #[allow(cyclomatic_complexity)]
    pub(crate) fn handle_event(&mut self, event: Result<GatewayEvent>) -> Result<Option<Event>> {
        match event {
            Ok(GatewayEvent::Dispatch(seq, event)) => {
                match event {
                    Event::Ready(ref ready) => {
                        self.session_id = Some(ready.ready.session_id.clone());
                        self.stage = ConnectionStage::Connected;

                        set_client_timeout(&mut self.client)?;
                    },
                    Event::Resumed(_) => {
                        info!("[Shard {:?}] Resumed", self.shard_info);

                        self.stage = ConnectionStage::Connected;
                    },
                    #[cfg_attr(rustfmt, rustfmt_skip)]
                    ref _other => {
                        #[cfg(feature = "voice")]
                        {
                            self.voice_dispatch(_other);
                        }
                    },
                }

                self.seq = seq;

                Ok(Some(event))
            },
            Ok(GatewayEvent::Heartbeat(s)) => {
                info!("[Shard {:?}] Received shard heartbeat", self.shard_info);

                // Received seq is off -- attempt to resume.
                if s > self.seq + 1 {
                    info!(
                        "[Shard {:?}] Received off sequence (them: {}; us: {}); resuming",
                        self.shard_info,
                        s,
                        self.seq
                    );

                    if self.stage == ConnectionStage::Handshake {
                        self.stage = ConnectionStage::Identifying;

                        self.identify()?;
                    } else {
                        warn!(
                            "[Shard {:?}] Heartbeat during non-Handshake; auto-reconnecting",
                            self.shard_info
                        );

                        return self.autoreconnect().and(Ok(None));
                    }
                }

                let map = json!({
                    "d": Value::Null,
                    "op": OpCode::Heartbeat.num(),
                });
                self.client.send_json(&map)?;

                Ok(None)
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.heartbeat_instants.1 = Some(Instant::now());
                self.last_heartbeat_acknowledged = true;

                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                if interval > 0 {
                    self.heartbeat_interval = Some(interval);
                }

                if self.stage == ConnectionStage::Handshake {
                    self.stage = ConnectionStage::Identifying;

                    Ok(None)
                } else {
                    self.autoreconnect().and(Ok(None))
                }
            },
            Ok(GatewayEvent::InvalidateSession) => {
                info!("[Shard {:?}] Received session invalidation; re-identifying",
                self.shard_info);

                self.seq = 0;
                self.session_id = None;

                self.identify()?;

                Ok(None)
            },
            Ok(GatewayEvent::Reconnect) => self.reconnect().and(Ok(None)),
            Err(Error::Gateway(GatewayError::Closed(data))) => {
                let num = data.as_ref().map(|d| d.status_code);
                let reason = data.map(|d| d.reason);
                let clean = num == Some(1000);

                {
                    let kind = if clean { "Cleanly" } else { "Uncleanly" };

                    info!("[Shard {:?}] {} closing with {:?}: {:?}",
                    self.shard_info,
                    kind,
                    num,
                    reason);
                }

                match num {
                    Some(close_codes::UNKNOWN_OPCODE) => warn!("Sent invalid opcode"),
                    Some(close_codes::DECODE_ERROR) => warn!("Sent invalid message"),
                    Some(close_codes::NOT_AUTHENTICATED) => {
                        warn!("Sent no authentication");

                        return Err(Error::Gateway(GatewayError::NoAuthentication));
                    },
                    Some(close_codes::AUTHENTICATION_FAILED) => {
                        warn!("Sent invalid authentication");

                        return Err(Error::Gateway(GatewayError::InvalidAuthentication));
                    },
                    Some(close_codes::ALREADY_AUTHENTICATED) => warn!("Already authenticated"),
                    Some(close_codes::INVALID_SEQUENCE) => {
                        warn!("[Shard {:?}] Sent invalid seq: {}", self.shard_info, self.seq);

                        self.seq = 0;
                    },
                    Some(close_codes::RATE_LIMITED) => warn!("Gateway ratelimited"),
                    Some(close_codes::INVALID_SHARD) => {
                        warn!("Sent invalid shard data");

                        return Err(Error::Gateway(GatewayError::InvalidShardData));
                    },
                    Some(close_codes::SHARDING_REQUIRED) => {
                        error!("Shard has too many guilds");

                        return Err(Error::Gateway(GatewayError::OverloadedShard));
                    },
                    Some(4006) |
                    Some(close_codes::SESSION_TIMEOUT) => {
                        info!("[Shard {:?}] Invalid session", self.shard_info);

                        self.session_id = None;
                    },
                    Some(other) if !clean => {
                        warn!("[Shard {:?}] Unknown unclean close {}: {:?}",
                        self.shard_info,
                        other,
                        reason);
                    },
                    _ => {},
                }

                let resume = num.map(|x| {
                    x != 1000 && x != close_codes::AUTHENTICATION_FAILED &&
                    self.session_id.is_some()
                }).unwrap_or(false);

                if resume {
                    self.resume().or_else(|_| self.reconnect()).and(Ok(None))
                } else {
                    self.reconnect().and(Ok(None))
                }
            },
            Err(Error::WebSocket(why)) => {
                if let WebSocketError::NoDataAvailable = why {
                    if self.heartbeat_instants.1.is_none() {
                        return Ok(None);
                    }
                }

                warn!("[Shard {:?}] Websocket error: {:?}", self.shard_info, why);
                info!("[Shard {:?}] Will attempt to auto-reconnect", self.shard_info);

                self.autoreconnect().and(Ok(None))
            },
            Err(error) => Err(error),
        }
    }

    /// Calculates the heartbeat latency between the shard and the gateway.
    ///
    /// # Examples
    ///
    /// When using the [`Client`], output the latency in response to a `"~ping"`
    /// message handled through [`Client::on_message`].
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "~ping" {
    ///             if let Some(latency) = ctx.shard.lock().latency() {
    ///                 let s = format!("{}.{}s", latency.as_secs(), latency.subsec_nanos());
    ///
    ///                 let _ = msg.channel_id.say(&s);
    ///             } else {
    ///                 let _ = msg.channel_id.say("N/A");
    ///             }
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Client`]: ../struct.Client.html
    /// [`EventHandler::on_message`]: ../event_handler/trait.EventHandler.html#method.on_message
    // Shamelessly stolen from brayzure's commit in eris:
    // <https://github.com/abalabahaha/eris/commit/0ce296ae9a542bcec0edf1c999ee2d9986bed5a6>
    pub fn latency(&self) -> Option<StdDuration> {
        if let (Some(received), Some(sent)) = self.heartbeat_instants {
            Some(sent - received)
        } else {
            None
        }
    }

    /// Shuts down the receiver by attempting to cleanly close the
    /// connection.
    pub fn shutdown_clean(&mut self) -> Result<()> {
        {
            let data = CloseData {
                status_code: 1000,
                reason: String::new(),
            };

            let message = OwnedMessage::Close(Some(data));

            self.client.send_message(&message)?;
        }

        let mut stream = self.client.stream_ref().as_tcp();

        stream.flush()?;
        stream.shutdown(Shutdown::Both)?;

        debug!("[Shard {:?}] Cleanly shutdown shard", self.shard_info);

        Ok(())
    }

    /// Uncleanly shuts down the receiver by not sending a close code.
    pub fn shutdown(&mut self) -> Result<()> {
        let mut stream = self.client.stream_ref().as_tcp();

        stream.flush()?;
        stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large
    /// guilds (250 members+). If a guild is over 250 members, then a full
    /// member list will not be downloaded, and must instead be requested to be
    /// sent in "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each
    /// chunk only contains a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be
    /// updated with member chunks.
    ///
    /// # Examples
    ///
    /// Chunk a single guild by Id, limiting to 2000 [`Member`]s, and not
    /// specifying a query parameter:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let mut shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// use serenity::model::GuildId;
    ///
    /// let guild_ids = vec![GuildId(81384788765712384)];
    ///
    /// shard.chunk_guilds(&guild_ids, Some(2000), None);
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, and specifying a
    /// query parameter of `"do"`:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("".to_owned()));
    /// #
    /// # let mut shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// use serenity::model::GuildId;
    ///
    /// let guild_ids = vec![GuildId(81384788765712384)];
    ///
    /// shard.chunk_guilds(&guild_ids, Some(20), Some("do"));
    /// ```
    ///
    /// [`Event::GuildMembersChunk`]:
    /// ../../model/event/enum.Event.html#variant.GuildMembersChunk
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`Member`]: ../../model/struct.Member.html
    pub fn chunk_guilds(&mut self, guild_ids: &[GuildId], limit: Option<u16>, query: Option<&str>) {
        let msg = json!({
            "op": OpCode::GetGuildMembers.num(),
            "d": {
                "guild_id": guild_ids.iter().map(|x| x.0).collect::<Vec<u64>>(),
                "limit": limit.unwrap_or(0),
                "query": query.unwrap_or(""),
            },
        });

        let _ = self.client.send_json(&msg);
    }

    /// Calculates the number of guilds that the shard is responsible for.
    ///
    /// If sharding is not being used (i.e. 1 shard), then the total number of
    /// [`Guild`] in the [`Cache`] will be used.
    ///
    /// **Note**: Requires the `cache` feature be enabled.
    ///
    /// # Examples
    ///
    /// Retrieve the number of guilds a shard is responsible for:
    ///
    /// ```rust,no_run
    /// # use serenity::client::gateway::Shard;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # let mutex = Arc::new(Mutex::new("will anyone read this".to_owned()));
    /// #
    /// # let shard = Shard::new(mutex.clone(), mutex, [0, 1]).unwrap();
    /// #
    /// let info = shard.shard_info();
    /// let guilds = shard.guilds_handled();
    ///
    /// println!("Shard {:?} is responsible for {} guilds", info, guilds);
    /// ```
    ///
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    /// [`Guild`]: ../model/struct.Guild.html
    #[cfg(feature = "cache")]
    pub fn guilds_handled(&self) -> u16 {
        let cache = CACHE.read().unwrap();

        let (shard_id, shard_count) = (self.shard_info[0], self.shard_info[1]);

        cache
            .guilds
            .keys()
            .filter(|guild_id| {
                utils::shard_id(guild_id.0, shard_count) == shard_id
            })
            .count() as u16
    }

    #[cfg(feature = "voice")]
    fn voice_dispatch(&mut self, event: &Event) {
        if let Event::VoiceStateUpdate(ref update) = *event {
            if let Some(guild_id) = update.guild_id {
                if let Some(handler) = self.manager.get(guild_id) {
                    handler.update_state(&update.voice_state);
                }
            }
        }

        if let Event::VoiceServerUpdate(ref update) = *event {
            if let Some(guild_id) = update.guild_id {
                if let Some(handler) = self.manager.get(guild_id) {
                    handler.update_server(&update.endpoint, &update.token);
                }
            }
        }
    }

    #[cfg(feature = "voice")]
    pub(crate) fn cycle_voice_recv(&mut self) {
        if let Ok(v) = self.manager_rx.try_recv() {
            if let Err(why) = self.client.send_json(&v) {
                warn!("[Shard {:?}] Err sending voice msg: {:?}", self.shard_info, why);
            }
        }
    }

    pub(crate) fn heartbeat(&mut self) -> Result<()> {
        let map = json!({
            "d": self.seq,
            "op": OpCode::Heartbeat.num(),
        });

        trace!("[Shard {:?}] Sending heartbeat d: {}", self.shard_info, self.seq);

        match self.client.send_json(&map) {
            Ok(_) => {
                self.heartbeat_instants.0 = Some(Instant::now());
                self.last_heartbeat_acknowledged = false;

                Ok(())
            },
            Err(why) => {
                match why {
                    Error::WebSocket(WebSocketError::IoError(err)) => {
                        if err.raw_os_error() != Some(32) {
                            debug!("[Shard {:?}] Err w/ heartbeating: {:?}", self.shard_info, err);
                        }
                    },
                    other => {
                        warn!("[Shard {:?}] Other err w/ keepalive: {:?}", self.shard_info, other);
                    },
                }

                Err(Error::Gateway(GatewayError::HeartbeatFailed))
            },
        }
    }

    pub(crate) fn check_heartbeat(&mut self) -> Result<()> {
        let heartbeat_interval = match self.heartbeat_interval {
            Some(heartbeat_interval) => heartbeat_interval,
            None => return Ok(()),
        };

        let wait = StdDuration::from_secs(heartbeat_interval / 1000);

        // If a duration of time less than the heartbeat_interval has passed,
        // then don't perform a keepalive or attempt to reconnect.
        if let Some(last_sent) = self.heartbeat_instants.0 {
            if last_sent.elapsed() <= wait {
                return Ok(());
            }
        }

        // If the last heartbeat didn't receive an acknowledgement, then
        // auto-reconnect.
        if !self.last_heartbeat_acknowledged {
            debug!("[Shard {:?}] Last heartbeat not acknowledged; re-connecting", self.shard_info);

            return self.reconnect().map_err(|why| {
                warn!("[Shard {:?}] Err auto-reconnecting from heartbeat check: {:?}",
                self.shard_info,
                why);

                why
            });
        }

        // Otherwise, we're good to heartbeat.
        if let Err(why) = self.heartbeat() {
            warn!("[Shard {:?}] Err heartbeating: {:?}", self.shard_info, why);

            self.reconnect()
        } else {
            self.heartbeat_instants.0 = Some(Instant::now());

            Ok(())
        }
    }

    pub(crate) fn autoreconnect(&mut self) -> Result<()> {
        if self.stage == ConnectionStage::Connecting {
            return Ok(());
        }

        if self.session_id.is_some() {
            debug!("[Shard {:?}] Autoreconnector choosing to resume", self.shard_info);

            self.resume()
        } else {
            debug!("[Shard {:?}] Autoreconnector choosing to reconnect", self.shard_info);

            self.reconnect()
        }
    }

    /// Retrieves the `heartbeat_interval`.
    #[inline]
    pub(crate) fn heartbeat_interval(&self) -> Option<u64> { self.heartbeat_interval }

    /// Retrieves the value of when the last heartbeat ack was received.
    #[inline]
    pub(crate) fn last_heartbeat_ack(&self) -> Option<Instant> { self.heartbeat_instants.1 }

    fn reconnect(&mut self) -> Result<()> {
        info!("[Shard {:?}] Attempting to reconnect", self.shard_info);
        self.reset();

        self.initialize()
    }

    // Attempts to send a RESUME message.
    //
    // # Examples
    //
    // Returns a `GatewayError::NoSessionId` is there is no `session_id`,
    // indicating that the shard should instead [`reconnect`].
    //
    // [`reconnect`]: #method.reconnect
    fn resume(&mut self) -> Result<()> {
        self.send_resume().or_else(|why| {
            warn!("Err sending resume: {:?}", why);

            self.reconnect()
        })
    }

    fn send_resume(&mut self) -> Result<()> {
        let session_id = match self.session_id.clone() {
            Some(session_id) => session_id,
            None => return Err(Error::Gateway(GatewayError::NoSessionId)),
        };

        self.client.send_json(&json!({
            "op": OpCode::Resume.num(),
            "d": {
                "session_id": session_id,
                "seq": self.seq,
                "token": &*self.token.lock().unwrap(),
            },
        }))
    }

    fn initialize(&mut self) -> Result<()> {
        self.stage = ConnectionStage::Connecting;
        self.client = connect(&self.ws_url.lock().unwrap())?;

        self.identify()
    }

    fn identify(&mut self) -> Result<()> {
        let identification = json!({
            "op": OpCode::Identify.num(),
            "d": {
                "compression": true,
                "large_threshold": constants::LARGE_THRESHOLD,
                "shard": self.shard_info,
                "token": &*self.token.lock().unwrap(),
                "v": constants::GATEWAY_VERSION,
                "properties": {
                    "$browser": "serenity",
                    "$device": "serenity",
                    "$os": consts::OS,
                },
            },
        });

        self.client.send_json(&identification)
    }

    fn reset(&mut self) {
        self.heartbeat_instants = (Some(Instant::now()), None);
        self.heartbeat_interval = None;
        self.last_heartbeat_acknowledged = true;
        self.stage = ConnectionStage::Disconnected;
        self.seq = 0;
    }

    fn update_presence(&mut self) {
        let (ref game, status, afk) = self.current_presence;
        let now = Utc::now().timestamp() as u64;

        let msg = json!({
            "op": OpCode::StatusUpdate.num(),
            "d": {
                "afk": afk,
                "since": now,
                "status": status.name(),
                "game": game.as_ref().map(|x| json!({
                    "name": x.name,
                    "type": x.kind,
                    "url": x.url,
                })),
            },
        });

        if let Err(why) = self.client.send_json(&msg) {
            warn!("[Shard {:?}] Err sending presence update: {:?}", self.shard_info, why);
        }

        #[cfg(feature = "cache")]
        {
            let mut cache = CACHE.write().unwrap();
            let current_user_id = cache.user.id;

            cache.presences.get_mut(&current_user_id).map(|presence| {
                presence.game = game.clone();
                presence.last_modified = Some(now);
            });
        }
    }
}

fn connect(base_url: &str) -> Result<WsClient> {
    let url = build_gateway_url(base_url)?;
    let client = ClientBuilder::from_url(&url).connect_secure(None)?;

    Ok(client)
}

fn set_client_timeout(client: &mut WsClient) -> Result<()> {
    let stream = client.stream_ref().as_tcp();
    stream.set_read_timeout(Some(StdDuration::from_millis(100)))?;
    stream.set_write_timeout(Some(StdDuration::from_secs(5)))?;

    Ok(())
}

fn build_gateway_url(base: &str) -> Result<Url> {
    Url::parse(&format!("{}?v={}", base, constants::GATEWAY_VERSION))
        .map_err(|_| Error::Gateway(GatewayError::BuildingUrl))
}
