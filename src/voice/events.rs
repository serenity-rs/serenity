use chrono::Duration;

pub type EventFn = fn(&mut EventContext) -> ();

pub enum EventContext {
	Track,
	Global,
}

pub enum Event {
	Periodic(Duration),
	Delayed(Duration),
	Track(TrackEvent)
}

pub enum TrackEvent {
	TrackEnd,
	TrackLoop,
}

pub struct EventData {
	event: Event,
	fire_time: Option<Duration>,
	action: EventFn,
}