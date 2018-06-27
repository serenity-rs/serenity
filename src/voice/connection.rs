use byteorder::{LittleEndian, NetworkEndian, ReadBytesExt, WriteBytesExt};
use constants::VOICE_GATEWAY_VERSION;
use future_utils::StreamExt;
use futures::{
    future::{
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
    time::{Duration, Instant},
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
use tokio::{
    self,
    net::{
        UdpFramed,
        UdpSocket,
    },
    util::FutureExt,
};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;

enum ReceiverStatus {
    Udp(RxVoicePacket),
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
    close_sender_ws: Option<OneShotSender<()>>,
    close_sender_udp: Option<OneShotSender<()>>,
    rx: Receiver<ReceiverStatus>,
    rx_pong: Receiver<Vec<u8>>,
}

impl Drop for ListenerItems {
    fn drop(&mut self) {
        let _ = mem::replace(&mut self.close_sender_ws, None)
            .expect("Formality to appease the borrow-lord.")
            .send(());
        let _ = mem::replace(&mut self.close_sender_udp, None)
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
    pub fn new(mut info: ConnectionInfo) -> Box<Future<Item = Self, Error = Error> + Send> {
        let url = generate_url(&mut info.endpoint);

        Box::new(result(url)
            // Build a (TLS'd) websocket.
            .and_then(move |url|
                connect_async(url).map_err(Error::from)
            )
            // Our init of the handshake.
            .and_then(move |(ws, _)|
                ws.send_json(&payload::build_identify(&info))
                    .map(|ws| (ws, info))
            )
            // The reply has TWO PARTS, which can come in any order.
            .and_then(|(ws, connection_info)| {
                let handshake = ProgressingVoiceHandshake {
                    connection_info,
                    ready: None,
                    hello: None
                };

                loop_fn((handshake, ws), |(mut handshake, ws)| {
                    ws.recv_json()
                        .and_then(move |(value_wrap, ws)| {

                            let value = match value_wrap {
                                Some(json_value) => json_value,
                                None => {return Ok(Loop::Continue((handshake, ws)));},
                            };

                            match VoiceEvent::deserialize(value)? {
                                VoiceEvent::Ready(r) => {
                                    handshake.ready = Some(r);
                                    if handshake.hello.is_some(){
                                        return Ok(Loop::Break((handshake, ws)));
                                    }
                                },
                                VoiceEvent::Hello(h) => {
                                    handshake.hello = Some(h);
                                    if handshake.ready.is_some() {
                                        return Ok(Loop::Break((handshake, ws)));
                                    }
                                },
                                other => {
                                    debug!("[Voice] Expected ready/hello; got: {:?}", other);

                                    return Err(Error::Voice(VoiceError::ExpectedHandshake));
                                },
                            }

                            Ok(Loop::Continue((handshake, ws)))
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

                let udp = UdpSocket::bind(&local)?;

                let mut bytes = Vec::with_capacity(70);
                bytes.write_u32::<NetworkEndian>(handshake.ready.ssrc)?;
                bytes.resize(70, 0u8);

                Ok(
                    udp.send_dgram(bytes, &destination)
                        .map_err(Error::from)
                        .and_then(|(udp, _)|
                            udp.recv_dgram(vec![0u8; 256])
                                .map_err(Error::from)
                                .deadline(Instant::now() + Duration::from_secs(4))
                                .map_err(|_|
                                    Error::Voice(VoiceError::ExpectedHandshake)
                                )
                        )
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
                                .map(move |ws|
                                    (handshake, ws, udp, destination)
                                ))
                    })
                )
            })
            .flatten()
            .flatten()
            .and_then(|(handshake, ws, udp, destination)|
                encryption_key(ws)
                    .map(move |(key, ws)| {
                        (handshake, ws, udp, key, destination)
                    })
            )
            .and_then(move |(handshake, ws, udp, key, destination)| {
                let VoiceHandshake { connection_info, hello, ready } = handshake;
                let codec = VoiceCodec::new(key, ready.ssrc)?;

                let (ws_send, ws_reader) = ws.split();
                let (udp_send, udp_reader) = UdpFramed::new(udp, codec).split();

                let ws_send = Some(ws_send);
                let udp_send = Some(udp_send);

                let listener_items = spawn_receive_handlers(ws_reader, Some(udp_reader));

                info!("[Voice] Connected to: {}", &connection_info.endpoint);

                // Encode for Discord in Stereo, as required.
                let soft_clip = SoftClip::new(Channels::Stereo);

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
                    ws_keepalive_timer: read_heartbeat_time(hello),
                    ws_send,
                })
            })
        )
    }

    fn ws(&mut self) -> SplitSink<WsClient> {
        self.ws_send.take()
            .expect("[voice] Failed to get websocket...")
    }

    fn restore_ws(mut self, ws: SplitSink<WsClient>) -> Self {
        self.ws_send = Some(ws);
        self
    }

    fn udp(&mut self) -> SplitSink<UdpFramed<VoiceCodec>> {
        self.udp_send.take()
            .expect("[voice] Failed to get udp...")
    }

    fn restore_udp(mut self, udp: SplitSink<UdpFramed<VoiceCodec>>) -> Self {
        self.udp_send = Some(udp);
        self
    }

    fn send_voice_packet(mut self, packet: TxVoicePacket) -> Box<Future<Item = Self, Error = Error> + Send> {
        Box::new(self.udp().send((packet, self.destination))
            .map_err(Error::from)
            .map(|udp|
                self.restore_udp(udp)
            )
        )
    }

    pub fn reconnect(mut self) -> Box<Future<Item = Self, Error = Error> + Send> {
        // A few steps to this.
        //  * Unconditionally terminate the voice ws connection, by dropping it.
        //  * Rebuild that connection (and listener).
        //  * Send Resume, await Resumed and Hello.
        //  * If connection closed/error, start a new connection.

        let url = generate_url(&mut self.connection_info.endpoint);
        let backup_info = self.connection_info.clone();
        let saved_udp = mem::replace(&mut self.listener_items.close_sender_udp, None);

        let _ = mem::replace(&mut self.listener_items.close_sender_ws, None)
            .expect("Formality to appease the borrow-lord.")
            .send(());
        let _ = self.ws();

        Box::new(result(url)
            .and_then(move |url| connect_async(url).map_err(Error::from))
            .and_then(move |(ws, _)| ws
                .send_json(&payload::build_resume(&self.connection_info))
                .map(|ws| (ws, self)))
            .and_then(|(ws, conn)|
                expect_resumed(ws)
                    .map(|ws| (ws, conn)))
            .and_then(|(ws, mut conn)|
                expect_hello(ws)
                    .map(|(ws, delay)| {
                        conn.ws_keepalive_timer = delay;
                        (ws, conn)
                    }))
            .and_then(move |(ws, mut conn)| {
                let (ws_send, ws_reader) = ws.split();

                let mut listener_items = spawn_receive_handlers(ws_reader, None);
                let saved_udp = saved_udp;

                // Swap the old udp handler over to the new struct, then swap the new struct in.
                // Note that we kept the udp sender untouched.
                listener_items.close_sender_udp = saved_udp;
                conn.listener_items = listener_items;

                Ok(conn.restore_ws(ws_send))
            })
            .or_else(move |_| Connection::new(backup_info))
        )
    }

    #[allow(unused_variables)]
    pub(crate) fn cycle(mut self, now: Instant, mut state: Box<TaskState>)
        -> Box<Future<Item = Box<TaskState>, Error = (Box<TaskState>, Error)> + Send>
    {
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

        // From this point, we need to be VERY careful with the
        // state object, so it MUST be passed along and handled
        // for each error block.

        Box::new(prepped_ws
            .then(move |result| match result {
                Ok(conn) => Ok((state, conn)),
                Err(e) => Err((state, e.into())),
            })
            .and_then(move |(state, mut conn)| {
                // Send the voice websocket keepalive if it's time
                match conn.ws_keepalive_timer.is_elapsed(now) {
                    true => {
                        let nonce = random::<u64>();
                        conn.last_heartbeat_nonce = Some(nonce);

                        conn.ws_keepalive_timer.reset();
                        Either::A(
                            conn.ws().send_json(&payload::build_heartbeat(nonce))
                                .then(move |result| match result {
                                    Ok(ws) => Ok((state, conn.restore_ws(ws))),
                                    Err(e) => Err((state, e.into())),
                                })
                        )
                    },
                    false => {
                        Either::B(ok((state, conn)))
                    },
                }
            })
            .and_then(move |(state, mut conn)| {
                // Send UDP keepalive if it's time
                match conn.udp_keepalive_timer.is_elapsed(now) {
                    true => {
                        conn.udp_keepalive_timer.reset();
                        Either::A(
                            conn.send_voice_packet(TxVoicePacket::KeepAlive)
                                .then(move |result| match result {
                                    Ok(conn) => Ok((state, conn)),
                                    Err(e) => Err((state, e.into())),
                                })
                        )
                    },
                    false => Either::B(ok((state, conn))),
                }
            })
            .and_then(move |(mut state, mut conn)| {
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

                            let source_stereo = stream.is_stereo();
                            let size_mod = if source_stereo { 2 } else { 1 };
                            let len_mod = if source_stereo { 1 } else { 2 };

                            let temp_len = match stream.get_type() {
                                AudioType::Opus => match stream.decode_and_add_opus_frame(&mut mix_buffer, vol) {
                                    Some(len) => len,
                                    None => 0,
                                },
                                AudioType::Pcm => {
                                    let buffer_len = 960 * size_mod;

                                    let len = match stream.read_pcm_frame(&mut buffer[..buffer_len]) {
                                        Some(len) => len * len_mod,
                                        None => 0,
                                    };

                                    if len > 0{
                                        combine_audio(&buffer, &mut mix_buffer, vol, source_stereo);
                                    }

                                    len
                                },
                            };

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
                    .then(move |result| match result {
                        Ok(conn) => Ok((state, conn)),
                        Err(e) => Err((state, e.into())),
                    })
                    .and_then(move |(state, mut conn)| {
                        conn.udp_keepalive_timer.reset();
                        match tx_packet {
                            Some(packet) => Either::A(
                                conn.send_voice_packet(packet)
                                    .then(move |result| match result {
                                        Ok(conn) => Ok((state, conn)),
                                        Err(e) => Err((state, e.into())),
                                    })
                            ),
                            None => Either::B(ok((state, conn))),
                        }
                    })
                    .map(move |(mut state, conn)| {
                        state.cycle_error = false;
                        state.restore_conn(conn);
                        state
                    })
            })
        )
    }

    fn set_speaking(mut self, speaking: bool) -> Box<Future<Item = Connection, Error = Error> + Send> {
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
    source_stereo: bool,
) {
    for i in 0..1920 {
        let sample_index = if source_stereo { i } else { i/2 };
        let sample = (raw_buffer[sample_index] as f32) / 32768.0;

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

// Maybe TODO: autogenerate these with a macro or something.

#[inline]
fn encryption_key(ws: WsClient) -> Box<Future<Item=(Key, WsClient), Error=Error> + Send> {
    Box::new(loop_fn(ws, |ws| {
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
    }))
}

#[inline]
fn expect_resumed(ws: WsClient) -> Box<Future<Item = WsClient, Error = Error> + Send> {
    Box::new(loop_fn(ws, |ws| {
        ws.recv_json()
            .and_then(|(value_wrap, ws)| {
                let value = match value_wrap {
                    Some(json_value) => json_value,
                    None => {return Ok(Loop::Continue(ws));},
                };

                match VoiceEvent::deserialize(value)? {
                    VoiceEvent::Resumed => {
                        return Ok(Loop::Break(ws))
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
    }))
}

#[inline]
fn expect_hello(ws: WsClient) -> Box<Future<Item = (WsClient, Delay), Error = Error> + Send> {
    Box::new(loop_fn(ws, |ws| {
        ws.recv_json()
            .and_then(|(value_wrap, ws)| {
                let value = match value_wrap {
                    Some(json_value) => json_value,
                    None => {return Ok(Loop::Continue(ws));},
                };

                match VoiceEvent::deserialize(value)? {
                    VoiceEvent::Hello(h) => {
                        return Ok(Loop::Break((ws, read_heartbeat_time(h))));
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
    }))
}

#[inline]
fn read_heartbeat_time(hello: VoiceHello) -> Delay {
    // Per discord dev team's current recommendations:
    // (https://discordapp.com/developers/docs/topics/voice-connections#heartbeating)
    // Multiply by 0.75.
    Delay::new((hello.heartbeat_interval as f64 * 0.75) as u64)
}


#[inline]
fn has_valid_mode<T, It> (modes: It) -> bool
where T: for<'a> PartialEq<&'a str>,
      It : IntoIterator<Item=T>
{
    modes.into_iter().any(|s| s == CRYPTO_MODE)
}

// The UDP param is only dropped in the event of a reconnection (where the full UDP pipeline exists and
// should still be alive)
#[inline]
fn spawn_receive_handlers(ws: SplitStream<WsClient>, udp_maybe: Option<SplitStream<UdpFramed<VoiceCodec>>>) -> ListenerItems {
    let (close_sender_ws, close_reader_ws) = oneshot_channel::<()>();

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    let (tx_pong, rx_pong) = mpsc::channel();

    let tx_pong_shared = repeat(Arc::new(Mutex::new(tx_pong)));

    tokio::spawn(ws.map_err(Error::from)
        .zip(tx_pong_shared)
        .until(close_reader_ws)
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

    let close_sender_udp = if let Some(udp) = udp_maybe {
        let (close_sender_udp, close_reader_udp) = oneshot_channel::<()>();

        tokio::spawn(udp.map_err(Error::from)
            .until(close_reader_udp)
            .for_each(move |(voice_frame, _src_addr)|
                tx_clone.send(ReceiverStatus::Udp(voice_frame))
                    .map_err(|_| Error::FutureMpsc("UDP event receiver hung up."))
            )
            .map_err(|e| {
                warn!("[voice] {}", e);

                ()
            })
        );

        Some(close_sender_udp)
    } else {
        None
    };

    ListenerItems {
        close_sender_ws: Some(close_sender_ws),
        close_sender_udp,
        rx,
        rx_pong,
    }
}
