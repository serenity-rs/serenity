use chrono::Utc;
use constants::{GATEWAY_VERSION, LARGE_THRESHOLD, OpCode};
use tokio_core::reactor::Handle;
use futures::future::Future;
use futures::stream::Stream as FuturesStream;
use futures::sync::mpsc::{self, UnboundedSender};
use futures::Sink;
use model::event::{Event, GatewayEvent};
use model::gateway::Game;
use model::id::GuildId;
use model::user::OnlineStatus;
use serde_json::{self, Error as JsonError, Value};
use std::cell::RefCell;
use std::env::consts;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::rc::Rc;
use std::time::{Duration, Instant};
use super::{ConnectionStage, CurrentPresence, ShardStream};
use tungstenite::{Error as TungsteniteError, Message as TungsteniteMessage};
use tokio_timer::Timer;
use tokio_tungstenite::connect_async;
use url::Url;
use std::str::FromStr;
use ::Error;

const CONNECTION: &'static str = "wss://gateway.discord.gg/?v=6&encoding=json";

#[derive(Copy, Clone, Debug)]
struct HeartbeatInfo {
    pub heartbeat_instants: (Option<Instant>, Option<Instant>),
    pub heartbeater: bool,
    pub last_heartbeat_acknowledged: bool,
    pub seq: u64,
    pub shard_info: [u64; 2],
}

pub struct Shard {
    current_presence: CurrentPresence,
    handle: Handle,
    heartbeat_info: Rc<RefCell<HeartbeatInfo>>,
    interval: Option<u64>,
    session_id: Option<String>,
    shard_info: [u64; 2],
    stage: ConnectionStage,
    stream: Option<ShardStream>,
    token: Rc<String>,
    tx: UnboundedSender<TungsteniteMessage>,
}

impl Shard {
    pub fn new(token: Rc<String>, shard_info: [u64; 2], handle: Handle)
        -> Box<Future<Item = Shard, Error = Error>> {
        let done = connect_async(Url::from_str(CONNECTION).unwrap(), handle.remote().clone())
            .map(move |(duplex, _)| {
                let (sink, stream) = duplex.split();
                let (tx, rx) = mpsc::unbounded();

                let done = rx
                    .map_err(|why| {
                        error!("Err select sink rx: {:?}", why);

                        TungsteniteError::Io(IoError::new(
                            IoErrorKind::Other,
                            "Err selecting sink rx",
                        ))
                    })
                    .forward(sink)
                    .map(|_| ())
                    .map_err(|_| ());

                handle.spawn(done);

                Self {
                    current_presence: (None, OnlineStatus::Online),
                    handle: handle.clone(),
                    heartbeat_info: Rc::new(RefCell::new(HeartbeatInfo {
                        heartbeat_instants: (None, None),
                        heartbeater: false,
                        last_heartbeat_acknowledged: true,
                        seq: 0,
                        shard_info,
                    })),
                    interval: None,
                    session_id: None,
                    stage: ConnectionStage::Handshake,
                    stream: Some(stream),
                    shard_info,
                    token,
                    tx,
                }
            })
            .map_err(From::from);

        Box::new(done)
    }

    pub fn parse(&self, msg: TungsteniteMessage) -> Result<GatewayEvent, JsonError> {
        match msg {
            TungsteniteMessage::Binary(v) => serde_json::from_slice(&v),
            TungsteniteMessage::Text(v) => serde_json::from_str(&v),
            _ => unreachable!("parse other"),
        }
    }


    /// Processes the given event to determine if something needs to be done.
    ///
    /// For example, an event may cause the shard to need to reconnect due to a
    /// session invalidating.
    pub fn process(&mut self, event: &GatewayEvent) {
        match *event {
            GatewayEvent::Dispatch(seq, ref event) => {
                let mut info = self.heartbeat_info.borrow_mut();
                let self_seq = info.seq;

                if seq > self_seq + 1 {
                    warn!(
                        "[Shard {:?}] Heartbeat off; them: {}, us: {}",
                        self.shard_info,
                        seq,
                        self_seq,
                    );
                }

                match *event {
                    Event::Ready(ref ready) => {
                        debug!("[Shard {:?}] Received Ready", self.shard_info);

                        self.session_id = Some(ready.ready.session_id.clone());
                        self.stage = ConnectionStage::Connected;
                    },
                    Event::Resumed(_) => {
                        info!("[Shard {:?}] Resumed", self.shard_info);

                        self.stage = ConnectionStage::Connected;
                        info.last_heartbeat_acknowledged = true;
                        info.heartbeat_instants = (Some(Instant::now()), None);
                    },
                    _ => {},
                }

                info.seq = seq;
            },
            GatewayEvent::Heartbeat(s) => {
                info!("[Shard {:?}] Received shard heartbeat", self.shard_info);

                if s > self.heartbeat_info.borrow().seq + 1 {
                    info!(
                        "[Shard {:?}] Received off sequence (them: {}; us: {}); resuming",
                        self.shard_info,
                        s,
                        self.heartbeat_info.borrow().seq
                    );
                }

                self.heartbeat().unwrap();
            },
            GatewayEvent::HeartbeatAck => {
                trace!("[Shard {:?}] Received heartbeat ack", self.shard_info);

                let mut info = self.heartbeat_info.borrow_mut();
                info.heartbeat_instants.1 = Some(Instant::now());
                info.last_heartbeat_acknowledged = true;
            },
            GatewayEvent::Hello(interval) => {
                debug!(
                    "[Shard {:?}] Received a Hello; interval: {}",
                    self.shard_info,
                    interval,
                );

                if self.stage == ConnectionStage::Resuming {
                    return;
                }

                if interval > 0 {
                    self.interval = Some(interval);
                }

                if self.stage == ConnectionStage::Handshake {
                    let heartbeat_info = Rc::clone(&self.heartbeat_info);
                    let mut tx = self.tx.clone();

                    let done = Timer::default()
                        .interval(Duration::from_millis(interval))
                        .for_each(move |_| {
                            let info = heartbeat_info.borrow();

                            heartbeat(
                                &mut tx,
                                info.seq,
                                info.shard_info,
                            ).unwrap();

                            Ok(())
                        }).map_err(|why| {
                            warn!("Err in shard heartbeat timer: {:?}", why);

                            ()
                        });

                    self.handle.spawn(done);

                    self.identify().unwrap();

                    return;
                }

                self.autoreconnect().unwrap();
            },
            GatewayEvent::InvalidateSession(resumable) => {
                info!(
                    "[Shard {:?}] Received session invalidation",
                    self.shard_info,
                );

                if resumable {
                    self.resume().unwrap();
                } else {
                    self.identify().unwrap();
                }
            },
            GatewayEvent::Reconnect => {
                self.reconnect().unwrap();
            },
        }
    }

    /// Returns a stream of WebSocket messages.
    ///
    /// These can be parsed via the [`parse`] method. This should be fed to the
    /// shard via [`process`], so that it can process actionable messages, such
    /// as heartbeats and session invalidations.
    ///
    /// This will _take_ the stream from the Shard, leaving the shard without a
    /// stream. Attempting to retrieve a stream of messages a second time will
    /// result in a panic.
    ///
    /// # Panics
    ///
    /// Panics if a stream of messages was already taken from the Shard. You can
    /// check this beforehand via [`messages_present`] if you need to.
    ///
    /// [`messages_present`]: #method.messages_present
    /// [`parse`]: #method.parse
    /// [`process`]: #method.process
    pub fn messages(&mut self) -> ShardStream {
        self.stream.take().unwrap()
    }

    pub fn current_presence(&self) -> &CurrentPresence {
        &self.current_presence
    }

    pub fn heartbeat_instants(&self) -> (Option<Instant>, Option<Instant>) {
        self.heartbeat_info.borrow().heartbeat_instants
    }

    pub fn heartbeat_interval(&self) -> Option<&u64> {
        self.interval.as_ref()
    }

    pub fn last_heartbeat_ack(&self) -> Option<Instant> {
        self.heartbeat_info.borrow().heartbeat_instants.1
    }

    pub fn last_heartbeat_acknowledged(&self) -> bool {
        self.heartbeat_info.borrow().last_heartbeat_acknowledged
    }

    pub fn last_heartbeat_sent(&self) -> Option<Instant> {
        self.heartbeat_info.borrow().heartbeat_instants.0
    }

    /// Calculates the heartbeat latency between the shard and the gateway.
    ///
    /// This will return `None` if:
    ///
    /// - a heartbeat acknowledgement has not been received yet (the shard just
    ///   started); or
    /// - a heartbeat was sent and the following acknowledgement has not been
    ///   received, which would result in a negative latency.
    pub fn latency(&self) -> Option<Duration> {
        if let (Some(sent), Some(received)) = self.heartbeat_info.borrow().heartbeat_instants {
            if received > sent {
                return Some(received - sent);
            }
        }

        None
    }

    /// Whether a stream of messages is present.
    ///
    /// If a stream of messages is present, it can be taken via the [`messages`]
    /// method.
    ///
    /// [`messages`]: #method.messages
    pub fn messages_present(&self) -> bool {
        self.stream.is_some()
    }

    pub fn seq(&self) -> u64 {
        self.heartbeat_info.borrow().seq
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_ref().map(AsRef::as_ref)
    }

    pub fn shard_info(&self) -> [u64; 2] {
        self.shard_info
    }

    pub fn stage(&self) -> ConnectionStage {
        self.stage
    }

    pub fn chunk_guilds<It: IntoIterator<Item = GuildId>>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<(), Error> {
        debug!("[Shard {:?}] Requesting member chunks", shard_info);

        self.send_value(json!({
            "op": OpCode::GetGuildMembers.num(),
            "d": {
                "guild_id": guild_ids.into_iter().map(|x| x.as_ref().0).collect::<Vec<u64>>(),
                "limit": limit.unwrap_or(0),
                "query": query.unwrap_or(""),
            },
        }))
    }

    pub fn set_game(&mut self, game: Option<Game>) -> Result<(), Error> {
        self._set_game(game);

        self.presence_update()
    }

    pub fn set_presence(&mut self, status: OnlineStatus, game: Option<Game>)
        -> Result<(), Error> {
        self._set_game(game);
        self._set_status(status);

        self.presence_update()
    }

    pub fn set_status(&mut self, status: OnlineStatus) -> Result<(), Error> {
        self._set_status(status);

        self.presence_update()
    }

    pub fn send(&mut self, msg: TungsteniteMessage) -> Result<(), Error> {
        send(&mut self.tx, msg)
    }

    fn autoreconnect(&mut self) -> Result<(), Error> {
        info!("[Shard {:?}] Autoreconnecting", self.shard_info);

        if self.session_id.is_some() {
            self.resume()
        } else {
            self.reconnect()
        }
    }

    fn heartbeat(&mut self) -> Result<(), Error> {
        trace!(
            "[Shard {:?}] Sending heartbeat d: {:?}",
            self.shard_info,
            self.heartbeat_info.borrow().seq,
        );

        heartbeat(
            &mut self.tx,
            self.heartbeat_info.borrow().seq,
            self.shard_info,
        )
    }

    fn identify(&mut self) -> Result<(), Error> {
        self.stage = ConnectionStage::Identifying;

        debug!("[Shard {:?}] Identifying", self.shard_info);

        let v = json!({
            "op": OpCode::Identify.num(),
            "d": {
                "compression": false,
                "large_threshold": LARGE_THRESHOLD,
                "shard": self.shard_info,
                "token": *self.token,
                "v": GATEWAY_VERSION,
                "properties": {
                    "$browser": "test",
                    "$device": "test",
                    "$os": consts::OS,
                },
            },
        });

        self.send_value(v)
    }

    fn presence_update(&mut self) -> Result<(), Error> {
        debug!("[Shard {:?}] Sending presence update", self.shard_info);

        let now = Utc::now().timestamp() as u64;

        let v = {
            let &(ref game, ref status) = &self.current_presence;

            json!({
                "op": OpCode::StatusUpdate.num(),
                "d": {
                    "afk": false,
                    "since": now,
                    "status": status.name(),
                    "game": game.as_ref().map(|x| json!({
                        "name": x.name,
                        "type": x.kind,
                        "url": x.url,
                    })),
                },
            })
        };

        self.send_value(v)
    }

    fn reconnect(&mut self) -> Result<(), Error> {
        self.stage = ConnectionStage::Connecting;
        info!("[Shard {:?}] Attempting to reconnect", self.shard_info);

        unreachable!("reconnect");
    }

    fn resume(&mut self) -> Result<(), Error> {
        self.stage = ConnectionStage::Resuming;

        debug!(
            "[Shard {:?}] Sending resume; seq: {}",
            self.shard_info,
            self.heartbeat_info.borrow().seq,
        );

        let v = json!({
            "op": OpCode::Resume.num(),
            "d": {
                "session_id": self.session_id,
                "seq": self.heartbeat_info.borrow().seq,
                "token": *self.token,
            },
        });

        self.send_value(v)
    }

    fn send_value(&mut self, value: Value) -> Result<(), Error> {
        let json = serde_json::to_string(&value)?;

        send(&mut self.tx, TungsteniteMessage::Text(json))
    }

    fn _set_game(&mut self, game: Option<Game>) {
        self.current_presence.0 = game;
    }

    fn _set_status(&mut self, mut status: OnlineStatus) {
        if status == OnlineStatus::Invisible {
            status = OnlineStatus::Offline;
        }

        self.current_presence.1 = status;
    }
}

fn heartbeat(
    tx: &mut UnboundedSender<TungsteniteMessage>,
    seq: u64,
    shard_info: [u64; 2],
) -> Result<(), Error> {
    trace!("[Shard {:?}] Sending heartbeat", shard_info);

    let v = serde_json::to_string(&json!({
        "op": OpCode::Heartbeat.num(),
        "d": seq,
    }))?;

    send(tx, TungsteniteMessage::Text(v))
}

fn send(tx: &mut UnboundedSender<TungsteniteMessage>, msg: TungsteniteMessage)
    -> Result<(), Error> {
    trace!("Sending message over gateway: {:?}", msg);

    tx.start_send(msg).map(|_| ()).map_err(From::from)
}
