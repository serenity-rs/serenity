use byteorder::{
    BigEndian,
    ByteOrder,
    LittleEndian,
    ReadBytesExt,
    WriteBytesExt
};
use crate::constants::VOICE_GATEWAY_VERSION;
use crate::gateway::WsClient;
use crate::internal::prelude::*;
use crate::internal::{
    ws_impl::{ReceiverExt, SenderExt},
    Timer
};
use crate::model::event::VoiceEvent;

use audiopus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    coder::Decoder as OpusDecoder,
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
};
use parking_lot::Mutex;
use rand::random;
use serde::Deserialize;
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{
        mpsc::{
            self,
            Receiver as MpscReceiver,
            Sender as MpscSender
        },
        Arc,
    },
    thread::{
        self,
        Builder as ThreadBuilder,
        JoinHandle
    },
    time::Duration
};

use super::audio::{AudioReceiver, AudioType, HEADER_LEN, SAMPLE_RATE, DEFAULT_BITRATE, LockedAudio};
use super::connection_info::ConnectionInfo;
use super::{payload, VoiceError, CRYPTO_MODE};
use url::Url;
use log::{debug, info, warn};

#[cfg(not(feature = "native_tls_backend"))]
use crate::internal::ws_impl::create_rustls_client;

enum ReceiverStatus {
    Udp(Vec<u8>),
    Websocket(VoiceEvent),
}

#[allow(dead_code)]
struct ThreadItems {
    rx: MpscReceiver<ReceiverStatus>,
    tx: MpscSender<ReceiverStatus>,
    udp_close_sender: MpscSender<i32>,
    udp_thread: JoinHandle<()>,
    ws_close_sender: MpscSender<i32>,
    ws_thread: JoinHandle<()>,
}

pub struct Connection {
    audio_timer: Timer,
    client: Arc<Mutex<WsClient>>,
    connection_info: ConnectionInfo,
    decoder_map: HashMap<(u32, Channels), OpusDecoder>,
    destination: SocketAddr,
    encoder: OpusEncoder,
    encoder_stereo: bool,
    keepalive_timer: Timer,
    key: Key,
    last_heartbeat_nonce: Option<u64>,
    sequence: u16,
    silence_frames: u8,
    soft_clip: SoftClip,
    speaking: bool,
    ssrc: u32,
    thread_items: ThreadItems,
    timestamp: u32,
    udp: UdpSocket,
}

impl Connection {
    pub fn new(mut info: ConnectionInfo) -> Result<Connection> {
        let url = generate_url(&mut info.endpoint)?;

        #[cfg(not(feature = "native_tls_backend"))]
        let mut client = create_rustls_client(url)?;

        #[cfg(feature = "native_tls_backend")]
        let mut client = tungstenite::connect(url)?.0;
        let mut hello = None;
        let mut ready = None;
        client.send_json(&payload::build_identify(&info))?;

        loop {
            let value = match client.recv_json()? {
                Some(value) => value,
                None => continue,
            };

            match VoiceEvent::deserialize(value)? {
                VoiceEvent::Ready(r) => {
                    ready = Some(r);
                    if hello.is_some(){
                        break;
                    }
                },
                VoiceEvent::Hello(h) => {
                    hello = Some(h);
                    if ready.is_some() {
                        break;
                    }
                },
                other => {
                    debug!("[Voice] Expected ready/hello; got: {:?}", other);

                    return Err(Error::Voice(VoiceError::ExpectedHandshake));
                },
            }
        };

        let hello = hello.expect("[Voice] Hello packet expected in connection initialisation, but not found.");
        let ready = ready.expect("[Voice] Ready packet expected in connection initialisation, but not found.");

        if !has_valid_mode(&ready.modes) {
            return Err(Error::Voice(VoiceError::VoiceModeUnavailable));
        }

        let destination = (&info.endpoint[..], ready.port)
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

            (&mut bytes[..]).write_u32::<BigEndian>(ready.ssrc)?;
            udp.send_to(&bytes, destination)?;

            let mut bytes = [0; 256];
            let (len, _addr) = udp.recv_from(&mut bytes)?;

            // Find the position in the bytes that contains the first byte of 0,
            // indicating the "end of the address".
            let index = bytes
                .iter()
                .skip(4)
                .position(|&x| x == 0)
                .ok_or(Error::Voice(VoiceError::FindingByte))?;

            let pos = 4 + index;
            let addr = String::from_utf8_lossy(&bytes[4..pos]);
            let port_pos = len - 2;
            let port = (&bytes[port_pos..]).read_u16::<LittleEndian>()?;

            client
                .send_json(&payload::build_select_protocol(addr, port))?;
        }

        let key = encryption_key(&mut client)?;

        unset_blocking(&mut client)?;
        let mutexed_client = Arc::new(Mutex::new(client));
        let thread_items = start_threads(Arc::clone(&mutexed_client), &udp)?;

        info!("[Voice] Connected to: {}", info.endpoint);

        // Encode for Discord in Stereo, as required.
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)?;
        encoder.set_bitrate(DEFAULT_BITRATE)?;
        let soft_clip = SoftClip::new(Channels::Stereo);

        // Per discord dev team's current recommendations:
        // (https://discordapp.com/developers/docs/topics/voice-connections#heartbeating)
        let temp_heartbeat = (hello.heartbeat_interval as f64 * 0.75) as u64;
        info!(
            "[Voice] WS heartbeat duration given as {}ms, adjusted to {}ms.",
            hello.heartbeat_interval,
            temp_heartbeat,
        );

        Ok(Connection {
            audio_timer: Timer::new(1000 * 60 * 4),
            client: mutexed_client,
            connection_info: info,
            decoder_map: HashMap::new(),
            destination,
            encoder,
            encoder_stereo: false,
            key,
            keepalive_timer: Timer::new(temp_heartbeat),
            last_heartbeat_nonce: None,
            udp,
            sequence: 0,
            silence_frames: 0,
            soft_clip,
            speaking: false,
            ssrc: ready.ssrc,
            thread_items,
            timestamp: 0,
        })
    }

    pub fn reconnect(&mut self) -> Result<()> {
        let url = generate_url(&mut self.connection_info.endpoint)?;

        // Thread may have died, we want to send to prompt a clean exit
        // (if at all possible) and then proceed as normal.
        let _ = self.thread_items.ws_close_sender.send(0);

        #[cfg(not(feature = "native_tls_backend"))]
        let mut client = create_rustls_client(url)?;

        #[cfg(feature = "native_tls_backend")]
        let mut client = tungstenite::connect(url)?.0;

        client.send_json(&payload::build_resume(&self.connection_info))?;

        let mut hello = None;
        let mut resumed = None;

        loop {
            let value = match client.recv_json()? {
                Some(value) => value,
                None => continue,
            };

            match VoiceEvent::deserialize(value)? {
                VoiceEvent::Resumed => {
                    resumed = Some(());
                    if hello.is_some(){
                        break;
                    }
                },
                VoiceEvent::Hello(h) => {
                    hello = Some(h);
                    if resumed.is_some() {
                        break;
                    }
                },
                other => {
                    debug!("[Voice] Expected resumed/hello; got: {:?}", other);

                    return Err(Error::Voice(VoiceError::ExpectedHandshake));
                },
            }
        };

        let hello = hello.expect("[Voice] Hello packet expected in connection initialisation, but not found.");

        self.keepalive_timer = Timer::new((hello.heartbeat_interval as f64 * 0.75) as u64);

        unset_blocking(&mut client)?;
        let mutexed_client = Arc::new(Mutex::new(client));
        let (ws_close_sender, ws_thread) = start_ws_thread(mutexed_client, &self.thread_items.tx)?;

        self.thread_items.ws_close_sender = ws_close_sender;
        self.thread_items.ws_thread = ws_thread;

        info!("[Voice] Reconnected to: {}", &self.connection_info.endpoint);
        Ok(())
    }

    #[inline]
    fn handle_received_udp(
        &mut self,
        receiver: &mut Option<Box<dyn AudioReceiver>>,
        buffer: &mut [i16; 1920],
        packet: &[u8],
        nonce: &mut Nonce,
        ) -> Result<()> {

        if let Some(receiver) = receiver.as_mut() {
            let mut handle = &packet[2..];
            let seq = handle.read_u16::<BigEndian>()?;
            let timestamp = handle.read_u32::<BigEndian>()?;
            let ssrc = handle.read_u32::<BigEndian>()?;

            nonce.0[..HEADER_LEN]
                .clone_from_slice(&packet[..HEADER_LEN]);

            if let Ok(mut decrypted) =
                secretbox::open(&packet[HEADER_LEN..], &nonce, &self.key) {
                let channels = opus_packet::nb_channels(&decrypted)?;

                let entry =
                    self.decoder_map.entry((ssrc, channels)).or_insert_with(
                        || OpusDecoder::new(SAMPLE_RATE, channels).unwrap(),
                    );

                // Strip RTP Header Extensions (one-byte)
                if decrypted[0] == 0xBE && decrypted[1] == 0xDE {
                    // Read the length bytes as a big-endian u16.
                    let header_extension_len = BigEndian::read_u16(&decrypted[2..4]);
                    let mut offset = 4;
                    for _ in 0..header_extension_len {
                        let byte = decrypted[offset];
                        offset += 1;
                        if byte == 0 {
                            continue;
                        }

                        offset += 1 + (0b1111 & (byte >> 4)) as usize;
                    }

                    while decrypted[offset] == 0 {
                        offset += 1;
                    }

                    decrypted = decrypted.split_off(offset);
                }

                let len = entry.decode(Some(&decrypted), &mut buffer[..], false)?;

                let is_stereo = channels == Channels::Stereo;

                let b = if is_stereo { len * 2 } else { len };

                receiver
                    .voice_packet(ssrc, seq, timestamp, is_stereo, &buffer[..b], decrypted.len());
            }
        }

        Ok(())
    }

    #[inline]
    fn check_audio_timer(&mut self) -> Result<()> {
        if self.audio_timer.check() {
            info!("[Voice] UDP keepalive");
            let mut bytes = [0; 4];
            (&mut bytes[..]).write_u32::<BigEndian>(self.ssrc)?;
            self.udp.send_to(&bytes, self.destination)?;
            info!("[Voice] UDP keepalive sent");
        }

        Ok(())
    }

    #[inline]
    fn check_keepalive_timer(&mut self) -> Result<()> {
        if self.keepalive_timer.check() {
            info!("[Voice] WS keepalive");
            let nonce = random::<u64>();
            self.last_heartbeat_nonce = Some(nonce);
            self.client.lock().send_json(&payload::build_heartbeat(nonce))?;
            info!("[Voice] WS keepalive sent");
        }

        Ok(())
    }

    #[inline]
    fn remove_unfinished_files(
        &mut self,
        sources: &mut Vec<LockedAudio>,
        opus_frame: &[u8],
        buffer: &mut [i16; 1920],
        mut mix_buffer: &mut [f32; 1920],
    ) -> Result<usize> {
        let mut len = 0;
        let mut i = 0;

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

                // Assume this for now, at least.
                // We'll be fusing streams, so we can either keep
                // as stereo or downmix to mono.
                let is_stereo = true;
                let source_stereo = stream.is_stereo();

                if is_stereo != self.encoder_stereo {
                    let channels = if is_stereo {
                        Channels::Stereo
                    } else {
                        Channels::Mono
                    };
                    self.encoder = OpusEncoder::new(SAMPLE_RATE, channels, CodingMode::Audio)?;
                    self.encoder_stereo = is_stereo;
                }

                let temp_len = match stream.get_type() {
                    AudioType::Opus => if stream.decode_and_add_opus_frame(&mut mix_buffer, vol).is_some() {
                            opus_frame.len()
                        } else {
                            0
                        },
                    AudioType::Pcm => {
                        let buffer_len = if source_stereo { 960 * 2 } else { 960 };

                        match stream.read_pcm_frame(&mut buffer[..buffer_len]) {
                            Some(len) => len,
                            None => 0,
                        }
                    },
                    AudioType::__Nonexhaustive => unreachable!(),
                };

                // May need to force interleave/copy.
                combine_audio(*buffer, &mut mix_buffer, source_stereo, vol);

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

        Ok(len)
    }

    #[allow(unused_variables)]
    pub fn cycle(&mut self,
                mut sources: &mut Vec<LockedAudio>,
                mut receiver: &mut Option<Box<dyn AudioReceiver>>,
                audio_timer: &mut Timer,
                bitrate: Bitrate)
                 -> Result<()> {
        // We need to actually reserve enough space for the desired bitrate.
        let size = match bitrate {
            // If user specified, we can calculate. 20ms means 50fps.
            Bitrate::BitsPerSecond(b) => b / 50,
            // Otherwise, just have a lot preallocated.
            _ => 5120,
        } + 16;

        let mut buffer = [0i16; 960 * 2];
        let mut mix_buffer = [0f32; 960 * 2];
        let mut packet = vec![0u8; size as usize].into_boxed_slice();
        let mut nonce = secretbox::Nonce([0; 24]);

        while let Ok(status) = self.thread_items.rx.try_recv() {
            match status {
                ReceiverStatus::Udp(packet) => {
                    self.handle_received_udp(&mut receiver, &mut buffer, &packet[..], &mut nonce)?;
                },
                ReceiverStatus::Websocket(VoiceEvent::Speaking(ev)) => {
                    if let Some(receiver) = receiver.as_mut() {
                        receiver.speaking_update(ev.ssrc, ev.user_id.0, ev.speaking);
                    }
                },
                ReceiverStatus::Websocket(VoiceEvent::ClientConnect(ev)) => {
                    if let Some(receiver) = receiver.as_mut() {
                        receiver.client_connect(ev.audio_ssrc, ev.user_id.0);
                    }
                },
                ReceiverStatus::Websocket(VoiceEvent::ClientDisconnect(ev)) => {
                    if let Some(receiver) = receiver.as_mut() {
                        receiver.client_disconnect(ev.user_id.0);
                    }
                },
                ReceiverStatus::Websocket(VoiceEvent::HeartbeatAck(ev)) => {
                    if let Some(nonce) = self.last_heartbeat_nonce {

                        if ev.nonce == nonce {
                            info!("[Voice] Heartbeat ACK received.");
                        } else {
                            warn!("[Voice] Heartbeat nonce mismatch! Expected {}, saw {}.", nonce, ev.nonce);
                        }

                        self.last_heartbeat_nonce = None;
                    }
                },
                ReceiverStatus::Websocket(other) => {
                    info!("[Voice] Received other websocket data: {:?}", other);
                },
            }
        }

        // Send the voice websocket keepalive if it's time
        self.check_keepalive_timer()?;

        // Send UDP keepalive if it's time
        self.check_audio_timer()?;

        // Reconfigure encoder bitrate.
        // From my testing, it seemed like this needed to be set every cycle.
        if let Err(e) = self.encoder.set_bitrate(bitrate) {
            warn!("[Voice] Bitrate set unsuccessfully: {:?}", e);
        }

        let mut opus_frame = Vec::new();

        // Walk over all the audio files, removing those which have finished.
        // For this purpose, we need a while loop in Rust.
        let len = self.remove_unfinished_files(&mut sources, &opus_frame, &mut buffer,&mut mix_buffer)?;

        self.soft_clip.apply(&mut mix_buffer[..])?;

        if len == 0 {
            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                // Explicit "Silence" frame.
                opus_frame.extend_from_slice(&[0xf8, 0xff, 0xfe]);
            } else {
                // Per official guidelines, send 5x silence BEFORE we stop speaking.
                self.set_speaking(false)?;

                audio_timer.r#await();

                return Ok(());
            }
        } else {
            self.silence_frames = 5;

            for value in &mut buffer[len..] {
                *value = 0;
            }
        }

        self.set_speaking(true)?;

        let index = self.prep_packet(&mut packet, mix_buffer, &opus_frame, nonce)?;
        audio_timer.r#await();

        self.udp.send_to(&packet[..index], self.destination)?;
        self.audio_timer.reset();

        Ok(())
    }

    fn prep_packet(&mut self,
                   packet: &mut [u8],
                   buffer: [f32; 1920],
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

        nonce.0[..HEADER_LEN]
            .clone_from_slice(&packet[..HEADER_LEN]);

        let sl_index = packet.len() - 16;
        let buffer_len = if self.encoder_stereo { 960 * 2 } else { 960 };

        let len = if opus_frame.is_empty() {
            self.encoder
                .encode_float(&buffer[..buffer_len], &mut packet[HEADER_LEN..sl_index])?
        } else {
            let len = opus_frame.len();
            packet[HEADER_LEN..HEADER_LEN + len]
                .clone_from_slice(opus_frame);
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

        info!("[Voice] Speaking update: {}", speaking);
        let o = self.client.lock().send_json(&payload::build_speaking(speaking));
        info!("[Voice] Speaking update confirmed.");
        o
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.thread_items.udp_close_sender.send(0);
        let _ = self.thread_items.ws_close_sender.send(0);

        info!("[Voice] Disconnected");
    }
}

#[inline]
fn combine_audio(
    raw_buffer: [i16; 1920],
    float_buffer: &mut [f32; 1920],
    true_stereo: bool,
    volume: f32,
) {
    for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
        let sample_index = if true_stereo { i } else { i / 2 };
        let sample = f32::from(raw_buffer[sample_index]) / 32768.0;

        *float_buffer_element += sample * volume;
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
fn encryption_key(client: &mut WsClient) -> Result<Key> {
    loop {
        let value = match client.recv_json()? {
            Some(value) => value,
            None => continue,
        };

        match VoiceEvent::deserialize(value)? {
            VoiceEvent::SessionDescription(desc) => {
                if desc.mode != CRYPTO_MODE {
                    return Err(Error::Voice(VoiceError::VoiceModeInvalid));
                }

                return Key::from_slice(&desc.secret_key)
                    .ok_or(Error::Voice(VoiceError::KeyGen));
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
fn has_valid_mode<T, It> (modes: It) -> bool
where T: for<'a> PartialEq<&'a str>,
      It : IntoIterator<Item=T>
{
    modes.into_iter().any(|s| s == CRYPTO_MODE)
}

#[inline]
fn start_threads(client: Arc<Mutex<WsClient>>, udp: &UdpSocket) -> Result<ThreadItems> {
    let (udp_close_sender, udp_close_reader) = mpsc::channel();

    let current_thread = thread::current();
    let thread_name = current_thread.name().unwrap_or("serenity voice");

    let (tx, rx) = mpsc::channel();
    let tx_udp = tx.clone();
    let udp_clone = udp.try_clone()?;

    let udp_thread = ThreadBuilder::new()
        .name(format!("{} UDP", thread_name))
        .spawn(move || {
            let _ = udp_clone.set_read_timeout(Some(Duration::from_millis(250)));

            let mut buffer = [0; 512];

            'outer: loop {
                if let Ok((len, _)) = udp_clone.recv_from(&mut buffer) {
                    let piece = buffer[..len].to_vec();
                    let send = tx_udp.send(ReceiverStatus::Udp(piece));

                    if send.is_err() {
                        break 'outer;
                    }
                } else if udp_close_reader.try_recv().is_ok() {
                    break 'outer;
                }
            }

            info!("[Voice] UDP thread exited.");
        })?;

    let (ws_close_sender, ws_thread) = start_ws_thread(client, &tx)?;

    Ok(ThreadItems {
        rx,
        tx,
        udp_close_sender,
        udp_thread,
        ws_close_sender,
        ws_thread,
    })
}

#[inline]
fn start_ws_thread(client: Arc<Mutex<WsClient>>, tx: &MpscSender<ReceiverStatus>) -> Result<(MpscSender<i32>, JoinHandle<()>)> {
    let current_thread = thread::current();
    let thread_name = current_thread.name().unwrap_or("serenity voice");

    let tx_ws = tx.clone();
    let (ws_close_sender, ws_close_reader) = mpsc::channel();

    let ws_thread = ThreadBuilder::new()
        .name(format!("{} WS", thread_name))
        .spawn(move || {
            'outer: loop {
                while let Ok(Some(value)) = client.lock().try_recv_json() {
                    let msg = match VoiceEvent::deserialize(value) {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };

                    if tx_ws.send(ReceiverStatus::Websocket(msg)).is_err() {
                        break 'outer;
                    }
                }

                if ws_close_reader.try_recv().is_ok() {
                    break 'outer;
                }

                thread::sleep(Duration::from_millis(25));
            }
            info!("[Voice] WS thread exited.");
        })?;

    Ok((ws_close_sender, ws_thread))
}

#[inline]
fn unset_blocking(client: &mut WsClient) -> Result<()> {
    #[cfg(not(feature = "native_tls_backend"))]
    let stream = &client.get_mut().sock;

    #[cfg(feature = "native_tls_backend")]
    let stream = match client.get_mut() {
        tungstenite::stream::Stream::Plain(stream) => stream,
        tungstenite::stream::Stream::Tls(stream) => stream.get_mut(),
    };

    stream.set_nonblocking(true)
        .map_err(Into::into)
}
