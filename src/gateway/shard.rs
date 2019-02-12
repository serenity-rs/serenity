use chrono::Utc;
use constants::{GATEWAY_VERSION, LARGE_THRESHOLD, OpCode};
use futures::future::Future;
use futures::stream::Stream as FuturesStream;
use futures::sync::mpsc::{self, UnboundedSender};
use futures::Sink;
use model::event::{Event, GatewayEvent};
use model::gateway::Activity;
use model::id::GuildId;
use model::user::OnlineStatus;
use parking_lot::Mutex;
use serde_json::{self, Value};
use std::env::consts;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::sync::Arc;
use std::time::{Duration, Instant};
use super::{ConnectionStage, CurrentPresence, ShardStream};
use tokio_tungstenite::tungstenite::{
    Error as TungsteniteError,
    Message as TungsteniteMessage,
};
use tokio::{
    self,
    timer::Interval,
};
use url::Url;
use std::str::FromStr;
use ::{Error, Result};

const CONNECTION: &'static str = "wss://gateway.discord.gg/?v=6&encoding=json";

pub enum Action {
    Autoreconnect,
    Identify,
    Reconnect,
    Resume,
}

#[derive(Copy, Clone, Debug)]
struct HeartbeatInfo {
    pub heartbeat_instants: (Option<Instant>, Option<Instant>),
    pub heartbeater: bool,
    pub last_heartbeat_acknowledged: bool,
    pub seq: u64,
    pub shard_info: [u64; 2],
}

impl HeartbeatInfo {
    fn new(shard_info: [u64; 2]) -> Self {
        Self {
            heartbeat_instants: (None, None),
            heartbeater: false,
            last_heartbeat_acknowledged: true,
            seq: 0,
            shard_info,
        }
    }
}

pub struct Shard {
    current_presence: CurrentPresence,
    heartbeat_info: Arc<Mutex<HeartbeatInfo>>,
    interval: Option<u64>,
    session_id: Option<String>,
    shard_info: [u64; 2],
    stage: Arc<Mutex<ConnectionStage>>,
    stream: Arc<Mutex<Option<ShardStream>>>,
    token: String,
    tx: Arc<Mutex<UnboundedSender<TungsteniteMessage>>>,
}

impl Shard {
    pub fn new(token: String, shard_info: [u64; 2])
        -> impl Future<Item = Shard, Error = Error> + Send {
        connect(CONNECTION).map(move |(sender, stream)| {
            Self {
                current_presence: (None, OnlineStatus::Online),
                heartbeat_info: Arc::new(Mutex::new(HeartbeatInfo::new(shard_info))),
                interval: None,
                session_id: None,
                stage: Arc::new(Mutex::new(ConnectionStage::Handshake)),
                stream: Arc::new(Mutex::new(Some(stream))),
                tx: Arc::new(Mutex::new(sender)),
                shard_info,
                token,
            }
        }).from_err()
    }

    pub fn parse(&self, msg: &TungsteniteMessage) -> Result<GatewayEvent> {
        match msg {
            TungsteniteMessage::Binary(v) => serde_json::from_slice(v),
            TungsteniteMessage::Text(v) => serde_json::from_str(v),
            _ => unreachable!("parse other"),
        }.map_err(From::from)
    }

    /// Processes the given event to determine if something needs to be done.
    ///
    /// For example, an event may cause the shard to need to reconnect due to a
    /// session invalidating.
    pub fn process(
        &mut self,
        event: &GatewayEvent,
    ) -> Result<Option<Action>> {
        match *event {
            GatewayEvent::Dispatch(seq, ref event) => {
                let mut info = self.heartbeat_info.lock();
                let self_seq = info.seq;

                if seq > self_seq + 1 {
                    warn!(
                        "[Shard {:?}] Sequence off; them: {}, us: {}",
                        self.shard_info,
                        seq,
                        self_seq,
                    );
                }

                match *event {
                    Event::Ready(ref ready) => {
                        debug!("[Shard {:?}] Received Ready", self.shard_info);

                        self.session_id = Some(ready.ready.session_id.clone());
                        *self.stage.lock() = ConnectionStage::Connected;
                    },
                    Event::Resumed(_) => {
                        info!("[Shard {:?}] Resumed", self.shard_info);

                        *self.stage.lock() = ConnectionStage::Connected;
                        info.last_heartbeat_acknowledged = true;
                        info.heartbeat_instants = (Some(Instant::now()), None);
                    },
                    _ => {},
                }

                info.seq = seq;

                Ok(None)
            },
            GatewayEvent::Heartbeat(s) => {
                info!("[Shard {:?}] Received shard heartbeat", self.shard_info);

                if s > self.heartbeat_info.lock().seq + 1 {
                    info!(
                        "[Shard {:?}] Received off sequence (them: {}; us: {}); resuming",
                        self.shard_info,
                        s,
                        self.heartbeat_info.lock().seq
                    );
                }

                self.heartbeat()?;

                Ok(None)
            },
            GatewayEvent::HeartbeatAck => {
                trace!("[Shard {:?}] Received heartbeat ack", self.shard_info);

                let mut info = self.heartbeat_info.lock();
                info.heartbeat_instants.1 = Some(Instant::now());
                info.last_heartbeat_acknowledged = true;

                Ok(None)
            },
            GatewayEvent::Hello(interval) => {
                debug!(
                    "[Shard {:?}] Received a Hello; interval: {}",
                    self.shard_info,
                    interval,
                );

                trace!("Unlocking stage");

                if self.stage.lock().clone() == ConnectionStage::Resuming {
                    trace!("Unlocked stage; it's a resume");

                    return Ok(None);
                }

                trace!("Unlocked stage");

                if interval > 0 {
                    self.interval = Some(interval);
                }

                trace!("Unlocking stage to check if it's a handshake");

                if self.stage.lock().clone() == ConnectionStage::Handshake {
                    trace!("Unlocked stage; it's a handshake");

                    let heartbeat_info = Arc::clone(&self.heartbeat_info);
                    let mut tx = self.tx.clone();
                    let duration = Duration::from_millis(interval);

                    let done = Interval::new(Instant::now(), duration)
                        .map_err(|why| {
                            warn!("Err starting heartbeat interval: {:?}", why);
                        }).for_each(move |_| {
                            let info = heartbeat_info.lock();

                            let heartbeat = heartbeat(
                                &tx,
                                info.seq,
                                info.shard_info,
                            );

                            if let Err(why) = heartbeat {
                                warn!("Err heartbeating: {:?}", why);

                                return Err(());
                            }

                            Ok(())
                        });

                    tokio::spawn(done);

                    trace!("Identifying");

                    return Ok(Some(Action::Identify));
                }

                trace!("Autoreconnecting");

                Ok(Some(Action::Autoreconnect))
            },
            GatewayEvent::InvalidateSession(resumable) => {
                info!(
                    "[Shard {:?}] Received session invalidation",
                    self.shard_info,
                );

                if resumable {
                    Ok(Some(Action::Resume))
                } else {
                    Ok(Some(Action::Identify))
                }
            },
            GatewayEvent::Reconnect => {
                Ok(Some(Action::Reconnect))
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
    /// result returning `None`.
    ///
    /// [`parse`]: #method.parse
    /// [`process`]: #method.process
    pub fn messages(&mut self) -> Option<ShardStream> {
        self.stream.lock().take()
    }

    pub fn current_presence(&self) -> &CurrentPresence {
        &self.current_presence
    }

    pub fn heartbeat_instants(&self) -> (Option<Instant>, Option<Instant>) {
        self.heartbeat_info.lock().heartbeat_instants
    }

    pub fn heartbeat_interval(&self) -> Option<&u64> {
        self.interval.as_ref()
    }

    pub fn last_heartbeat_ack(&self) -> Option<Instant> {
        self.heartbeat_info.lock().heartbeat_instants.1
    }

    pub fn last_heartbeat_acknowledged(&self) -> bool {
        self.heartbeat_info.lock().last_heartbeat_acknowledged
    }

    pub fn last_heartbeat_sent(&self) -> Option<Instant> {
        self.heartbeat_info.lock().heartbeat_instants.0
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
        if let (Some(sent), Some(received)) = self.heartbeat_info.lock().heartbeat_instants {
            if received > sent {
                return Some(received - sent);
            }
        }

        None
    }

    pub fn seq(&self) -> u64 {
        self.heartbeat_info.lock().seq
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_ref().map(AsRef::as_ref)
    }

    pub fn shard_info(&self) -> [u64; 2] {
        self.shard_info
    }

    pub fn stage(&self) -> ConnectionStage {
        *self.stage.lock()
    }

    pub fn chunk_guilds<It: IntoIterator<Item = GuildId>>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<()> {
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

    pub fn set_activity(&mut self, activity: Option<Activity>) -> Result<()> {
        self._set_activity(activity);

        self.presence_update()
    }

    pub fn set_presence(&mut self, status: OnlineStatus, activity: Option<Activity>)
        -> Result<()> {
        self._set_activity(activity);
        self._set_status(status);

        self.presence_update()
    }

    pub fn set_status(&mut self, status: OnlineStatus) -> Result<()> {
        self._set_status(status);

        self.presence_update()
    }

    pub fn send(&self, msg: TungsteniteMessage) -> Result<()> {
        send(&self.tx, msg)
    }

    pub fn autoreconnect(&mut self) -> impl Future<Item = (), Error = Error> + Send {
        info!("[Shard {:?}] Autoreconnecting", self.shard_info);

        use futures::future::Either;
        if self.session_id.is_some() && self.seq() > 0 {
            Either::A(self.resume())
        } else {
            Either::B(self.reconnect())
        }
    }

    pub fn heartbeat(&mut self) -> Result<()> {
        trace!(
            "[Shard {:?}] Sending heartbeat d: {:?}",
            self.shard_info,
            self.heartbeat_info.lock().seq,
        );

        heartbeat(
            &self.tx,
            self.heartbeat_info.lock().seq,
            self.shard_info,
        )
    }

    pub fn identify(&mut self) -> Result<()> {
        *self.stage.lock() = ConnectionStage::Identifying;

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
                    "$browser": "serenity.rs",
                    "$device": "likely some water aerosol in the sky",
                    "$os": consts::OS,
                },
            },
        });

        self.send_value(v)
    }

    fn initialize(&mut self) -> impl Future<Item = (), Error = Error> + Send {
        debug!("[Shard {:?}] Initializing", self.shard_info);

        *self.stage.lock() = ConnectionStage::Connecting;

        let heartbeat_info = Arc::clone(&self.heartbeat_info);
        let shard_info = self.shard_info;
        let stage = Arc::clone(&self.stage);
        let self_stream = Arc::clone(&self.stream);
        let tx = Arc::clone(&self.tx);

        connect(CONNECTION).map(move |(sender, stream)| {
            *heartbeat_info.lock() = HeartbeatInfo::new(shard_info);
            *stage.lock() = ConnectionStage::Handshake;
            *self_stream.lock() = Some(stream);
            *tx.lock() = sender;
        })
    }

    fn presence_update(&mut self) -> Result<()> {
        debug!("[Shard {:?}] Sending presence update", self.shard_info);

        let now = Utc::now().timestamp() as u64;

        let v = {
            let &(ref activity, ref status) = &self.current_presence;

            json!({
                "op": OpCode::StatusUpdate.num(),
                "d": {
                    "afk": false,
                    "since": now,
                    "status": status.name(),
                    "game": activity.as_ref().map(|x| json!({
                        "name": x.name,
                        "type": x.kind,
                        "url": x.url,
                    })),
                },
            })
        };

        self.send_value(v)
    }

    pub fn reconnect(&mut self) -> impl Future<Item = (), Error = Error> + Send {
        *self.stage.lock() = ConnectionStage::Connecting;
        info!("[Shard {:?}] Attempting to reconnect", self.shard_info);

        self.reset().expect("Shard reset failed");

        self.initialize()
    }

    fn reset(&mut self) -> Result<()> {
        self.interval = None;
        self.session_id = None;
        *self.stage.lock() = ConnectionStage::Disconnected;

        let mut info = self.heartbeat_info.lock();
        info.last_heartbeat_acknowledged = true;
        info.seq = 0;

        Ok(())
    }

    pub fn resume(&mut self) -> impl Future<Item = (), Error = Error> + Send {
        let seq = self.heartbeat_info.lock().seq;
        let session_id = self.session_id.clone();
        let shard_info = self.shard_info;
        let stage = Arc::clone(&self.stage);
        let token = self.token.clone();
        let tx = Arc::clone(&self.tx);

        self.initialize().then(move |res| {
            if res.is_err() {
                return res;
            }

            *stage.lock() = ConnectionStage::Resuming;

            debug!(
                "[Shard {:?}] Sending resume; seq: {}",
                shard_info,
                seq,
            );

            let v = serde_json::to_string(&json!({
                "op": OpCode::Resume.num(),
                "d": {
                    "session_id": session_id,
                    "seq": seq,
                    "token": *token,
                },
            }))?;

            send(&tx, TungsteniteMessage::Text(v))?;

            Ok(())
        })
    }

    fn send_value(&mut self, value: Value) -> Result<()> {
        let json = serde_json::to_string(&value)?;

        send(&self.tx, TungsteniteMessage::Text(json))
    }

    fn _set_activity(&mut self, activity: Option<Activity>) {
        self.current_presence.0 = activity;
    }

    fn _set_status(&mut self, mut status: OnlineStatus) {
        if status == OnlineStatus::Invisible {
            status = OnlineStatus::Offline;
        }

        self.current_presence.1 = status;
    }
}

// Some code copy-pasted from:
// <https://github.com/snapview/tokio-tungstenite/blob/master/src/connect.rs>
//
// Ignore how bad this is for now, I just wanted it working.

mod encryption {
    extern crate native_tls;
    extern crate tokio_io;
    extern crate tokio_tls;

    use self::native_tls::TlsConnector;
    use self::tokio_tls::{TlsConnector as TokioTlsConnector, TlsStream};

    use futures::{future, Future};
    use self::tokio_io::{AsyncRead, AsyncWrite};

    use tokio_tungstenite::tungstenite::Error;
    use tokio_tungstenite::tungstenite::stream::Mode;

    use tokio_tungstenite::stream::Stream as StreamSwitcher;

    /// A stream that might be protected with TLS.
    pub type MaybeTlsStream<S> = StreamSwitcher<S, TlsStream<S>>;

    pub type AutoStream<S> = MaybeTlsStream<S>;

    pub fn wrap_stream<S>(socket: S, domain: String, mode: Mode)
        -> impl Future<Item=AutoStream<S>, Error=Error> + Send
    where
        S: 'static + AsyncRead + AsyncWrite + Send,
    {
        match mode {
            Mode::Plain => future::Either::A(future::ok(StreamSwitcher::Plain(socket))),
            Mode::Tls => {
                future::Either::B(future::result(TlsConnector::new())
                            .map(TokioTlsConnector::from)
                            .and_then(move |connector| connector.connect(&domain, socket))
                            .map(|s| StreamSwitcher::Tls(s))
                            .map_err(|e| Error::Tls(e)))
            }
        }
    }
}

use tokio_tungstenite::tungstenite::handshake::client::Request;

#[inline]
fn domain(request: &Request) -> Result<String> {
    match request.url.host_str() {
        Some(d) => Ok(d.to_string()),
        None => Err(Error::Url("no host name in the url".into())),
    }
}

fn connect(
    uri: &str,
) -> impl Future<Item = (UnboundedSender<TungsteniteMessage>, ShardStream), Error = Error> + Send {
    extern crate tokio_dns;

    use futures::future;
    use tokio_tungstenite;
    use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

    let config = WebSocketConfig {
        max_message_size: Some(usize::max_value()),
        max_frame_size: Some(usize::max_value()),
        ..Default::default()
    };

    let url = Url::from_str(uri).expect("Err parsing url");

    let request = Request::from(url);
    let domain = match domain(&request) {
        Ok(domain) => domain,
        Err(why) => return future::Either::A(future::err(Error::from(why))),
    };
    let port = request.url.port_or_known_default().expect("Bug: port unknown");

    let mode = match tokio_tungstenite::tungstenite::client::url_mode(&request.url) {
        Ok(mode) => mode,
        Err(why) => return future::Either::A(future::err(Error::from(why))),
    };

    future::Either::B(tokio_dns::TcpStream::connect((domain.as_str(), port)).map_err(|why| {
        warn!("Err connecting to remote: {:?}", why);

        why
    }).from_err().and_then(move |stream| {
        encryption::wrap_stream(stream, domain, mode).and_then(|mut stream| {
            tokio_tungstenite::stream::NoDelay::set_nodelay(&mut stream, true)
                .map(move |()| stream)
                .map_err(|e| e.into())
        })
    }).and_then(move |stream| {
        debug!("Connected to remote; request: {:?}", request);
        tokio_tungstenite::client_async_with_config(
            request,
            stream,
            Some(config),
        )
    }).map(move |(duplex, _)| {
        debug!("Shook hands with remote 🤝");
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

        tokio::spawn(done);

        (tx, stream)
    }).from_err())
}

fn heartbeat(
    tx: &Arc<Mutex<UnboundedSender<TungsteniteMessage>>>,
    seq: u64,
    shard_info: [u64; 2],
) -> Result<()> {
    trace!("[Shard {:?}] Sending heartbeat", shard_info);

    let v = serde_json::to_string(&json!({
        "op": OpCode::Heartbeat.num(),
        "d": seq,
    }))?;

    send(tx, TungsteniteMessage::Text(v))
}

fn send(
    tx: &Arc<Mutex<UnboundedSender<TungsteniteMessage>>>,
    msg: TungsteniteMessage,
) -> Result<()> {
    trace!("Sending message over gateway: {}", msg);

    tx.lock().start_send(msg).map(|_| ()).map_err(From::from)
}
