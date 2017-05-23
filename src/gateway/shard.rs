use std::io::Write;
use std::net::Shutdown;
use std::sync::mpsc::{self, Sender as MpscSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, Builder as ThreadBuilder};
use std::time::{Duration as StdDuration, Instant};
use std::mem;
use super::{GatewayError, GatewayStatus, prep};
use time;
use websocket::client::{Client as WsClient, Sender, Receiver};
use websocket::message::Message as WsMessage;
use websocket::result::WebSocketError;
use websocket::stream::WebSocketStream;
use websocket::ws::sender::Sender as WsSender;
use ::constants::OpCode;
use ::http;
use ::internal::prelude::*;
use ::internal::ws_impl::{ReceiverExt, SenderExt};
use ::model::event::{Event, GatewayEvent, ReadyEvent};
use ::model::{Game, GuildId, OnlineStatus};

#[cfg(feature="cache")]
use ::client::CACHE;
#[cfg(feature="voice")]
use ::ext::voice::Manager as VoiceManager;
#[cfg(feature="cache")]
use ::utils;

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
#[derive(Clone, Debug)]
pub struct Shard {
    current_presence: CurrentPresence,
    /// A tuple of the last instant that a heartbeat was sent, and the last that
    /// an acknowledgement was received.
    ///
    /// This can be used to calculate [`latency`].
    ///
    /// [`latency`]: fn.latency.html
    heartbeat_instants: (Arc<Mutex<Instant>>, Option<Instant>),
    keepalive_channel: MpscSender<GatewayStatus>,
    seq: u64,
    session_id: Option<String>,
    shard_info: Option<[u64; 2]>,
    token: String,
    ws_url: String,
    /// The voice connections that this Shard is responsible for. The Shard will
    /// update the voice connections' states.
    #[cfg(feature="voice")]
    pub manager: VoiceManager,
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
    pub fn new(base_url: &str,
               token: &str,
               shard_info: Option<[u64; 2]>)
               -> Result<(Shard, ReadyEvent, Receiver<WebSocketStream>)> {
        let url = prep::build_gateway_url(base_url)?;

        let response = WsClient::connect(url)?.send()?;
        response.validate()?;

        let (mut sender, mut receiver) = response.begin().split();

        let identification = prep::identify(token, shard_info);
        sender.send_json(&identification)?;

        let heartbeat_interval = match receiver.recv_json(GatewayEvent::decode)? {
            GatewayEvent::Hello(interval) => interval,
            other => {
                debug!("Unexpected event during shard start: {:?}", other);

                return Err(Error::Gateway(GatewayError::ExpectedHello));
            },
        };

        let (tx, rx) = mpsc::channel();
        let thread_name = match shard_info {
            Some(info) => format!("serenity keepalive [shard {}/{}]",
                                  info[0],
                                  info[1] - 1),
            None => "serenity keepalive [unsharded]".to_owned(),
        };

        let heartbeat_sent = Arc::new(Mutex::new(Instant::now()));
        let heartbeat_clone = heartbeat_sent.clone();

        ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || {
                prep::keepalive(heartbeat_interval, heartbeat_clone, sender, &rx)
            })?;

        // Parse READY
        let event = receiver.recv_json(GatewayEvent::decode)?;
        let (ready, sequence) = prep::parse_ready(event,
                                                  &tx,
                                                  &mut receiver,
                                                  identification)?;

        Ok((feature_voice! {{
            Shard {
                current_presence: (None, OnlineStatus::Online, false),
                heartbeat_instants: (heartbeat_sent, None),
                keepalive_channel: tx.clone(),
                seq: sequence,
                token: token.to_owned(),
                session_id: Some(ready.ready.session_id.clone()),
                shard_info: shard_info,
                ws_url: base_url.to_owned(),
                manager: VoiceManager::new(tx, ready.ready.user.id),
            }
        } else {
            Shard {
                current_presence: (None, OnlineStatus::Online, false),
                heartbeat_instants: (heartbeat_sent, None),
                keepalive_channel: tx.clone(),
                seq: sequence,
                token: token.to_owned(),
                session_id: Some(ready.ready.session_id.clone()),
                shard_info: shard_info,
                ws_url: base_url.to_owned(),
            }
        }}, ready, receiver))
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
    /// #
    /// # let (shard, _, _) = Shard::new("", "", Some([1, 2])).unwrap();
    /// #
    /// assert_eq!(shard.shard_info(), Some([1, 2]));
    /// ```
    pub fn shard_info(&self) -> Option<[u64; 2]> {
        self.shard_info
    }

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
    /// #
    /// # let (mut shard, _, _) = Shard::new("", "", Some([0, 1])).unwrap();
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
    /// #
    /// # let (mut shard, _, _) = Shard::new("", "", Some([0, 1])).unwrap();
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
    /// #
    /// # let (mut shard, _, _) = Shard::new("", "", Some([0, 1])).unwrap();
    /// #
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// shard.set_presence(Some(Game::playing("Heroes of the Storm")), OnlineStatus::Online, false);
    /// ```
    pub fn set_presence(&mut self,
                        game: Option<Game>,
                        mut status: OnlineStatus,
                        afk: bool) {
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
    #[allow(cyclomatic_complexity)]
    #[doc(hidden)]
    pub fn handle_event(&mut self,
                        event: Result<GatewayEvent>,
                        mut receiver: &mut Receiver<WebSocketStream>)
                        -> Result<Option<(Event, Option<Receiver<WebSocketStream>>)>> {
        match event {
            Ok(GatewayEvent::Dispatch(seq, event)) => {
                let status = GatewayStatus::Sequence(seq);
                let _ = self.keepalive_channel.send(status);

                self.seq = seq;

                self.handle_dispatch(&event);

                Ok(Some((event, None)))
            },
            Ok(GatewayEvent::Heartbeat(s)) => {
                info!("Received shard heartbeat");

                // Received seq is off -- attempt to resume.
                if s > self.seq + 1 {
                    info!("Received off sequence (them: {}; us: {}); resuming",
                          s,
                          self.seq);

                    return if self.session_id.is_some() {
                        self.resume(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
                    } else {
                        self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
                    };
                }

                let map = json!({
                    "d": Value::Null,
                    "op": OpCode::Heartbeat.num(),
                });
                let status = GatewayStatus::SendMessage(map);
                let _ = self.keepalive_channel.send(status);

                Ok(None)
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.heartbeat_instants.1 = Some(Instant::now());

                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                if interval > 0 {
                    let status = GatewayStatus::Interval(interval);
                    let _ = self.keepalive_channel.send(status);
                }

                if self.session_id.is_some() {
                    self.resume(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
                } else {
                    self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
                }
            },
            Ok(GatewayEvent::InvalidateSession) => {
                info!("Received session invalidation; re-identifying");
                self.seq = 0;
                self.session_id = None;

                let identification = prep::identify(&self.token, self.shard_info);
                let status = GatewayStatus::SendMessage(identification);
                let _ = self.keepalive_channel.send(status);

                Ok(None)
            },
            Ok(GatewayEvent::Reconnect) => {
                self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
            },
            Err(Error::Gateway(GatewayError::Closed(num, message))) => {
                let clean = num == Some(1000);

                {
                    let kind = if clean { "Cleanly" } else { "Uncleanly" };

                    info!("{} closing with {:?}: {}", kind, num, message);
                }

                match num {
                    Some(4001) => warn!("Sent invalid opcode"),
                    Some(4002) => warn!("Sent invalid message"),
                    Some(4003) => warn!("Sent no authentication"),
                    Some(4004) => warn!("Sent invalid authentication"),
                    Some(4005) => warn!("Already authenticated"),
                    Some(4007) => {
                        warn!("Sent invalid seq: {}", self.seq);

                        self.seq = 0;
                    },
                    Some(4008) => warn!("Gateway ratelimited"),
                    Some(4010) => warn!("Sent invalid shard"),
                    Some(4011) => error!("Bot requires more shards"),
                    Some(4006) | Some(4009) => {
                        info!("Invalid session");

                        self.session_id = None;
                    },
                    Some(other) if !clean => {
                        warn!("Unknown unclean close {}: {:?}", other, message);
                    },
                    _ => {},
                }

                let resume = num.map(|x| x != 1000 && x != 4004 && self.session_id.is_some())
                    .unwrap_or(false);

                if resume {
                    info!("Attempting to resume");

                    if self.session_id.is_some() {
                        match self.resume(receiver) {
                            Ok((ev, rec)) => {
                                info!("Resumed");

                                return Ok(Some((ev, Some(rec))));
                            },
                            Err(why) => {
                                warn!("Error resuming: {:?}", why);
                                info!("Falling back to reconnecting");
                            },
                        }
                    }
                }

                info!("Reconnecting");

                self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
            },
            Err(Error::WebSocket(why)) => {
                if let WebSocketError::NoDataAvailable = why {
                    if self.heartbeat_instants.1.is_none() {
                        return Ok(None);
                    }
                }

                warn!("Websocket error: {:?}", why);
                info!("Will attempt to reconnect or resume");

                // Attempt to resume if the following was not received:
                //
                // - InvalidateSession.
                //
                // Otherwise, fallback to reconnecting.
                if self.session_id.is_some() {
                    info!("Attempting to resume");

                    match self.resume(&mut receiver) {
                        Ok((ev, rec)) => {
                            info!("Resumed");

                            return Ok(Some((ev, Some(rec))));
                        },
                        Err(why) => {
                            warn!("Error resuming: {:?}", why);
                            info!("Falling back to reconnecting");
                        },
                    }
                }

                info!("Reconnecting");

                self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
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
    /// # use serenity::client::Client;
    /// #
    /// # let mut client = Client::login("hello source code viewer <3");
    /// client.on_message(|ctx, msg| {
    ///     if msg.content == "~ping" {
    ///         if let Some(latency) = ctx.shard.lock().unwrap().latency() {
    ///             let s = format!("{}.{}s", latency.as_secs(), latency.subsec_nanos());
    ///
    ///             let _ = msg.channel_id.say(&s);
    ///         } else {
    ///             let _ = msg.channel_id.say("N/A");
    ///         }
    ///     }
    /// });
    /// ```
    ///
    /// [`Client`]: ../struct.Client.html
    /// [`Client::on_message`]: ../struct.Client.html#method.on_message
    // Shamelessly stolen from brayzure's commit in eris:
    // <https://github.com/abalabahaha/eris/commit/0ce296ae9a542bcec0edf1c999ee2d9986bed5a6>
    pub fn latency(&self) -> Option<StdDuration> {
        self.heartbeat_instants.1.map(|send| send - *self.heartbeat_instants.0.lock().unwrap())
    }

    /// Shuts down the receiver by attempting to cleanly close the
    /// connection.
    #[doc(hidden)]
    pub fn shutdown_clean(receiver: &mut Receiver<WebSocketStream>)
        -> Result<()> {
        let r = receiver.get_mut().get_mut();

        {
            let mut sender = Sender::new(r.by_ref(), true);
            let message = WsMessage::close_because(1000, "");

            sender.send_message(&message)?;
        }

        r.flush()?;
        r.shutdown(Shutdown::Both)?;

        debug!("Cleanly shutdown shard");

        Ok(())
    }

    /// Uncleanly shuts down the receiver by not sending a close code.
    #[doc(hidden)]
    pub fn shutdown(receiver: &mut Receiver<WebSocketStream>) -> Result<()> {
        let r = receiver.get_mut().get_mut();

        r.flush()?;
        r.shutdown(Shutdown::Both)?;

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
    /// #
    /// # let (shard, _, _) = Shard::new("", "", Some([0, 1])).unwrap();
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
    /// #
    /// # let (shard, _, _) = Shard::new("", "", Some([0, 1])).unwrap();
    /// #
    /// use serenity::model::GuildId;
    ///
    /// let guild_ids = vec![GuildId(81384788765712384)];
    ///
    /// shard.chunk_guilds(&guild_ids, Some(20), Some("do"));
    /// ```
    ///
    /// [`Event::GuildMembersChunk`]: ../../model/event/enum.Event.html#variant.GuildMembersChunk
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [`Member`]: ../../model/struct.Member.html
    pub fn chunk_guilds(&self, guild_ids: &[GuildId], limit: Option<u16>, query: Option<&str>) {
        let msg = json!({
            "op": OpCode::GetGuildMembers.num(),
            "d": {
                "guild_id": guild_ids.iter().map(|x| x.0).collect::<Vec<u64>>(),
                "limit": limit.unwrap_or(0),
                "query": query.unwrap_or(""),
            },
        });

        let _ = self.keepalive_channel.send(GatewayStatus::SendMessage(msg));
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
    /// #
    /// # let (shard, _, _) = Shard::new("will anyone read this", "", Some([0, 1])).unwrap();
    /// #
    /// let info = shard.shard_info();
    /// let guilds = shard.guilds_handled();
    ///
    /// println!("Shard {:?} is responsible for {} guilds", info, guilds);
    /// ```
    ///
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    /// [`Guild`]: ../model/struct.Guild.html
    #[cfg(feature="cache")]
    pub fn guilds_handled(&self) -> u16 {
        let cache = CACHE.read().unwrap();

        if let Some((shard_id, shard_count)) = self.shard_info.map(|s| (s[0], s[1])) {
            cache.guilds
                .keys()
                .filter(|guild_id| utils::shard_id(guild_id.0, shard_count) == shard_id)
                .count() as u16
        } else {
            cache.guilds.len() as u16
        }
    }

    #[allow(unused_variables)]
    fn handle_dispatch(&mut self, event: &Event) {
        #[cfg(feature="voice")]
        {
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
    }

    fn reconnect(&mut self, mut receiver: &mut Receiver<WebSocketStream>)
        -> Result<(Event, Receiver<WebSocketStream>)> {
        info!("Attempting to reconnect");

        // Take a few attempts at reconnecting.
        for i in 1u64..11u64 {
            let gateway_url = http::get_gateway()?.url;

            let shard = Shard::new(&gateway_url,
                                   &self.token,
                                   self.shard_info);

            if let Ok((shard, ready, receiver_new)) = shard {
                let _ = Shard::shutdown(&mut receiver);

                mem::replace(self, shard);
                self.session_id = Some(ready.ready.session_id.clone());

                return Ok((Event::Ready(ready), receiver_new));
            }

            let seconds = i.pow(2);

            debug!("Exponentially backing off for {} seconds", seconds);

            // Exponentially back off.
            thread::sleep(StdDuration::from_secs(seconds));
        }

        // Reconnecting failed; just return an error instead.
        Err(Error::Gateway(GatewayError::ReconnectFailure))
    }

    #[doc(hidden)]
    pub fn resume(&mut self, receiver: &mut Receiver<WebSocketStream>)
        -> Result<(Event, Receiver<WebSocketStream>)> {
        let session_id = match self.session_id.clone() {
            Some(session_id) => session_id,
            None => return Err(Error::Gateway(GatewayError::NoSessionId)),
        };

        let _ = receiver.shutdown_all();
        let url = prep::build_gateway_url(&self.ws_url)?;

        let response = WsClient::connect(url)?.send()?;
        response.validate()?;

        let (mut sender, mut receiver) = response.begin().split();

        sender.send_json(&json!({
            "op": OpCode::Resume.num(),
            "d": {
                "session_id": session_id,
                "seq": self.seq,
                "token": self.token,
            },
        }))?;

        // Note to self when this gets accepted in a decade:
        // https://github.com/rust-lang/rfcs/issues/961
        let ev;

        loop {
            match receiver.recv_json(GatewayEvent::decode)? {
                GatewayEvent::Dispatch(seq, event) => {
                    match event {
                        Event::Ready(ref ready) => {
                            self.session_id = Some(ready.ready.session_id.clone());
                        },
                        Event::Resumed(_) => info!("Resumed"),
                        ref other => warn!("Unknown resume event: {:?}", other),
                    }

                    self.seq = seq;
                    ev = event;

                    break;
                },
                GatewayEvent::Hello(i) => {
                    let _ = self.keepalive_channel.send(GatewayStatus::Interval(i));
                }
                GatewayEvent::InvalidateSession => {
                    sender.send_json(&prep::identify(&self.token, self.shard_info))?;
                },
                other => {
                    debug!("Unexpected event: {:?}", other);

                    return Err(Error::Gateway(GatewayError::InvalidHandshake));
                },
            }
        }

        let _ = self.keepalive_channel.send(GatewayStatus::Sender(sender));

        Ok((ev, receiver))
    }

    fn update_presence(&self) {
        let (ref game, status, afk) = self.current_presence;
        let now = time::get_time().sec as u64;

        let msg = json!({
            "op": OpCode::StatusUpdate.num(),
            "d": {
                "afk": afk,
                "since": now,
                "status": status.name(),
                "game": game.as_ref().map(|x| json!({
                    "name": x.name,
                })),
            },
        });

        let _ = self.keepalive_channel.send(GatewayStatus::SendMessage(msg));

        #[cfg(feature="cache")]
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
