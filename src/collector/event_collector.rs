use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use crate::collector::{CollectorError, FilterFn, LazyArc};
use crate::model::event::{Event, EventType, RelatedIdsForEventType};
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};
use crate::{Error, Result};

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct EventFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<Event>>,

    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<FilterFn<Event>>,
}

impl EventFilter {
    /// Creates a new filter
    fn new(
        options: FilterOptions,
        filter_limit: Option<u32>,
        collect_limit: Option<u32>,
        filter: Option<FilterFn<Event>>,
    ) -> (Self, Receiver<Arc<Event>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            filter,
            options,
            filter_limit,
            collect_limit,
        };

        (filter, receiver)
    }

    /// Sends a `event` to the consuming collector if the `event` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_event(&mut self, event: &mut LazyArc<'_, Event>) -> bool {
        // Only events with matching types count towards the filtered limit.
        if !self.is_matching_event_type(event) {
            return !self.sender.is_closed();
        }

        if self.is_passing_constraints(event) {
            self.collected += 1;

            if self.sender.send(event.as_arc()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }

    /// Checks if the `event` is one of the types we're looking for.
    fn is_matching_event_type(&self, event: &Event) -> bool {
        self.options.event_types.contains(&event.event_type())
    }

    /// Checks if the `event` passes set constraints.
    /// Constraints are optional, as it is possible to limit events to
    /// be sent by a specific user or in a specifc guild.
    fn is_passing_constraints(&self, event: &Event) -> bool {
        fn empty_or_any<T, F>(slice: &[T], f: F) -> bool
        where
            F: Fn(&T) -> bool,
        {
            slice.is_empty() || slice.iter().any(f)
        }

        empty_or_any(&self.options.guild_id, |id| event.guild_id().contains(id))
            && empty_or_any(&self.options.user_id, |id| event.user_id().contains(id))
            && empty_or_any(&self.options.channel_id, |id| event.channel_id().contains(id))
            && empty_or_any(&self.options.message_id, |id| event.message_id().contains(id))
            && self.filter.as_ref().map_or(true, |f| f.0(event))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A event is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.filter_limit.as_ref().map_or(true, |limit| self.filtered < *limit)
            && self.collect_limit.as_ref().map_or(true, |limit| self.collected < *limit)
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterOptions {
    event_types: Vec<EventType>,
    channel_id: Vec<ChannelId>,
    guild_id: Vec<GuildId>,
    user_id: Vec<UserId>,
    message_id: Vec<MessageId>,
}

impl super::CollectorBuilder<'_, Event> {
    fn validate_related_ids(self) -> Result<Self> {
        let related = self.filter_options.event_types.iter().map(EventType::related_ids).fold(
            RelatedIdsForEventType::default(),
            |mut acc, e| {
                acc.user_id |= e.user_id;
                acc.guild_id |= e.guild_id;
                acc.channel_id |= e.channel_id;
                acc.message_id |= e.message_id;
                acc
            },
        );

        if (self.filter_options.user_id.is_empty() || related.user_id)
            && (self.filter_options.guild_id.is_empty() || related.guild_id)
            && (self.filter_options.channel_id.is_empty() || related.channel_id)
            && (self.filter_options.message_id.is_empty() || related.message_id)
        {
            Ok(self)
        } else {
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        }
    }

    /// Adds an [`EventType`] that this collector will collect.
    /// If an event does not have one of these types, it won't be received.
    pub fn add_event_type(mut self, event_type: EventType) -> Self {
        self.filter_options.event_types.push(event_type);
        self
    }

    /// Sets the required user ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_user_id(mut self, user_id: impl Into<UserId>) -> Result<Self> {
        self.filter_options.user_id.push(user_id.into());
        self.validate_related_ids()
    }

    /// Sets the required channel ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_channel_id(mut self, channel_id: impl Into<ChannelId>) -> Result<Self> {
        self.filter_options.channel_id.push(channel_id.into());
        self.validate_related_ids()
    }

    /// Sets the required guild ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_guild_id(mut self, guild_id: impl Into<GuildId>) -> Result<Self> {
        self.filter_options.guild_id.push(guild_id.into());
        self.validate_related_ids()
    }

    /// Sets the required message ID of an event.
    /// If an event does not have this ID, it won't be received.
    ///
    /// # Errors
    /// Errors if a relevant [`EventType`] has not been added.
    pub fn add_message_id(mut self, message_id: impl Into<MessageId>) -> Result<Self> {
        self.filter_options.message_id.push(message_id.into());
        self.validate_related_ids()
    }
}

/// An event collector receives events matching the given filter for a set duration.
pub type EventCollector = super::Collector<Event>;
pub type EventCollectorBuilder<'a> = super::CollectorBuilder<'a, Event>;

// No deprecated CollectSingle alias as EventCollector never had a CollectSingle version.

impl super::FilterOptions<Event> for FilterOptions {
    type FilterItem = Event;

    fn build(
        self,
        messenger: &crate::client::bridge::gateway::ShardMessenger,
        filter_limit: Option<u32>,
        collect_limit: Option<u32>,
        filter: Option<FilterFn<Self::FilterItem>>,
    ) -> Receiver<Arc<Event>> {
        let (filter, recv) = EventFilter::new(self, filter_limit, collect_limit, filter);
        messenger.set_event_filter(filter);

        recv
    }
}

impl super::Collectable for Event {
    type FilterOptions = FilterOptions;
}

#[cfg(test)]
mod test {
    use futures::channel::mpsc::unbounded;

    use super::*;
    use crate::client::bridge::gateway::ShardMessenger;

    #[test]
    fn test_build_with_single_id_filter() {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_user_id(UserId::new(1)),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_event_type(EventType::GuildRoleCreate)
                .add_user_id(UserId::new(1)),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildBanAdd)
                .add_user_id(UserId::new(1)),
            Ok(_)
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildBanAdd)
                .add_event_type(EventType::GuildCreate)
                .add_user_id(UserId::new(1)),
            Ok(_)
        ));
    }

    #[test]
    fn test_build_with_multiple_id_filters() -> Result<()> {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::UserUpdate)
                .add_user_id(UserId::new(1))?
                .add_guild_id(GuildId::new(1)),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::UserUpdate)
                .add_user_id(UserId::new(1)),
            Ok(_)
        ));

        Ok(())
    }

    #[test]
    fn test_build_with_multiple_event_types() {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);

        // If at least one event type has the filtered ID type(s), we go ahead and build the
        // collector, even though one or more of the event types may never be yielded.
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_event_type(EventType::GuildMemberAdd)
                .add_user_id(UserId::new(1)),
            Ok(_)
        ));
        // But if none of the events have that ID type, that's an error.
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_event_type(EventType::UserUpdate)
                .add_channel_id(ChannelId::new(1)),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
    }
}
