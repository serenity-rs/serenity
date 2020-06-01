use crate::{
    internal::Timer,
    model::id::GuildId,
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
use std::{
    collections::HashMap,
    io::Result as IoResult,
    sync::mpsc::{
        self,
        Receiver as MpscReceiver,
        Sender as MpscSender,
        TryRecvError,
    },
    thread::Builder as ThreadBuilder,
    time::Duration,
};

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
        packet_transmitter: pkt_out_tx,
        packet_receiver: pkt_in_tx,
    };

    // FIXME: clean this up...
    // Might need to keep join-handles etc.
    let name = format!("Serenity Voice Event Dispatcher (G{})", guild_id);

    let ic = interconnect.clone();
    ThreadBuilder::new()
        .name(name)
        .spawn(move || evt_runner(ic, evt_rx))
        .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

    // let name = format!("Serenity Voice Event Dispatcher (G{})", guild_id);
    // let ic = interconnect.clone();
    // ThreadBuilder::new()
    //     .name(name)
    //     .spawn(move || evt_runner(ic, evt_rx))
    //     .unwrap_or_else(|_| panic!("[Voice] Error starting guild: {:?}", guild_id));

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
            Ok(FireCoreEvent(evt, ctx)) => {
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

#[derive(Clone, Debug)]
pub(crate) struct Interconnect {
    pub(crate) events: MpscSender<EventMessage>,
    pub(crate) mixer: MpscSender<()>,
    pub(crate) packet_transmitter: MpscSender<()>,
    pub(crate) packet_receiver: MpscSender<()>,
}

pub(crate) enum EventMessage {
    // Event related.
    // Track events should fire off the back of state changes.
    AddGlobalEvent(EventData),
    AddTrackEvent(usize, EventData),
    FireCoreEvent(CoreEvent, EventContext),

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

fn runner(guild_id: GuildId, rx: &MpscReceiver<Status>) {
    let mut senders = Vec::new();
    let mut receiver = None;
    let mut connection = None;
    let mut timer = Timer::new(20);
    let mut bitrate = DEFAULT_BITRATE;
    let mut events = GlobalEvents::default();
    let mut fired_track_evts = HashMap::new();
    let mut time_in_call = Duration::default();
    let mut entry_points = 0u64;

    let interconnect = start_internals(guild_id);

    'runner: loop {
        loop {
            match rx.try_recv() {
                Ok(Status::Connect(info)) => {
                    connection = match Connection::new(info) {
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
                Ok(Status::SetReceiver(r)) => {
                    receiver = r;
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
                    events.add_event(evt);
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
                let cycle = connection.cycle(&mut senders, &mut receiver, &mut fired_track_evts, &mut timer, bitrate, &mut time_in_call, &mut entry_points, &interconnect);

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
            connection = conn.reconnect()
                .ok()
                .map(|_| conn);
        }
    }
}
