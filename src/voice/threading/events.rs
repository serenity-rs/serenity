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
use log::{debug, error, info, trace, warn};
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
                info!("[Voice] Global event added.");
                global.add_event(data);
            },
            Ok(AddTrackEvent(i, data)) => {
                info!("[Voice] Adding event to track {}.", i);

                let event_store = events.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal store index for AddTrackEvent.");
                let state = states.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal state index for AddTrackEvent.");

                event_store.add_event(data, state.position);
            },
            Ok(FireCoreEvent(ctx)) => {
                let ctx = ctx.to_user_context();
                let evt = ctx.to_core_event()
                    .expect("[Voice] Event thread was passed a non-core event in FireCoreEvent.");

                trace!("[Voice] Firing core event {:?}.", evt);

                global.fire_core_event(evt, ctx);
            },
            Ok(AddTrack(store, state, handle)) => {
                events.push(store);
                states.push(state);
                handles.push(handle);

                info!("[Voice] Event state for track {} added", events.len());
            },
            Ok(ChangeState(i, change)) => {
                use TrackStateChange::*;

                let max_states = states.len();
                let state = states.get_mut(i)
                    .expect("[Voice] Event thread was given an illegal state index for ChangeState.");

                debug!("[Voice] Changing state for track {} of {}: {:?}", i, max_states, change);

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
                        // Massive, unprecedented state changes.
                        *state = new;
                    },
                }
            },
            Ok(RemoveTrack(i)) => {
                info!("[Voice] Event state for track {} of {} removed.", i, events.len());

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
