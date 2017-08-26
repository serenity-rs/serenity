use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use opus::{Application as CodingMode, Channels, Decoder as OpusDecoder, Encoder as OpusEncoder,
           packet as opus_packet};
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, Builder as ThreadBuilder, JoinHandle};
use std::time::Duration;
use super::audio::{AudioReceiver, AudioSource, AudioType, HEADER_LEN, SAMPLE_RATE};
use super::connection_info::ConnectionInfo;
use super::{CRYPTO_MODE, VoiceError, payload};
use websocket::client::Url as WebsocketUrl;
use websocket::sync::client::ClientBuilder;
use websocket::sync::stream::{AsTcpStream, TcpStream, TlsStream};
use websocket::sync::Client as WsClient;
use internal::prelude::*;
use internal::ws_impl::{ReceiverExt, SenderExt};
use internal::Timer;
use model::event::VoiceEvent;
use model::UserId;

type Client = WsClient<TlsStream<TcpStream>>;

enum ReceiverStatus {
    Udp(Vec<u8>),
    Websocket(VoiceEvent),
}

#[allow(dead_code)]
struct ThreadItems {
    rx: MpscReceiver<ReceiverStatus>,
    udp_close_sender: MpscSender<i32>,
    udp_thread: JoinHandle<()>,
    ws_close_sender: MpscSender<i32>,
    ws_thread: JoinHandle<()>,
}

#[allow(dead_code)]
pub struct Connection {
    audio_timer: Timer,
    client: Arc<Mutex<Client>>,
    decoder_map: HashMap<(u32, Channels), OpusDecoder>,
    destination: SocketAddr,
    encoder: OpusEncoder,
    encoder_stereo: bool,
    keepalive_timer: Timer,
    key: Key,
    sequence: u16,
    silence_frames: u8,
    speaking: bool,
    ssrc: u32,
    thread_items: ThreadItems,
    timestamp: u32,
    udp: UdpSocket,
    user_id: UserId,
}

impl Connection {
    pub fn new(mut info: ConnectionInfo) -> Result<Connection> {
        let url = generate_url(&mut info.endpoint)?;

        let mut client = ClientBuilder::from_url(&url).connect_secure(None)?;

        client.send_json(&payload::build_identify(&info))?;

        let hello = loop {
            match client.recv_json(VoiceEvent::decode)? {
                VoiceEvent::Hello(received_hello) => {
                    break received_hello;
                },
                VoiceEvent::Heartbeat(_) => continue,
                other => {
                    debug!("[Voice] Expected hello/heartbeat; got: {:?}", other);

                    return Err(Error::Voice(VoiceError::ExpectedHandshake));
                },
            }
        };

        if !has_valid_mode(&hello.modes) {
            return Err(Error::Voice(VoiceError::VoiceModeUnavailable));
        }

        let destination = (&info.endpoint[..], hello.port)
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
        let udp = UdpSocket::bind("0.0.0.0:0")?;

        {
            let mut bytes = [0; 70];

            (&mut bytes[..]).write_u32::<BigEndian>(hello.ssrc)?;
            udp.send_to(&bytes, destination)?;

            let mut bytes = [0; 256];
            let (len, _addr) = udp.recv_from(&mut bytes)?;

            // Find the position in the bytes that contains the first byte of 0,
            // indicating the "end of the address".
            let index = bytes.iter().skip(4).position(|&x| x == 0).ok_or(
                Error::Voice(
                    VoiceError::FindingByte,
                ),
            )?;

            let pos = 4 + index;
            let addr = String::from_utf8_lossy(&bytes[4..pos]);
            let port_pos = len - 2;
            let port = (&bytes[port_pos..]).read_u16::<LittleEndian>()?;

            client.send_json(
                &payload::build_select_protocol(addr, port),
            )?;
        }

        let key = encryption_key(&mut client)?;

        let _ = client.stream_ref().as_tcp().set_read_timeout(
            Some(Duration::from_millis(25)),
        );

        let mutexed_client = Arc::new(Mutex::new(client));
        let thread_items = start_threads(mutexed_client.clone(), &udp)?;

        info!("[Voice] Connected to: {}", info.endpoint);

        let encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Mono, CodingMode::Audio)?;

        Ok(Connection {
            audio_timer: Timer::new(1000 * 60 * 4),
            client: mutexed_client,
            decoder_map: HashMap::new(),
            destination: destination,
            encoder: encoder,
            encoder_stereo: false,
            key: key,
            keepalive_timer: Timer::new(hello.heartbeat_interval),
            udp: udp,
            sequence: 0,
            silence_frames: 0,
            speaking: false,
            ssrc: hello.ssrc,
            thread_items: thread_items,
            timestamp: 0,
            user_id: info.user_id,
        })
    }

    #[allow(unused_variables)]
    pub fn cycle(&mut self,
                 source: &mut Option<Box<AudioSource>>,
                 receiver: &mut Option<Box<AudioReceiver>>,
                 audio_timer: &mut Timer)
                 -> Result<()> {
        let mut buffer = [0i16; 960 * 2];
        let mut packet = [0u8; 512];
        let mut nonce = secretbox::Nonce([0; 24]);

        if let Some(receiver) = receiver.as_mut() {
            while let Ok(status) = self.thread_items.rx.try_recv() {
                match status {
                    ReceiverStatus::Udp(packet) => {
                        let mut handle = &packet[2..];
                        let seq = handle.read_u16::<BigEndian>()?;
                        let timestamp = handle.read_u32::<BigEndian>()?;
                        let ssrc = handle.read_u32::<BigEndian>()?;

                        nonce.0[..HEADER_LEN].clone_from_slice(
                            &packet[..HEADER_LEN],
                        );

                        if let Ok(decrypted) = secretbox::open(
                            &packet[HEADER_LEN..],
                            &nonce,
                            &self.key,
                        ) {
                            let channels = opus_packet::get_nb_channels(&decrypted)?;

                            let entry =
                                self.decoder_map.entry((ssrc, channels)).or_insert_with(|| {
                                    OpusDecoder::new(SAMPLE_RATE, channels).unwrap()
                                });

                            let len = entry.decode(&decrypted, &mut buffer, false)?;

                            let is_stereo = channels == Channels::Stereo;

                            let b = if is_stereo { len * 2 } else { len };

                            receiver.voice_packet(
                                ssrc,
                                seq,
                                timestamp,
                                is_stereo,
                                &buffer[..b],
                            );
                        }
                    },
                    ReceiverStatus::Websocket(VoiceEvent::Speaking(ev)) => {
                        receiver.speaking_update(ev.ssrc, ev.user_id.0, ev.speaking);
                    },
                    ReceiverStatus::Websocket(other) => {
                        info!("[Voice] Received other websocket data: {:?}", other);
                    },
                }
            }
        } else {
            loop {
                if self.thread_items.rx.try_recv().is_err() {
                    break;
                }
            }
        }

        // Send the voice websocket keepalive if it's time
        if self.keepalive_timer.check() {
            self.client.lock().unwrap().send_json(
                &payload::build_keepalive(),
            )?;
        }

        // Send UDP keepalive if it's time
        if self.audio_timer.check() {
            let mut bytes = [0; 4];
            (&mut bytes[..]).write_u32::<BigEndian>(self.ssrc)?;
            self.udp.send_to(&bytes, self.destination)?;
        }


        let mut opus_frame = Vec::new();

        let len = match source.as_mut() {
            Some(stream) => {
                let is_stereo = stream.is_stereo();

                if is_stereo != self.encoder_stereo {
                    let channels = if is_stereo {
                        Channels::Stereo
                    } else {
                        Channels::Mono
                    };
                    self.encoder = OpusEncoder::new(SAMPLE_RATE, channels, CodingMode::Audio)?;
                    self.encoder_stereo = is_stereo;
                }

                match stream.get_type() {
                    AudioType::Opus => {
                        match stream.read_opus_frame() {
                            Some(frame) => {
                                opus_frame = frame;
                                opus_frame.len()
                            },
                            None => 0,
                        }
                    },
                    AudioType::Pcm => {
                        let buffer_len = if is_stereo { 960 * 2 } else { 960 };

                        match stream.read_pcm_frame(&mut buffer[..buffer_len]) {
                            Some(len) => len,
                            None => 0,
                        }
                    },
                }
            },
            None => 0,
        };

        if len == 0 {
            self.set_speaking(false)?;

            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                for value in &mut buffer[..] {
                    *value = 0;
                }
            } else {
                audio_timer.await();

                return Ok(());
            }
        } else {
            self.silence_frames = 5;

            for value in &mut buffer[len..] {
                *value = 0;
            }
        }

        self.set_speaking(true)?;

        let index = self.prep_packet(&mut packet, buffer, &opus_frame, nonce)?;
        audio_timer.await();

        self.udp.send_to(&packet[..index], self.destination)?;
        self.audio_timer.reset();

        Ok(())
    }

    fn prep_packet(&mut self,
                   packet: &mut [u8; 512],
                   buffer: [i16; 1920],
                   opus_frame: &[u8],
                   mut nonce: Nonce)
                   -> Result<usize> {
        {
            let mut cursor = &mut packet[..HEADER_LEN];
            cursor.write_all(&[0x80, 0x78])?;
            cursor.write_u16::<BigEndian>(self.sequence)?;
            cursor.write_u32::<BigEndian>(self.timestamp)?;
            cursor.write_u32::<BigEndian>(self.ssrc)?;
        }

        nonce.0[..HEADER_LEN].clone_from_slice(
            &packet[..HEADER_LEN],
        );

        let sl_index = packet.len() - 16;
        let buffer_len = if self.encoder_stereo { 960 * 2 } else { 960 };

        let len = if opus_frame.is_empty() {
            self.encoder.encode(
                &buffer[..buffer_len],
                &mut packet[HEADER_LEN..sl_index],
            )?
        } else {
            let len = opus_frame.len();
            packet[HEADER_LEN..HEADER_LEN + len].clone_from_slice(
                opus_frame,
            );
            len
        };

        let crypted = {
            let slice = &packet[HEADER_LEN..HEADER_LEN + len];
            secretbox::seal(slice, &nonce, &self.key)
        };
        let index = HEADER_LEN + crypted.len();
        packet[HEADER_LEN..index].clone_from_slice(&crypted);

        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(960);

        Ok(HEADER_LEN + crypted.len())
    }

    fn set_speaking(&mut self, speaking: bool) -> Result<()> {
        if self.speaking == speaking {
            return Ok(());
        }

        self.speaking = speaking;

        self.client.lock().unwrap().send_json(
            &payload::build_speaking(
                speaking,
            ),
        )
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.thread_items.udp_close_sender.send(0);
        let _ = self.thread_items.ws_close_sender.send(0);

        info!("[Voice] Disconnected");
    }
}

fn generate_url(endpoint: &mut String) -> Result<WebsocketUrl> {
    if endpoint.ends_with(":80") {
        let len = endpoint.len();

        endpoint.truncate(len - 3);
    }

    WebsocketUrl::parse(&format!("wss://{}", endpoint)).or(Err(
        Error::Voice(VoiceError::EndpointUrl),
    ))
}

#[inline]
fn encryption_key(client: &mut Client) -> Result<Key> {
    loop {
        match client.recv_json(VoiceEvent::decode)? {
            VoiceEvent::Ready(ready) => {
                if ready.mode != CRYPTO_MODE {
                    return Err(Error::Voice(VoiceError::VoiceModeInvalid));
                }

                return Key::from_slice(&ready.secret_key).ok_or(Error::Voice(
                    VoiceError::KeyGen,
                ));
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
    }
}

#[inline]
fn has_valid_mode(modes: &[String]) -> bool { modes.iter().any(|s| s == CRYPTO_MODE) }

#[inline]
fn start_threads(client: Arc<Mutex<Client>>, udp: &UdpSocket) -> Result<ThreadItems> {
    let (udp_close_sender, udp_close_reader) = mpsc::channel();
    let (ws_close_sender, ws_close_reader) = mpsc::channel();

    let current_thread = thread::current();
    let thread_name = current_thread.name().unwrap_or("serenity voice");

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    let udp_clone = udp.try_clone()?;

    let udp_thread = ThreadBuilder::new()
        .name(format!("{} UDP", thread_name))
        .spawn(move || {
            let _ = udp_clone.set_read_timeout(Some(Duration::from_millis(250)));

            let mut buffer = [0; 512];

            loop {
                if let Ok((len, _)) = udp_clone.recv_from(&mut buffer) {
                    let piece = buffer[..len].to_vec();
                    let send = tx.send(ReceiverStatus::Udp(piece));

                    if send.is_err() {
                        return;
                    }
                } else if udp_close_reader.try_recv().is_ok() {
                    return;
                }
            }
        })?;

    let ws_thread = ThreadBuilder::new()
        .name(format!("{} WS", thread_name))
        .spawn(move || loop {
            while let Ok(msg) = client.lock().unwrap().recv_json(VoiceEvent::decode) {
                if tx_clone.send(ReceiverStatus::Websocket(msg)).is_ok() {
                    return;
                }
            }

            if ws_close_reader.try_recv().is_ok() {
                return;
            }

            thread::sleep(Duration::from_millis(25));
        })?;

    Ok(ThreadItems {
        rx: rx,
        udp_close_sender: udp_close_sender,
        udp_thread: udp_thread,
        ws_close_sender: ws_close_sender,
        ws_thread: ws_thread,
    })
}
