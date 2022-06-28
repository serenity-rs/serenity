use std::num::NonZeroU64;
use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{CommonFilterOptions, LazyArc};
use crate::model::channel::Reaction;

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
    common_options: CommonFilterOptions<Reaction>,
}

impl ReactionFilter {
    /// Creates a new filter
    fn new(
        options: FilterOptions,
        common_options: CommonFilterOptions<Reaction>,
    ) -> (Self, Receiver<Arc<ReactionAction>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
            common_options,
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
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(reaction))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A reaction is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.common_options.filter_limit.map_or(true, |limit| self.filtered < limit)
            && self.common_options.collect_limit.map_or(true, |limit| self.collected < limit)
    }
}

#[derive(Clone, Debug)]
pub struct FilterOptions {
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
            channel_id: None,
            guild_id: None,
            author_id: None,
            message_id: None,
            accept_added: true,
            accept_removed: false,
        }
    }
}

impl super::CollectorBuilder<'_, ReactionAction> {
    /// If set to `true`, added reactions will be collected.
    ///
    /// Set to `true` by default.
    pub fn added(mut self, is_accepted: bool) -> Self {
        self.filter_options.accept_added = is_accepted;

        self
    }

    /// If set to `true`, removed reactions will be collected.
    ///
    /// Set to `false` by default.
    pub fn removed(mut self, is_accepted: bool) -> Self {
        self.filter_options.accept_removed = is_accepted;

        self
    }

    impl_channel_id!("Sets the channel on which the reaction must occur. If a reaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the reaction must occur. If a reaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the reaction must occur. If a reaction is not on a message with this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of a reaction. If a reaction is not issued by a user with this ID, it won't be received.");
}

impl super::FilterOptions<ReactionAction> for FilterOptions {
    type FilterItem = Reaction;

    fn build(
        self,
        messenger: &ShardMessenger,
        common_options: CommonFilterOptions<Self::FilterItem>,
    ) -> Receiver<Arc<ReactionAction>> {
        let (filter, recv) = ReactionFilter::new(self, common_options);
        messenger.set_reaction_filter(filter);

        recv
    }
}

impl super::Collectable for ReactionAction {
    type FilterOptions = FilterOptions;
}

/// A reaction collector receives reactions matching a the given filter for a set duration.
pub type ReactionCollector = super::Collector<ReactionAction>;
pub type ReactionCollectorBuilder<'a> = super::CollectorBuilder<'a, ReactionAction>;

#[deprecated = "Use ReactionCollectorBuilder::collect_single"]
pub type CollectReaction<'a> = ReactionCollectorBuilder<'a>;
