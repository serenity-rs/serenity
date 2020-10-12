use super::*;
use std::{cmp::Ordering, time::Duration};

/// Internal representation of an event, as handled by the audio context.
pub struct EventData {
    pub(crate) event: Event,
    pub(crate) fire_time: Option<Duration>,
    pub(crate) action: Box<dyn EventHandler>,
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
    pub fn new<F: EventHandler + 'static>(event: Event, action: F) -> Self {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Event {{ event: {:?}, fire_time: {:?}, action: <fn> }}",
            self.event, self.fire_time
        )
    }
}

/// Events are ordered/compared based on their firing time.
impl Ord for EventData {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.fire_time.is_some() && other.fire_time.is_some() {
            let t1 = self
                .fire_time
                .as_ref()
                .expect("T1 known to be well-defined by above.");
            let t2 = other
                .fire_time
                .as_ref()
                .expect("T2 known to be well-defined by above.");

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
