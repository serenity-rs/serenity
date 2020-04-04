use std::{
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio::{
    sync::Mutex,
    time::timeout,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context as FutContext, Poll},
};

use futures::future::BoxFuture;
use crate::client::bridge::gateway::ShardMessenger;
use crate::model::channel::Reaction;

type ForeachFunction = for<'fut> fn(&'fut Arc<ReactionAction>) -> BoxFuture<'fut, bool>;
type Shard = Arc<Mutex<ShardMessenger>>;

macro_rules! impl_reaction_collector {
    ($($name:ident;)*) => {
        $(
            impl<'a> $name<'a> {
                /// Limits how many messages will attempt to be filtered.
                ///
                /// The filter checks whether the message has been sent
                /// in the right guild, channel, and by the right author.
                pub fn filter_limit(mut self, limit: u32) -> Self {
                    self.filter.as_mut().unwrap().filter_limit = Some(limit);

                    self
                }

                /// Limits how many reactions can be collected.
                ///
                /// A reaction is considered *collected*, if the reaction
                /// passes all the requirements.
                pub fn collect_limit(mut self, limit: u32) -> Self {
                    self.filter.as_mut().unwrap().collect_limit = Some(limit);

                    self
                }

                /// Sets a filter function where reactions passed to the function must
                /// return `true`, otherwise the reaction won't be collected.
                /// This is the last instance to pass for a reaction to count as *collected*.
                ///
                /// This function is intended to be a reaction content filter.
                pub fn filter<F: Fn(&Arc<Reaction>) -> bool + 'static + Send + Sync>(mut self, function: F) -> Self {
                    self.filter.as_mut().unwrap().filter = Some(Arc::new(function));

                    self
                }

                /// Sets a duration the collector shall await reactions.
                /// Once the timeout is reached, the collector will *close* and be unable
                /// to receive any new reactions, already received reactions will be yielded.
                pub fn timeout(mut self, timeout: Duration) -> Self {
                    self.collector.as_mut().unwrap().timeout = Some(timeout);

                    self
                }

                /// Sets the required author ID of a reaction.
                /// If a reaction is not issued by a user with this ID, it won't be received.
                pub fn author_id(mut self, author_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().author_id = Some(author_id.into());

                    self
                }

                /// Sets the message on which the reaction must occur.
                /// If a reaction is not on a message with this ID, it won't be received.
                pub fn message_id(mut self, message_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().message_id = Some(message_id.into());

                    self
                }

                /// Sets the guild in which the reaction must occur.
                /// If a reaction is not on a message with this ID, it won't be received.
                pub fn guild_id(mut self, guild_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().guild_id = Some(guild_id.into());

                    self
                }

                /// Sets the channel on which the reaction must occur.
                /// If a reaction is not on a message with this ID, it won't be received.
                pub fn channel_id(mut self, channel_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().channel_id = Some(channel_id.into());

                    self
                }

                /// Performs a function whenever a reaction is received,
                /// if the function returns true, it will continue receiving
                /// more reactions, if it returns false, it will stop receiving.
                pub fn for_each_received(mut self, function: ForeachFunction) -> Self {
                    self.collector.as_mut().unwrap().for_each = Some(Arc::new(function));

                    self
                }

                /// If set to `true`, added reactions will be collected.
                ///
                /// Set to `true` by default.
                pub fn added(mut self, is_accepted: bool) -> Self {
                    self.filter.as_mut().unwrap().accept_added = is_accepted;

                    self
                }

                /// If set to `false`, removed reactions will be collected.
                ///
                /// Set to `false` by default.
                pub fn removed(mut self, is_accepted: bool) -> Self {
                    self.filter.as_mut().unwrap().accept_removed = is_accepted;

                    self
                }
            }
        )*
    }
}

/// Marks whether the reaction has been added or removed.
#[derive(Debug)]
pub enum ReactionAction {
    Added(Arc<Reaction>),
    Removed(Arc<Reaction>),
}

impl ReactionAction {
    pub fn as_inner_ref(&self) -> &Arc<Reaction> {
        match self {
            Self::Added(inner) => inner,
            Self::Removed(inner) => inner,
        }
    }

    pub fn is_added(&self) -> bool {
        if let Self::Added(_) = &self {
            true
        } else {
            false
        }
    }

    pub fn is_removed(&self) -> bool {
        if let Self::Removed(_) = &self {
            true
        } else {
            false
        }
    }
}

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct ReactionFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<ReactionAction>>,
}

impl ReactionFilter {
    /// Creates a new filter
    fn new(options: FilterOptions) -> (Self, Receiver<Arc<ReactionAction>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
        };

        (filter, receiver)
    }

    /// Sends a `reaction` to the consuming collector if the `reaction` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_reaction(&mut self, reaction: &Arc<ReactionAction>) -> bool {
        if self.is_passing_constraints(&reaction) {
            self.collected += 1;

            if self.sender.send(Arc::clone(reaction)).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits()
    }

    /// Checks if the `reaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit reactions to
    /// be sent by a specific author or in a specifc guild.
    fn is_passing_constraints(&self, reaction: &Arc<ReactionAction>) -> bool {
        let reaction = match **reaction {
            ReactionAction::Added(ref reaction) => if self.options.accept_added {
                reaction
            } else {
                return false;
            },
            ReactionAction::Removed(ref reaction) => if self.options.accept_removed {
                reaction
            } else {
                return false;
            },
        };

        self.options.guild_id.map_or(true, |id| { Some(id) == reaction.guild_id.map(|g| g.0) })
        && self.options.message_id.map_or(true, |id| { id == reaction.message_id.0 })
        && self.options.channel_id.map_or(true, |id| { id == reaction.channel_id.0 })
        && self.options.author_id.map_or(true, |id| { id == reaction.user_id.0 })
        && self.options.filter.as_ref().map_or(true, |f| f(&reaction))
    }


    /// Checks if the filter is within set receive and collect limits.
    /// A reaction is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.map_or(true, |limit| { self.filtered < limit })
        && self.options.collect_limit.map_or(true, |limit| { self.collected < limit })
    }
}

#[derive(Default)]
struct CollectorOptions {
    for_each: Option<Arc<ForeachFunction>>,
    timeout: Option<Duration>,
}

#[derive(Clone)]
struct FilterOptions {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<Arc<dyn Fn(&Arc<Reaction>) -> bool + 'static + Send + Sync>>,
    channel_id: Option<u64>,
    guild_id: Option<u64>,
    author_id: Option<u64>,
    message_id: Option<u64>,
    accept_added: bool,
    accept_removed: bool,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            filter_limit: None,
            collect_limit: None,
            filter: None,
            channel_id: None,
            guild_id: None,
            author_id: None,
            message_id: None,
            accept_added: true,
            accept_removed: false,
        }
    }
}

// Implement the common setters for all reaction collector types.
// This avoids using a trait that the user would need to import in
// order to use any of these methods.
impl_reaction_collector! {
    CollectOneReaction;
    CollectNReactions;
    CollectAllReactions;
    ReactionCollectorBuilder;
}

pub struct ReactionCollectorBuilder<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, ReactionCollector>>,
}

impl<'a> ReactionCollectorBuilder<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }
}

impl<'a> Future for ReactionCollectorBuilder<'a> {
    type Output = ReactionCollector;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = ReactionFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_reaction_filter(filter);

                ReactionCollector {
                    receiver,
                    timeout,
                    for_each,
                }
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

pub struct CollectOneReaction<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, Option<Arc<ReactionAction>>>>,
}

impl<'a> CollectOneReaction<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }
}

impl<'a> Future for CollectOneReaction<'a> {
    type Output = Option<Arc<ReactionAction>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = ReactionFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_reaction_filter(filter);

                ReactionCollector {
                    receiver,
                    timeout,
                    for_each,
                }.receive_one().await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

pub struct CollectNReactions<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, Vec<Arc<ReactionAction>>>>,
}

impl<'a> CollectNReactions<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }
}

impl<'a> Future for CollectNReactions<'a> {
    type Output = Vec<Arc<ReactionAction>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let number = self.filter.as_ref().unwrap().collect_limit.unwrap_or(1);
            let (filter, receiver) = ReactionFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_reaction_filter(filter);

                ReactionCollector {
                    receiver,
                    timeout,
                    for_each,
                }.receive_n(number).await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

/// Future to collect all reactions.
pub struct CollectAllReactions<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, Vec<Arc<ReactionAction>>>>,
}

impl<'a> CollectAllReactions<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }
}

impl<'a> Future for CollectAllReactions<'a> {
    type Output = Vec<Arc<ReactionAction>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = ReactionFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_reaction_filter(filter);

                ReactionCollector {
                    receiver,
                    timeout,
                    for_each,
                }.receive_all().await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

impl std::fmt::Debug for FilterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactionFilter")
            .field("collect_limit", &self.collect_limit)
            .field("filter", &"Option<Arc<dyn Fn(&Arc<Reaction>) -> bool + 'static + Send + Sync>>")
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("author_id", &self.author_id)
            .finish()
    }
}

/// A reaction collector receives reactions matching a the given filter for a
/// set duration.
pub struct ReactionCollector {
    timeout: Option<Duration>,
    receiver: Receiver<Arc<ReactionAction>>,
    for_each: Option<Arc<ForeachFunction>>,
}

impl ReactionCollector {
    /// Internal method to receive reactions until limitations are reached.
    /// The method fills `reactions` while it is awaited, you cancel awaiting.
    async fn receive(&mut self, reactions: &mut Vec<Arc<ReactionAction>>, max_reactions: Option<u32>) {
        let mut received_reactions = 0;

        while let Some(reaction) = self.receiver.recv().await {
            if max_reactions.map_or(false, |max| received_reactions >= max) {
                break;
            }

            received_reactions += 1;

            if let Some(for_each) = &self.for_each {

                if !for_each(&reaction).await {
                    break;
                }
            }

            reactions.push(reaction);
        }
    }

    /// Receives all reactions until set limitations are reached and
    /// returns the reactions.
    ///
    /// This will consume the collector and end the collection.
    pub async fn receive_all(mut self) -> Vec<Arc<ReactionAction>> {
        let mut reactions = Vec::new();

        if let Some(time) = self.timeout {
            let _ = timeout(time, self.receive(&mut reactions, None)).await;
        } else {
            self.receive(&mut reactions, None).await;
        }

        self.receiver.close();

        reactions
    }

    /// Receives `number` amount of reactions until set limitations are reached
    /// and returns the reactions.
    ///
    /// This will consume the collector and end the collection.
    pub async fn receive_n(mut self, number: u32) -> Vec<Arc<ReactionAction>> {
        let mut reactions = Vec::new();

        if let Some(time) = self.timeout {
            let _ = timeout(time, self.receive(&mut reactions, Some(number))).await;
        } else {
            self.receive(&mut reactions, Some(number)).await;
        }

        self.receiver.close();

        reactions
    }

   /// Receives a single reaction and returns it.
   /// The timer is applied each time called, the internal reaction collected
   /// counter carries over.
    pub async fn receive_one(&mut self) -> Option<Arc<ReactionAction>> {
        if let Some(time) = self.timeout {
            timeout(time, self.receiver.recv()).await.ok().flatten()
        } else {
            self.receiver.recv().await
        }
    }

    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(mut self) {
        self.receiver.close();
    }
}

impl Drop for ReactionCollector {
    fn drop(&mut self) {
        self.receiver.close();
    }
}
