use std::{
	cmp::Ordering,
	collections::{
		BinaryHeap,
		HashMap,
	},
	time::Duration,
};
use super::{Audio, AudioHandle, AudioState};

/// Information about which tracks, if any, fired an event.
///
/// Local events ([`Audio`]-specific) are guaranteed to have
/// an attached track, while global timing events will not.
///
/// [`Audio`]: struct.Audio.html
/// [`Handler::add_global_event`]: struct.Handler.html#method.add_global_event
pub enum EventContext<'a> {
	/// Local event context, passed to events created via [`AudioHandle::add_event`]
	/// or [`EventStore::add_event`].
	///
	/// [`EventStore::add_event`]: struct.EventStore.html#method.add_event
	/// [`AudioHandle::add_event`]: struct.AudioHandle.html#method.add_event
	Track(&'a AudioState, &'a AudioHandle),

	/// Global event context, passed to events created via [`Handler::add_global_event`].
	///
	/// [`Handler::add_global_event`]: struct.Handler.html#method.add_global_event
	Global(Option<Vec<&'a mut Audio>>),
}

#[derive(Debug, Eq, Hash, PartialEq)]
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

	/// Cancels the event, if it was intended to persist.
	Cancel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Events corresponding to important state changes in an audio track.
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
	action: Box<dyn FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static>,
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
		where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
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
	track: HashMap<TrackEvent, Vec<EventData>>,
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
			Track(t) => {
				self.track.entry(t)
					.or_insert_with(|| vec![])
					.push(evt);
			}
			Delayed(_) | Periodic(_, _) => {
				self.timed.push(evt);
			},
			_ => {
				// Event cancelled.
			},
		}
	}

	/// Processes all events due up to and including `now`.
	pub(crate) fn process_timed(&mut self, now: Duration, mut ctx: EventContext<'_>) {
		while let Some(evt) = self.timed.peek() {
			if evt.fire_time.as_ref().expect("Timed event must have a fire_time.") > &now {
				break;
			}
			let mut evt = self.timed.pop().expect("Can only succeed due to peek = Some(...).");

			let old_evt_type = evt.event;
			if let Some(new_evt_type) = (evt.action)(&mut ctx) {
				evt.event = new_evt_type;
				self.add_event(evt, now);
			} else if let Event::Periodic(d, _) = old_evt_type {
				evt.event = Event::Periodic(d, None);
				self.add_event(evt, now);
			}
		}
	}

	/// Processes all events attached to the given track event.
	pub(crate) fn process_track(&mut self, now: Duration, track_event: TrackEvent, mut ctx: EventContext<'_>) {
		// move a Vec in and out: not too expensive, but could be better.
		// Although it's obvious that moving an event out of one vec and into
		// another necessitates that they be different event types, thus entries,
		// convincing the compiler of this is non-trivial without making them dedicated
		// fields.
		let events = self.track.remove(&track_event);
		if let Some(mut events) = events {
			// FIXME: Possibly use tombstones to prevent realloc/memcpys?
			let mut i = 0;
			while i < events.len() {
				let evt = &mut events[i];
				// Only remove/readd if the event type changes (i.e., Some AND new != old)
				if let Some(new_evt_type) = (evt.action)(&mut ctx) {
					if evt.event == new_evt_type {

						let mut evt = events.remove(i);

						evt.event = new_evt_type;
						self.add_event(evt, now);
					} else { i += 1; }
				} else { i += 1; };
			}
			self.track.insert(track_event, events);
		}
	}
}

#[derive(Debug, Default)]
pub(crate) struct GlobalEvents {
	pub(crate) store: EventStore,
	pub(crate) time: Duration,
}

impl GlobalEvents {
	pub(crate) fn march_and_process(&mut self, sources: &mut Vec<Audio>, events: &mut HashMap<TrackEvent, Vec<usize>>) {
		self.time += Duration::from_millis(20);
		self.store.process_timed(self.time, EventContext::Global(None));

		for (evt, indices) in events.iter() {
			// Peek to see if there are any listeners and events at all...
			let should_work = !indices.is_empty();

			if should_work {
				let mut local_sources = &mut sources[..];
				let mut auds = Vec::with_capacity(local_sources.len());

				// filter_map etc. on the indices wouldn't work here...
				// So far as I can tell, this was the only way to convince the compiler
				// that the references would survive.
				let mut removed = 0;
				for i in indices {
					let (_, edge) = local_sources.split_at_mut(i - removed);
					let (val, rest) = edge.split_at_mut(1);
					local_sources = rest;
					auds.push(&mut val[0]);
					removed += i;
				}

				for audio in &mut auds {
					let state = audio.get_state();
		            let handle = audio.handle.clone();
		            audio.events.process_track(audio.position, *evt, EventContext::Track(&state, &handle));
				}

				self.store.process_track(self.time, *evt, EventContext::Global(Some(auds)));
			}
		}

		// remove audios, too.
		let to_cull = events.entry(TrackEvent::End).or_default();
		for (count, index) in to_cull.iter().enumerate() {
			let _ = sources.remove(index - count);
		}

		// Now drain vecs.
		for (_evt, indices) in events.iter_mut() {
			indices.clear();
		}
	}
}
