use std::num::NonZeroU64;
use std::sync::Arc;

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{Filter, LazyArc};
use crate::model::channel::Reaction;

/// Marks whether the reaction has been added or removed.
#[derive(Debug, Clone)]
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
pub struct LazyReactionAction<'a> {
    reaction: LazyArc<'a, Reaction>,
    added: bool,
    arc: Option<Arc<ReactionAction>>,
}

impl<'a> LazyReactionAction<'a> {
    #[must_use]
    pub fn new(reaction: &'a Reaction, added: bool) -> Self {
        Self {
            reaction: LazyArc::new(reaction),
            added,
            arc: None,
        }
    }
}

impl super::LazyItem<ReactionAction> for LazyReactionAction<'_> {
    fn as_arc(&mut self) -> &mut Arc<ReactionAction> {
        let added = self.added;
        let reaction = &mut self.reaction;
        self.arc.get_or_insert_with(|| {
            Arc::new(if added {
                ReactionAction::Added(reaction.as_arc().clone())
            } else {
                ReactionAction::Removed(reaction.as_arc().clone())
            })
        })
    }
}

impl super::FilterTrait<ReactionAction> for Filter<ReactionAction> {
    fn register(self, messenger: &ShardMessenger) {
        messenger.set_reaction_filter(self);
    }

    /// Checks if the `reaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit reactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, reaction: &mut LazyReactionAction<'_>) -> bool {
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

impl super::Collectable for ReactionAction {
    type FilterItem = Reaction;
    type FilterOptions = FilterOptions;
    type LazyItem<'a> = LazyReactionAction<'a>;
}

/// A reaction collector receives reactions matching a the given filter for a set duration.
pub type ReactionCollector = super::Collector<ReactionAction>;
pub type ReactionCollectorBuilder<'a> = super::CollectorBuilder<'a, ReactionAction>;
pub type ReactionFilter = Filter<ReactionAction>;

#[deprecated = "Use ReactionCollectorBuilder::collect_single"]
pub type CollectReaction<'a> = ReactionCollectorBuilder<'a>;
