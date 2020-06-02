use audiopus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    coder::Decoder as OpusDecoder,
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
};
use crate::{
    gateway::WsClient,
    internal::{
        ws_impl::{ReceiverExt, SenderExt},
        Timer,
    },
    model::{
        event::{VoiceEvent, VoiceSpeakingState},
        id::GuildId,
    },
    voice::{
        connection::Connection,
        constants::*,
        events::{
            CoreEvent,
            EventContext,
            EventData,
            EventStore,
            GlobalEvents,
            TrackEvent,
            UntimedEvent,
        },
        payload,
        tracks::{
            LoopState,
            PlayMode,
            TrackHandle,
            TrackState,
        },
        Status,
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
        MutableKeepalivePacket,
    },
    rtp::{
        MutableRtpPacket,
        RtpPacket,
        RtpType,
    },
    MutablePacket,
    Packet,
};
use log::{debug, error, info, warn};
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
    io::Result as IoResult,
    net::{
        SocketAddr,
        UdpSocket,
    },
    sync::mpsc::{
        self,
        Receiver as MpscReceiver,
        Sender as MpscSender,
        TryRecvError,
    },
    thread::Builder as ThreadBuilder,
    time::Duration,
};

#[derive(Clone, Debug)]
pub(crate) struct Interconnect {
    pub(crate) events: MpscSender<EventMessage>,
    pub(crate) mixer: MpscSender<()>,
    pub(crate) voice_packets: MpscSender<()>,
    pub(crate) aux_packets: MpscSender<AuxPacketMessage>,
}

impl Interconnect {
    fn poison(&self) {
        self.events.send(EventMessage::Poison);
        self.mixer.send(());
        self.voice_packets.send(());
        self.aux_packets.send(AuxPacketMessage::Poison);
    }
}

pub(crate) enum EventMessage {
    // Event related.
    // Track events should fire off the back of state changes.
    AddGlobalEvent(EventData),
    AddTrackEvent(usize, EventData),
    FireCoreEvent(EventContext),

    AddTrack(EventStore, TrackState, TrackHandle),
    ChangeState(usize, TrackStateChange),
    RemoveTrack(usize),
    Tick,

    Poison,
}

pub(crate) enum TrackStateChange {
    Mode(PlayMode),
    Volume(f32),
    Position(Duration),
    // Bool indicates user-set.
    Loops(LoopState, bool),
    Total(TrackState),
}

pub(crate) enum AuxPacketMessage {
    Udp(UdpSocket),
    UdpDestination(SocketAddr),
    UdpKey(Key),
    Ws(WsClient),

    SetSsrc(u32),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice Iface (G{})", guild_id);

    ThreadBuilder::new()
        .name(name)
        .spawn(move || runner(guild_id, &rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));
}

fn start_internals(guild_id: GuildId) -> Interconnect {
    let (evt_tx, evt_rx) = mpsc::channel();
    let (mixer_tx, mixer_rx) = mpsc::channel();
    let (pkt_out_tx, pkt_out_rx) = mpsc::channel();
    let (pkt_in_tx, pkt_in_rx) = mpsc::channel();

    let interconnect = Interconnect {
        events: evt_tx,
        mixer: mixer_tx,
        voice_packets: pkt_out_tx,
        aux_packets: pkt_in_tx,
    };

    // FIXME: clean this up...
    // Might need to keep join-handles etc.
    let name = format!("Serenity Voice Event Dispatcher (G{})", guild_id);

    let ic = interconnect.clone();
    ThreadBuilder::new()
        .name(name)
        .spawn(move || evt_runner(ic, evt_rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

    let name = format!("Serenity Voice Auxiliary Network (G{})", guild_id);

    let ic = interconnect.clone();
    ThreadBuilder::new()
        .name(name)
        .spawn(move || aux_runner(ic, pkt_in_rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

    interconnect
}

fn evt_runner(interconnect: Interconnect, evt_rx: MpscReceiver<EventMessage>) {
    let mut global = GlobalEvents::default();

    let mut events: Vec<EventStore> = vec![];
    let mut states: Vec<TrackState> = vec![];
    let mut handles: Vec<TrackHandle> = vec![];

    loop {
        use EventMessage::*;
        match evt_rx.recv() {
            Ok(AddGlobalEvent(data)) => {
                global.add_event(data);
            },
            Ok(AddTrackEvent(i, data)) => {
                let event_store = events.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal store index for AddTrackEvent.");
                let state = states.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal state index for AddTrackEvent.");

                event_store.add_event(data, state.position);
            },
            Ok(FireCoreEvent(ctx)) => {
                let evt = ctx.to_core_event()
                    .expect("[Voice] Event thread was passed a non-core event in FireCoreEvent.");
                global.fire_core_event(evt, ctx);
            },
            Ok(AddTrack(store, state, handle)) => {
                events.push(store);
                states.push(state);
                handles.push(handle);
            },
            Ok(ChangeState(i, change)) => {
                use TrackStateChange::*;

                let event_store = events.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal store index for AddTrackEvent.");
                let state = states.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal state index for ChangeState.");

                match change {
                    Mode(mode) => {
                        let old = state.playing;
                        state.playing = mode;
                        if old != mode && mode.is_done() {
                            global.fire_track_event(TrackEvent::End, i);

                            // Save this for the tick!
                            // event_store.process_untimed(global.time, TrackEvent::into(), );
                        }
                    },
                    Volume(vol) => {state.volume = vol;},
                    Position(pos) => {
                        // Currently, only Tick should fire time events.
                        state.position = pos;
                    },
                    Loops(loops, user_set) => {
                        state.loops = loops;
                        if !user_set {
                            global.fire_track_event(TrackEvent::Loop, i);
                        }
                    },
                    Total(new) => {
                        // Massive, unprecendented state changes.
                        *state = new;
                    },
                }
            },
            Ok(RemoveTrack(i)) => {
                events.remove(i);
                states.remove(i);
                handles.remove(i);
            },
            Ok(Tick) => {
                // NOTE: this should fire saved up blocks of state change evts.
                global.tick(&mut events, &mut states, &mut handles);
            },
            Err(_) | Ok(Poison) => {
                break;
            },
        }
    }
}

struct SsrcState {
    silent_frame_count: usize,
    decoder: OpusDecoder,
}

fn decrypt_udp_in_place() -> Result<(), ()> {
    Ok(())
}

fn aux_runner(interconnect: Interconnect, evt_rx: MpscReceiver<AuxPacketMessage>) {
    // FIXME: should be hinted at by event thread.
    let mut should_decrypt = true;

    let mut udp_buffer = [0u8; VOICE_PACKET_MAX];
    let mut udp_key = None;
    let mut udp_socket: Option<UdpSocket> = None;
    let mut ws_client: Option<WsClient> = None;
    let mut destination = None;

    let mut ws_ka_time = Timer::new(45_000);
    let mut udp_ka_time = Timer::new(5_000);

    let mut ssrc = 0;
    let mut keepalive_bytes = [0u8; MutableKeepalivePacket::minimum_packet_size()];

    let mut speaking = VoiceSpeakingState::empty();
    let mut last_heartbeat_nonce: Option<u64> = None;
    let mut decoder_map: HashMap<u32, SsrcState> = Default::default();

    'aux_runner: loop {
        if let Some(ws) = ws_client.as_mut() {
            if ws_ka_time.check() {
                let nonce = random::<u64>();
                last_heartbeat_nonce = Some(nonce);
                ws.send_json(&payload::build_heartbeat(nonce));
                ws_ka_time.reset_from_deadline();
            }

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
                        interconnect.events.send(EventMessage::FireCoreEvent(
                            EventContext::SpeakingStateUpdate {
                                ssrc: ev.ssrc,
                                user_id: ev.user_id.0,
                                speaking_state: ev.speaking,
                            }
                        ));
                    },
                    VoiceEvent::ClientConnect(ev) => {
                        interconnect.events.send(EventMessage::FireCoreEvent(
                            EventContext::ClientConnect {
                                audio_ssrc: ev.audio_ssrc,
                                video_ssrc: ev.video_ssrc,
                                user_id: ev.user_id.0,
                            }
                        ));
                    },
                    VoiceEvent::ClientDisconnect(ev) => {
                        interconnect.events.send(EventMessage::FireCoreEvent(
                            EventContext::ClientDisconnect {
                                user_id: ev.user_id.0,
                            }
                        ));
                    },
                    VoiceEvent::HeartbeatAck(ev) => {
                        if let Some(nonce) = last_heartbeat_nonce {
                            if ev.nonce == nonce {
                                info!("[Voice] Heartbeat ACK received.");
                            } else {
                                warn!("[Voice] Heartbeat nonce mismatch! Expected {}, saw {}.", nonce, ev.nonce);
                            }

                            last_heartbeat_nonce = None;
                        }
                    },
                    other => {
                        info!("[Voice] Received other websocket data: {:?}", other);
                    },
                }
            }
        }

        if let Some(udp) = udp_socket.as_mut() {
            if udp_ka_time.check() {
                udp.send_to(
                    &keepalive_bytes,
                    destination.expect("[Voice] Tried to send keepalive without valid destination.")
                );
                udp_ka_time.reset_from_deadline();
            }

            while let Ok((len, _addr)) = udp.recv_from(&mut udp_buffer[..]) {
                if !should_decrypt {
                    continue;
                }

                let packet = &udp_buffer[..len];

                match demux::demux(packet) {
                    Demuxed::Rtp(rtp) => {
                        println!("{:?}", rtp);
                        println!("{:?}", rtp.payload());
                    },
                    Demuxed::Rtcp(rtcp) => {
                        println!("{:?}", rtcp);
                        println!("{:?}", rtcp.payload());
                    },
                    Demuxed::FailedParse(t) => {
                        warn!("[Voice] Failed to parse message of type {:?}.", t);
                    }
                    _ => {
                        warn!("[Voice] Illegal UDP packet from voice server.");
                    }
                }
            }
        }

        loop {
            use AuxPacketMessage::*;
            match evt_rx.try_recv() {
                Ok(Udp(udp)) => {
                    let _ = udp.set_read_timeout(Some(TIMESTEP_LENGTH / 2));

                    udp_socket = Some(udp);
                    udp_ka_time.reset();
                },
                Ok(UdpKey(new_key)) => {
                    udp_key = Some(new_key);
                },
                Ok(UdpDestination(addr)) => {
                    destination = Some(addr);
                }
                Ok(Ws(data)) => {
                    ws_client = Some(data);
                    ws_ka_time.reset();
                },
                Ok(SetSsrc(new_ssrc)) => {
                    ssrc = new_ssrc;
                    let mut ka = MutableKeepalivePacket::new(&mut keepalive_bytes[..])
                        .expect("[Voice] Insufficient bytes given to keepalive packet.");
                    ka.set_ssrc(new_ssrc);
                },
                Ok(SetKeepalive(keepalive)) => {
                    ws_ka_time = Timer::new(keepalive as u64);
                }
                Ok(Speaking(is_speaking)) => {
                    if speaking.contains(VoiceSpeakingState::MICROPHONE) != is_speaking {
                        speaking.set(VoiceSpeakingState::MICROPHONE, is_speaking);    
                        if let Some(client) = ws_client.as_mut() {
                            client.send_json(&payload::build_speaking(speaking, ssrc));
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
    }
}

fn runner(guild_id: GuildId, rx: &MpscReceiver<Status>) {
    let mut senders = Vec::new();
    let mut connection = None;
    let mut timer = Timer::new(20);
    let mut bitrate = DEFAULT_BITRATE;
    let mut time_in_call = Duration::default();
    let mut entry_points = 0u64;

    let interconnect = start_internals(guild_id);

    'runner: loop {
        loop {
            match rx.try_recv() {
                Ok(Status::Connect(info)) => {
                    connection = match Connection::new(info, &interconnect) {
                        Ok(connection) => Some(connection),
                        Err(why) => {
                            warn!("[Voice] Error connecting: {:?}", why);

                            None
                        },
                    };
                },
                Ok(Status::Disconnect) => {
                    connection = None;
                },
                Ok(Status::SetTrack(s)) => {
                    senders.clear();

                    if let Some(aud) = s {
                        senders.push(aud);
                    }
                },
                Ok(Status::AddTrack(mut s)) => {
                    let evts = s.events.take()
                        .unwrap_or_default();
                    let state = s.state();
                    let handle = s.handle.clone();

                    senders.push(s);

                    interconnect.events.send(EventMessage::AddTrack(evts, state, handle));
                },
                Ok(Status::SetBitrate(b)) => {
                    bitrate = b;
                },
                Ok(Status::AddEvent(evt)) => {
                    interconnect.events.send(EventMessage::AddGlobalEvent(evt));
                }
                Err(TryRecvError::Empty) => {
                    // If we received nothing, then we can perform an update.
                    break;
                },
                Err(TryRecvError::Disconnected) => {
                    break 'runner;
                },
            }
        }

        // Overall here, check if there's an error.
        //
        // If there is a connection, try to send an update. This should not
        // error. If there is though for some spurious reason, then set `error`
        // to `true`.
        //
        // Otherwise, wait out the timer and do _not_ error and wait to receive
        // another event.
        let error = match connection.as_mut() {
            Some(connection) => {
                let cycle = connection.cycle(&mut senders, &mut timer, bitrate, &mut time_in_call, &mut entry_points, &interconnect);

                match cycle {
                    Ok(()) => {
                        // Tick
                        interconnect.events.send(EventMessage::Tick);

                        // Strip expired sources.
                        let mut i = 0;
                        while i < senders.len() {
                            let aud = senders.get_mut(i)
                                .expect("[Voice] Tried to remove an illegal track index.");

                            if aud.playing.is_done() {
                                senders.remove(i);
                                interconnect.events.send(EventMessage::RemoveTrack(i));
                            } else {
                                i += 1;
                            }
                        }
                        false
                    },
                    Err(why) => {
                        error!(
                            "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                            why
                        );

                        true
                    },
                }
            },
            None => {
                timer.r#await();

                false
            },
        };

        // If there was an error, then just reset the connection and try to get
        // another.
        if error {
            let mut conn = connection.expect("[Voice] Shouldn't have had a voice connection error without a connection.");
            connection = conn.reconnect(&interconnect)
                .ok()
                .map(|_| conn);
        }
    }
}
