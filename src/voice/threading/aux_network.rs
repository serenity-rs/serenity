use audiopus::{
    Channels,
    coder::Decoder as OpusDecoder,
};
use crate::{
    error::Result,
    gateway::WsClient,
    internal::{
        ws_impl::{ReceiverExt, SenderExt},
        Timer,
    },
    model::event::{VoiceEvent, VoiceSpeakingState},
    voice::{
        constants::*,
        error::VoiceError,
        events::{
            CoreContext,
        },
        payload,
        threading::{
            AuxPacketMessage,
            CoreMessage,
            EventMessage,
            Interconnect,
        },
    },
};
use discortp::{
    demux::{
        self,
        DemuxedMut,
    },
    discord::MutableKeepalivePacket,
    rtp::{
        RtpExtensionPacket,
        RtpPacket,
    },
    FromPacket,
    MutablePacket,
    Packet,
    PacketSize,
};
use log::{debug, error, info, warn};
use rand::random;
use serde::Deserialize;
use sodiumoxide::crypto::secretbox::{
    self,
    MACBYTES,
    NONCEBYTES,
    Key,
    Tag,
};
use std::{
    collections::HashMap,
    net::{
        SocketAddr,
        UdpSocket,
    },
    sync::mpsc::{
        Receiver as MpscReceiver,
        TryRecvError,
    },
};

#[derive(Debug)]
struct SsrcState {
    silent_frame_count: u16,
    decoder: OpusDecoder,
    last_seq: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpeakingDelta {
    Same,
    Start,
    Stop,
}

impl SsrcState {
    fn new(pkt: RtpPacket<'_>) -> Self {
        Self {
            silent_frame_count: 5, // We do this to make the first speech packet fire an event.
            decoder: OpusDecoder::new(SAMPLE_RATE, Channels::Stereo)
                .expect("[Voice] Failed to create new Opus decoder for source.",),
            last_seq: pkt.get_sequence().into(),
        }
    }

    fn process(&mut self, pkt: RtpPacket<'_>, data_offset: usize) -> Result<(SpeakingDelta, Vec<i16>)> {
        let new_seq: u16 = pkt.get_sequence().into();

        let extensions = pkt.get_extension() != 0;
        let seq_delta = new_seq.wrapping_sub(self.last_seq);
        Ok(if seq_delta >= (1 << 15) {
            // Overflow, reordered (previously missing) packet.
            (SpeakingDelta::Same, vec![])
        } else {
            self.last_seq = new_seq;
            let missed_packets = seq_delta.saturating_sub(1);
            let (audio, pkt_size) = self.scan_and_decode(&pkt.payload()[data_offset..], extensions, missed_packets)?;

            let delta = if pkt_size == SILENT_FRAME.len() {
                // Frame is silent.
                let old = self.silent_frame_count;
                self.silent_frame_count = self.silent_frame_count.saturating_add(1 + missed_packets);

                if self.silent_frame_count >= 5 && old < 5 {
                    SpeakingDelta::Stop
                } else {
                    SpeakingDelta::Same
                }
            } else {
                // Frame has meaningful audio.
                let out = if self.silent_frame_count >= 5 {
                    SpeakingDelta::Start
                } else {
                    SpeakingDelta::Same
                };
                self.silent_frame_count = 0;
                out
            };

            (delta, audio)
        })
    }

    fn scan_and_decode(&mut self, data: &[u8], extension: bool, missed_packets: u16) -> Result<(Vec<i16>, usize)> {
        let mut out = vec![0; STEREO_FRAME_SIZE];
        let start = if extension {
            RtpExtensionPacket::new(data)
                .map(|pkt| pkt.packet_size())
                .ok_or_else(|| {
                    error!("[Voice] Extension packet indicated, but insufficient space.");
                    VoiceError::IllegalVoicePacket
                })
        } else {
            Ok(0)
        }?;

        for _ in 0..missed_packets {
            let missing_frame: Option<&[u8]> = None;
            if let Err(e) = self.decoder.decode(missing_frame, &mut out[..], false) {
                warn!("[Voice] Issue while decoding for missed packet: {:?}.", e);
            }
        }

        let audio_len = self.decoder.decode(Some(&data[start..]), &mut out[..], false)
            .map_err(|e| {
                error!("[Voice] Failed to decode received packet: {:?}.", e);
                e
            })?;

        out.truncate(audio_len);

        Ok((out, data.len() - start))
    }
}

struct AuxNetwork {
    rx: MpscReceiver<AuxPacketMessage>,

    udp_socket: Option<UdpSocket>,
    ws_client: Option<WsClient>,
    destination: Option<SocketAddr>,
    key: Option<Key>,
    packet_buffer: [u8; VOICE_PACKET_MAX],

    ssrc: u32,
    keepalive_bytes: [u8; MutableKeepalivePacket::minimum_packet_size()],
    ws_keepalive_time: Timer,
    udp_keepalive_time: Timer,

    speaking: VoiceSpeakingState,
    last_heartbeat_nonce: Option<u64>,
    decoder_map: HashMap<u32, SsrcState>,

    should_parse: bool,
}

impl AuxNetwork {
    pub(crate) fn new(evt_rx: MpscReceiver<AuxPacketMessage>) -> Self {
        Self {
            rx: evt_rx,

            udp_socket: None,
            ws_client: None,
            destination: None,
            key: None,
            packet_buffer: [0u8; VOICE_PACKET_MAX],

            ssrc: 0,
            keepalive_bytes: [0u8; MutableKeepalivePacket::minimum_packet_size()],
            ws_keepalive_time: Timer::new(45_000),
            udp_keepalive_time: Timer::new(UDP_KEEPALIVE_GAP_MS),

            speaking: VoiceSpeakingState::empty(),
            last_heartbeat_nonce: None,
            decoder_map: Default::default(),

            // FIXME: should be hinted at by event thread.
            should_parse: true,
        }
    }

    fn run(&mut self, interconnect: &Interconnect) {
        'aux_runner: loop {
            let mut ws_error = match self.process_ws_messages(interconnect) {
                Err(e) => {
                    error!("[Voice] Error processing ws {:?}.", e);
                    true
                },
                _ => false,
            };

            self.process_udp_messages(interconnect);

            loop {
                use AuxPacketMessage::*;
                match self.rx.try_recv() {
                    Ok(Udp(udp)) => {
                        let _ = udp.set_read_timeout(Some(TIMESTEP_LENGTH / 2));

                        self.udp_socket = Some(udp);
                        self.udp_keepalive_time.reset();
                    },
                    Ok(UdpKey(new_key)) => {
                        self.key = Some(new_key);
                    },
                    Ok(UdpDestination(addr)) => {
                        self.destination = Some(addr);
                    }
                    Ok(Ws(data)) => {
                        self.ws_client = Some(*data);
                        self.ws_keepalive_time.reset();
                    },
                    Ok(SetSsrc(new_ssrc)) => {
                        self.ssrc = new_ssrc;
                        let mut ka = MutableKeepalivePacket::new(&mut self.keepalive_bytes[..])
                            .expect("[Voice] Insufficient bytes given to keepalive packet.");
                        ka.set_ssrc(new_ssrc);
                    },
                    Ok(SetKeepalive(keepalive)) => {
                        self.ws_keepalive_time = Timer::new(keepalive as u64);
                    }
                    Ok(Speaking(is_speaking)) => {
                        if self.speaking.contains(VoiceSpeakingState::MICROPHONE) != is_speaking {
                            self.speaking.set(VoiceSpeakingState::MICROPHONE, is_speaking);    
                            if let Some(client) = self.ws_client.as_mut() {
                                ws_error |= match client.send_json(&payload::build_speaking(self.speaking, self.ssrc)) {
                                    Err(e) => {
                                        error!("[Voice] Issue sending speaking update {:?}.", e);
                                        true
                                    },
                                    _ => false,
                                }
                            }
                        }
                    }
                    Err(TryRecvError::Disconnected) | Ok(Poison) => {
                        break 'aux_runner;
                    },
                    Err(_) => {
                        // No message.
                        break;
                    }
                }
            }

            if ws_error {
                self.ws_client = None;
                let _ = interconnect.core.send(CoreMessage::Reconnect);
            }
        }

        info!("[Voice] Auxiliary network thread exited");
    }

    fn process_ws_messages(&mut self, interconnect: &Interconnect) -> Result<()> {
        if let Some(ws) = self.ws_client.as_mut() {
            if self.ws_keepalive_time.check() {
                let nonce = random::<u64>();
                self.last_heartbeat_nonce = Some(nonce);
                ws.send_json(&payload::build_heartbeat(nonce))?;
                self.ws_keepalive_time.reset_from_deadline();
            }

            // FIXME: need to propagate WS disconnection back to main thread to trigger reconnect.

            while let Ok(Some(value)) = ws.try_recv_json() {
                let msg = match VoiceEvent::deserialize(&value) {
                    Ok(m) => m,
                    Err(_) => {
                        warn!("[Voice] Unexpected Websocket message: {:?}", value);
                        break
                    },
                };

                match msg {
                    VoiceEvent::Speaking(ev) => {
                        let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                            CoreContext::SpeakingStateUpdate(ev)
                        ));
                    },
                    VoiceEvent::ClientConnect(ev) => {
                        let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                            CoreContext::ClientConnect(ev)
                        ));
                    },
                    VoiceEvent::ClientDisconnect(ev) => {
                        let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                            CoreContext::ClientDisconnect(ev)
                        ));
                    },
                    VoiceEvent::HeartbeatAck(ev) => {
                        if let Some(nonce) = self.last_heartbeat_nonce.take() {
                            if ev.nonce == nonce {
                                info!("[Voice] Heartbeat ACK received.");
                            } else {
                                warn!("[Voice] Heartbeat nonce mismatch! Expected {}, saw {}.", nonce, ev.nonce);
                            }
                        }
                    },
                    other => {
                        info!("[Voice] Received other websocket data: {:?}", other);
                    },
                }
            }
        }
        Ok(())
    }

    fn process_udp_messages(&mut self, interconnect: &Interconnect) {
        // NOTE: errors here (and in general for UDP) are not fatal to the connection.
        // Panics should be avoided due to adversarial nature of rx'd packets,
        // but correct handling should not prompt a reconnect.
        if let Some(udp) = self.udp_socket.as_mut() {
            if self.udp_keepalive_time.check() {
                let _ = udp.send_to(
                    &self.keepalive_bytes,
                    self.destination.expect("[Voice] Tried to send keepalive without valid destination.")
                );
                self.udp_keepalive_time.reset_from_deadline();
            }

            while let Ok((len, _addr)) = udp.recv_from(&mut self.packet_buffer[..]) {
                if !self.should_parse {
                    continue;
                }

                let packet = &mut self.packet_buffer[..len];
                let key = self.key.as_ref().expect("[Voice] Tried to decrypt without a valid key.");

                match demux::demux_mut(packet) {
                    DemuxedMut::Rtp(mut rtp) => {
                        if !rtp_valid(rtp.to_immutable()) {
                            error!("[Voice] Illegal RTP message received.");
                            continue;
                        }

                        let rtp_body_start = decrypt_in_place(
                            &mut rtp,
                            key,
                        ).expect("[Voice] RTP decryption failed.");

                        let entry = self.decoder_map.entry(rtp.get_ssrc())
                            .or_insert_with(
                                || SsrcState::new(rtp.to_immutable()),
                            );

                        if let Ok((delta, audio)) = entry.process(rtp.to_immutable(), rtp_body_start) {
                            match delta {
                                SpeakingDelta::Start => {
                                    let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                                        CoreContext::SpeakingUpdate {
                                            ssrc: rtp.get_ssrc(),
                                            speaking: true,
                                        },
                                    ));
                                },
                                SpeakingDelta::Stop => {
                                    let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                                        CoreContext::SpeakingUpdate {
                                            ssrc: rtp.get_ssrc(),
                                            speaking: false,
                                        },
                                    ));
                                },
                                _ => {},
                            }

                            let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                                CoreContext::VoicePacket {
                                    audio,
                                    packet: rtp.from_packet(),
                                    payload_offset: rtp_body_start,
                                },
                            ));
                        } else {
                            warn!("[Voice] RTP decoding/decrytion failed.");
                        }
                    },
                    DemuxedMut::Rtcp(mut rtcp) => {
                        let rtcp_body_start = decrypt_in_place(
                            &mut rtcp,
                            key,
                        );

                        if let Ok(start) = rtcp_body_start {
                            let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                                CoreContext::RtcpPacket {
                                    packet: rtcp.from_packet(),
                                    payload_offset: start,
                                },
                            ));
                        } else {
                            warn!("[Voice] RTCP decryption failed.");
                        }
                    },
                    DemuxedMut::FailedParse(t) => {
                        warn!("[Voice] Failed to parse message of type {:?}.", t);
                    }
                    _ => {
                        warn!("[Voice] Illegal UDP packet from voice server.");
                    }
                }
            }
        }
    }
}

pub(crate) fn runner(interconnect: Interconnect, evt_rx: MpscReceiver<AuxPacketMessage>) {
    let mut aux = AuxNetwork::new(evt_rx);

    aux.run(&interconnect);
}

#[inline]
fn decrypt_in_place(packet: &mut impl MutablePacket, key: &Key) -> Result<usize> {
    // Applies discord's cheapest.
    // In future, might want to make a choice...
    let header_len = packet.packet().len() - packet.payload().len();
    let mut nonce = secretbox::Nonce([0; NONCEBYTES]);
    nonce.0[..header_len]
        .copy_from_slice(&packet.packet()[..header_len]);

    let data = packet.payload_mut();
    let (tag_bytes, data_bytes) = data.split_at_mut(MACBYTES);
    let tag = Tag::from_slice(tag_bytes)
        .ok_or_else(|| {
            error!("[Voice] failed to extract enough bytes for encryption tag.");
            VoiceError::Decryption
        })?;

    secretbox::open_detached(data_bytes, &tag, &nonce, key)
        .map(|_| MACBYTES)
        .map_err(|_| VoiceError::Decryption.into())
}

#[inline]
fn rtp_valid(packet: RtpPacket<'_>) -> bool {
    packet.get_version() == RTP_VERSION
        && packet.get_payload_type() == RTP_PROFILE_TYPE
}
