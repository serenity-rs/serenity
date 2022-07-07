use std::num::NonZeroU64;

use super::Filter;
use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::LazyArc;
use crate::model::channel::Message;

impl super::FilterTrait<Message> for Filter<Message> {
    fn register(self, messenger: &ShardMessenger) {
        messenger.set_message_filter(self);
    }

    /// Checks if the `message` passes set constraints.
    /// Constraints are optional, as it is possible to limit messages to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, message: &mut LazyArc<'_, Message>) -> bool {
        self.options.guild_id.map_or(true, |g| Some(g) == message.guild_id.map(|g| g.0))
            && self.options.channel_id.map_or(true, |g| g == message.channel_id.0)
            && self.options.author_id.map_or(true, |g| g == message.author.id.0)
            && self.common_options.filter.as_ref().map_or(true, |f| f.0(message))
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterOptions {
    channel_id: Option<NonZeroU64>,
    guild_id: Option<NonZeroU64>,
    author_id: Option<NonZeroU64>,
}

impl super::CollectorBuilder<'_, Message> {
    impl_channel_id!("Sets the required channel ID of a message. If a message does not meet this ID, it won't be received.");
    impl_author_id!("Sets the required author ID of a message. If a message does not meet this ID, it won't be received.");
    impl_guild_id!("Sets the required guild ID of a message. If a message does not meet this ID, it won't be received.");
}

#[nougat::gat]
impl super::Collectable for Message {
    type FilterItem = Message;
    type FilterOptions = FilterOptions;
    type LazyItem<'a> = LazyArc<'a, Message>;
}

/// A message collector receives messages matching the given filter for a set duration.
pub type MessageCollectorBuilder<'a> = super::CollectorBuilder<'a, Message>;
pub type MessageCollector = super::Collector<Message>;
pub type MessageFilter = Filter<Message>;

#[deprecated = "Use MessageCollectorBuilder::collect_single"]
pub type CollectReply<'a> = MessageCollectorBuilder<'a>;
