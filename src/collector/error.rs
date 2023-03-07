use std::error::Error as StdError;
use std::fmt;

/// An error that occurred while working with a collector.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No event types were passed to [add_event_type].
    ///
    /// [add_event_type]: crate::collector::EventCollectorBuilder::add_event_type
    NoEventTypes,
    /// The combination of event types and ID filters used with [EventCollectorBuilder] is invalid
    /// and will never match any events.
    ///
    /// For example, the following always errors because GuildCreate never has a related user ID:
    /// ```rust
    /// # use serenity::{prelude::*, collector::{CollectorError, EventCollectorBuilder}, model::prelude::*};
    /// # let (sender, _) = futures::channel::mpsc::unbounded();
    /// # let ctx = serenity::client::bridge::gateway::ShardMessenger::new(sender);
    /// assert!(matches!(
    ///     EventCollectorBuilder::new(&ctx)
    ///         .add_event_type(EventType::GuildCreate)
    ///         .add_user_id(UserId::default())
    ///         .build(),
    ///     Err(SerenityError::Collector(CollectorError::InvalidEventIdFilters)),
    /// ));
    /// ```
    /// [EventCollectorBuilder]: crate::collector::EventCollectorBuilder
    InvalidEventIdFilters,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEventTypes => f.write_str("No event types provided"),
            Self::InvalidEventIdFilters => {
                f.write_str("Invalid event type + id filters, would never match any events")
            },
        }
    }
}

impl StdError for Error {}
