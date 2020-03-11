use chrono::Duration;
use super::Audio;

pub type EventFn = fn(&mut EventContext<'_>) -> ();

// #[derive(Debug)]
pub enum EventContext<'a> {
	Track(&'a mut Audio),
	Global(Option<&'a mut Audio>),
}

#[derive(Debug)]
pub enum EventData {
	Periodic(Duration),
	Delayed(Duration),
	Track(TrackEvent)
}

#[derive(Debug)]
pub enum TrackEvent {
	TrackEnd,
	TrackLoop,
}

pub struct Event<F>
	where F: Fn(&mut EventContext<'_>) -> ()
{
	event: EventData,
	fire_time: Option<Duration>,
	action: F,
}

impl Event {
	pub fn new<F: Fn(&mut EventContext<'_>) -> ()>(event: EventData, action: F) -> Self {
		Self {
			event,
			fire_time: None,
			action,
		}
	}
}

impl std::fmt::Debug for Event<> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        write!(f, "Event {{ event: {:?}, fire_time: {:?}, action: <fn> }}", self.event, self.fire_time)
    }
}