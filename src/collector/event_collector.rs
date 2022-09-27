use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as FutContext, Poll};
use std::time::Duration;

use futures::stream::Stream;
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio::time::{sleep, Sleep};

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::{CollectorError, LazyArc};
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
}

impl EventFilter {
    /// Creates a new filter
    fn new(options: FilterOptions) -> Result<(Self, Receiver<Arc<Event>>)> {
        Self::validate_options(&options)?;

        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
        };

        Ok((filter, receiver))
    }

    fn validate_options(options: &FilterOptions) -> Result<()> {
        if options.event_types.is_empty() {
            return Err(Error::Collector(CollectorError::NoEventTypes));
        }
        let related = options.event_types.iter().map(EventType::related_ids).fold(
            RelatedIdsForEventType::default(),
            |mut acc, e| {
                acc.user_id |= e.user_id;
                acc.guild_id |= e.guild_id;
                acc.channel_id |= e.channel_id;
                acc.message_id |= e.message_id;
                acc
            },
        );
        if (options.user_id.is_empty() || related.user_id)
            && (options.guild_id.is_empty() || related.guild_id)
            && (options.channel_id.is_empty() || related.channel_id)
            && (options.message_id.is_empty() || related.message_id)
        {
            Ok(())
        } else {
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        }
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
    /// be sent by a specific user or in a specific guild.
    #[allow(clippy::wrong_self_convention)]
    fn is_passing_constraints(&mut self, event: &mut LazyArc<'_, Event>) -> bool {
        fn empty_or_any<T, F>(slice: &[T], f: F) -> bool
        where
            F: Fn(&T) -> bool,
        {
            slice.is_empty() || slice.iter().any(f)
        }

        // TODO: On next branch, switch filter arg to &T so this as_arc() call can be removed.
        empty_or_any(&self.options.guild_id, |id| event.guild_id().contains(id))
            && empty_or_any(&self.options.user_id, |id| event.user_id().contains(id))
            && empty_or_any(&self.options.channel_id, |id| event.channel_id().contains(id))
            && empty_or_any(&self.options.message_id, |id| event.message_id().contains(id))
            && self.options.filter.as_mut().map_or(true, |f| f.0(&event.as_arc()))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A event is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.as_ref().map_or(true, |limit| self.filtered < *limit)
            && self.options.collect_limit.as_ref().map_or(true, |limit| self.collected < *limit)
    }
}

// TODO: inline this struct
#[derive(Clone)]
struct FilterFn(super::FilterFn<Event>);

impl fmt::Debug for FilterFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("super::FilterFn<Event>")
    }
}

#[derive(Clone, Debug, Default)]
struct FilterOptions {
    event_types: Vec<EventType>,
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<FilterFn>,
    channel_id: Vec<ChannelId>,
    guild_id: Vec<GuildId>,
    user_id: Vec<UserId>,
    message_id: Vec<MessageId>,
}

/// Future building a stream of events.
#[must_use = "Builders do nothing unless built"]
pub struct EventCollectorBuilder {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl EventCollectorBuilder {
    /// A future that builds an [`EventCollector`] based on the settings.
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
        }
    }

    /// Limits how many events will attempt to be filtered.
    ///
    /// The filter checks whether the event has the right related guild, channel, user, and message.
    /// Only events with types passed to [`Self::add_event_type`] as counted towards this limit.
    #[allow(clippy::unwrap_used)]
    pub fn filter_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().filter_limit = Some(limit);

        self
    }

    /// Limits how many events can be collected.
    ///
    /// An event is considered *collected*, if the event
    /// passes all the requirements.
    #[allow(clippy::unwrap_used)]
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().collect_limit = Some(limit);

        self
    }

    /// Sets a filter function where events passed to the `function` must
    /// return `true`, otherwise the event won't be collected and failed the filter
    /// process.
    /// This is the last step to pass for a event to count as *collected*.
    #[allow(clippy::unwrap_used)]
    pub fn filter<F: Fn(&Arc<Event>) -> bool + 'static + Send + Sync>(
        mut self,
        function: F,
    ) -> Self {
        self.filter.as_mut().unwrap().filter = Some(FilterFn(Arc::new(function)));

        self
    }

    /// Adds an [`EventType`] that this collector will collect.
    /// If an event does not have one of these types, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn add_event_type(mut self, event_type: EventType) -> Self {
        self.filter.as_mut().unwrap().event_types.push(event_type);

        self
    }

    /// Sets the required user ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn add_user_id(mut self, user_id: impl Into<UserId>) -> Self {
        self.filter.as_mut().unwrap().user_id.push(user_id.into());

        self
    }

    /// Sets the required channel ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn add_channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.filter.as_mut().unwrap().channel_id.push(channel_id.into());

        self
    }

    /// Sets the required guild ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn add_guild_id(mut self, guild_id: impl Into<GuildId>) -> Self {
        self.filter.as_mut().unwrap().guild_id.push(guild_id.into());

        self
    }

    /// Sets the required message ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn add_message_id(mut self, message_id: impl Into<MessageId>) -> Self {
        self.filter.as_mut().unwrap().message_id.push(message_id.into());

        self
    }

    /// Sets a `duration` for how long the collector shall receive
    /// events.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(Box::pin(sleep(duration)));

        self
    }

    /// Use the given configuration to build the [`EventCollector`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Collector`] if the filter option validation fails.
    #[allow(clippy::unwrap_used)]
    pub fn build(self) -> Result<EventCollector> {
        let shard_messenger = self.shard.unwrap();
        let (filter, receiver) = EventFilter::new(self.filter.unwrap())?;
        let timeout = self.timeout;

        shard_messenger.set_event_filter(filter);

        Ok(EventCollector {
            receiver: Box::pin(receiver),
            timeout,
        })
    }
}

/// An event collector receives events matching the given filter for a set duration.
pub struct EventCollector {
    receiver: Pin<Box<Receiver<Arc<Event>>>>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl EventCollector {
    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(mut self) {
        self.receiver.close();
    }
}

impl Stream for EventCollector {
    type Item = Arc<Event>;
    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Option<Self::Item>> {
        if let Some(ref mut timeout) = self.timeout {
            match timeout.as_mut().poll(ctx) {
                Poll::Ready(_) => {
                    return Poll::Ready(None);
                },
                Poll::Pending => (),
            }
        }

        self.receiver.as_mut().poll_recv(ctx)
    }
}

impl Drop for EventCollector {
    fn drop(&mut self) {
        self.receiver.close();
    }
}

#[cfg(test)]
mod test {
    use futures::channel::mpsc::unbounded;

    use super::*;
    use crate::client::bridge::gateway::ShardMessenger;

    #[test]
    fn test_no_event_types() {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);
        assert!(matches!(
            EventCollectorBuilder::new(&msg).build(),
            Err(Error::Collector(CollectorError::NoEventTypes))
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg).add_channel_id(ChannelId::default()).build(),
            Err(Error::Collector(CollectorError::NoEventTypes))
        ));
    }

    #[test]
    fn test_build_with_single_id_filter() {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_user_id(UserId::default())
                .build(),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_event_type(EventType::GuildRoleCreate)
                .add_user_id(UserId::default())
                .build(),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildBanAdd)
                .add_user_id(UserId::default())
                .build(),
            Ok(_)
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildBanAdd)
                .add_event_type(EventType::GuildCreate)
                .add_user_id(UserId::default())
                .build(),
            Ok(_)
        ));
    }

    #[test]
    fn test_build_with_multiple_id_filters() {
        let (sender, _) = unbounded();
        let msg = ShardMessenger::new(sender);

        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::UserUpdate)
                .add_user_id(UserId::default())
                .add_guild_id(GuildId::default())
                .build(),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::UserUpdate)
                .add_user_id(UserId::default())
                .build(),
            Ok(_)
        ));
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
                .add_user_id(UserId::default())
                .build(),
            Ok(_)
        ));
        // But if none of the events have that ID type, that's an error.
        assert!(matches!(
            EventCollectorBuilder::new(&msg)
                .add_event_type(EventType::GuildCreate)
                .add_event_type(EventType::UserUpdate)
                .add_channel_id(ChannelId::default())
                .build(),
            Err(Error::Collector(CollectorError::InvalidEventIdFilters))
        ));
    }
}
