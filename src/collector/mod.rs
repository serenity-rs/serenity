//! Collectors will receive events from the contextual shard, check if the
//! filter lets them pass, and collects if the receive, collect, or time limits
//! are not reached yet.
#[cfg(feature = "unstable_discord_api")]
pub mod component_interaction_collector;
pub mod message_collector;
pub mod reaction_collector;

#[cfg(feature = "unstable_discord_api")]
pub use component_interaction_collector::*;
pub use message_collector::*;
pub use reaction_collector::*;
