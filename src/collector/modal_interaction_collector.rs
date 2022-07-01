use std::num::NonZeroU64;

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{Filter, LazyArc};
use crate::model::application::interaction::modal::ModalSubmitInteraction;

impl super::FilterTrait<ModalSubmitInteraction> for Filter<ModalSubmitInteraction> {
    fn register(self, messenger: &ShardMessenger) {
        messenger.set_modal_interaction_filter(self);
    }

    fn is_passing_constraints(
        &self,
        interaction: &mut LazyArc<'_, ModalSubmitInteraction>,
    ) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id.map(|g| g.0))
            && self
                .options
                .message_id
                .map_or(true, |id| Some(id) == interaction.message.as_ref().map(|m| m.id.0))
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id.as_ref().0)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id.0)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(interaction))
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterOptions {
    channel_id: Option<NonZeroU64>,
    guild_id: Option<NonZeroU64>,
    author_id: Option<NonZeroU64>,
    message_id: Option<NonZeroU64>,
}

impl super::CollectorBuilder<'_, ModalSubmitInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received");
}

impl super::Collectable for ModalSubmitInteraction {
    type FilterOptions = FilterOptions;
    type FilterItem = ModalSubmitInteraction;
    type LazyItem<'a> = LazyArc<'a, ModalSubmitInteraction>;
}

/// A modal interaction collector receives interactions matching a the given filter for a set duration.
pub type ModalInteractionCollector = super::Collector<ModalSubmitInteraction>;
pub type ModalInteractionCollectorBuilder<'a> = super::CollectorBuilder<'a, ModalSubmitInteraction>;
pub type ModalInteractionFilter = Filter<ModalSubmitInteraction>;

#[deprecated = "Use ModalInteractionCollectorBuilder::collect_single"]
pub type CollectModalInteraction<'a> = ModalInteractionCollectorBuilder<'a>;
