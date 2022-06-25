use std::num::NonZeroU64;
use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use super::macros::*;
use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::{CommonFilterOptions, LazyArc};
use crate::model::application::interaction::message_component::MessageComponentInteraction;

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct ComponentInteractionFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<MessageComponentInteraction>>,
    common_options: CommonFilterOptions<MessageComponentInteraction>,
}

impl ComponentInteractionFilter {
    /// Creates a new filter
    fn new(
        options: FilterOptions,
        common_options: CommonFilterOptions<MessageComponentInteraction>,
    ) -> (Self, Receiver<Arc<MessageComponentInteraction>>) {
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
    fn is_passing_constraints(&self, interaction: &MessageComponentInteraction) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id.map(|g| g.0))
            && self.options.message_id.map_or(true, |id| interaction.message.id.0 == id)
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id.as_ref().0)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id.0)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(interaction))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// An interaction is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.common_options.filter_limit.map_or(true, |limit| self.filtered < limit)
            && self.common_options.collect_limit.map_or(true, |limit| self.collected < limit)
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterOptions {
    channel_id: Option<NonZeroU64>,
    guild_id: Option<NonZeroU64>,
    author_id: Option<NonZeroU64>,
    message_id: Option<NonZeroU64>,
}

impl super::CollectorBuilder<'_, MessageComponentInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received");
}

impl super::FilterOptions<MessageComponentInteraction> for FilterOptions {
    type FilterItem = MessageComponentInteraction;

    fn build(
        self,
        messenger: &ShardMessenger,
        common_options: CommonFilterOptions<Self::FilterItem>,
    ) -> Receiver<Arc<MessageComponentInteraction>> {
        let (filter, recv) = ComponentInteractionFilter::new(self, common_options);
        messenger.set_component_interaction_filter(filter);

        recv
    }
}

impl super::Collectable for MessageComponentInteraction {
    type FilterOptions = FilterOptions;
}

/// A component interaction collector receives interactions matching a the given filter for a set duration.
pub type ComponentInteractionCollector = super::Collector<MessageComponentInteraction>;
pub type ComponentInteractionCollectorBuilder<'a> =
    super::CollectorBuilder<'a, MessageComponentInteraction>;

#[deprecated = "Use ComponentInteractionCollectorBuilder::collect_single"]
pub type CollectComponentInteraction<'a> = ComponentInteractionCollectorBuilder<'a>;
