use std::num::NonZeroU64;
use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use crate::client::bridge::gateway::ShardMessenger;
use crate::collector::macros::*;
use crate::collector::{FilterFn, LazyArc};
use crate::model::channel::Message;

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct MessageFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<Message>>,

    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<FilterFn<Message>>,
}

impl MessageFilter {
    /// Creates a new filter
    fn new(
        options: FilterOptions,
        filter_limit: Option<u32>,
        collect_limit: Option<u32>,
        filter: Option<FilterFn<Message>>,
    ) -> (Self, Receiver<Arc<Message>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            filter,
            options,
            filter_limit,
            collect_limit,
        };

        (filter, receiver)
    }

    /// Sends a `message` to the consuming collector if the `message` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_message(&mut self, message: &mut LazyArc<'_, Message>) -> bool {
        if self.is_passing_constraints(message) {
            self.collected += 1;

            if self.sender.send(message.as_arc()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }

    /// Checks if the `message` passes set constraints.
    /// Constraints are optional, as it is possible to limit messages to
    /// be sent by a specific author or in a specific guild.
    fn is_passing_constraints(&self, message: &Message) -> bool {
        self.options.guild_id.map_or(true, |g| Some(g) == message.guild_id.map(|g| g.0))
            && self.options.channel_id.map_or(true, |g| g == message.channel_id.0)
            && self.options.author_id.map_or(true, |g| g == message.author.id.0)
            && self.filter.as_ref().map_or(true, |f| f.0(message))
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A message is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.filter_limit.as_ref().map_or(true, |limit| self.filtered < *limit)
            && self.collect_limit.as_ref().map_or(true, |limit| self.collected < *limit)
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

impl super::FilterOptions<Message> for FilterOptions {
    type FilterItem = Message;

    fn build(
        self,
        messenger: &ShardMessenger,
        filter_limit: Option<u32>,
        collect_limit: Option<u32>,
        filter: Option<FilterFn<Self::FilterItem>>,
    ) -> Receiver<Arc<Message>> {
        let (filter, recv) = MessageFilter::new(self, filter_limit, collect_limit, filter);
        messenger.set_message_filter(filter);

        recv
    }
}

impl super::Collectable for Message {
    type FilterOptions = FilterOptions;
}

/// A message collector receives messages matching the given filter for a set duration.
pub type MessageCollectorBuilder<'a> = super::CollectorBuilder<'a, Message>;
pub type MessageCollector = super::Collector<Message>;

#[deprecated = "Use MessageCollectorBuilder::collect_single"]
pub type CollectReply<'a> = MessageCollectorBuilder<'a>;
