use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{Filter, FilterTrait, LazyArc};
use crate::model::application::interaction::message_component::MessageComponentInteraction;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};

impl FilterTrait<MessageComponentInteraction> for Filter<MessageComponentInteraction> {
    fn register(self, messenger: &ShardMessenger) {
        messenger.set_component_interaction_filter(self);
    }

    /// Checks if the `interaction` passes set constraints.
    /// Constraints are optional, as it is possible to limit interactions to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(
        &self,
        interaction: &mut LazyArc<'_, MessageComponentInteraction>,
    ) -> bool {
        self.options.guild_id.map_or(true, |id| Some(id) == interaction.guild_id)
            && self
                .options
                .custom_ids
                .as_ref()
                .map_or(true, |id| id.contains(&interaction.data.custom_id))
            && self.options.message_id.map_or(true, |id| interaction.message.id == id)
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

impl super::CollectorBuilder<'_, MessageComponentInteraction> {
    impl_channel_id!("Sets the channel on which the interaction must occur. If an interaction is not on a message with this channel ID, it won't be received.");
    impl_guild_id!("Sets the guild in which the interaction must occur. If an interaction is not on a message with this guild ID, it won't be received.");
    impl_message_id!("Sets the message on which the interaction must occur. If an interaction is not on a message with this ID, it won't be received.");
    impl_custom_ids!("Sets acceptable custom IDs for the interaction. If an interaction does not contain one of the custom IDs, it won't be received.");
    impl_author_id!("Sets the required author ID of an interaction. If an interaction is not triggered by a user with this ID, it won't be received.");
}

#[nougat::gat]
impl super::Collectable for MessageComponentInteraction {
    type FilterOptions = FilterOptions;
    type FilterItem = MessageComponentInteraction;
    type LazyItem<'a> = LazyArc<'a, MessageComponentInteraction>;
}

/// A component interaction collector receives interactions matching a the given filter for a set duration.
pub type ComponentInteractionCollector = super::Collector<MessageComponentInteraction>;
pub type ComponentInteractionFilter = Filter<MessageComponentInteraction>;
pub type ComponentInteractionCollectorBuilder<'a> =
    super::CollectorBuilder<'a, MessageComponentInteraction>;

#[deprecated = "Use ComponentInteractionCollectorBuilder::collect_single"]
pub type CollectComponentInteraction<'a> = ComponentInteractionCollectorBuilder<'a>;
