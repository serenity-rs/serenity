use serde_json::builder::ObjectBuilder;
use std::io::Write;
use std::net::Shutdown;
use std::sync::mpsc::{self, Sender as MpscSender};
use std::thread::{self, Builder as ThreadBuilder};
use std::time::Duration as StdDuration;
use std::mem;
use super::super::login_type::LoginType;
use super::super::rest;
use super::{GatewayError, GatewayStatus, prep};
use time;
use websocket::client::{Client as WsClient, Sender, Receiver};
use websocket::message::Message as WsMessage;
use websocket::result::WebSocketError;
use websocket::stream::WebSocketStream;
use websocket::ws::sender::Sender as WsSender;
use ::constants::OpCode;
use ::internal::prelude::*;
use ::internal::ws_impl::{ReceiverExt, SenderExt};
use ::model::event::{Event, GatewayEvent, ReadyEvent};
use ::model::{ChannelId, Game, GuildId, OnlineStatus};

#[cfg(feature="cache")]
use ::client::CACHE;
#[cfg(feature="voice")]
use ::ext::voice::Manager as VoiceManager;

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
/// [`Client`]: struct.Client.html
/// [`new`]: #method.new
/// [`receive`]: #method.receive
/// [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding
/// [module docs]: index.html#sharding
pub struct Shard {
    current_presence: CurrentPresence,
    keepalive_channel: MpscSender<GatewayStatus>,
    seq: u64,
    login_type: LoginType,
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
    /// use serenity::client::gateway::Shard;
    /// use serenity::client::{LoginType, rest};
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_BOT_TOKEN").expect("Token in environment");
    /// // retrieve the gateway response, which contains the URL to connect to
    /// let gateway = rest::get_gateway().expect("Valid gateway response").url;
    /// let shard = Shard::new(&gateway, &token, None, LoginType::Bot)
    ///     .expect("Working shard");
    ///
    /// // at this point, you can create a `loop`, and receive events and match
    /// // their variants
    /// ```
    pub fn new(base_url: &str,
               token: &str,
               shard_info: Option<[u64; 2]>,
               login_type: LoginType)
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
        ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || prep::keepalive(heartbeat_interval, sender, rx))?;

        // Parse READY
        let event = receiver.recv_json(GatewayEvent::decode)?;
        let (ready, sequence) = prep::parse_ready(event,
                                                  &tx,
                                                  &mut receiver,
                                                  identification)?;

        Ok((feature_voice! {{
            Shard {
                current_presence: (None, OnlineStatus::Online, false),
                keepalive_channel: tx.clone(),
                seq: sequence,
                login_type: login_type,
                token: token.to_owned(),
                session_id: Some(ready.ready.session_id.clone()),
                shard_info: shard_info,
                ws_url: base_url.to_owned(),
                manager: VoiceManager::new(tx, ready.ready.user.id),
            }
        } else {
            Shard {
                current_presence: (None, OnlineStatus::Online, false),
                keepalive_channel: tx.clone(),
                seq: sequence,
                login_type: login_type,
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
    pub fn set_game(&mut self, game: Option<Game>) {
        self.current_presence.0 = game;

        self.update_presence();
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid presence, so it is automatically
    /// converted to [`Invisible`].
    ///
    /// Other presence settings are maintained.
    ///
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
    /// ```rust,ignore
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// // assuming you are in a context
    ///
    /// context.shard.lock()
    ///     .unwrap()
    ///     .set_presence(Some(Game::playing("Heroes of the Storm")),
    ///                   OnlineStatus::Online,
    ///                   false);
    /// ```
    pub fn set_presence(&mut self,
                        game: Option<Game>,
                        status: OnlineStatus,
                        afk: bool) {
        let status = match status {
            OnlineStatus::Offline => OnlineStatus::Invisible,
            other => other,
        };

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

                    return if let Some(session_id) = self.session_id.clone() {
                        self.resume(session_id, receiver)
                            .map(|(ev, rec)| Some((ev, Some(rec))))
                    } else {
                        self.reconnect(receiver)
                            .map(|(ev, rec)| Some((ev, Some(rec))))
                    };
                }

                let map = ObjectBuilder::new()
                    .insert("d", Value::Null)
                    .insert("op", OpCode::Heartbeat.num())
                    .build();
                let status = GatewayStatus::SendMessage(map);
                let _ = self.keepalive_channel.send(status);

                Ok(None)
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                Ok(None)
            },
            Ok(GatewayEvent::Hello(interval)) => {
                if interval > 0 {
                    let status = GatewayStatus::Interval(interval);
                    let _ = self.keepalive_channel.send(status);
                }

                if let Some(session_id) = self.session_id.clone() {
                    self.resume(session_id, receiver)
                        .map(|(ev, rec)| Some((ev, Some(rec))))
                } else {
                    self.reconnect(receiver)
                        .map(|(ev, rec)| Some((ev, Some(rec))))
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
                    Some(4006) | Some(4009) => {
                        info!("Invalid session");

                        self.session_id = None;
                    },
                    Some(other) if !clean => {
                        warn!("Unknown unclean close {}: {:?}", other, message);
                    },
                    _ => {},
                }

                if !clean && num != Some(1000) && num != Some(4004) {
                    if let Some(session_id) = self.session_id.clone() {
                        match self.resume(session_id, receiver) {
                            Ok((ev, rec)) => return Ok(Some((ev, Some(rec)))),
                            Err(why) => debug!("Error resuming: {:?}", why),
                        }
                    }
                }

                self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
            },
            Err(Error::WebSocket(WebSocketError::NoDataAvailable)) => Ok(None),
            Err(Error::WebSocket(why)) => {
                warn!("Websocket error: {:?}", why);
                info!("Will attempt to reconnect or resume");

                // Attempt to resume if the following was not received:
                //
                // - InvalidateSession.
                //
                // Otherwise, fallback to reconnecting.
                if let Some(session_id) = self.session_id.clone() {
                    match self.resume(session_id, &mut receiver) {
                        Ok((ev, rec)) => return Ok(Some((ev, Some(rec)))),
                        Err(why) => info!("Error resuming: {:?}", why),
                    }
                }

                info!("Reconnecting");

                self.reconnect(receiver).map(|(ev, rec)| Some((ev, Some(rec))))
            },
            Err(error) => Err(error),
        }
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

        Ok(())
    }

    pub fn shutdown(receiver: &mut Receiver<WebSocketStream>) -> Result<()> {
        let r = receiver.get_mut().get_mut();

        r.flush()?;
        r.shutdown(Shutdown::Both)?;

        Ok(())
    }

    /// Syncs a number of [`Call`]s, given by their associated channel Ids. This
    /// will allow the current user to know what calls are currently occurring,
    /// as otherwise events will not be received.
    pub fn sync_calls(&self, channels: &[ChannelId]) {
        for &channel in channels {
            let msg = ObjectBuilder::new()
                .insert("op", OpCode::SyncCall.num())
                .insert_object("d", |obj| obj
                    .insert("channel_id", channel.0)
                )
                .build();

            let _ = self.keepalive_channel.send(GatewayStatus::SendMessage(msg));
        }
    }

    /// Requests that one or multiple [`Guild`]s be synced.
    ///
    /// This will ask Discord to start sending member chunks for large guilds
    /// (250 members+). If a guild is over 250 members, then a full member list
    /// will not be downloaded, and must instead be requested to be sent in
    /// "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each
    /// chunk only contains a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be
    /// updated with member chunks.
    pub fn sync_guilds(&self, guild_ids: &[GuildId]) {
        let msg = ObjectBuilder::new()
            .insert("op", OpCode::SyncGuild.num())
            .insert_array("d", |a| guild_ids.iter().fold(a, |a, s| a.push(s.0)))
            .build();

        let _ = self.keepalive_channel.send(GatewayStatus::SendMessage(msg));
    }

    fn handle_dispatch(&mut self, event: &Event) {
        if let Event::Resumed(ref ev) = *event {
            let status = GatewayStatus::Interval(ev.heartbeat_interval);

            let _ = self.keepalive_channel.send(status);
        }

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
            let gateway_url = rest::get_gateway()?.url;

            let shard = Shard::new(&gateway_url,
                                   &self.token,
                                   self.shard_info,
                                   self.login_type);

            if let Ok((shard, ready, receiver_new)) = shard {
                let _ = Shard::shutdown(&mut receiver);

                mem::replace(self, shard);
                self.session_id = Some(ready.ready.session_id.clone());

                return Ok((Event::Ready(ready), receiver_new));
            }

            // Exponentially back off.
            thread::sleep(StdDuration::from_secs(i.pow(2)));
        }

        // Reconnecting failed; just return an error instead.
        Err(Error::Gateway(GatewayError::ReconnectFailure))
    }

    fn resume(&mut self, session_id: String, receiver: &mut Receiver<WebSocketStream>)
        -> Result<(Event, Receiver<WebSocketStream>)> {
        receiver.get_mut().get_mut().shutdown(Shutdown::Both)?;
        let url = prep::build_gateway_url(&self.ws_url)?;

        let response = WsClient::connect(url)?.send()?;
        response.validate()?;

        let (mut sender, mut receiver) = response.begin().split();

        sender.send_json(&ObjectBuilder::new()
            .insert_object("d", |o| o
                .insert("session_id", session_id)
                .insert("seq", self.seq)
                .insert("token", &self.token))
            .insert("op", OpCode::Resume.num())
            .build())?;

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
                        Event::Resumed { .. } => info!("Resumed"),
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

        let msg = ObjectBuilder::new()
            .insert("op", OpCode::StatusUpdate.num())
            .insert_object("d", move |mut object| {
                object = object.insert("afk", afk)
                    .insert("since", now)
                    .insert("status", status.name());

                match game.as_ref() {
                    Some(game) => {
                        object.insert_object("game", move |o| o
                            .insert("name", &game.name))
                    },
                    None => object.insert("game", Value::Null),
                }
            })
            .build();

        let _ = self.keepalive_channel.send(GatewayStatus::SendMessage(msg));

        #[cfg(feature="cache")]
        {
            let mut cache = CACHE.write().unwrap();
            let current_user_id = cache.user.id;

            for (user_id, presence) in &mut cache.presences {
                if *user_id != current_user_id {
                    continue;
                }

                presence.game = game.clone();
                presence.last_modified = Some(now);
            }
        }
    }
}
