use std::{
    boxed::Box,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context as FutContext, Poll},
    time::Duration,
};

use futures::{future::BoxFuture, stream::Stream};
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
#[cfg(all(feature = "tokio_compat", not(feature = "tokio")))]
use tokio::time::{delay_for as sleep, Delay as Sleep};
#[cfg(feature = "tokio")]
use tokio::time::{sleep, Sleep};

use crate::{
    client::bridge::gateway::ShardMessenger,
    collector::LazyArc,
    model::{
        event::Event,
        id::{ChannelId, GuildId, MessageId, UserId},
    },
};

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
    fn new(options: FilterOptions) -> (Self, Receiver<Arc<Event>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
        };

        (filter, receiver)
    }

    /// Sends a `event` to the consuming collector if the `event` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_event(&mut self, event: &mut LazyArc<'_, Event>) -> bool {
        // TODO: Don't count events of other types as "filtered"

        if self.is_passing_constraints(event) {
            self.collected += 1;

            if let Err(_) = self.sender.send(event.as_arc()) {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }

    /// Checks if the `event` passes set constraints.
    /// Constraints are optional, as it is possible to limit events to
    /// be sent by a specific author or in a specifc guild.
    fn is_passing_constraints(&self, event: &mut LazyArc<'_, Event>) -> bool {
        // TODO: On next branch, switch filter arg to &T so this as_arc() call can be removed.
        self.options.guild_id.map_or(true, |id| event.guild_id().contains(&id))
            && self.options.user_id.map_or(true, |id| event.user_id().contains(&id))
            && self.options.channel_id.map_or(true, |id| event.channel_id().contains(&id))
            && self.options.message_id.map_or(true, |id| event.message_id().contains(&id))
            && self.options.filter.as_ref().map_or(true, |f| f(&event.as_arc()))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A event is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.as_ref().map_or(true, |limit| self.filtered < *limit)
            && self.options.collect_limit.as_ref().map_or(true, |limit| self.collected < *limit)
    }
}

#[derive(Clone, Default)]
struct FilterOptions {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<Arc<dyn Fn(&Arc<Event>) -> bool + 'static + Send + Sync>>,
    channel_id: Option<ChannelId>,
    guild_id: Option<GuildId>,
    user_id: Option<UserId>,
    message_id: Option<MessageId>,
}

impl std::fmt::Debug for FilterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventFilter")
            .field("filter_limit", &self.filter_limit)
            .field("collect_limit", &self.collect_limit)
            .field("filter", &"Option<Arc<dyn Fn(&Arc<Event>) -> bool + 'static + Send + Sync>>")
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("user_id", &self.user_id)
            .field("user_id", &self.message_id)
            .finish()
    }
}

/// Future building a stream of events.
pub struct EventCollectorBuilder<'a> {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
    fut: Option<BoxFuture<'a, EventCollector>>,
}

impl<'a> EventCollectorBuilder<'a> {
    /// A future that builds an [`EventCollector`] based on the settings.
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
            fut: None,
        }
    }

    /// Limits how many events will attempt to be filtered.
    ///
    /// The filter checks whether the event has the right related guild, channel, user,
    /// and message.
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
        self.filter.as_mut().unwrap().filter = Some(Arc::new(function));

        self
    }

    /// Sets the required user ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn user_id(mut self, user_id: impl Into<UserId>) -> Self {
        self.filter.as_mut().unwrap().user_id = Some(user_id.into());

        self
    }

    /// Sets the required channel ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.filter.as_mut().unwrap().channel_id = Some(channel_id.into());

        self
    }

    /// Sets the required guild ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn guild_id(mut self, guild_id: impl Into<GuildId>) -> Self {
        self.filter.as_mut().unwrap().guild_id = Some(guild_id.into());

        self
    }

    /// Sets the required message ID of an event.
    /// If an event does not have this ID, it won't be received.
    #[allow(clippy::unwrap_used)]
    pub fn message_id(mut self, message_id: impl Into<MessageId>) -> Self {
        self.filter.as_mut().unwrap().message_id = Some(message_id.into());

        self
    }

    /// Sets a `duration` for how long the collector shall receive
    /// events.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(Box::pin(sleep(duration)));

        self
    }
}

impl<'a> Future for EventCollectorBuilder<'a> {
    // TODO: Check whether the filter will never match any events based on the event types and
    // possible related IDs and switch to Result<EventCollector>.
    type Output = EventCollector;
    #[allow(clippy::unwrap_used)]
    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let (filter, receiver) = EventFilter::new(self.filter.take().unwrap());
            let timeout = self.timeout.take();

            self.fut = Some(Box::pin(async move {
                shard_messenger.set_event_filter(filter);

                EventCollector {
                    receiver: Box::pin(receiver),
                    timeout,
                }
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
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
