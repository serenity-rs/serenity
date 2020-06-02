use crate::{
    voice::{
        events::{
            EventStore,
            GlobalEvents,
            TrackEvent,
        },
        threading::{
            EventMessage,
            Interconnect,
            TrackStateChange,
        },
        tracks::{
            TrackHandle,
            TrackState,
        },
    },
};
use log::{debug, error, info, warn};
use std::sync::mpsc::Receiver as MpscReceiver;

pub(crate) fn runner(_interconnect: Interconnect, evt_rx: MpscReceiver<EventMessage>) {
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

                let state = states.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal state index for ChangeState.");

                match change {
                    Mode(mode) => {
                        let old = state.playing;
                        state.playing = mode;
                        if old != mode && mode.is_done() {
                            global.fire_track_event(TrackEvent::End, i);
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
