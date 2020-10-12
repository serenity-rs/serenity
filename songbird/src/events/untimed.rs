use super::*;

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