mod aux_network;
mod events;

use crate::{
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
use log::{error, warn};
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
    pub(crate) events: MpscSender<EventMessage>,
    pub(crate) aux_packets: MpscSender<AuxPacketMessage>,
}

impl Interconnect {
    fn poison(&self) {
        self.events.send(EventMessage::Poison);
        self.aux_packets.send(AuxPacketMessage::Poison);
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

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice Iface (G{})", guild_id);

    ThreadBuilder::new()
        .name(name)
        .spawn(move || runner(guild_id, &rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));
}

fn start_internals(guild_id: GuildId) -> Interconnect {
    let (evt_tx, evt_rx) = mpsc::channel();
    let (pkt_aux_tx, pkt_aux_rx) = mpsc::channel();

    let interconnect = Interconnect {
        events: evt_tx,
        aux_packets: pkt_aux_tx,
    };

    // FIXME: clean this up...
    // Might need to keep join-handles etc.
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
                let cycle = connection.cycle(&mut tracks, &mut timer, bitrate, &interconnect);

                match cycle {
                    Ok(()) => {
                        // Tick
                        interconnect.events.send(EventMessage::Tick);

                        // Strip expired sources.
                        let mut i = 0;
                        while i < tracks.len() {
                            let aud = tracks.get_mut(i)
                                .expect("[Voice] Tried to remove an illegal track index.");

                            if aud.playing.is_done() {
                                tracks.remove(i);
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
