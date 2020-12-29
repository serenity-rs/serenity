//! Collectors will receive events from the contextual shard, check if the
//! filter lets them pass, and collects if the receive, collect, or time limits
//! are not reached yet.
pub mod message_collector;
pub mod reaction_collector;

pub use message_collector::*;
pub use reaction_collector::*;
