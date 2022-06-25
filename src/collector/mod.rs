//! Collectors will receive events from the contextual shard, check if the
//! filter lets them pass, and collects if the receive, collect, or time limits
//! are not reached yet.

use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;
use tokio::time::Sleep;

use crate::client::bridge::gateway::ShardMessenger;

mod error;
mod macros;
pub use error::Error as CollectorError;

mod base_collector;
use base_collector::Collector;

pub mod component_interaction_collector;
pub mod event_collector;
pub mod message_collector;
pub mod modal_interaction_collector;
pub mod reaction_collector;

pub use component_interaction_collector::*;
pub use event_collector::*;
pub use message_collector::*;
pub use modal_interaction_collector::*;
pub use reaction_collector::*;

#[derive(Clone)]
pub struct FilterFn<Arg: ?Sized>(Arc<dyn Fn(&Arg) -> bool + 'static + Send + Sync>);
impl<Arg> fmt::Debug for FilterFn<Arg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FilterFn")
            .field(&format_args!("Arc<dyn Fn({}) -> bool", stringify!(Arg)))
            .finish()
    }
}

/// Wraps a &T and clones the value into an Arc<T> lazily. Used with collectors to allow inspecting
/// the value in filters while only cloning values that actually match.
#[derive(Debug)]
pub(crate) struct LazyArc<'a, T> {
    value: &'a T,
    arc: Option<Arc<T>>,
}

impl<'a, T: Clone> LazyArc<'a, T> {
    pub fn new(value: &'a T) -> Self {
        LazyArc {
            value,
            arc: None,
        }
    }

    pub fn as_arc(&mut self) -> Arc<T> {
        let value = self.value;
        self.arc.get_or_insert_with(|| Arc::new(value.clone())).clone()
    }
}

impl<'a, T> std::ops::Deref for LazyArc<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

mod sealed {
    use crate::model::prelude::*;

    pub trait Sealed {}

    impl Sealed for Event {}
    impl Sealed for Message {}
    impl Sealed for crate::collector::ReactionAction {}
    impl Sealed for interaction::modal::ModalSubmitInteraction {}
    impl Sealed for interaction::message_component::MessageComponentInteraction {}
}

pub trait FilterOptions<Item: ?Sized> {
    type FilterItem;

    fn build(
        self,
        messenger: &ShardMessenger,
        common: CommonFilterOptions<Self::FilterItem>,
    ) -> Receiver<Arc<Item>>;
}

pub trait Collectable {
    type FilterOptions: FilterOptions<Self> + Default;
}

#[derive(Clone, Debug)]
pub struct CommonFilterOptions<FilterItem> {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<FilterFn<FilterItem>>,
}

// Needed to prevent `FilterItem` needing a `Default` bound, which `derive` adds.
impl<FilterItem> Default for CommonFilterOptions<FilterItem> {
    fn default() -> Self {
        Self {
            collect_limit: None,
            filter_limit: None,
            filter: None,
        }
    }
}

#[must_use = "Builders must be built"]
pub struct CollectorBuilder<'a, Item: Collectable + sealed::Sealed> {
    common_options: CommonFilterOptions<<Item::FilterOptions as FilterOptions<Item>>::FilterItem>,
    filter_options: Item::FilterOptions,
    shard_messenger: &'a ShardMessenger,
    timeout: Option<Pin<Box<Sleep>>>,
}

impl<'a, Item: Collectable + sealed::Sealed> CollectorBuilder<'a, Item> {
    pub fn new(shard_messenger: &'a ShardMessenger) -> Self {
        Self {
            shard_messenger,

            timeout: None,
            common_options: CommonFilterOptions::default(),
            filter_options: Item::FilterOptions::default(),
        }
    }

    pub fn build(self) -> Collector<Item> {
        let receiver = self.filter_options.build(self.shard_messenger, self.common_options);

        Collector {
            timeout: self.timeout,
            receiver: Box::pin(receiver),
        }
    }

    /// Sets a filter function where items passed to the `function` must return `true`,
    /// otherwise the item won't be collected and failed the filter process.
    ///
    /// This is the last instance to pass for an item to count as *collected*.
    pub fn filter<
        F: Fn(&<Item::FilterOptions as FilterOptions<Item>>::FilterItem) -> bool
            + 'static
            + Send
            + Sync,
    >(
        mut self,
        function: F,
    ) -> Self {
        self.common_options.filter = Some(FilterFn(Arc::new(function)));

        self
    }

    /// Limits how many items can be collected.
    ///
    /// An item is considered *collected*, if the message passes all the requirements.
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.common_options.collect_limit = Some(limit);

        self
    }

    /// Limits how many events will attempt to be filtered.
    pub fn filter_limit(mut self, limit: u32) -> Self {
        self.common_options.filter_limit = Some(limit);

        self
    }

    /// Sets a [`Duration`] for how long the collector shall receive events.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(Box::pin(tokio::time::sleep(duration)));

        self
    }
}

impl<Item: Collectable + Send + Sync + sealed::Sealed + 'static> CollectorBuilder<'_, Item> {
    pub async fn collect_single(self) -> Option<Arc<Item>> {
        let mut collector = self.build();
        collector.next().await
    }
}
