use audiopus::{
    Application as CodingMode,
    Bitrate,
    Channels,
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
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
        payload,
        threading::{
            AuxPacketMessage,
            EventMessage,
            Interconnect,
            TrackStateChange,
        },
        tracks::{Track, PlayMode},
        CRYPTO_MODE,
        VoiceError,
    },
};
use discortp::{
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
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    thread,
    time::Duration,
    time::Instant,
};
use url::Url;

#[cfg(not(feature = "native_tls_backend"))]
use crate::internal::ws_impl::create_rustls_client;

pub(crate) struct Connection {
    audio_timer: Timer,
    connection_info: ConnectionInfo,
    destination: SocketAddr,
    encoder: OpusEncoder,
    key: Key,
    packet: [u8; VOICE_PACKET_MAX],
    silence_frames: u8,
    soft_clip: SoftClip,
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
            silence_frames: 0,
            soft_clip,
            udp,
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

    #[inline]
    fn mix_tracks(
        &mut self,
        tracks: &mut Vec<Track>,
        _opus_frame: &[u8],
        mut mix_buffer: &mut [f32; STEREO_FRAME_SIZE],
        interconnect: &Interconnect,
    ) -> Result<usize> {
        let mut len = 0;
        let mut i = 0;

        while i < tracks.len() {
            let aud = tracks.get_mut(i).unwrap();

            let vol = aud.volume;

            let stream = &mut aud.source;

            if aud.playing != PlayMode::Play {          
                i += 1;
                continue;
            }

            let now = Instant::now();
            let temp_len = stream.mix(&mut mix_buffer, vol);
            let later = Instant::now();

            len = len.max(temp_len);
            if temp_len > 0 {
                aud.step_frame();
            } else if aud.do_loop() {
                if let Some(time) = aud.seek_time(Default::default()) {
                    interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Position(time)));
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
    fn audio_commands_events(&mut self, tracks: &mut Vec<Track>, interconnect: &Interconnect) {
        for (i, audio) in tracks.iter_mut().enumerate() {
            audio.process_commands(i, interconnect);
        }
    }

    #[allow(unused_variables)]
    pub(crate) fn cycle(&mut self,
                mut tracks: &mut Vec<Track>,
                audio_timer: &mut Timer,
                bitrate: Bitrate,
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

        // Reconfigure encoder bitrate.
        // From my testing, it seemed like this needed to be set every cycle.
        // FIXME: test ths again...
        if let Err(e) = self.encoder.set_bitrate(bitrate) {
            warn!("[Voice] Bitrate set unsuccessfully: {:?}", e);
        }

        let mut opus_frame = Vec::new();

        // Walk over all the audio files, removing those which have finished.
        // For this purpose, we need a while loop in Rust.
        let len = self.mix_tracks(&mut tracks, &opus_frame, &mut mix_buffer, interconnect)?;

        self.soft_clip.apply(&mut mix_buffer[..])?;

        if len == 0 {
            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                // Explicit "Silence" frame.
                opus_frame.extend_from_slice(&SILENT_FRAME);
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

        self.prep_and_send_packet(mix_buffer, &opus_frame)?;
        audio_timer.r#await();

        self.audio_timer.reset_from_deadline();

        self.audio_commands_events(&mut tracks, interconnect);

        Ok(())
    }

    fn prep_and_send_packet(&mut self, buffer: [f32; 1920], opus_frame: &[u8]) -> Result<()> {
        let mut nonce = secretbox::Nonce([0; NONCEBYTES]);
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
