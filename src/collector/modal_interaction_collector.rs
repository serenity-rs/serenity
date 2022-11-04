use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{Filter, LazyArc};
use crate::model::application::interaction::modal::ModalInteraction;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};

impl super::FilterTrait<ModalInteraction> for Filter<ModalInteraction> {
    fn register(self, messenger: &ShardMessenger) {
        messenger.set_modal_interaction_filter(self);
    }

    fn is_passing_constraints(&self, interaction: &mut LazyArc<'_, ModalInteraction>) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id)
            && self
                .options
                .custom_ids
                .as_ref()
                .map_or(true, |id| id.contains(&interaction.data.custom_id))
            && self
                .options
                .message_id
                .map_or(true, |id| Some(id) == interaction.message.as_ref().map(|m| m.id))
            && self.options.channel_id.map_or(true, |id| id == interaction.channel_id)
            && self.options.author_id.map_or(true, |id| id == interaction.user.id)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(interaction))
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterOptions {
    channel_id: Option<ChannelId>,
    guild_id: Option<GuildId>,
    author_id: Option<UserId>,
    message_id: Option<MessageId>,
    custom_ids: Option<Vec<String>>,
}

impl super::CollectorBuilder<'_, ModalInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_custom_ids!("Sets acceptable custom IDs for the interaction. If an interaction does not contain one of the custom IDs, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received.");
}

impl super::Collectable for ModalInteraction {
    type FilterOptions = FilterOptions;
    type FilterItem = ModalInteraction;
    type LazyItem<'a> = LazyArc<'a, ModalInteraction>;
}

/// A modal interaction collector receives interactions matching a the given filter for a set duration.
pub type ModalInteractionCollector = super::Collector<ModalInteraction>;
pub type ModalInteractionCollectorBuilder<'a> = super::CollectorBuilder<'a, ModalInteraction>;
pub type ModalInteractionFilter = Filter<ModalInteraction>;

#[deprecated = "Use ModalInteractionCollectorBuilder::collect_single"]
pub type CollectModalInteraction<'a> = ModalInteractionCollectorBuilder<'a>;
