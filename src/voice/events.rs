use crate::{
	model::event::VoiceSpeakingState,
	voice::{
		constants::*,
		tracks::{
			PlayMode,
			Track,
			TrackHandle,
			TrackState,
		},
	},
};
use std::{
	cmp::Ordering,
	collections::{
		BinaryHeap,
		HashMap,
	},
	sync::Arc,
	time::Duration,
};

/// Information about which tracks, if any, fired an event.
///
/// Local events ([`Track`]-specific) are guaranteed to have
/// an attached track, while global timing events will not.
///
/// [`Track`]: struct.Track.html
/// [`Handler::add_global_event`]: struct.Handler.html#method.add_global_event
#[derive(Clone, Debug)]
pub enum EventContext {
	/// Local event context, passed to events created via [`TrackHandle::add_event`]
	/// or [`EventStore::add_event`].
	///
	/// [`EventStore::add_event`]: struct.EventStore.html#method.add_event
	/// [`TrackHandle::add_event`]: struct.TrackHandle.html#method.add_event
	Track(Vec<(TrackState, TrackHandle)>),

	SpeakingStateUpdate {
		ssrc: u32,
		user_id: u64,
		speaking_state: VoiceSpeakingState,
	},

	SpeakingUpdate {
		ssrc: u32,
		speaking: bool,
	},

	VoicePacket {
		ssrc: u32,
        sequence: u16,
        timestamp: u32,
        stereo: bool,
        data: Vec<i16>,
        compressed_size: usize
	},

	RtcpPacket {
		data: u32, // FIXME.
	},

	ClientConnect {
		ssrc: u32,
		user_id: u64,
	},

	ClientDisconnect {
		user_id: u64,	
	},
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Classes of event which may occur, triggering a handler
/// at the local (track-specific) or global level.
///
/// Local time-based events rely upon the current playback
/// time of a track, and so will not fire if a track becomes paused
/// or stops. In case this is required, global events are a better
/// fit.
///
/// Event handlers themselves are described in [`EventData::action`].
///
/// [`EventData::action`]: struct.EventData.html#method.action
pub enum Event {
	/// Periodic events rely upon two parameters: a *period*
	/// and an optional *phase*.
	///
	/// If the *phase* is `None`, then the event will first fire
	/// in one *period*. Periodic events repeat automatically
	/// so long as the `action` in [`EventData`] returns `None`.
	///
	/// [`EventData`]: struct.EventData.html
	Periodic(Duration, Option<Duration>),

	/// Delayed events rely upon a *delay* parameter, and
	/// fire one *delay* after the audio context processes them.
	///
	/// Delayed events are automatically removed once fired,
	/// so long as the `action` in [`EventData`] returns `None`.
	///
	/// [`EventData`]: struct.EventData.html
	Delayed(Duration),

	/// Track events correspond to certain actions or changes
	/// of state, such as a track finishing, looping, or being
	/// manually stopped.
	///
	/// Track events persist while the `action` in [`EventData`]
	/// returns `None`.
	///
	/// [`EventData`]: struct.EventData.html
	Track(TrackEvent),

	/// Core events
	///
	/// Track events persist while the `action` in [`EventData`]
	/// returns `None`. Core events **must** be applied globally,
	/// as attaching them to a track is a no-op.
	///
	/// [`EventData`]: struct.EventData.html
	Core(CoreEvent),

	/// Cancels the event, if it was intended to persist.
	Cancel,
}

impl Event {
	pub(crate) fn is_global_only(&self) -> bool {
		matches!(self, Self::Core(_))
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Voice core events occur on receipt of
/// voice packets and telemetry.
///
/// Core events persist while the `action` in [`EventData`]
/// returns `None`.
///
/// [`EventData`]: struct.EventData.html
pub enum CoreEvent {
    /// Fired on receipt of a speaking state update from another host.
    ///
    /// Note: this will fire when a user starts speaking for the first time,
    /// or changes their capabilities. 
	SpeakingStateUpdate,

    /// Fires when a source starts speaking, or stops speaking
    /// (*i.e.*, 5 consecutive silent frames).
	SpeakingUpdate,

	/// Fires on receipt of a voice packet from another stream in the voice call.
    ///
    /// As RTP packets do not map to Discord's notion of users, SSRCs must be mapped
    /// back using the user IDs seen through client connection, disconnection,
    /// or speaking state update.
	VoicePacket,

	/// Fires on receipt of an RTCP packet, containing various call stats
    /// such as latency reports.
	RtcpPacket,

	/// Fires whenever a user connects to the same stream as the bot.
	ClientConnect,

	/// Fires whenever a user disconnects from the same stream as the bot.
	ClientDisconnect,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Track and voice core events.
///
/// Untimed events persist while the `action` in [`EventData`]
/// returns `None`.
///
/// [`EventData`]: struct.EventData.html
pub enum UntimedEvent {
	Track(TrackEvent),
	Core(CoreEvent),
}

impl From<TrackEvent> for UntimedEvent {
	fn from(evt: TrackEvent) -> Self {
		UntimedEvent::Track(evt)
	}
}

impl From<CoreEvent> for UntimedEvent {
	fn from(evt: CoreEvent) -> Self {
		UntimedEvent::Core(evt)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Track events correspond to certain actions or changes
/// of state, such as a track finishing, looping, or being
/// manually stopped. Voice core events occur on receipt of
/// voice packets and telemetry.
///
/// Track events persist while the `action` in [`EventData`]
/// returns `None`.
///
/// [`EventData`]: struct.EventData.html
pub enum TrackEvent {
	/// The attached track has ended. (// TODO: separate actual end with deliberate)
	End,

	/// The attached track has looped.
	Loop,
}

/// Internal representation of an event, as handled by the audio context.
pub struct EventData {
	event: Event,
	fire_time: Option<Duration>,
	action: Box<dyn FnMut(EventContext) -> Option<Event> + Send + Sync + 'static>,
}

impl EventData {
	/// Create a representation of an event and its associated handler.
	///
	/// An event handler, `action`, receives an [`EventContext`] and optionally
	/// produces a new [`Event`] type for itself. Returning `None` will
	/// maintain the same event type, while removing any [`Delayed`] entries.
	/// Event handlers will be re-added with their new trigger condition,
	/// or removed if [`Cancel`]led
	///
	/// [`EventContext`]: enum.EventContext.html
	/// [`Event`]: enum.Event.html
	/// [`Delayed`]: enum.Event.html#variant.Delayed
	/// [`Cancel`]: enum.Event.html#variant.Cancel
	pub fn new<F>(event: Event, action: F) -> Self
		where F: FnMut(EventContext) -> Option<Event> + Send + Sync + 'static
	{
		Self {
			event,
			fire_time: None,
			action: Box::new(action),
		}
	}

	/// Computes the next firing time for a timer event.
	pub fn compute_activation(&mut self, now: Duration) {
		match self.event {
			Event::Periodic(period, phase) => {
				self.fire_time = Some(now + phase.unwrap_or(period));
			},
			Event::Delayed(offset) => {
				self.fire_time = Some(now + offset);
			},
			_ => {},
		}
	}
}

impl std::fmt::Debug for EventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        write!(f, "Event {{ event: {:?}, fire_time: {:?}, action: <fn> }}", self.event, self.fire_time)
    }
}

// Events are ordered/compared based on their firing time.
impl Ord for EventData {
	fn cmp(&self, other: &Self) -> Ordering {
		if self.fire_time.is_some() && other.fire_time.is_some() {
			let t1 = self.fire_time.as_ref().expect("T1 known to be well-defined by above.");
			let t2 = other.fire_time.as_ref().expect("T2 known to be well-defined by above.");

			t1.cmp(&t2)
		} else {
			Ordering::Equal
		}
	}
}

impl PartialOrd for EventData {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for EventData {
	fn eq(&self, other: &Self) -> bool {
		self.fire_time == other.fire_time
	}
}

impl Eq for EventData {}

#[derive(Debug, Default)]
/// Storage for [`EventData`], designed to be used for both local and global contexts.
///
/// Timed events are stored in a binary heap for fast selection, and have custom `Eq`,
/// `Ord`, etc. implementations to support (only) this.
///
/// [`EventData`]: struct.EventData.html
pub struct EventStore {
	timed: BinaryHeap<EventData>,
	untimed: HashMap<UntimedEvent, Vec<EventData>>,
}

impl EventStore {
	pub fn new() -> Self {
		Default::default()
	}

	/// Add an event to this store.
	///
	/// Updates `evt` according to [`EventData::compute_activation`].
	///
	/// [`EventData::compute_activation`]: struct.EventData.html#method.compute_activation
	pub fn add_event(&mut self, mut evt: EventData, now: Duration) {
		evt.compute_activation(now);

		use Event::*;
		match evt.event {
			Core(c) => {
				self.untimed.entry(c.into())
					.or_insert_with(|| vec![])
					.push(evt);
			},
			Track(t) => {
				self.untimed.entry(t.into())
					.or_insert_with(|| vec![])
					.push(evt);
			},
			Delayed(_) | Periodic(_, _) => {
				self.timed.push(evt);
			},
			_ => {
				// Event cancelled.
			},
		}
	}

	/// Processes all events due up to and including `now`.
	pub(crate) fn process_timed(&mut self, now: Duration, mut ctx: EventContext) {
		while let Some(evt) = self.timed.peek() {
			if evt.fire_time.as_ref().expect("Timed event must have a fire_time.") > &now {
				break;
			}
			let mut evt = self.timed.pop().expect("Can only succeed due to peek = Some(...).");

			let old_evt_type = evt.event;
			if let Some(new_evt_type) = (evt.action)(ctx.clone()) {
				evt.event = new_evt_type;
				self.add_event(evt, now);
			} else if let Event::Periodic(d, _) = old_evt_type {
				evt.event = Event::Periodic(d, None);
				self.add_event(evt, now);
			}
		}
	}

	/// Processes all events attached to the given track event.
	pub(crate) fn process_untimed(&mut self, now: Duration, untimed_event: UntimedEvent, mut ctx: EventContext) {
		// move a Vec in and out: not too expensive, but could be better.
		// Although it's obvious that moving an event out of one vec and into
		// another necessitates that they be different event types, thus entries,
		// convincing the compiler of this is non-trivial without making them dedicated
		// fields.
		let events = self.untimed.remove(&untimed_event);
		if let Some(mut events) = events {
			// FIXME: Possibly use tombstones to prevent realloc/memcpys?
			let mut i = 0;
			while i < events.len() {
				let evt = &mut events[i];
				// Only remove/readd if the event type changes (i.e., Some AND new != old)
				if let Some(new_evt_type) = (evt.action)(ctx.clone()) {
					if evt.event == new_evt_type {

						let mut evt = events.remove(i);

						evt.event = new_evt_type;
						self.add_event(evt, now);
					} else { i += 1; }
				} else { i += 1; };
			}
			self.untimed.insert(untimed_event, events);
		}
	}
}

#[derive(Debug, Default)]
pub(crate) struct GlobalEvents {
	pub(crate) store: EventStore,
	pub(crate) time: Duration,
	pub(crate) awaiting_tick: HashMap<TrackEvent, Vec<usize>>,
}

impl GlobalEvents {
	// pub(crate) fn march_and_process(&mut self, sources: &mut Vec<Track>, events: &mut HashMap<UntimedEvent, EventContext>) {
	// 	self.time += Duration::from_millis(20);
	// 	// FIXME: make this return all active?
	// 	self.store.process_timed(self.time, EventContext::Track(vec![]));

	// 	for (evt, ctx) in events.iter() {
	// 		if let EventContext::Track(indices) = ctx {
	// 			// Peek to see if there are any listeners and events at all...
	// 			let should_work = !indices.is_empty();

	// 			if should_work {
	// 				let mut local_sources = &mut sources[..];
	// 				let mut auds = Vec::with_capacity(local_sources.len());

	// 				// filter_map etc. on the indices wouldn't work here...
	// 				// So far as I can tell, this was the only way to convince the compiler
	// 				// that the references would survive.
	// 				let mut removed = 0;
	// 				for i in indices {
	// 					let (_, edge) = local_sources.split_at_mut(i - removed);
	// 					let (val, rest) = edge.split_at_mut(1);
	// 					local_sources = rest;
	// 					auds.push(&mut val[0]);
	// 					removed += i;
	// 				}

	// 				for audio in &mut auds {
	// 					let state = audio.state();
	// 		            let handle = audio.handle.clone();
	// 		            audio.events.process_untimed(audio.position, *evt, EventContext::Track(&state, &handle));
	// 				}

	// 				self.store.process_untimed(self.time, *evt, EventContext::Track(Some(auds.iter().map(|a| &a.handle).collect())));
	// 			}
	// 		} else {
	// 			// TODO: other untimed events
	// 			// self.store.process_untimed(self.time, *evt, EventContext::Global(GlobalContext::Track(Some(auds))));
	// 		}
	// 	}

	// 	// remove audios, too.
	// 	let to_cull = events.entry(TrackEvent::End.into()).or_insert_with(|| EventContext::Track(vec![]));
	// 	if let EventContext::Track(to_cull) = to_cull {
	// 		for (count, index) in to_cull.iter().enumerate() {
	// 			let _ = sources.remove(index - count);
	// 		}
	// 	}

	// 	// Now drain vecs.
	// 	for (_evt, indices) in events.iter_mut() {
	// 		if let EventContext::Track(indices) = indices {
	// 			indices.clear();
	// 		}
	// 	}
	// }

	pub(crate) fn add_event(&mut self, evt: EventData) {
		self.store.add_event(evt, self.time);
	}

	pub(crate) fn fire_core_event(&mut self, evt: CoreEvent, ctx: EventContext) {
		self.store.process_untimed(self.time, evt.into(), ctx);
	}

	pub(crate) fn fire_track_event(&mut self, evt: TrackEvent, index: usize) {
		let holder = self.awaiting_tick.entry(evt)
			.or_insert_with(|| vec![]);

		holder.push(index);
	}

	pub(crate) fn tick(
		&mut self,
		events: &mut Vec<EventStore>,
		states: &mut Vec<TrackState>,
		handles: &mut Vec<TrackHandle>,
	) {
		// FIXME: Make this into references...

		// Global timed events
		self.time += TIMESTEP_LENGTH;
		self.store.process_timed(self.time, EventContext::Track(vec![]));

		// Local timed events
		for (i, state) in states.iter_mut().enumerate() {
			if state.playing == PlayMode::Play {
				state.position += TIMESTEP_LENGTH;
				state.play_time += TIMESTEP_LENGTH;

				let event_store = events.get_mut(i)
                    .expect("[Voice] Missing store index for Tick (local timed).");
                let handle = handles.get_mut(i)
                    .expect("[Voice] Missing handle index for Tick (local timed).");

                event_store.process_timed(state.position, EventContext::Track(vec![(state.clone(), handle.clone())]));
			}
		}

		for (evt, indices) in self.awaiting_tick.iter() {
			let untimed = (*evt).into();

			// Local untimed track events.
			for &i in indices.iter() {
				let event_store = events.get_mut(i)
                    .expect("[Voice] Missing store index for Tick (local untimed).");
                let handle = handles.get_mut(i)
                    .expect("[Voice] Missing handle index for Tick (local untimed).");
                let state = states.get_mut(i)
                    .expect("[Voice] Missing state index for Tick (local untimed).");

                event_store.process_untimed(state.position, untimed, EventContext::Track(vec![(state.clone(), handle.clone())]));
			}

			// Global untimed track events.
			if self.store.untimed.contains_key(&untimed) {
				let global_ctx = indices.iter().map(|i| (
					states.get(*i).expect("[Voice] Missing state index for Tick (global untimed)").clone(),
					handles.get(*i).expect("[Voice] Missing handle index for Tick (global untimed)").clone(),
				)).collect();

				self.store.process_untimed(self.time, untimed, EventContext::Track(global_ctx))
			}
			// let global_ctx = indices.map(|i| (
			// 		states.get(i).expect("[Voice] Missing state index for "),
			// 	))
		}

		// Now drain vecs.
		for (_evt, indices) in self.awaiting_tick.iter_mut() {
			indices.clear();
		}
	}
}
