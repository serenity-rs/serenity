mod aux_network;
mod events;

use crate::{
    error::Error,
    gateway::WsClient,
    internal::Timer,
    model::id::GuildId,
    voice::{
        connection::Connection,
        constants::*,
        events::{
            CoreContext,
            EventData,
            EventStore,
        },
        tracks::{
            LoopState,
            PlayMode,
            TrackHandle,
            TrackState,
        },
        Status,
    },
};
use log::{error, info, warn};
use sodiumoxide::crypto::secretbox::Key;
use std::{
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
    pub(crate) core: MpscSender<CoreMessage>,
    pub(crate) events: MpscSender<EventMessage>,
    pub(crate) aux_packets: MpscSender<AuxPacketMessage>,
}

impl Interconnect {
    fn poison(&self) {
        let _ = self.events.send(EventMessage::Poison);
        let _ = self.aux_packets.send(AuxPacketMessage::Poison);
    }

    fn restart(self, guild_id: GuildId) -> Self {
        self.poison();
        start_internals(guild_id, self.core)
    }
}

pub(crate) enum EventMessage {
    // Event related.
    // Track events should fire off the back of state changes.
    AddGlobalEvent(EventData),
    AddTrackEvent(usize, EventData),
    FireCoreEvent(CoreContext),

    AddTrack(EventStore, TrackState, TrackHandle),
    ChangeState(usize, TrackStateChange),
    RemoveTrack(usize),
    Tick,

    Poison,
}

#[derive(Debug)]
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
    Ws(Box<WsClient>),

    SetSsrc(u32),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}

pub(crate) enum CoreMessage {
    Reconnect,
}

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice Iface (G{})", guild_id);

    ThreadBuilder::new()
        .name(name)
        .spawn(move || runner(guild_id, &rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));
}

fn start_internals(guild_id: GuildId, core: MpscSender<CoreMessage>) -> Interconnect {
    let (evt_tx, evt_rx) = mpsc::channel();
    let (pkt_aux_tx, pkt_aux_rx) = mpsc::channel();

    let interconnect = Interconnect {
        core,
        events: evt_tx,
        aux_packets: pkt_aux_tx,
    };

    let name = format!("Serenity Voice Event Dispatcher (G{})", guild_id);
    let ic = interconnect.clone();
    ThreadBuilder::new()
        .name(name)
        .spawn(move || events::runner(ic, evt_rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

    let name = format!("Serenity Voice Auxiliary Network (G{})", guild_id);
    let ic = interconnect.clone();
    ThreadBuilder::new()
        .name(name)
        .spawn(move || aux_network::runner(ic, pkt_aux_rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

    interconnect
}

fn runner(guild_id: GuildId, rx: &MpscReceiver<Status>) {
    let mut tracks = Vec::new();
    let mut connection = None;
    let mut timer = Timer::new(20);
    let mut bitrate = DEFAULT_BITRATE;

    let (reconnect_tx, reconnect_rx) = mpsc::channel();

    let mut interconnect = start_internals(guild_id, reconnect_tx);

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
                    tracks.clear();

                    if let Some(aud) = s {
                        tracks.push(aud);
                    }
                },
                Ok(Status::AddTrack(mut s)) => {
                    let evts = s.events.take()
                        .unwrap_or_default();
                    let state = s.state();
                    let handle = s.handle.clone();

                    tracks.push(s);

                    let _ = interconnect.events.send(EventMessage::AddTrack(evts, state, handle));
                },
                Ok(Status::SetBitrate(b)) => {
                    bitrate = b;
                },
                Ok(Status::AddEvent(evt)) => {
                    let _ = interconnect.events.send(EventMessage::AddGlobalEvent(evt));
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
                let cycle = connection.cycle(&mut tracks, &mut timer, bitrate, &interconnect);

                match cycle {
                    Ok(()) => {
                        // Send state changes
                        let mut i = 0;
                        let mut to_remove = Vec::with_capacity(tracks.len());
                        while i < tracks.len() {
                            let aud = tracks.get_mut(i)
                                .expect("[Voice] Tried to remove an illegal track index.");

                            if aud.playing.is_done() {
                                let p_state = aud.playing();
                                tracks.remove(i);
                                to_remove.push(i);
                                let _ = interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Mode(p_state)));
                            } else {
                                i += 1;
                            }
                        }

                        // Tick
                        let _ = interconnect.events.send(EventMessage::Tick);

                        // Then do removals.
                        for i in &to_remove[..] {
                            let _ = interconnect.events.send(EventMessage::RemoveTrack(*i));
                        }

                        false
                    },
                    Err(why) => {
                        if matches!(why, Error::VoiceInterconnectFailure) {
                            interconnect = interconnect.restart(guild_id);
                        }

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

        let remote_error = match reconnect_rx.try_recv() {
            Err(TryRecvError::Empty) => {
                false
            },
            _ => {
                true
            },
        };

        // If there was an error, then just reset the connection and try to get
        // another.
        if error || remote_error {
            if let Some(mut conn) = connection.take() {
                // try once: if interconnect, try again.
                // if still issue, full connect.
                let info = conn.connection_info.clone();

                let full_connect = match conn.reconnect(&interconnect) {
                    Ok(()) => {
                        connection = Some(conn);
                        false
                    },
                    Err(Error::VoiceInterconnectFailure) => {
                        interconnect = interconnect.restart(guild_id);

                        match conn.reconnect(&interconnect) {
                            Ok(()) => {
                                connection = Some(conn);
                                false
                            },
                            _ => true,
                        }
                    },
                    _ => true,
                };

                if full_connect {
                    connection = Connection::new(info, &interconnect)
                        .map_err(|e| {error!("[Voice] Catastrophic connection failure. Stopping. {:?}", e); e})
                        .ok();
                }
            }
        }
    }

    info!("[Voice] Main thread exited");
    interconnect.poison();
}
