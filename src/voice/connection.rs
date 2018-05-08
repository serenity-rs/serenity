use byteorder::{ByteOrder, LittleEndian, NetworkEndian, ReadBytesExt, WriteBytesExt};
use constants::VOICE_GATEWAY_VERSION;
use future_utils::{
    mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use futures::{
    future::{
        err,
        loop_fn,
        ok,
        result,
        Either,
        Future,
        Loop,
    },
    stream::{
        SplitSink,
        SplitStream,
    },
    sync::{
        mpsc::{
            self as future_mpsc,
            Receiver as FutureMpscReceiver,
            Sender as FutureMpscSender,
        },
        oneshot::{
            channel as oneshot_channel,
            Sender as OneShotSender,
        },
    },
    Sink,
    Stream,
};
use internal::prelude::*;
use internal::ws_ext::{
    message_to_json,
    ReceiverExt,
    SenderExt,
    WsClient,
};
use internal::Delay;
use model::event::{
    VoiceEvent,
    VoiceHello,
    VoiceReady,
};
use model::id::UserId;
use opus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    Decoder as OpusDecoder,
    Encoder as OpusEncoder,
    SoftClip,
};
use parking_lot::Mutex;
use rand::random;
use serde::Deserialize;
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use std::{
    collections::HashMap,
    io::{Error as IoError, Write},
    net::{SocketAddr, ToSocketAddrs},
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    sync::Arc,
    thread::{self, Builder as ThreadBuilder, JoinHandle},
    time::{Duration, Instant},
};
use super::{
    audio::{
        AudioReceiver,
        AudioType,
        DEFAULT_BITRATE,
        HEADER_LEN,
        LockedAudio,
        SAMPLE_RATE,
        SILENT_FRAME,
    },
    codec::{
        VoiceCodec,
        TxVoicePacket,
        RxVoicePacket,
    },
    connection_info::ConnectionInfo,
    payload,
    CRYPTO_MODE,
    VoiceError, 
};
use tokio_core::{
    net::{
        TcpStream,
        UdpCodec,
        UdpFramed,
        UdpSocket,
    },
    reactor::{Core, Handle, Remote},
};
use tokio_tungstenite::{connect_async, WebSocketStream};
use tungstenite::Message;
use url::Url;

enum ReceiverStatus {
    Udp(<VoiceCodec as UdpCodec>::In),
    Websocket(VoiceEvent),
}

struct ProgressingVoiceHandshake {
    connection_info: ConnectionInfo,
    hello: Option<VoiceHello>,
    ready: Option<VoiceReady>,
    // ws: WsClient,
}

impl ProgressingVoiceHandshake {
    fn finalize(self) -> Result<VoiceHandshake> {
        let ready = self.ready.ok_or(Error::Voice(VoiceError::ExpectedHandshake))?;
        let hello = self.hello.ok_or(Error::Voice(VoiceError::ExpectedHandshake))?;

        Ok(VoiceHandshake {
            connection_info: self.connection_info,
            ready,
            hello,
            // ws: self.ws,
        })
    }
}

struct VoiceHandshake {
    connection_info: ConnectionInfo,
    hello: VoiceHello,
    ready: VoiceReady,
    // ws: WsClient,
}

struct ListenerItems {
    close_sender: OneShotSender<()>,
    rx: Receiver<ReceiverStatus>,
    rx_pong: Receiver<Vec<u8>>,
}

#[allow(dead_code)]
pub struct Connection {
    connection_info: ConnectionInfo,
    destination: SocketAddr,
    last_heartbeat_nonce: Option<u64>,
    listener_items: ListenerItems,
    silence_frames: u8,
    soft_clip: SoftClip,
    speaking: bool,
    udp_keepalive_timer: Delay,
    udp_send: SplitSink<UdpFramed<VoiceCodec>>,
    ws_keepalive_timer: Delay,
    ws_send: SplitSink<WsClient>,
}

impl Connection {
    pub fn new(mut info: ConnectionInfo, handle: Handle)
            -> Box<Future<Item = Connection, Error = Error>> {

        // let mut core = Core::new().unwrap();
        // let handle = core.handle();

        let url = generate_url(&mut info.endpoint);
        let local_remote_ws = handle.remote().clone();
        let local_remote_listeners = handle.remote().clone();

        let done = result(url)
            // Build a (TLS'd) websocket.
            .and_then(move |url| connect_async(url, local_remote_ws).map_err(Error::from))
            // Our init of the handshake.
            .and_then(move |(ws, _)| {
                ws.send_json(&payload::build_identify(&info))
                    .map(|ws| (ws, info))
            })
            // The reply has TWO PARTS, which can come in any order.
            .and_then(|(ws, connection_info)| {
                loop_fn((ProgressingVoiceHandshake {connection_info, ready: None, hello: None}, ws),
                        |(mut state, ws)| {
                    ws.recv_json()
                        .and_then(move |(value_wrap, ws)| {

                            let value = match value_wrap {
                                Some(json_value) => json_value,
                                None => {return Ok(Loop::Continue((state, ws)));},
                            };

                            match VoiceEvent::deserialize(value)? {
                                VoiceEvent::Ready(r) => {
                                    state.ready = Some(r);
                                    if state.hello.is_some(){
                                        return Ok(Loop::Break((state, ws)));
                                    }
                                },
                                VoiceEvent::Hello(h) => {
                                    state.hello = Some(h);
                                    if state.ready.is_some() {
                                        return Ok(Loop::Break((state, ws)));
                                    }
                                },
                                other => {
                                    debug!("[Voice] Expected ready/hello; got: {:?}", other);

                                    return Err(Error::Voice(VoiceError::ExpectedHandshake));
                                },
                            }

                            Ok(Loop::Continue((state, ws)))
                        })
                })
            })
            // With the ws handshake, we need to open a UDP voice channel.
            // 
            .and_then(move |(state, ws)| {
                let handshake = state.finalize()?;

                if !has_valid_mode(&handshake.ready.modes) {
                    return Err(Error::Voice(VoiceError::VoiceModeUnavailable));
                }

                let destination = (&handshake.connection_info.endpoint[..], handshake.ready.port)
                    .to_socket_addrs()?
                    .next()
                    .ok_or(Error::Voice(VoiceError::HostnameResolve))?;

                // Important to note here: the length of the packet can be of either 4
                // or 70 bytes. If it is 4 bytes, then we need to send a 70-byte packet
                // to determine the IP.
                //
                // Past the initial 4 bytes, the packet _must_ be completely empty data.
                //
                // The returned packet will be a null-terminated string of the IP, and
                // the port encoded in LE in the last two bytes of the packet.

                // TODO: compute local socket addr as a lazy static
                let local = "0.0.0.0:0"
                    .to_socket_addrs()?
                    .next()
                    .ok_or(Error::Voice(VoiceError::HostnameResolve))?;

                let udp = UdpSocket::bind(&local, &handle)?;

                let mut bytes = Vec::with_capacity(70);
                bytes.write_u32::<NetworkEndian>(handshake.ready.ssrc)?;
                bytes.resize(70, 0u8);

                Ok(
                    udp.send_dgram(bytes, destination)
                        .and_then(|(udp, _)| udp.recv_dgram(vec![0u8; 256]))
                        .map_err(Error::from)
                        .and_then(move |(udp, data, len, _)| {
                            // Find the position in the bytes that contains the first byte of 0,
                            // indicating the "end of the address".
                            let index = data.iter()
                                .skip(4)
                                .position(|&x| x == 0)
                                .ok_or(Error::Voice(VoiceError::FindingByte))?;

                            let pos = 4 + index;
                            let addr = String::from_utf8_lossy(&data[4..pos]);
                            let port_pos = len - 2;
                            let port = (&data[port_pos..]).read_u16::<LittleEndian>()?;

                            Ok(ws.send_json(&payload::build_select_protocol(addr, port))
                                .map_err(Error::from)
                                .map(move |ws| {
                                    (handshake, ws, udp, destination)
                                }))
                    })
                )
            })
            .flatten()
            .flatten()
            .and_then(|(handshake, ws, udp, destination)| {
                Ok(encryption_key(ws)
                    .map(move |(key, ws)| {
                        (handshake, ws, udp, key, destination)
                    })
                )
            })
            .flatten()
            .and_then(move |(handshake, ws, udp, key, destination)| {
                let VoiceHandshake { connection_info, hello, ready } = handshake;
                let codec = VoiceCodec::new(destination, key, ready.ssrc)?;

                let (ws_send, ws_reader) = ws.split();
                let (udp_send, udp_reader) = udp.framed(codec).split();

                let listener_items = spawn_receive_handlers(ws_reader, udp_reader, &local_remote_listeners);

                info!("[Voice] Connected to: {}", &connection_info.endpoint);

                // Encode for Discord in Stereo, as required.
                let soft_clip = SoftClip::new(Channels::Stereo);

                // Per discord dev team's current recommendations:
                // (https://discordapp.com/developers/docs/topics/voice-connections#heartbeating)
                let temp_heartbeat = (hello.heartbeat_interval as f64 * 0.75) as u64;

                // Putting most of the timeout events onto the old-style tokio timer wheel
                // means we either need a HUGE wheel to avoid collision, or multiple wheels.
                //
                // Just put in some simple checks since we run the update loop every 20ms anyhow.
                Ok(Connection {
                    connection_info,
                    destination,
                    last_heartbeat_nonce: None,
                    listener_items,
                    silence_frames: 100,
                    soft_clip,
                    speaking: false,
                    udp_keepalive_timer: Delay::new(1000 * 60 * 4),
                    udp_send,
                    ws_keepalive_timer: Delay::new(temp_heartbeat),
                    ws_send,
                })
            });

        Box::new(done)
    }

    pub fn reconnect(self, handle: Handle) -> Box<Future<Item = Connection, Error = Error>> {
        // A few steps to this.
        //  * Unconditionally terminate the voice and udp connections.
        //  * Rebuild those connections (and listeners).
        //  * Send Resume, await Resumed.
        //  * If conneciton closed, start a new connection.

        // TODO.
        // Need to figure out the interaction with kick/ban etc.
        Box::new(err(Error::Voice(VoiceError::VoiceModeUnavailable)))
    }

    #[allow(unused_variables)]
    pub fn cycle(mut self,
                 // sources: &'static mut Vec<LockedAudio>,
                 sources: Arc<Mutex<Vec<LockedAudio>>>,
                 // receiver: &mut Option<Box<AudioReceiver>>,
                 receiver: Arc<Mutex<Option<Box<AudioReceiver>>>>,
                 bitrate: Bitrate)
                 -> Box<Future<Item = (), Error = Error>> {
        // Process events the listeners have batched out.
        // let client_receiver = receiver.as_mut();

        {
            let mut client_receiver = receiver.lock();
            let forward_events = client_receiver.is_some();

            while let Ok(status) = self.listener_items.rx.try_recv() {
                match status {
                    ReceiverStatus::Udp(packet) => {
                        if forward_events {
                            let RxVoicePacket {
                                is_stereo,
                                seq,
                                ssrc,
                                timestamp,
                                voice,
                            } = packet;

                            let len = if is_stereo { 960 * 2 } else { 960 };

                            client_receiver.as_mut().expect("Receiver is already 'Some'")
                                .voice_packet(ssrc, seq, timestamp, is_stereo, &voice[..len]);
                        }
                    },
                    ReceiverStatus::Websocket(VoiceEvent::Speaking(ev)) => {
                        if forward_events {
                            client_receiver.as_mut().expect("Receiver is already 'Some'")
                                .speaking_update(ev.ssrc, ev.user_id.0, ev.speaking);
                        }
                    },
                    ReceiverStatus::Websocket(VoiceEvent::HeartbeatAck(ev)) => {
                        match self.last_heartbeat_nonce {
                            Some(nonce) => {
                                if ev.nonce != nonce {
                                    warn!("[Voice] Heartbeat nonce mismatch! Expected {}, saw {}.", nonce, ev.nonce);
                                }

                                self.last_heartbeat_nonce = None;
                            },
                            None => {},
                        }
                    },
                    ReceiverStatus::Websocket(other) => {
                        info!("[Voice] Received other websocket data: {:?}", other);
                    },
                }
            }
        }

        let udp_now = Instant::now();
        let ws_now = udp_now.clone();

        // https://tools.ietf.org/html/rfc6455#section-5.5.3
        // "the endpoint MAY elect to send a Pong frame
        // for only the most recently processed Ping frame."
        let ws_ping = {
            let mut pong = None;

            while let Ok(data) = self.listener_items.rx_pong.try_recv() {
                pong = Some(data);
            }

            pong
        };

        // Send WS pong if need be.
        let prepped_ws = match ws_ping {
            Some(data) => Either::A(self.ws_send.send(Message::Pong(data))),
            None => Either::B(ok(self.ws_send))
        };

        let out = prepped_ws
            .map_err(Error::from)
            .and_then(move |ws_send| {
                // Send the voice websocket keepalive if it's time
                match self.ws_keepalive_timer.is_elapsed(ws_now) {
                    true => {
                        let nonce = random::<u64>();
                        self.last_heartbeat_nonce = Some(nonce);

                        self.ws_keepalive_timer.reset();
                        Either::A(
                            self.ws_send.send_json(&payload::build_heartbeat(nonce))
                                .map_err(Error::from)
                        )
                    },
                    false => {
                        Either::B(ok(self.ws_send))
                    },
                }
            })
            .and_then(move |ws_send| {
                // Send UDP keepalive if it's time
                match self.udp_keepalive_timer.is_elapsed(udp_now) {
                    true => {
                        self.udp_keepalive_timer.reset();
                        Either::A(
                            self.udp_send.send(TxVoicePacket::KeepAlive)
                                .map_err(Error::from)
                        )
                    },
                    false => Either::B(ok(self.udp_send)),
                }
            })
            .and_then(move |udp_send| {
                let mut buffer = [0i16; 960 * 2];
                let mut mix_buffer = [0f32; 960 * 2];
                let mut len = 0;

                // TODO: Could we parallelise this across futures?
                // It's multiple I/O operations, potentially.

                // Walk over all the audio files, removing those which have finished.
                let mut i = 0;

                let sources = sources.lock();

                while i < sources.len() {
                    let mut finished = false;

                    let aud_lock = (&sources[i]).clone();
                    let mut aud = aud_lock.lock();

                    let vol = aud.volume;
                    let skip = !aud.playing;

                    {
                        let stream = &mut aud.source;

                        if skip {
                            i += 1;

                            continue;
                        }

                        let temp_len = match stream.get_type() {
                            AudioType::Opus => match stream.decode_and_add_opus_frame(&mut mix_buffer, vol) {
                                Some(len) => len,
                                None => 0,
                            },
                            AudioType::Pcm => {
                                let buffer_len = 960 * 2;

                                match stream.read_pcm_frame(&mut buffer[..buffer_len]) {
                                    Some(len) => len,
                                    None => 0,
                                }
                            },
                        };

                        // May need to force interleave/copy.
                        combine_audio(&buffer, &mut mix_buffer, vol);

                        len = len.max(temp_len);
                        i += if temp_len > 0 {
                            1
                        } else {
                            sources.remove(i);
                            finished = true;

                            0
                        };
                    }

                    aud.finished = finished;

                    if !finished {
                        aud.step_frame();
                    }
                };

                let tx_packet = if len == 0 {
                    if self.silence_frames > 0 {
                        // Per official guidelines, send 5x silence BEFORE we stop speaking.
                        self.silence_frames -= 1;
                        Some(TxVoicePacket::Silence)
                    } else {
                        // Okay, NOW we stop speaking.
                        None
                    }
                } else {
                    self.silence_frames = 5;

                    self.soft_clip.apply(&mut mix_buffer);
                    Some(TxVoicePacket::Audio(&mix_buffer[..len], bitrate))
                };


                self.set_speaking(tx_packet.is_some())
                    .and_then(|ws_send| {
                        self.udp_keepalive_timer.reset();
                        match tx_packet {
                            Some(packet) => Either::A(
                                self.udp_send.send(packet)
                                    .map_err(Error::from)
                                    .map(|_| ())
                            ),
                            None => Either::B(ok(())),
                        }
                    })
            });

        Box::new(out)
    }

    fn set_speaking(&mut self, speaking: bool) -> Box<Future<Item = SplitSink<WsClient>, Error = Error>> {
        let out = match self.speaking == speaking {
            true => Either::A(ok(self.ws_send)),
            false => {
                self.speaking = speaking;
                self.ws_keepalive_timer.reset();
                Either::B(
                    self.ws_send.send_json(&payload::build_speaking(speaking))
                        .map_err(Error::from)
                )
            },
        };

        Box::new(out)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.listener_items.close_sender.send(());

        info!("[Voice] Disconnected");
    }
}

#[inline]
fn combine_audio(
    raw_buffer: &[i16; 1920],
    float_buffer: &mut [f32; 1920],
    volume: f32,
) {
    for i in 0..1920 {
        let sample = (raw_buffer[i] as f32) / 32768.0;

        float_buffer[i] = float_buffer[i] + sample * volume;
    }
}

fn generate_url(endpoint: &mut String) -> Result<Url> {
    if endpoint.ends_with(":80") {
        let len = endpoint.len();

        endpoint.truncate(len - 3);
    }

    Url::parse(&format!("wss://{}/?v={}", endpoint, VOICE_GATEWAY_VERSION))
        .or(Err(Error::Voice(VoiceError::EndpointUrl)))
}

#[inline]
fn encryption_key(ws: WsClient) -> Box<Future<Item=(Key, WsClient), Error=Error>> {
    let out = loop_fn(ws, |ws| {
        ws.recv_json()
            .and_then(|(value_wrap, ws)| {
                let value = match value_wrap {
                    Some(json_value) => json_value,
                    None => {return Ok(Loop::Continue(ws));},
                };

                match VoiceEvent::deserialize(value)? {
                    VoiceEvent::SessionDescription(desc) => {
                        if desc.mode != CRYPTO_MODE {
                            return Err(Error::Voice(VoiceError::VoiceModeInvalid));
                        }

                        let key = Key::from_slice(&desc.secret_key)
                            .ok_or(Error::Voice(VoiceError::KeyGen))?;

                        return Ok(Loop::Break((key, ws)))
                    },
                    VoiceEvent::Unknown(op, value) => {
                        debug!(
                            "[Voice] Expected ready for key; got: op{}/v{:?}",
                            op.num(),
                            value
                        );
                    },
                    _ => {},
                }

                Ok(Loop::Continue(ws))
            })
    });

    Box::new(out)
}

#[inline]
fn has_valid_mode<T, It> (modes: It) -> bool
where T: for<'a> PartialEq<&'a str>,
      It : IntoIterator<Item=T>
{
    modes.into_iter().any(|s| s == CRYPTO_MODE)
}

#[inline]
fn spawn_receive_handlers(ws: SplitStream<WsClient>, udp: SplitStream<UdpFramed<VoiceCodec>>, context: &Remote) -> ListenerItems {
    let (close_sender, close_reader) = oneshot_channel::<()>();

    let close_reader = close_reader;
    let close_reader1 = close_reader.shared();
    let close_reader2 = close_reader1.clone();

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    let (tx_pong, rx_pong) = mpsc::channel();

    context.spawn(move |_|
        ws.map_err(Error::from)
            .until(close_reader1.map(|v| *v))
            .for_each(|message| {
                message_to_json(message, tx_pong).and_then(
                    |maybe_value| match maybe_value {
                        Some(value) => match VoiceEvent::deserialize(value) {
                            Ok(msg) => tx.send(ReceiverStatus::Websocket(msg))
                                .map_err(|e| Error::FutureMpsc("WS event receiver hung up.")),
                            Err(why) => {
                                warn!("Error deserializing voice event: {:?}", why);

                                Err(Error::Json(why))
                            },
                        },
                        None => Ok(()),
                    })
            })
            .map_err(|e| {
                warn!("[voice] {}", e);

                ()
            })
    );

    context.spawn(move |_|
        udp.map_err(Error::from)
            .until(close_reader2.map(|v| *v))
            .for_each(|voice_frame|
                tx_clone.send(ReceiverStatus::Udp(voice_frame))
                    .map_err(|e| Error::FutureMpsc("UDP event receiver hung up."))
            )
            .map_err(|e| {
                warn!("[voice] {}", e);

                ()
            })
    );

    ListenerItems {
        close_sender,
        rx,
        rx_pong,
    }
}
