use byteorder::{LittleEndian, NetworkEndian, ReadBytesExt, WriteBytesExt};
use constants::VOICE_GATEWAY_VERSION;
use future_utils::StreamExt;
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
        repeat,
        SplitSink,
        SplitStream,
    },
    sync::oneshot::{
        channel as oneshot_channel,
        Sender as OneShotSender,
    },
    Sink,
    Stream,
};
use internal::{
    long_lock::LongLock,
    prelude::*,
    ws_ext::{
        message_to_json,
        ReceiverExt,
        SenderExt,
        WsClient,
    },
    Delay
};
use model::event::{
    VoiceEvent,
    VoiceHello,
    VoiceReady,
};
use opus::{
    Channels,
    SoftClip,
};
use parking_lot::Mutex;
use rand::random;
use serde::Deserialize;
use sodiumoxide::crypto::secretbox::Key;
use std::{
    mem,
    net::{SocketAddr, ToSocketAddrs},
    sync::mpsc::{self, Receiver},
    sync::Arc,
    time::Instant,
};
use super::{
    audio::{
        AudioType,
    },
    codec::{
        VoiceCodec,
        TxVoicePacket,
        RxVoicePacket,
    },
    connection_info::ConnectionInfo,
    payload,
    threading::TaskState,
    VoiceError, 
    CRYPTO_MODE,
};
use tokio_core::{
    net::{
        UdpCodec,
        UdpFramed,
        UdpSocket,
    },
    reactor::{Handle, Remote},
};
use tokio_tungstenite::connect_async;
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
    close_sender: Option<OneShotSender<()>>,
    rx: Receiver<ReceiverStatus>,
    rx_pong: Receiver<Vec<u8>>,
}

impl Drop for ListenerItems {
    fn drop(&mut self) {
        let _ = mem::replace(&mut self.close_sender, None)
            .expect("Formality to appease the borrow-lord.")
            .send(());

        info!("[Voice] Disconnected");
    }
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
    udp_send: Option<SplitSink<UdpFramed<VoiceCodec>>>,
    ws_keepalive_timer: Delay,
    ws_send: Option<SplitSink<WsClient>>,
}

impl Connection {
    pub fn new(mut info: ConnectionInfo, handle: Handle)
            -> impl Future<Item = Connection, Error = Error> {

        // let mut core = Core::new().unwrap();
        // let handle = core.handle();

        let url = generate_url(&mut info.endpoint);
        let local_remote_ws = handle.remote().clone();
        let local_remote_listeners = handle.remote().clone();

        result(url)
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

                let ws_send = Some(ws_send);
                let udp_send = Some(udp_send);

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
            })
    }

    fn ws(&mut self) -> SplitSink<WsClient> {
        mem::replace(&mut self.ws_send, None)
            .expect("[voice] Failed to get websocket...")
    }

    fn restore_ws(mut self, ws: SplitSink<WsClient>) -> Self {
        self.ws_send = Some(ws);
        self
    }

    fn udp(&mut self) -> SplitSink<UdpFramed<VoiceCodec>> {
        mem::replace(&mut self.udp_send, None)
            .expect("[voice] Failed to get udp...")
    }

    fn restore_udp(mut self, udp: SplitSink<UdpFramed<VoiceCodec>>) -> Self {
        self.udp_send = Some(udp);
        self
    }

    pub fn reconnect(self, handle: Handle) -> impl Future<Item = Connection, Error = Error> {
        // A few steps to this.
        //  * Unconditionally terminate the voice and udp connections.
        //  * Rebuild those connections (and listeners).
        //  * Send Resume, await Resumed.
        //  * If conneciton closed, start a new connection.

        // TODO.
        // Need to figure out the interaction with kick/ban etc.
        err(Error::Voice(VoiceError::VoiceModeUnavailable))
    }

    #[allow(unused_variables)]
    pub(crate) fn cycle(mut self, now: Instant, mut state: LongLock<TaskState>)
            -> impl Future<Item = (), Error = Error> {

    // On success, this is unset.
    state.cycle_error = true;

        {
            // Process events the listeners have batched out.
            let client_receiver = &mut state.receiver.as_mut();
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
            Some(data) => Either::A(
                self.ws().send(Message::Pong(data))
                    .map(move |ws| self.restore_ws(ws))
            ),
            None => Either::B(
                ok(self)
            )
        };

        prepped_ws
            .map_err(Error::from)
            .and_then(move |mut conn| {
                // Send the voice websocket keepalive if it's time
                match conn.ws_keepalive_timer.is_elapsed(now) {
                    true => {
                        let nonce = random::<u64>();
                        conn.last_heartbeat_nonce = Some(nonce);

                        conn.ws_keepalive_timer.reset();
                        Either::A(
                            conn.ws().send_json(&payload::build_heartbeat(nonce))
                                .map_err(Error::from)
                                .map(move |ws| conn.restore_ws(ws))
                        )
                    },
                    false => {
                        Either::B(ok(conn))
                    },
                }
            })
            .and_then(move |mut conn| {
                // Send UDP keepalive if it's time
                match conn.udp_keepalive_timer.is_elapsed(now) {
                    true => {
                        conn.udp_keepalive_timer.reset();
                        Either::A(
                            conn.udp().send(TxVoicePacket::KeepAlive)
                                .map_err(Error::from)
                                .map(move |udp| conn.restore_udp(udp))
                        )
                    },
                    false => Either::B(ok(conn)),
                }
            })
            .and_then(move |mut conn| {
                let mut buffer = [0i16; 960 * 2];
                let mut mix_buffer = vec![0f32; 960 * 2];
                let mut len = 0;

                // TODO: Could we parallelise this across futures?
                // It's multiple I/O operations, potentially.
                {
                    // Walk over all the audio files, removing those which have finished.
                    let mut i = 0;

                    let sources = &mut state.senders;

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
                }

                let tx_packet = if len == 0 {
                    if conn.silence_frames > 0 {
                        // Per official guidelines, send 5x silence BEFORE we stop speaking.
                        conn.silence_frames -= 1;
                        Some(TxVoicePacket::Silence)
                    } else {
                        // Okay, NOW we stop speaking.
                        None
                    }
                } else {
                    conn.silence_frames = 5;

                    conn.soft_clip.apply(&mut mix_buffer);
                    Some(TxVoicePacket::Audio(mix_buffer, len, state.bitrate))
                };


                conn.set_speaking(tx_packet.is_some())
                    .and_then(move |mut conn| {
                        conn.udp_keepalive_timer.reset();
                        match tx_packet {
                            Some(packet) => Either::A(
                                conn.udp().send(packet)
                                    .map_err(Error::from)
                                    .map(|udp|
                                        conn.restore_udp(udp)
                                    )
                            ),
                            None => Either::B(ok(conn)),
                        }
                    })
                    .map(move |conn| {
                        // MutexGuard::map(state, |a| a.restore_conn(conn));
                        // IMPORTANT
                        state.cycle_error = false;
                        state.restore_conn(conn);
                        ()
                    })
            })
    }

    fn set_speaking(mut self, speaking: bool) -> Box<Future<Item = Connection, Error = Error>> {
        let out = match self.speaking == speaking {
            true => Either::A(ok(self)),
            false => {
                self.speaking = speaking;
                self.ws_keepalive_timer.reset();
                Either::B(
                    self.ws().send_json(&payload::build_speaking(speaking))
                        .map_err(Error::from)
                        .map(|ws| self.restore_ws(ws))
                )
            },
        };

        Box::new(out)
    }
}

#[inline]
fn combine_audio(
    raw_buffer: &[i16; 1920],
    float_buffer: &mut Vec<f32>,
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
fn encryption_key(ws: WsClient) -> impl Future<Item=(Key, WsClient), Error=Error> {
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

    let tx_pong_shared = repeat(Arc::new(Mutex::new(tx_pong)));

    context.spawn(move |_|
        ws.map_err(Error::from)
            .zip(tx_pong_shared)
            .until(close_reader1.map(|v| *v))
            .for_each(move |(message, tx_pong_lock)| {
                message_to_json(message, tx_pong_lock).and_then(
                    |maybe_value| match maybe_value {
                        Some(value) => match VoiceEvent::deserialize(value) {
                            Ok(msg) => tx.send(ReceiverStatus::Websocket(msg))
                                .map_err(|_| Error::FutureMpsc("WS event receiver hung up.")),
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
            .for_each(move |voice_frame|
                tx_clone.send(ReceiverStatus::Udp(voice_frame))
                    .map_err(|_| Error::FutureMpsc("UDP event receiver hung up."))
            )
            .map_err(|e| {
                warn!("[voice] {}", e);

                ()
            })
    );

    ListenerItems {
        close_sender: Some(close_sender),
        rx,
        rx_pong,
    }
}
