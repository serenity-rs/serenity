use std::sync::Arc;

use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};

use super::{Collectable, CommonFilterOptions, LazyItem, LazyItemGat};
use crate::client::bridge::gateway::ShardMessenger;

pub trait FilterTrait<Item: Collectable> {
    fn register(self, messenger: &ShardMessenger);
    fn is_passing_constraints(&self, item: &mut LazyItemGat<'_, Item>) -> bool;
}

#[derive(Clone, Debug)]
pub struct Filter<Item: Collectable> {
    pub(super) filtered: u32,
    pub(super) collected: u32,
    pub(crate) sender: Sender<Arc<Item>>,
    pub(super) options: Item::FilterOptions,
    pub(super) common_options: CommonFilterOptions<Item::FilterItem>,
}

impl<Item: Collectable> Filter<Item> {
    /// Creates a new filter
    pub fn new(
        options: Item::FilterOptions,
        common_options: CommonFilterOptions<Item::FilterItem>,
    ) -> (Self, Receiver<Arc<Item>>) {
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

    fn is_within_limits(&self) -> bool {
        self.common_options.filter_limit.map_or(true, |limit| self.filtered < limit.get())
            && self.common_options.collect_limit.map_or(true, |limit| self.collected < limit.get())
    }
}

impl<Item: Collectable> Filter<Item>
where
    Filter<Item>: FilterTrait<Item>,
{
    /// Sends an item to the consuming collector if the item conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn process_item(&mut self, item: &mut LazyItemGat<'_, Item>) -> bool {
        if self.is_passing_constraints(item) {
            self.collected += 1;

            if self.sender.send(item.as_arc().clone()).is_err() {
                return false;
            }
        }

        self.filtered += 1;

        self.is_within_limits() && !self.sender.is_closed()
    }
}
