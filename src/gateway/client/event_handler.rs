use async_trait::async_trait;

use super::context::Context;
#[cfg(doc)]
use crate::gateway::ShardRunner;
use crate::http::RatelimitInfo;
use crate::model::event::{Event, FullEvent};

#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Checks if the `event` should be dispatched or ignored. Returns Some(event) by default.
    ///
    /// Returning `None` will drop the event and never process it.
    ///
    /// Returning a mutated `Some` is a supported use case, but may cause contradictions to
    /// documented behaviour or otherwise cause unexpected errors within serenity/library
    /// consumers. Mutate with extreme caution.
    ///
    /// ## Warning
    ///
    /// Similar to [`RawEventHandler`], this method runs synchronously to the [`ShardRunner`], keep
    /// runtime complexity low.
    fn filter_event(&self, _context: &Context, event: Box<Event>) -> Option<Box<Event>> {
        Some(event)
    }

    /// Dispatches an event through this handler, allowing for event matching and handling
    /// based on individual event variants.
    async fn dispatch(&self, _context: &Context, _event: &FullEvent) {}

    /// Dispatched when an HTTP rate limit is hit.
    async fn ratelimit(&self, _data: RatelimitInfo) {}
}

/// An event handler that receives raw `dispatch` events.
///
/// ## Warning
/// As this is a low level trait, the methods of this trait are run on the same tokio task as the
/// [`ShardRunner`].
///
/// This means that if any of these methods take too long to return, the shard may drop events or be
/// disconnected entirely.
///
/// It is recommended to clone the fields needed out of [`Event`], then spawn a task to run
/// concurrently to the shard loop.
#[async_trait]
pub trait RawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&self, _ctx: Context, _ev: &Event) {}

    /// Checks if the `event` should be dispatched or ignored. Returns Some(event) by default.
    ///
    /// Returning `None` will drop the event and never process it.
    ///
    /// Returning a mutated `Some` is a supported use case, but may cause contradictions to
    /// documented behaviour or otherwise cause unexpected errors within serenity/library
    /// consumers. Mutate with extreme caution.
    fn filter_event(&self, _context: &Context, event: Box<Event>) -> Option<Box<Event>> {
        Some(event)
    }
}
