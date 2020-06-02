use audiopus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    coder::Decoder as OpusDecoder,
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
};
use byteorder::{
    BigEndian,
    ByteOrder,
    ReadBytesExt,
    WriteBytesExt
};
use crate::{
    constants::VOICE_GATEWAY_VERSION,
    gateway::WsClient,
    internal::prelude::*,
    internal::{
        ws_impl::{ReceiverExt, SenderExt},
        Timer,
    },
    model::event::{VoiceEvent, VoiceSpeakingState},
    voice::{
        connection_info::ConnectionInfo,
        constants::*,
        events::{EventContext, UntimedEvent},
        payload,
        threading::{
            AuxPacketMessage,
            EventMessage,
            Interconnect,
            TrackStateChange,
        },
        tracks::{Track, PlayMode},
        CRYPTO_MODE,
        TrackEvent,
        VoiceError,
    },
};
use discortp::{
    demux::{
        self,
        Demuxed,
    },
    discord::{
        IpDiscoveryPacket,
        IpDiscoveryType,
        MutableIpDiscoveryPacket,
    },
    rtp::{
        MutableRtpPacket,
        RtpPacket,
        RtpType,
    },
    MutablePacket,
    Packet,
};
use log::{debug, info, warn};
use parking_lot::Mutex;
use rand::random;
use serde::Deserialize;
use sodiumoxide::crypto::secretbox::{
    self,
    MACBYTES,
    NONCEBYTES,
    Key,
    Nonce,
};
use std::{
    collections::HashMap,
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
    time::Duration,
    time::Instant,
};
use url::Url;

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

pub(crate) struct Connection {
    audio_timer: Timer,
    connection_info: ConnectionInfo,
    destination: SocketAddr,
    encoder: OpusEncoder,
    key: Key,
    packet: [u8; VOICE_PACKET_MAX],
    silence_frames: u8,
    soft_clip: SoftClip,
    speaking: VoiceSpeakingState,
    ssrc: u32,
    udp: UdpSocket,
}

impl Connection {
    pub(crate) fn new(mut info: ConnectionInfo, interconnect: &Interconnect) -> Result<Connection> {
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

        let destination = (&ready.ip[..], ready.port)
            .to_socket_addrs()?
            .next()
            .ok_or(Error::Voice(VoiceError::HostnameResolve))?;

        let udp = UdpSocket::bind("0.0.0.0:0")?;

        // Follow Discord's IP Discovery procedures, in case NAT tunnelling is needed.
        let mut bytes = [0; IpDiscoveryPacket::const_packet_size()];
        {
            let mut view = MutableIpDiscoveryPacket::new(&mut bytes[..])
                .expect(
                    "[Voice] Too few bytes in 'bytes' for IPDiscovery packet.\
                    (Blame: IpDiscoveryPacket::const_packet_size()?)"
                );
            view.set_pkt_type(IpDiscoveryType::Request);
            view.set_length(70);
            view.set_ssrc(ready.ssrc);
        }

        udp.send_to(&bytes, destination)?;

        let (len, _addr) = udp.recv_from(&mut bytes)?;
        {
            let view = IpDiscoveryPacket::new(&bytes[..len])
                .ok_or_else(|| Error::Voice(VoiceError::IllegalDiscoveryResponse))?;

            if view.get_pkt_type() != IpDiscoveryType::Response {
                return Err(Error::Voice(VoiceError::IllegalDiscoveryResponse));
            }

            let addr = String::from_utf8_lossy(&view.get_address_raw());
            client.send_json(&payload::build_select_protocol(addr, view.get_port()))?;
        }

        let key = encryption_key(&mut client)?;

        unset_blocking(&mut client)?;

        info!("[Voice] Connected to: {}", info.endpoint);

        // Encode for Discord in Stereo, as required.
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)?;
        encoder.set_bitrate(DEFAULT_BITRATE)?;
        let soft_clip = SoftClip::new(Channels::Stereo);

        info!(
            "[Voice] WS heartbeat duration {}ms.",
            hello.heartbeat_interval,
        );

        let mut packet = [0u8; VOICE_PACKET_MAX];

        let mut rtp = MutableRtpPacket::new(&mut packet[..])
            .expect(
                "[Voice] Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)"
            );
        rtp.set_version(2);
        rtp.set_payload_type(RtpType::Dynamic(120));
        rtp.set_sequence(random::<u16>().into());
        rtp.set_timestamp(random::<u32>().into());
        rtp.set_ssrc(ready.ssrc);

        interconnect.aux_packets.send(AuxPacketMessage::UdpDestination(destination));
        interconnect.aux_packets.send(AuxPacketMessage::UdpKey(key.clone()));
        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval));
        interconnect.aux_packets.send(AuxPacketMessage::SetSsrc(ready.ssrc));
        interconnect.aux_packets.send(AuxPacketMessage::Udp(
            udp.try_clone()
                .expect("[Voice] Failed to clone UDP")
        ));
        interconnect.aux_packets.send(AuxPacketMessage::Ws(client));

        Ok(Connection {
            audio_timer: Timer::new(1000 * 60 * 4),
            connection_info: info,
            destination,
            encoder,
            key,
            packet,
            udp,
            silence_frames: 0,
            soft_clip,
            speaking: VoiceSpeakingState::empty(),
            ssrc: ready.ssrc,
            // thread_items,
        })
    }

    pub fn reconnect(&mut self, interconnect: &Interconnect) -> Result<()> {
        let url = generate_url(&mut self.connection_info.endpoint)?;

        // Thread may have died, we want to send to prompt a clean exit
        // (if at all possible) and then proceed as normal.
        // let _ = self.thread_items.ws_close_sender.send(0);

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

        unset_blocking(&mut client)?;

        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval));
        interconnect.aux_packets.send(AuxPacketMessage::Ws(client));

        info!("[Voice] Reconnected to: {}", &self.connection_info.endpoint);
        Ok(())
    }

    // #[inline]
    // fn handle_received_udp(
    //     &mut self,
    //     interconnect: &Interconnect,
    //     buffer: &mut [i16; 1920],
    //     packet: &[u8],
    //     nonce: &mut Nonce,
    //     ) -> Result<()> {
    //     match demux::demux(packet) {
    //         Demuxed::Rtp(rtp) => {
    //             println!("{:?}", rtp);
    //             println!("{:?}", rtp.payload());
    //         },
    //         Demuxed::Rtcp(rtcp) => {
    //             println!("{:?}", rtcp);
    //             println!("{:?}", rtcp.payload());
    //         },
    //         Demuxed::FailedParse(t) => {
    //             warn!("[Voice] Failed to parse message of type {:?}.", t);
    //         }
    //         _ => {
    //             warn!("[Voice] Illegal UDP packet from voice server.");
    //         }
    //     }

    //     // Old code: make this nicer and integrate with above Demux.
    //     let mut handle = &packet[2..];
    //     let sequence = handle.read_u16::<BigEndian>()?;
    //     let timestamp = handle.read_u32::<BigEndian>()?;
    //     let ssrc = handle.read_u32::<BigEndian>()?;

    //     let rtp_len = RtpPacket::minimum_packet_size();
    //     nonce.0[..rtp_len]
    //         .clone_from_slice(&packet[..rtp_len]);

    //     if let Ok(mut decrypted) =
    //         secretbox::open(&packet[rtp_len..], &nonce, &self.key) {
    //         let channels = opus_packet::nb_channels(&decrypted)?;

    //         let entry =
    //             self.decoder_map.entry((ssrc, channels)).or_insert_with(
    //                 || OpusDecoder::new(SAMPLE_RATE, channels).unwrap(),
    //             );

    //         // Strip RTP Header Extensions (one-byte)
    //         if decrypted[0] == 0xBE && decrypted[1] == 0xDE {
    //             // Read the length bytes as a big-endian u16.
    //             let header_extension_len = BigEndian::read_u16(&decrypted[2..4]);
    //             let mut offset = 4;
    //             for _ in 0..header_extension_len {
    //                 let byte = decrypted[offset];
    //                 offset += 1;
    //                 if byte == 0 {
    //                     continue;
    //                 }

    //                 offset += 1 + (0b1111 & (byte >> 4)) as usize;
    //             }

    //             // Skip over undocumented Discord byte
    //             offset += 1;

    //             decrypted = decrypted.split_off(offset);
    //         }

    //         let len = entry.decode(Some(&decrypted), &mut buffer[..], false)?;

    //         let is_stereo = channels == Channels::Stereo;

    //         let b = if is_stereo { len * 2 } else { len };

    //         interconnect.events.send(EventMessage::FireCoreEvent(
    //             EventContext::VoicePacket {
    //                 ssrc,
    //                 sequence,
    //                 timestamp,
    //                 stereo: is_stereo,
    //                 data: buffer[..b].to_vec(),
    //                 compressed_size: decrypted.len(),
    //             }
    //         ));
    //     }

    //     Ok(())
    // }

    #[inline]
    fn mix_tracks(
        &mut self,
        sources: &mut Vec<Track>,
        _opus_frame: &[u8],
        mut mix_buffer: &mut [f32; STEREO_FRAME_SIZE],
        time_in_call: &mut Duration,
        entry_points: &mut u64,
        interconnect: &Interconnect,
    ) -> Result<usize> {
        let mut len = 0;
        let mut i = 0;

        while i < sources.len() {
            let aud = sources.get_mut(i).unwrap();

            let vol = aud.volume;

            let stream = &mut aud.source;

            if aud.playing != PlayMode::Play {          
                i += 1;
                continue;
            }

            let now = Instant::now();
            let temp_len = stream.mix(&mut mix_buffer, vol);
            let later = Instant::now();

            *time_in_call += later - now;
            *entry_points += 1;

            if *entry_points % 1000 == 0 {
                println!("Average cost {:?}ns", time_in_call.as_nanos()/(*entry_points as u128));

                *time_in_call = Duration::default();
                *entry_points = 0;
            }

            len = len.max(temp_len);
            if temp_len > 0 {
                aud.step_frame();
            } else if aud.do_loop() {
                if aud.seek_time(Default::default()).is_some() {
                    interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Loops(aud.loops, false)));
                }

            } else {
                aud.end();
            }

            i += 1;
        };

        Ok(len)
    }

    #[inline]
    fn audio_commands_events(&mut self, sources: &mut Vec<Track>, interconnect: &Interconnect) {
        for (i, audio) in sources.iter_mut().enumerate() {
            audio.process_commands(i, interconnect);
        }
    }

    #[allow(unused_variables)]
    pub(crate) fn cycle(&mut self,
                mut sources: &mut Vec<Track>,
                audio_timer: &mut Timer,
                bitrate: Bitrate,
                mut time_in_call: &mut Duration,
                mut entry_points: &mut u64,
                interconnect: &Interconnect)
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
        let mut nonce = secretbox::Nonce([0; NONCEBYTES]);

        // Reconfigure encoder bitrate.
        // From my testing, it seemed like this needed to be set every cycle.
        // FIXME: test ths again...
        if let Err(e) = self.encoder.set_bitrate(bitrate) {
            warn!("[Voice] Bitrate set unsuccessfully: {:?}", e);
        }

        let mut opus_frame = Vec::new();

        // Walk over all the audio files, removing those which have finished.
        // For this purpose, we need a while loop in Rust.
        let len = self.mix_tracks(&mut sources, &opus_frame, &mut mix_buffer, &mut time_in_call, &mut entry_points, interconnect)?;

        self.soft_clip.apply(&mut mix_buffer[..])?;

        if len == 0 {
            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                // Explicit "Silence" frame.
                opus_frame.extend_from_slice(&[0xf8, 0xff, 0xfe]);
            } else {
                // Per official guidelines, send 5x silence BEFORE we stop speaking.
                interconnect.aux_packets.send(AuxPacketMessage::Speaking(false));

                audio_timer.r#await();

                return Ok(());
            }
        } else {
            self.silence_frames = 5;

            for value in &mut buffer[len..] {
                *value = 0;
            }
        }

        interconnect.aux_packets.send(AuxPacketMessage::Speaking(true));

        self.prep_and_send_packet(mix_buffer, &opus_frame, nonce)?;
        audio_timer.r#await();

        self.audio_timer.reset_from_deadline();

        self.audio_commands_events(&mut sources, interconnect);

        Ok(())
    }

    fn prep_and_send_packet(&mut self,
                   buffer: [f32; 1920],
                   opus_frame: &[u8],
                   mut nonce: Nonce)
                   -> Result<()> {
        let index = {
            let mut rtp = MutableRtpPacket::new(&mut self.packet[..])
                .expect(
                    "[Voice] Too few bytes in self.packet for RTP header.\
                    (Blame: VOICE_PACKET_MAX?)"
                );

            let pkt = rtp.packet();
            let rtp_len = RtpPacket::minimum_packet_size();
            nonce.0[..rtp_len]
                .copy_from_slice(&pkt[..rtp_len]);

            let payload = rtp.payload_mut();

            let payload_len = if opus_frame.is_empty() {
                self.encoder
                    .encode_float(&buffer[..STEREO_FRAME_SIZE], &mut payload[MACBYTES..])?
            } else {
                let len = opus_frame.len();
                payload[MACBYTES..MACBYTES + len]
                    .clone_from_slice(opus_frame);
                len
            };

            let final_payload_size = MACBYTES + payload_len;

            let tag = secretbox::seal_detached(&mut payload[MACBYTES..final_payload_size], &nonce, &self.key);
            payload[..MACBYTES].copy_from_slice(&tag.0[..]);

            rtp_len + final_payload_size
        };

        self.udp.send_to(&self.packet[..index], self.destination)?;

        let mut rtp = MutableRtpPacket::new(&mut self.packet[..])
            .expect(
                "[Voice] Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)"
            );
        rtp.set_sequence(rtp.get_sequence() + 1);
        rtp.set_timestamp(rtp.get_timestamp() + 960);

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        info!("[Voice] Disconnected");
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
                    debug!("VOX WS {:#?}", value);
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
