use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as FutContext, Poll};
use std::time::Duration;

use futures::future::BoxFuture;
use futures::stream::{Stream, StreamExt};
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio::time::{sleep, Sleep};

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::LazyArc;
use crate::model::application::interaction::message_component::MessageComponentInteraction;

macro_rules! impl_component_interaction_collector {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Limits how many interactions will attempt to be filtered.
                ///
                /// The filter checks whether the message has been sent
                /// in the right guild, channel, and by the right author.
                pub fn filter_limit(mut self, limit: u32) -> Self {
                    self.filter.as_mut().unwrap().filter_limit = Some(limit);

                    self
                }

                /// Limits how many interactions can be collected.
                ///
                /// An interaction is considered *collected*, if the interaction
                /// passes all the requirements.
                pub fn collect_limit(mut self, limit: u32) -> Self {
                    self.filter.as_mut().unwrap().collect_limit = Some(limit);

                    self
                }

                /// Sets a filter function where interactions passed to the function must
                /// return `true`, otherwise the interaction won't be collected.
                /// This is the last instance to pass for an interaction to count as *collected*.
                ///
                /// This function is intended to be an interaction filter.
                pub fn filter<F: Fn(&Arc<MessageComponentInteraction>) -> bool + 'static + Send + Sync>(mut self, function: F) -> Self {
                    self.filter.as_mut().unwrap().filter = Some(Arc::new(function));

                    self
                }

                /// Sets the required author ID of an interaction.
                /// If an interaction is not triggered by a user with this ID, it won't be received.
                pub fn author_id(mut self, author_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().author_id = Some(author_id.into());

                    self
                }

                /// Sets the message on which the interaction must occur.
                /// If an interaction is not on a message with this ID, it won't be received.
                pub fn message_id(mut self, message_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().message_id = Some(message_id.into());

                    self
                }

                /// Sets the guild in which the interaction must occur.
                /// If an interaction is not on a message with this guild ID, it won't be received.
                pub fn guild_id(mut self, guild_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().guild_id = Some(guild_id.into());

                    self
                }

                /// Sets the channel on which the interaction must occur.
                /// If an interaction is not on a message with this channel ID, it won't be received.
                pub fn channel_id(mut self, channel_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().channel_id = Some(channel_id.into());

                    self
                }

                /// Sets a `duration` for how long the collector shall receive
                /// interactions.
                pub fn timeout(mut self, duration: Duration) -> Self {
                    self.timeout = Some(Box::pin(sleep(duration)));

                    self
                }
            }
        )*
    }
}

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct ComponentInteractionFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<MessageComponentInteraction>>,
}

impl ComponentInteractionFilter {
    /// Creates a new filter
    fn new(options: FilterOptions) -> (Self, Receiver<Arc<MessageComponentInteraction>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
        };

        (filter, receiver)
    }

    /// Sends an `interaction` to the consuming collector if the `interaction` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_interaction(
        &mut self,
        interaction: &mut LazyArc<'_, MessageComponentInteraction>,
    ) -> bool {
        if self.is_passing_constraints(interaction) {
            self.collected += 1;

            if self.sender.send(interaction.as_arc()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }

    /// Checks if the `interaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit interactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(
        &self,
        interaction: &mut LazyArc<'_, MessageComponentInteraction>,
    ) -> bool {
        // TODO: On next branch, switch filter arg to &T so this as_arc() call can be removed.
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id.map(|g| g.0))
            && self.options.message_id.map_or(true, |id| interaction.message.id.0 == id)
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id.as_ref().0)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id.0)
            && self.options.filter.as_ref().map_or(true, |f| f(&interaction.as_arc()))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// An interaction is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.map_or(true, |limit| self.filtered < limit)
            && self.options.collect_limit.map_or(true, |limit| self.collected < limit)
    }
}

#[derive(Clone, Default)]
struct FilterOptions {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<super::FilterFn<MessageComponentInteraction>>,
    channel_id: Option<u64>,
    guild_id: Option<u64>,
    author_id: Option<u64>,
    message_id: Option<u64>,
}

impl fmt::Debug for FilterOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentInteractionFilter")
            .field("collect_limit", &self.collect_limit)
            .field("filter", &"Option<super::FilterFn<Reaction>>")
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("author_id", &self.author_id)
            .finish()
    }
}

// Implement the common setters for all component interaction collector types.
// This avoids using a trait that the user would need to import in
// order to use any of these methods.
impl_component_interaction_collector! {
    CollectComponentInteraction;
    ComponentInteractionCollectorBuilder;
}

#[must_use = "Builders do nothing unless built"]
pub struct ComponentInteractionCollectorBuilder {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl ComponentInteractionCollectorBuilder {
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
        }
    }

    /// Use the given configuration to build the [`ComponentInteractionCollector`].
    #[allow(clippy::unwrap_used)]
    #[must_use]
    pub fn build(self) -> ComponentInteractionCollector {
        let shard_messenger = self.shard.unwrap();
        let (filter, receiver) = ComponentInteractionFilter::new(self.filter.unwrap());
        let timeout = self.timeout;

        shard_messenger.set_component_interaction_filter(filter);

        ComponentInteractionCollector {
            receiver: Box::pin(receiver),
            timeout,
        }
    }
}

#[must_use = "Builders do nothing unless awaited"]
pub struct CollectComponentInteraction {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
    fut: Option<BoxFuture<'static, Option<Arc<MessageComponentInteraction>>>>,
}

impl CollectComponentInteraction {
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
            fut: None,
        }
    }
}

impl Future for CollectComponentInteraction {
    type Output = Option<Arc<MessageComponentInteraction>>;
    #[allow(clippy::unwrap_used)]
    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let (filter, receiver) = ComponentInteractionFilter::new(self.filter.take().unwrap());
            let timeout = self.timeout.take();

            self.fut = Some(Box::pin(async move {
                shard_messenger.set_component_interaction_filter(filter);

                ComponentInteractionCollector {
                    receiver: Box::pin(receiver),
                    timeout,
                }
                .next()
                .await
            }));
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

/// A component interaction collector receives interactions matching a the given filter for a
/// set duration.
pub struct ComponentInteractionCollector {
    receiver: Pin<Box<Receiver<Arc<MessageComponentInteraction>>>>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl ComponentInteractionCollector {
    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(mut self) {
        self.receiver.close();
    }
}

impl Stream for ComponentInteractionCollector {
    type Item = Arc<MessageComponentInteraction>;
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

impl Drop for ComponentInteractionCollector {
    fn drop(&mut self) {
        self.receiver.close();
    }
}
