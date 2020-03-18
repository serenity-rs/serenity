use std::{
	cmp::{
		Ordering,
		Reverse,
	},
	collections::{
		BinaryHeap,
		HashMap,
	},
	time::Duration,
};
use super::{Audio, AudioHandle, AudioState};

pub type EventFn = dyn FnMut(&mut EventContext<'_>) -> Option<Event> + Send + 'static;

pub enum EventContext<'a> {
	Track(&'a AudioState, &'a AudioHandle),
	Global(Option<&'a mut Audio>),
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Event {
	Periodic(Duration, Option<Duration>),
	Delayed(Duration),
	Track(TrackEvent),
	Cancel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TrackEvent {
	End,
	Loop,
}

pub struct EventData {
	event: Event,
	fire_time: Option<Duration>,
	action: Box<dyn FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static>,
}

impl EventData {
	pub fn new<F>(event: Event, action: F) -> Self
		where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
	{
		Self {
			event,
			fire_time: None,
			action: Box::new(action),
		}
	}

	pub fn compute_activation(&mut self, now: Duration) {
		match self.event {
			Event::Periodic(period, offset) => {
				self.fire_time = Some(now + offset.unwrap_or(period));
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

pub struct EventStore {
	timed: BinaryHeap<EventData>,
	track: HashMap<TrackEvent, Vec<EventData>>,
}

impl EventStore {
	pub fn new() -> Self {
		Self {
			timed: BinaryHeap::new(),
			track: HashMap::new(),
		}
	}

	pub fn add_event(&mut self, mut evt: EventData, now: Duration) {
		evt.compute_activation(now);

		use Event::*;
		match evt.event {
			Track(t) => {
				self.track.entry(t)
					.or_insert(vec![])
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

	pub fn process_timed(&mut self, now: Duration, mut ctx: EventContext<'_>) {
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

	pub fn process_track(&mut self, now: Duration, track_event: TrackEvent, mut ctx: EventContext<'_>) {
		// move a Vec in and out: not too expensive, but could be better.
		// Although it's obvious that moving an event out of one vec and into
		// another necessitates that they be different event types, thus entries,
		// convincing the compiler of this is non-trivial without making them dedicated
		// fields.
		// FIXME: not working...
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
