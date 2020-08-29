use async_tungstenite::tungstenite::protocol::Message;
use audiopus::{
    Application as CodingMode,
    Bitrate,
    Channels,
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
};
use crate::{
    constants::VOICE_GATEWAY_VERSION,
    gateway::WsStream,
    internal::{
        prelude::*,
        ws_impl::{ReceiverExt, SenderExt},
        Timer,
    },
    model::event::VoiceEvent,
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
    },
    MutablePacket,
    Packet,
};
use futures::{
    channel::mpsc::{
        self,
        Receiver as MpscReceiver,
        Sender as MpscSender,
        TryRecvError,
    },
    sink::SinkExt,
};
use log::{debug, info, warn};
use rand::random;
use serde::Deserialize;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::{
    time::{delay_for, timeout},
    net::{
        UdpSocket,
        udp::{RecvHalf, SendHalf},
    },
};
use url::Url;
use xsalsa20poly1305::{
    aead::{AeadInPlace, NewAead},
    KEY_SIZE,
    TAG_SIZE,
    Key,
    Nonce, 
    XSalsa20Poly1305 as Cipher,
};


#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
use crate::internal::ws_impl::create_rustls_client;

#[cfg(feature = "native_tls_backend")]
use crate::internal::ws_impl::create_native_tls_client;

pub(crate) struct Connection {
    cipher: Cipher,
    pub(crate) connection_info: ConnectionInfo,
    destination: SocketAddr,
    encoder: OpusEncoder,
    packet: [u8; VOICE_PACKET_MAX],
    silence_frames: u8,
    soft_clip: SoftClip,
    udp: SendHalf,
}

impl Connection {
    pub(crate) async fn new(mut info: ConnectionInfo, interconnect: &Interconnect, bitrate: Bitrate) -> Result<Connection> {
        let url = generate_url(&mut info.endpoint)?;

        #[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
        let mut client = create_rustls_client(url).await?;

        #[cfg(feature = "native_tls_backend")]
        let mut client = create_native_tls_client(url).await?;

        info!("Made thing");

        let mut hello = None;
        let mut ready = None;
        client.send_json(&payload::build_identify(&info)).await?;

        info!("Sent thing");

        loop {
            let value = match client.recv_json().await? {
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

        info!("visited HR");

        if !has_valid_mode(&ready.modes) {
            return Err(Error::Voice(VoiceError::VoiceModeUnavailable));
        }

        let destination = (&ready.ip[..], ready.port)
            .to_socket_addrs()?
            .next()
            .ok_or(Error::Voice(VoiceError::HostnameResolve))?;

        let mut udp = UdpSocket::bind("0.0.0.0:0").await?;

        info!("Made udp");

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

        udp.send_to(&bytes, &destination).await?;

        info!("sent disco");

        let (len, _addr) = udp.recv_from(&mut bytes).await?;
        {
            let view = IpDiscoveryPacket::new(&bytes[..len])
                .ok_or_else(|| Error::Voice(VoiceError::IllegalDiscoveryResponse))?;

            if view.get_pkt_type() != IpDiscoveryType::Response {
                return Err(Error::Voice(VoiceError::IllegalDiscoveryResponse));
            }

            let addr = String::from_utf8_lossy(&view.get_address_raw());
            client.send_json(&payload::build_select_protocol(addr, view.get_port())).await?;
        }

        info!("rx'd disco");

        let cipher = init_cipher(&mut client).await?;

        info!("Made cipher");

        info!("[Voice] Connected to: {}", info.endpoint);

        // Encode for Discord in Stereo, as required.
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)?;
        encoder.set_bitrate(bitrate)?;
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
        rtp.set_version(RTP_VERSION);
        rtp.set_payload_type(RTP_PROFILE_TYPE);
        rtp.set_sequence(random::<u16>().into());
        rtp.set_timestamp(random::<u32>().into());
        rtp.set_ssrc(ready.ssrc);

        let(udp_rx, udp_tx) = udp.split();

        interconnect.aux_packets.send(AuxPacketMessage::UdpDestination(destination))?;
        interconnect.aux_packets.send(AuxPacketMessage::UdpCipher(cipher.clone()))?;
        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval))?;
        interconnect.aux_packets.send(AuxPacketMessage::SetSsrc(ready.ssrc))?;
        interconnect.aux_packets.send(AuxPacketMessage::Udp(udp_rx))?;
        interconnect.aux_packets.send(AuxPacketMessage::Ws(Box::new(client)))?;

        Ok(Connection {
            cipher,
            connection_info: info,
            destination,
            encoder,
            packet,
            silence_frames: 0,
            soft_clip,
            udp: udp_tx,
        })
    }

    pub async fn reconnect(&mut self, interconnect: &Interconnect) -> Result<()> {
        let url = generate_url(&mut self.connection_info.endpoint)?;

        // Thread may have died, we want to send to prompt a clean exit
        // (if at all possible) and then proceed as normal.

        #[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
        let mut client = create_rustls_client(url).await?;

        #[cfg(feature = "native_tls_backend")]
        let mut client = create_native_tls_client(url).await?;

        client.send_json(&payload::build_resume(&self.connection_info)).await?;

        let mut hello = None;
        let mut resumed = None;

        loop {
            let value = match client.recv_json().await? {
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

        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval))?;
        interconnect.aux_packets.send(AuxPacketMessage::Ws(Box::new(client)))?;

        info!("[Voice] Reconnected to: {}", &self.connection_info.endpoint);
        Ok(())
    }

    #[inline]
    fn mix_tracks<'a>(
        &mut self,
        tracks: &mut Vec<Track>,
        opus_frame: &'a mut [u8],
        mix_buffer: &mut [f32; STEREO_FRAME_SIZE],
        interconnect: &Interconnect,
    ) -> Result<(usize, &'a[u8])> {
        let mut len = 0;

        for (i, track) in tracks.iter_mut().enumerate() {
            let vol = track.volume;
            let stream = &mut track.source;

            if track.playing != PlayMode::Play {
                continue;
            }

            let temp_len = stream.mix(mix_buffer, vol);

            len = len.max(temp_len);
            if temp_len > 0 {
                track.step_frame();
            } else if track.do_loop() {
                if let Some(time) = track.seek_time(Default::default()) {
                    let _ = interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Position(time)));
                    let _ = interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Loops(track.loops, false)));
                }
            } else {
                track.end();
            }
        };

        // If opus frame passthorugh were supported, we'd do it in this function.
        // This requires that we have only one track, who has volume 1.0, and an
        // Opus codec type.
        //
        // For now, we make this override possible but don't perform the work itself.
        Ok((len, &opus_frame[..0]))
    }

    #[inline]
    fn audio_commands_events(&mut self, tracks: &mut Vec<Track>, interconnect: &Interconnect) {
        for (i, audio) in tracks.iter_mut().enumerate() {
            audio.process_commands(i, interconnect);
        }
    }

    #[allow(unused_variables)]
    pub(crate) async fn cycle(&mut self,
                mut tracks: &mut Vec<Track>,
                audio_timer: &mut Timer,
                bitrate: Bitrate,
                interconnect: &Interconnect,
                muted: bool
            ) -> Result<()> {
        let mut opus_frame_backing = [0u8; STEREO_FRAME_SIZE];
        let mut mix_buffer = [0f32; STEREO_FRAME_SIZE];

        // Slice which mix tracks may use to passthrough direct Opus frames.
        let mut opus_space = &mut opus_frame_backing[..];

        // Walk over all the audio files, combining into one audio frame according
        // to volume, play state, etc.
        let (mut len, mut opus_frame) = self.mix_tracks(&mut tracks, &mut opus_space, &mut mix_buffer, interconnect)?;

        self.soft_clip.apply(&mut mix_buffer[..])?;

        if muted {
            len = 0;
        }

        if len == 0 {
            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                // Explicit "Silence" frame.
                opus_frame = &SILENT_FRAME[..];
            } else {
                // Per official guidelines, send 5x silence BEFORE we stop speaking.
                interconnect.aux_packets.send(AuxPacketMessage::Speaking(false))?;

                audio_timer.hold().await;

                return Ok(());
            }
        } else {
            self.silence_frames = 5;
        }

        interconnect.aux_packets.send(AuxPacketMessage::Speaking(true))?;

        self.prep_and_send_packet(mix_buffer, opus_frame).await?;
        audio_timer.hold().await;

        self.audio_commands_events(&mut tracks, interconnect);

        Ok(())
    }

    pub(crate) fn set_bitrate(&mut self, bitrate: Bitrate) -> Result<()> {
        self.encoder.set_bitrate(bitrate)
            .map_err(Into::into)
    }

    async fn prep_and_send_packet(&mut self, buffer: [f32; 1920], opus_frame: &[u8]) -> Result<()> {
        let mut nonce = Nonce::default();
        let index = {
            let mut rtp = MutableRtpPacket::new(&mut self.packet[..])
                .expect(
                    "[Voice] Too few bytes in self.packet for RTP header.\
                    (Blame: VOICE_PACKET_MAX?)"
                );

            let pkt = rtp.packet();
            let rtp_len = RtpPacket::minimum_packet_size();
            nonce[..rtp_len]
                .copy_from_slice(&pkt[..rtp_len]);

            let payload = rtp.payload_mut();

            let payload_len = if opus_frame.is_empty() {
                self.encoder
                    .encode_float(&buffer[..STEREO_FRAME_SIZE], &mut payload[TAG_SIZE..])?
            } else {
                let len = opus_frame.len();
                payload[TAG_SIZE..TAG_SIZE + len]
                    .clone_from_slice(opus_frame);
                len
            };

            let final_payload_size = TAG_SIZE + payload_len;

            let tag = self.cipher.encrypt_in_place_detached(&nonce, b"", &mut payload[TAG_SIZE..final_payload_size])
                .expect("[Voice] Encryption failed?");
            payload[..TAG_SIZE].copy_from_slice(&tag[..]);

            rtp_len + final_payload_size
        };

        self.udp.send_to(&self.packet[..index], &self.destination).await?;

        let mut rtp = MutableRtpPacket::new(&mut self.packet[..])
            .expect(
                "[Voice] Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)"
            );
        rtp.set_sequence(rtp.get_sequence() + 1);
        rtp.set_timestamp(rtp.get_timestamp() + MONO_FRAME_SIZE as u32);

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
async fn init_cipher(client: &mut WsStream) -> Result<Cipher> {
    loop {
        let value = match client.recv_json().await? {
            Some(value) => value,
            None => continue,
        };

        match VoiceEvent::deserialize(value)? {
            VoiceEvent::SessionDescription(desc) => {
                if desc.mode != CRYPTO_MODE {
                    return Err(Error::Voice(VoiceError::VoiceModeInvalid));
                }

                // TODO: use `XSalsa20Poly1305::new_varkey`. See:
                // <https://github.com/RustCrypto/traits/pull/191>
                if desc.secret_key.len() == KEY_SIZE {
                    let key = Key::from_slice(&desc.secret_key);
                    return Ok(Cipher::new(key));
                } else {
                    return Err(Error::Voice(VoiceError::KeyGen));
                }
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
