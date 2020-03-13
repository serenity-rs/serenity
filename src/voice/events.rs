use chrono::Duration;
use super::Audio;

pub type EventFn = dyn FnMut(&mut EventContext<'_>) -> Option<Event> + Send + 'static;

// #[derive(Debug)]
pub enum EventContext<'a> {
	Track(&'a mut Audio),
	Global(Option<&'a mut Audio>),
}

#[derive(Debug)]
pub enum Event {
	Periodic(Duration, Option<Duration>),
	Delayed(Duration),
	Track(TrackEvent),
	Cancel,
}

#[derive(Debug)]
pub enum TrackEvent {
	TrackEnd,
	TrackLoop,
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
}

impl std::fmt::Debug for EventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        write!(f, "Event {{ event: {:?}, fire_time: {:?}, action: <fn> }}", self.event, self.fire_time)
    }
}