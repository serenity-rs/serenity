use chrono::Duration;

pub type EventFn = fn(&mut EventContext) -> ();

#[derive(Debug)]
pub enum EventContext {
	Track,
	Global,
}

#[derive(Debug)]
pub enum Event {
	Periodic(Duration),
	Delayed(Duration),
	Track(TrackEvent)
}

#[derive(Debug)]
pub enum TrackEvent {
	TrackEnd,
	TrackLoop,
}

pub struct EventData {
	event: Event,
	fire_time: Option<Duration>,
	action: EventFn,
}