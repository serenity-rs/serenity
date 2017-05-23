//! The gateway module contains the pieces - primarily the `Shard` -
//! responsible for maintaing a WebSocket connection with Discord.
//!
//! A shard is an interface for the lower-level receiver and sender. It provides
//! what can otherwise be thought of as "sugar methods". A shard represents a
//! single connection to Discord. If acting as a [`Bot`] user, you can make
//! use of a method named "sharding" to have multiple shards, potentially
//! offloading some server load to another server(s).
//!
//! # Sharding
//!
//! Sharding is a method to split portions of bots into separate processes. This
//! is an enforced strategy by Discord once a bot reaches a certain number of
//! guilds (2500). Once this number is reached, a bot must be sharded in a way
//! that only 2500 guilds maximum may be allocated per shard.
//!
//! The "recommended" number of guilds per shard is _around_ 1000. Sharding can
//! be useful for splitting processes across separate servers. Often you may
//! want some or all shards to be in the same process, allowing for a shared
//! State. This is possible through this library.
//!
//! See [Discord's documentation][docs] for more information.
//!
//! If you are not using a bot account or do not require sharding - such as for
//! a small bot - then use [`Client::start`].
//!
//! There are a few methods of sharding available:
//!
//! - [`Client::start_autosharded`]: retrieves the number of shards Discord
//! recommends using from the API, and then automatically starts that number of
//! shards.
//! - [`Client::start_shard`]: starts a single shard for use in the instance,
//! handled by the instance of the Client. Use this if you only want 1 shard
//! handled by this instance.
//! - [`Client::start_shards`]: starts all shards in this instance. This is best
//! for when you want a completely shared State.
//! - [`Client::start_shard_range`]: start a range of shards within this
//! instance. This should be used when you, for example, want to split 10 shards
//! across 3 instances.
//!
//! **Note**: User accounts can not shard. Use [`Client::start`].
//!
//! [`Bot`]: ../enum.LoginType.html#variant.Bot
//! [`Client`]: ../struct.Client.html
//! [`Client::start`]: ../struct.Client.html#method.start
//! [`Client::start_autosharded`]: ../struct.Client.html#method.start_autosharded
//! [`Client::start_shard`]: ../struct.Client.html#method.start_shard
//! [`Client::start_shard_range`]: ../struct.Client.html#method.start_shard_range
//! [`Client::start_shards`]: ../struct.Client.html#method.start_shards
//! [docs]: https://discordapp.com/developers/docs/topics/gateway#sharding

mod error;
mod prep;
mod shard;
mod status;

pub use self::error::Error as GatewayError;
pub use self::shard::Shard;
pub use self::status::Status as GatewayStatus;
