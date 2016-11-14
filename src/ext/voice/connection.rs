use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::builder::ObjectBuilder;
use sodiumoxide::crypto::secretbox::Key;
use std::net::{Shutdown, SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver as MpscReceiver};
use std::thread::{self, Builder as ThreadBuilder};
use super::audio::{AudioReceiver, AudioSource};
use super::connection_info::ConnectionInfo;
use super::{CRYPTO_MODE, VoiceError};
use websocket::client::request::Url as WebsocketUrl;
use websocket::client::{
    Client as WsClient,
    Receiver as WsReceiver,
    Sender as WsSender
};
use websocket::stream::WebSocketStream;
use ::client::STATE;
use ::constants::VoiceOpCode;
use ::internal::prelude::*;
use ::internal::ws_impl::{ReceiverExt, SenderExt};
use ::internal::Timer;
use ::model::VoiceEvent;

pub enum ReceiverStatus {
    Udp(Vec<u8>),
    Websocket(VoiceEvent),
}

#[allow(dead_code)]
pub struct Connection {
    audio_timer: Timer,
    destination: SocketAddr,
    keepalive_timer: Timer,
    key: Key,
    receive_channel: MpscReceiver<ReceiverStatus>,
    sender: WsSender<WebSocketStream>,
    sequence: u64,
    speaking: bool,
    ssrc: u32,
    timestamp: u32,
    udp: UdpSocket,
}

impl Connection {
    pub fn new(mut info: ConnectionInfo) -> Result<Connection> {
        let url = try!(generate_url(&mut info.endpoint));

        let response = try!(try!(WsClient::connect(url)).send());
        try!(response.validate());
        let (mut sender, mut receiver) = response.begin().split();

        try!(sender.send_json(&identify(&info)));

        let handshake = match try!(receiver.recv_json(VoiceEvent::decode)) {
            VoiceEvent::Handshake(handshake) => handshake,
            _ => return Err(Error::Voice(VoiceError::ExpectedHandshake)),
        };

        if !has_valid_mode(handshake.modes) {
            return Err(Error::Voice(VoiceError::VoiceModeUnavailable));
        }

        let destination = {
            try!(try!((&info.endpoint[..], handshake.port)
                .to_socket_addrs())
                .next()
                .ok_or(Error::Voice(VoiceError::HostnameResolve)))
        };
        let udp = try!(UdpSocket::bind("0.0.0.0:0"));

        {
            let mut bytes = [0; 70];
            try!((&mut bytes[..]).write_u32::<BigEndian>(handshake.ssrc));
            try!(udp.send_to(&bytes, destination));
        }

        try!(send_acknowledgement(&mut sender, &udp));

        let key = try!(get_encryption_key(&mut receiver));

        let receive_channel = try!(start_threads(receiver, &udp));

        info!("[Voice] Connected to: {}", info.endpoint);

        Ok(Connection {
            audio_timer: Timer::new(1000 * 60 * 4),
            destination: destination,
            key: key,
            keepalive_timer: Timer::new(handshake.heartbeat_interval),
            receive_channel: receive_channel,
            udp: udp,
            sender: sender,
            sequence: 0,
            speaking: false,
            ssrc: handshake.ssrc,
            timestamp: 0,
        })
    }

    #[allow(unused_variables)]
    pub fn update(&mut self,
                  source: &mut Option<Box<AudioSource>>,
                  receiver: &mut Option<Box<AudioReceiver>>,
                  audio_timer: &mut Timer)
                  -> Result<()> {
        if let Some(receiver) = receiver.as_mut() {
            while let Ok(status) = self.receive_channel.try_recv() {
                match status {
                    ReceiverStatus::Udp(packet) => {
                        debug!("[Voice] Received UDP packet: {:?}", packet);
                    },
                    ReceiverStatus::Websocket(VoiceEvent::Speaking(ev)) => {
                        receiver.speaking_update(ev.ssrc,
                                                 &ev.user_id,
                                                 ev.speaking);
                    },
                    ReceiverStatus::Websocket(other) => {
                        info!("[Voice] Received other websocket data: {:?}",
                              other);
                    },
                }
            }
        } else {
            while let Ok(_) = self.receive_channel.try_recv() {}
        }

        // Send the voice websocket keepalive if it's time
        if self.keepalive_timer.check() {
            try!(self.sender.send_json(&keepalive()));
        }

        // Send the UDP keepalive if it's time
        if self.audio_timer.check() {
            let mut bytes = [0; 4];
            try!((&mut bytes[..]).write_u32::<BigEndian>(self.ssrc));
            try!(self.udp.send_to(&bytes, self.destination));
        }

        try!(self.speaking(true));

        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(960);

        audio_timer.await();
        self.audio_timer.reset();
        Ok(())
    }

    fn speaking(&mut self, speaking: bool) -> Result<()> {
        if self.speaking == speaking {
            return Ok(());
        }

        self.speaking = speaking;

        let map = ObjectBuilder::new()
            .insert("op", VoiceOpCode::Speaking.num())
            .insert_object("d", |object| object
                .insert("delay", 0))
                .insert("speaking", speaking)
            .build();

        self.sender.send_json(&map)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.sender.get_mut().shutdown(Shutdown::Both);

        info!("Voice disconnected");
    }
}

fn generate_url(endpoint: &mut String) -> Result<WebsocketUrl> {
    if endpoint.ends_with(":80") {
        let len = endpoint.len();

        endpoint.truncate(len - 3);
    }

    WebsocketUrl::parse(&format!("wss://{}", endpoint))
        .or(Err(Error::Voice(VoiceError::EndpointUrl)))
}

pub fn get_encryption_key(receiver: &mut WsReceiver<WebSocketStream>)
    -> Result<Key> {
    loop {
        match try!(receiver.recv_json(VoiceEvent::decode)) {
            VoiceEvent::Ready(ready) => {
                if ready.mode != CRYPTO_MODE {
                    return Err(Error::Voice(VoiceError::VoiceModeInvalid));
                }

                return Key::from_slice(&ready.secret_key)
                    .ok_or(Error::Voice(VoiceError::KeyGen));
            },
            VoiceEvent::Unknown(op, value) => {
                debug!("Unknown message type: {}/{:?}", op.num(), value);
            },
            _ => {},
        }
    }
}

fn identify(info: &ConnectionInfo) -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::Identify.num())
        .insert_object("d", |o| o
            .insert("server_id", info.server_id)
            .insert("session_id", &info.session_id)
            .insert("token", &info.token)
            .insert("user_id", STATE.lock().unwrap().user.id.0))
        .build()
}

#[inline(always)]
fn has_valid_mode(modes: Vec<String>) -> bool {
    modes.iter().any(|s| s == CRYPTO_MODE)
}

fn keepalive() -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::KeepAlive.num())
        .insert("d", Value::Null)
        .build()
}

#[inline]
fn select_protocol(address: &[u8], port: u16) -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::SelectProtocol.num())
        .insert_object("d", |o| o
            .insert("protocol", "udp")
            .insert_object("data", |o| o
                .insert("address", address)
                .insert("mode", "xsalsa20_poly1305")))
                .insert("port", port)
        .build()
}

#[inline]
fn send_acknowledgement(sender: &mut WsSender<WebSocketStream>, udp: &UdpSocket)
    -> Result<()> {
    let mut bytes = [0; 256];

    let (len, _) = try!(udp.recv_from(&mut bytes));

    let zero_index = bytes.iter()
        .skip(4)
        .position(|&x| x == 0)
        .unwrap();

    let address = &bytes[4..4 + zero_index];

    let port = try!((&bytes[len - 2..]).read_u16::<LittleEndian>());

    // send the acknowledgement websocket message
    let map = select_protocol(address, port);
    sender.send_json(&map).map(|_| ())
}

#[inline]
fn start_threads(mut receiver: WsReceiver<WebSocketStream>, udp: &UdpSocket)
    -> Result<MpscReceiver<ReceiverStatus>> {
    let thread = thread::current();
    let thread_name = thread.name().unwrap_or("serenity.rs voice");

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    let udp_clone = try!(udp.try_clone());

    try!(ThreadBuilder::new()
        .name(format!("{} WS", thread_name))
        .spawn(move || {
            loop {
                let msg = receiver.recv_json(VoiceEvent::decode);

                if let Ok(msg) = msg {
                    let send = tx.send(ReceiverStatus::Websocket(msg));

                    if let Err(_why) = send {
                        return;
                    }
                } else {
                    break;
                }
            }
        }));

    try!(ThreadBuilder::new()
        .name(format!("{} UDP", thread_name))
        .spawn(move || {
            let mut buffer = [0; 512];

            loop {
                let (len, _) = udp_clone.recv_from(&mut buffer).unwrap();
                let req = tx_clone.send(ReceiverStatus::Udp(buffer[..len]
                    .iter()
                    .cloned()
                    .collect()));

                if let Err(_why) = req {
                    return;
                }
            }
        }));

    Ok(rx)
}
