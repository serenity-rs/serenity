use std::fmt;
use std::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as FutContext, Poll};

use futures::future::BoxFuture;
use futures::stream::{Stream, StreamExt};
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio::time::Sleep;

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{FilterFn, LazyArc};
use crate::model::channel::Reaction;

macro_rules! impl_reaction_collector {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Sets a filter function where reactions passed to the function must
                /// return `true`, otherwise the reaction won't be collected.
                /// This is the last instance to pass for a reaction to count as *collected*.
                ///
                /// This function is intended to be a reaction content filter.
                pub fn filter<F: Fn(&Reaction) -> bool + 'static + Send + Sync>(mut self, function: F) -> Self {
                    self.filter.as_mut().unwrap().filter = Some(FilterFn(Arc::new(function)));

                    self
                }

                /// If set to `true`, added reactions will be collected.
                ///
                /// Set to `true` by default.
                pub fn added(mut self, is_accepted: bool) -> Self {
                    self.filter.as_mut().unwrap().accept_added = is_accepted;

                    self
                }

                /// If set to `true`, removed reactions will be collected.
                ///
                /// Set to `false` by default.
                pub fn removed(mut self, is_accepted: bool) -> Self {
                    self.filter.as_mut().unwrap().accept_removed = is_accepted;

                    self
                }

                impl_filter_limit!("Limits how many messages will attempt to be filtered.\n\nThe filter checks whether the message has been sent in the right guild, channel, and by the right author.");
                impl_collect_limit!("Limits how many reactions can be collected. A reaction is considered *collected*, if the reaction passes all the requirements.");
                impl_channel_id!("Sets the channel on which the reaction must occur. If a reaction is not on a message with this channel ID, it won't be received.");
                impl_guild_id!("Sets the guild in which the reaction must occur. If a reaction is not on a message with this guild ID, it won't be received.");
                impl_message_id!("Sets the message on which the reaction must occur. If a reaction is not on a message with this ID, it won't be received.");
                impl_author_id!("Sets the required author ID of a reaction. If a reaction is not issued by a user with this ID, it won't be received.");
                impl_timeout!("Sets a `duration` for how long the collector shall receive reactions.");
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
    #[must_use]
    pub fn as_inner_ref(&self) -> &Arc<Reaction> {
        match self {
            Self::Added(inner) | Self::Removed(inner) => inner,
        }
    }

    #[must_use]
    pub fn is_added(&self) -> bool {
        matches!(self, Self::Added(_))
    }

    #[must_use]
    pub fn is_removed(&self) -> bool {
        matches!(self, Self::Removed(_))
    }
}

#[derive(Debug)]
pub(crate) struct LazyReactionAction<'a> {
    reaction: LazyArc<'a, Reaction>,
    added: bool,
    arc: Option<Arc<ReactionAction>>,
}

impl<'a> LazyReactionAction<'a> {
    pub fn new(reaction: &'a Reaction, added: bool) -> Self {
        Self {
            reaction: LazyArc::new(reaction),
            added,
            arc: None,
        }
    }

    pub fn as_arc(&mut self) -> Arc<ReactionAction> {
        let added = self.added;
        let reaction = &mut self.reaction;
        self.arc
            .get_or_insert_with(|| {
                if added {
                    Arc::new(ReactionAction::Added(reaction.as_arc()))
                } else {
                    Arc::new(ReactionAction::Removed(reaction.as_arc()))
                }
            })
            .clone()
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
    pub(crate) fn send_reaction(&mut self, reaction: &mut LazyReactionAction<'_>) -> bool {
        if self.is_passing_constraints(reaction) {
            self.collected += 1;

            if self.sender.send(reaction.as_arc()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }

    /// Checks if the `reaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit reactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, reaction: &LazyReactionAction<'_>) -> bool {
        let reaction = match (reaction.added, &reaction.reaction) {
            (true, reaction) => {
                if self.options.accept_added {
                    reaction
                } else {
                    return false;
                }
            },
            (false, reaction) => {
                if self.options.accept_removed {
                    reaction
                } else {
                    return false;
                }
            },
        };

        self.options.guild_id.map_or(true, |id| Some(id) == reaction.guild_id.map(|g| g.0))
            && self.options.message_id.map_or(true, |id| id == reaction.message_id.0)
            && self.options.channel_id.map_or(true, |id| id == reaction.channel_id.0)
            && self.options.author_id.map_or(true, |id| Some(id) == reaction.user_id.map(|u| u.0))
            && self.options.filter.as_ref().map_or(true, |f| f.0(reaction))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A reaction is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.map_or(true, |limit| self.filtered < limit)
            && self.options.collect_limit.map_or(true, |limit| self.collected < limit)
    }
}

#[derive(Clone)]
struct FilterOptions {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<FilterFn<Reaction>>,
    channel_id: Option<NonZeroU64>,
    guild_id: Option<NonZeroU64>,
    author_id: Option<NonZeroU64>,
    message_id: Option<NonZeroU64>,
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
    CollectReaction;
    ReactionCollectorBuilder;
}

#[must_use = "builders do nothing until built"]
pub struct ReactionCollectorBuilder {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl ReactionCollectorBuilder {
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
        }
    }

    /// Use the given configuration to build the [`ReactionCollector`].
    #[allow(clippy::unwrap_used)]
    #[must_use]
    pub fn build(self) -> ReactionCollector {
        let shard_messenger = self.shard.unwrap();
        let (filter, receiver) = ReactionFilter::new(self.filter.unwrap());
        let timeout = self.timeout;

        shard_messenger.set_reaction_filter(filter);

        ReactionCollector {
            receiver: Box::pin(receiver),
            timeout,
        }
    }
}

#[must_use = "builders do nothing unless awaited"]
pub struct CollectReaction {
    filter: Option<FilterOptions>,
    shard: Option<ShardMessenger>,
    timeout: Option<Pin<Box<Sleep>>>,
    fut: Option<BoxFuture<'static, Option<Arc<ReactionAction>>>>,
}

impl CollectReaction {
    pub fn new(shard_messenger: impl AsRef<ShardMessenger>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            shard: Some(shard_messenger.as_ref().clone()),
            timeout: None,
            fut: None,
        }
    }
}

impl Future for CollectReaction {
    type Output = Option<Arc<ReactionAction>>;
    #[allow(clippy::unwrap_used)]
    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let (filter, receiver) = ReactionFilter::new(self.filter.take().unwrap());
            let timeout = self.timeout.take();

            self.fut = Some(Box::pin(async move {
                shard_messenger.set_reaction_filter(filter);

                ReactionCollector {
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

impl fmt::Debug for FilterOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReactionFilter")
            .field("collect_limit", &self.collect_limit)
            .field("filter", &"Option<super::FilterFn<Reaction>>")
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("author_id", &self.author_id)
            .finish()
    }
}

/// A reaction collector receives reactions matching a the given filter for a
/// set duration.
pub struct ReactionCollector {
    receiver: Pin<Box<Receiver<Arc<ReactionAction>>>>,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl ReactionCollector {
    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(mut self) {
        self.receiver.close();
    }
}

impl Stream for ReactionCollector {
    type Item = Arc<ReactionAction>;
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

impl Drop for ReactionCollector {
    fn drop(&mut self) {
        self.receiver.close();
    }
}
