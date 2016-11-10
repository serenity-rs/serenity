use flate2::read::ZlibDecoder;
use serde_json::builder::ObjectBuilder;
use serde_json;
use std::fmt::{self, Display};
use std::io::Write;
use std::net::Shutdown;
use std::sync::mpsc::{
    self,
    Receiver as MpscReceiver,
    Sender as MpscSender,
    TryRecvError
};
use std::thread::{self, Builder as ThreadBuilder};
use std::time::Duration as StdDuration;
use std::{env, mem};
use super::login_type::LoginType;
use super::Client;
use time::{self, Duration};
use websocket::client::request::Url as RequestUrl;
use websocket::client::{Client as WsClient, Sender, Receiver};
use websocket::message::{Message as WsMessage, Type as WsType};
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WsReceiver;
use websocket::ws::sender::Sender as WsSender;
use ::constants::{self, OpCode};
use ::model::*;
use ::prelude_internal::*;

enum Status {
    SendMessage(Value),
    Sequence(u64),
    ChangeInterval(u64),
    ChangeSender(Sender<WebSocketStream>),
}

#[derive(Clone, Debug)]
pub enum ConnectionError {
    /// The connection closed
    Closed(Option<u16>, String),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionError::Closed(s, ref v) => {
                f.write_str(&format!("Connection closed {:?}: {:?}", s, v))
            },
            ConnectionError::ExpectedHello => {
                f.write_str("Expected Hello during handshake")
            },
            ConnectionError::InvalidHandshake => {
                f.write_str("Expected Ready or InvalidateSession")
            },
        }
    }
}

/// A connection is a handler for a websocket connection to Discord's gateway.
/// The connection allows for sending and receiving messages over the websocket,
/// such as setting the active game, reconnecting, syncing guilds, and more.
///
/// # Sharding
///
/// Sharding is a method to split portions of bots into separate processes. This
/// is an enforced strategy by Discord once a bot reaches a certain number of
/// guilds (2500). Once this number is reached, a bot must be sharded in a way
/// that only 2500 guilds maximum may be allocated per shard.
///
/// The "recommended" number of guilds per shard is _around_ 1000. Sharding can
/// be useful for splitting processes across separate servers. Often you may
/// want some or all shards to be in the same process, allowing for a shared
/// State. This is possible through this library.
///
/// See [Discord's documentation][docs] for more information.
///
/// If you are not using a bot account or do not require sharding - such as for
/// a small bot - then use [`Client::start`].
///
/// There are a few methods of sharding available:
///
/// - [`Client::start_autosharded`]: retrieves the number of shards Discord
/// recommends using from the API, and then automatically starts that number of
/// shards.
/// - [`Client::start_shard`]: starts a single shard for use in the instance,
/// handled by the instance of the Client. Use this if you only want 1 shard
/// handled by this instance.
/// - [`Client::start_shards`]: starts all shards in this instance. This is best
/// for when you want a completely shared State.
/// - [`Client::start_shard_range`]: start a range of shards within this
/// instance. This should be used when you, for example, want to split 10 shards
/// across 3 instances.
///
/// **Note**: User accounts can not shard. Use [`Client::start`].
///
/// # Stand-alone connections
///
/// You may instantiate a connection yourself if you need to, which is
/// completely decoupled from the client. For most use cases, you will not need
/// to do this, and you can leave the client to do it.
///
/// This can be done by passing in the required parameters to [`new`]. You can
/// then manually handle the connection yourself and receive events via
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
/// [`Client::start`]: struct.Client.html#method.start
/// [`Client::start_autosharded`]: struct.Client.html#method.start_autosharded
/// [`Client::start_shard`]: struct.Client.html#method.start_shard
/// [`Client::start_shard_range`]: struct.Client.html#method.start_shard_range
/// [`Client::start_shards`]: struct.Client.html#method.start_shards
/// [`new`]: #method.new
/// [`receive`]: #method.receive
/// [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding
pub struct Connection {
    keepalive_channel: MpscSender<Status>,
    last_sequence: u64,
    login_type: LoginType,
    receiver: Receiver<WebSocketStream>,
    session_id: Option<String>,
    shard_info: Option<[u8; 2]>,
    token: String,
    ws_url: String,
}

impl Connection {
    /// Instantiates a new instance of a connection, bypassing the client.
    ///
    /// **Note**: You should likely never need to do this yourself.
    ///
    /// # Examples
    ///
    /// Instantiating a new Connection manually for a bot with no shards, and
    /// then listening for events:
    ///
    /// ```rust,ignore
    /// use serenity::client::{Connection, LoginType, http};
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_BOT_TOKEN").expect("Token in environment");
    /// // retrieve the gateway response, which contains the URL to connect to
    /// let gateway = http::get_gateway().expect("Valid gateway response").url;
    /// let connection = Connection::new(&gateway, &token, None, LoginType::Bot)
    ///     .expect("Working connection");
    ///
    /// // at this point, you can create a `loop`, and receive events and match
    /// // their variants
    /// ```
    pub fn new(base_url: &str,
               token: &str,
               shard_info: Option<[u8; 2]>,
               login_type: LoginType)
               -> Result<(Connection, ReadyEvent)> {
        let url = try!(build_gateway_url(base_url));

        let response = try!(try!(WsClient::connect(url)).send());
        try!(response.validate());

        let (mut sender, mut receiver) = response.begin().split();

        let identification = identify(token, shard_info);
        try!(sender.send_json(&identification));

        let heartbeat_interval = match try!(receiver.recv_json(GatewayEvent::decode)) {
            GatewayEvent::Hello(interval) => interval,
            other => {
                debug!("Unexpected event during connection start: {:?}", other);

                return Err(Error::Connection(ConnectionError::ExpectedHello));
            },
        };

        let (tx, rx) = mpsc::channel();
        try!(ThreadBuilder::new()
            .name("serenity keepalive".into())
            .spawn(move || keepalive(heartbeat_interval, sender, rx)));

        // Parse READY
        let event = try!(receiver.recv_json(GatewayEvent::decode));
        let (ready, sequence) = try!(parse_ready(event,
                                                 &tx,
                                                 &mut receiver,
                                                 identification));

        Ok((Connection {
            keepalive_channel: tx,
            last_sequence: sequence,
            login_type: login_type,
            receiver: receiver,
            token: token.to_owned(),
            session_id: Some(ready.ready.session_id.clone()),
            shard_info: shard_info,
            ws_url: base_url.to_owned(),
        }, ready))
    }

    pub fn shard_info(&self) -> Option<[u8; 2]> {
        self.shard_info
    }

    pub fn set_game(&self, game: Option<Game>) {
        self.set_presence(game, OnlineStatus::Online, false)
    }

    pub fn set_presence(&self,
                        game: Option<Game>,
                        status: OnlineStatus,
                        afk: bool) {
        let status = match status {
            OnlineStatus::Offline => OnlineStatus::Invisible,
            other => other,
        };
        let msg = ObjectBuilder::new()
            .insert("op", OpCode::StatusUpdate.num())
            .insert_object("d", move |mut object| {
                object = object.insert("since", 0)
                    .insert("afk", afk)
                    .insert("status", status.name());

                match game {
                    Some(game) => {
                        object.insert_object("game", move |o|
                            o.insert("name", game.name))
                    },
                    None => object.insert("game", Value::Null),
                }
            })
            .build();

        let _ = self.keepalive_channel.send(Status::SendMessage(msg));
    }

    pub fn receive(&mut self) -> Result<Event> {
        match self.receiver.recv_json(GatewayEvent::decode) {
            Ok(GatewayEvent::Dispatch(sequence, event)) => {
                self.last_sequence = sequence;

                let _ = self.keepalive_channel.send(Status::Sequence(sequence));

                if let Event::Resumed(ref ev) = event {
                    let _ = self.keepalive_channel.send(Status::ChangeInterval(ev.heartbeat_interval));
                }

                Ok(event)
            },
            Ok(GatewayEvent::Heartbeat(sequence)) => {
                let map = ObjectBuilder::new()
                    .insert("d", sequence)
                    .insert("op", OpCode::Heartbeat.num())
                    .build();
                let _ = self.keepalive_channel.send(Status::SendMessage(map));

                self.receive()
            },
            Ok(GatewayEvent::HeartbeatAck) => {
                self.receive()
            },
            Ok(GatewayEvent::Hello(interval)) => {
                let _ = self.keepalive_channel.send(Status::ChangeInterval(interval));

                self.receive()
            },
            Ok(GatewayEvent::InvalidateSession) => {
                self.session_id = None;

                let status = Status::SendMessage(identify(&self.token,
                                                          self.shard_info));

                let _ = self.keepalive_channel.send(status);

                self.receive()
            },
            Ok(GatewayEvent::Reconnect) => {
                self.reconnect()
            },
            Err(Error::Connection(ConnectionError::Closed(num, message))) => {
                warn!("Closing with {:?}: {:?}", num, message);

                // Attempt to resume if the following was not received:
                //
                // - 1000: Close.
                //
                // Otherwise, fallback to reconnecting.
                if num != Some(1000) {
                    if let Some(session_id) = self.session_id.clone() {
                        match self.resume(session_id) {
                            Ok(event) => return Ok(event),
                            Err(why) => debug!("Err resuming: {:?}", why),
                        }
                    }
                }

                self.reconnect()
            },
            Err(Error::WebSocket(why)) => {
                warn!("Websocket error: {:?}", why);
                info!("Reconnecting");

                // Attempt to resume if the following was not received:
                //
                // - InvalidateSession.
                //
                // Otherwise, fallback to reconnecting.
                if let Some(session_id) = self.session_id.clone() {
                    match self.resume(session_id) {
                        Ok(event) => return Ok(event),
                        Err(why) => debug!("Err resuming: {:?}", why),
                    }
                }

                self.reconnect()
            },
            Err(error) => Err(error),
        }
    }

    fn reconnect(&mut self) -> Result<Event> {
        debug!("Reconnecting");

        // Take a few attempts at reconnecting; otherwise fall back to
        // re-instantiating the connection.
        for _ in 0..3 {
            let connection = Connection::new(&self.ws_url,
                                             &self.token,
                                             self.shard_info,
                                             self.login_type);

            if let Ok((connection, ready)) = connection {
                try!(mem::replace(self, connection).shutdown());

                self.session_id = Some(ready.ready.session_id.clone());

                return Ok(Event::Ready(ready));
            }

            thread::sleep(StdDuration::from_secs(1));
        }

        // If all else fails: get a new endpoint.
        //
        // A bit of complexity here: instantiate a temporary instance of a
        // Client. This client _does not_ replace the current client(s) that the
        // user has. This client will then connect to gateway. This new
        // connection will be used to replace _this_ connection.
        let (connection, ready) = {
            let mut client = Client::login_raw(&self.token.clone(),
                                               self.login_type);

            try!(client.boot_connection(self.shard_info))
        };

        // Replace this connection with a new one, and shutdown the now-old
        // connection.
        try!(mem::replace(self, connection).shutdown());

        self.session_id = Some(ready.ready.session_id.clone());

        Ok(Event::Ready(ready))
    }

    fn resume(&mut self, session_id: String) -> Result<Event> {
        try!(self.receiver.get_mut().get_mut().shutdown(Shutdown::Both));
        let url = try!(build_gateway_url(&self.ws_url));

        let response = try!(try!(WsClient::connect(url)).send());
        try!(response.validate());

        let (mut sender, mut receiver) = response.begin().split();

        try!(sender.send_json(&ObjectBuilder::new()
            .insert_object("d", |o| o
                .insert("session_id", session_id)
                .insert("seq", self.last_sequence)
                .insert("token", &self.token)
            )
            .insert("op", OpCode::Resume.num())
            .build()));

        let first_event;

        loop {
            match try!(receiver.recv_json(GatewayEvent::decode)) {
                GatewayEvent::Dispatch(seq, event) => {
                    if let Event::Ready(ref event) = event {
                        self.session_id = Some(event.ready.session_id.clone());
                    }

                    self.last_sequence = seq;
                    first_event = event;

                    break;
                },
                GatewayEvent::InvalidateSession => {
                    try!(sender.send_json(&identify(&self.token, self.shard_info)));
                }
                other => {
                    debug!("Unexpected event: {:?}", other);

                    return Err(Error::Connection(ConnectionError::InvalidHandshake));
                }
            }
        }

        self.receiver = receiver;
        let _ = self.keepalive_channel.send(Status::ChangeSender(sender));

        Ok(first_event)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        let stream = self.receiver.get_mut().get_mut();

        {
            let mut sender = Sender::new(stream.by_ref(), true);
            let message = WsMessage::close_because(1000, "");

            try!(sender.send_message(&message));
        }

        try!(stream.flush());
        try!(stream.shutdown(Shutdown::Both));

        Ok(())
    }

    pub fn sync_guilds(&self, guild_ids: &[GuildId]) {
        let msg = ObjectBuilder::new()
            .insert("op", OpCode::SyncGuild.num())
            .insert_array("d", |a| guild_ids.iter().fold(a, |a, s| a.push(s.0)))
            .build();

        let _ = self.keepalive_channel.send(Status::SendMessage(msg));
    }

    pub fn sync_calls(&self, channels: &[ChannelId]) {
        for &channel in channels {
            let msg = ObjectBuilder::new()
                .insert("op", OpCode::SyncCall.num())
                .insert_object("d", |obj| obj
                    .insert("channel_id", channel.0)
                )
                .build();

            let _ = self.keepalive_channel.send(Status::SendMessage(msg));
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        match self.shutdown() {
            Ok(()) => {
                if let Some(info) = self.shard_info {
                    println!("Correctly shutdown shard {}/{}", info[0], info[1] - 1);
                } else {
                    println!("Correctly shutdown connection");
                }
            },
            Err(why) => {
                if let Some(info) = self.shard_info {
                    println!("Failed to shutdown shard {}/{}: {:?}",
                           info[0],
                           info[1] - 1,
                           why);
                } else {
                    println!("Failed to shutdown connection: {:?}", why);
                }
            }
        }
    }
}

trait ReceiverExt {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T>
        where F: FnOnce(Value) -> Result<T>;
}

trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for Receiver<WebSocketStream> {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T> where F: FnOnce(Value) -> Result<T> {
        let message: WsMessage = try!(self.recv_message());

        if message.opcode == WsType::Close {
            let representation = String::from_utf8_lossy(&message.payload)
                .into_owned();

            Err(Error::Connection(ConnectionError::Closed(message.cd_status_code,
                                                          representation)))
        } else if message.opcode == WsType::Binary || message.opcode == WsType::Text {
            let json: Value = if message.opcode == WsType::Binary {
                try!(serde_json::from_reader(ZlibDecoder::new(&message.payload[..])))
            } else {
                try!(serde_json::from_reader(&message.payload[..]))
            };

            decode(json).map_err(|err| {
                warn!("Error decoding: {}",
                      String::from_utf8_lossy(&message.payload));

                err
            })
        } else {
            let representation = String::from_utf8_lossy(&message.payload)
                .into_owned();

            Err(Error::Connection(ConnectionError::Closed(None,
                                                          representation)))
        }
    }
}

impl SenderExt for Sender<WebSocketStream> {
    fn send_json(&mut self, value: &Value) -> Result<()> {
        serde_json::to_string(value)
            .map(WsMessage::text)
            .map_err(Error::from)
            .and_then(|m| self.send_message(&m).map_err(Error::from))
    }
}

fn parse_ready(event: GatewayEvent,
               tx: &MpscSender<Status>,
               receiver: &mut Receiver<WebSocketStream>,
               identification: Value)
               -> Result<(ReadyEvent, u64)> {
    match event {
        GatewayEvent::Dispatch(seq, Event::Ready(event)) => {
            Ok((event, seq))
        },
        GatewayEvent::InvalidateSession => {
            debug!("Session invalidation");

            let _ = tx.send(Status::SendMessage(identification));

            match try!(receiver.recv_json(GatewayEvent::decode)) {
                GatewayEvent::Dispatch(seq, Event::Ready(event)) => {
                    Ok((event, seq))
                },
                other => {
                    debug!("Unexpected event: {:?}", other);

                    Err(Error::Connection(ConnectionError::InvalidHandshake))
                },
            }
        },
        other => {
            debug!("Unexpected event: {:?}", other);

            Err(Error::Connection(ConnectionError::InvalidHandshake))
        },
    }
}

fn identify(token: &str, shard_info: Option<[u8; 2]>) -> serde_json::Value {
    ObjectBuilder::new()
        .insert("op", OpCode::Identify.num())
        .insert_object("d", |mut object| {
            object = identify_compression(object)
                .insert("large_threshold", 250) // max value
                .insert_object("properties", |object| object
                    .insert("$browser", "Feature-full and ergonomic discord rust library")
                    .insert("$device", "serenity")
                    .insert("$os", env::consts::OS)
                    .insert("$referrer", "")
                    .insert("$referring_domain", "")
                )
                .insert("token", token)
                .insert("v", constants::GATEWAY_VERSION);

            if let Some(shard_info) = shard_info {
                object = object
                    .insert_array("shard", |array| array
                        .push(shard_info[0])
                        .push(shard_info[1]));
            }

            object
        })
        .build()
}

#[cfg(not(feature = "debug"))]
fn identify_compression(object: ObjectBuilder) -> ObjectBuilder {
    object.insert("compression", true)
}

#[cfg(feature = "debug")]
fn identify_compression(object: ObjectBuilder) -> ObjectBuilder {
    object.insert("compression", false)
}

fn build_gateway_url(base: &str) -> Result<::websocket::client::request::Url> {
    RequestUrl::parse(&format!("{}?v={}", base, constants::GATEWAY_VERSION))
        .map_err(|_| Error::Client(ClientError::Gateway))
}

fn keepalive(interval: u64,
             mut sender: Sender<WebSocketStream>,
             channel: MpscReceiver<Status>) {
    let mut base_interval = Duration::milliseconds(interval as i64);
    let mut next_tick = time::get_time() + base_interval;

    let mut last_sequence = 0;

    'outer: loop {
        thread::sleep(StdDuration::from_millis(100));

        loop {
            match channel.try_recv() {
                Ok(Status::ChangeInterval(interval)) => {
                    base_interval = Duration::milliseconds(interval as i64);
                },
                Ok(Status::ChangeSender(new_sender)) => {
                    sender = new_sender;
                },
                Ok(Status::SendMessage(val)) => {
                    match sender.send_json(&val) {
                        Ok(()) => {},
                        Err(e) => warn!("Err sending message: {:?}", e),
                    }
                },
                Ok(Status::Sequence(seq)) => {
                    last_sequence = seq;
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'outer,
            }
        }

        if time::get_time() >= next_tick {
            next_tick = next_tick + base_interval;

            let map = ObjectBuilder::new()
                .insert("d", last_sequence)
                .insert("op", OpCode::Heartbeat.num())
                .build();

            match sender.send_json(&map) {
                Ok(()) => {},
                Err(e) => warn!("Error sending gateway keeaplive: {:?}", e)
            }
        }
    }

    let _ = sender.get_mut().shutdown(Shutdown::Both);
}
