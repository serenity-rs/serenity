//! The HTTP module which provides functions for performing requests to endpoints in Discord's API.
//!
//! An important function of the REST API is ratelimiting. Requests to endpoints are ratelimited to
//! prevent spam, and once ratelimited Discord will stop performing requests. The library
//! implements protection to pre-emptively ratelimit, to ensure that no wasted requests are made.
//!
//! The HTTP module comprises of two types of requests:
//! - REST API requests, which require an authorization token;
//! - Other requests, which do not require an authorization token.
//!
//! The former require a [`Client`] to have logged in, while the latter may be made regardless of
//! any other usage of the library.
//!
//! If a request spuriously fails, it will be retried once.
//!
//! Note that you may want to perform requests through a [model]s' instance methods where possible,
//! as they each offer different levels of a high-level interface to the HTTP module.
//!
//! [`Client`]: crate::Client
//! [model]: crate::model

mod client;
mod error;
mod multipart;
mod ratelimiting;
mod request;
mod routing;

use std::time::Duration;

use reqwest::Method;
pub use reqwest::StatusCode;
use serde::{Deserialize, Deserializer};

pub use self::client::*;
pub use self::error::*;
pub use self::multipart::*;
pub use self::ratelimiting::*;
pub use self::request::*;
pub use self::routing::*;
use crate::internal::prelude::*;
use crate::model::channel::{GuildThread, Message, ThreadMember};
use crate::model::id::*;

/// An method used for ratelimiting special routes.
///
/// This is needed because [`reqwest`]'s [`Method`] enum does not derive Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum LightMethod {
    /// Indicates that a route is for the `DELETE` method only.
    Delete,
    /// Indicates that a route is for the `GET` method only.
    Get,
    /// Indicates that a route is for the `PATCH` method only.
    Patch,
    /// Indicates that a route is for the `POST` method only.
    Post,
    /// Indicates that a route is for the `PUT` method only.
    Put,
}

impl LightMethod {
    #[must_use]
    pub const fn reqwest_method(self) -> Method {
        match self {
            Self::Delete => Method::DELETE,
            Self::Get => Method::GET,
            Self::Patch => Method::PATCH,
            Self::Post => Method::POST,
            Self::Put => Method::PUT,
        }
    }
}

/// Representation of the method of a query to send for the [`Http::get_guilds`] function.
#[non_exhaustive]
pub enum GuildPagination {
    /// The Id to get the guilds after.
    After(GuildId),
    /// The Id to get the guilds before.
    Before(GuildId),
}

/// Representation of the method of a query to send for the [`Http::get_scheduled_event_users`] and
/// [`Http::get_bans`] functions.
#[non_exhaustive]
pub enum UserPagination {
    /// The Id to get the users after.
    After(UserId),
    /// The Id to get the users before.
    Before(UserId),
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum MessagePagination {
    After(MessageId),
    Around(MessageId),
    Before(MessageId),
}

/// Outcome of a guild message search.
///
/// This is an unusual response type; it can hold either an error-like response or search results.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages).
#[derive(Clone, Debug)]
pub enum MessageSearchOutcome {
    NotIndexed(MessageSearchNotIndexed),
    Results(MessageSearchResults),
}

fn duration_via_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // It's actually an i32 in the spec, but that's because JSON doesn't have unsigned types.
    // It would make no sense for the value to ever be negative, so we can act as if it's a u64.
    u64::deserialize(deserializer).map(Duration::from_secs)
}

/// Error-like response to a guild message search when indexing is not yet complete.
///
/// > You should retry the request after the timeframe specified in the `retry_after` field. If the
/// > `retry_after` field is 0, you should retry the request after a short delay.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages).
#[derive(Clone, Debug, Deserialize)]
pub struct MessageSearchNotIndexed {
    pub code: JsonErrorCode,
    pub message: FixedString,
    pub documents_indexed: u32,
    #[serde(deserialize_with = "duration_via_secs")]
    pub retry_after: Duration,
}

/// Unnest a `Vec` of single-element `Vec`s.
fn de_flatten_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    <Vec<[T; 1]>>::deserialize(deserializer).map(Vec::into_flattened)
}

/// Successful result of a guild message search.
///
/// > Due to speed optimizations, search may return slightly fewer results than the limit specified
/// > when messages have not been accessed for a long time. Clients should not rely on the length of
/// > the `messages` array to paginate results.
/// >
/// > Additionally, when messages are actively being created or deleted, the `total_results` field
/// > may not be accurate.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#search-guild-messages).
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageSearchResults {
    /// Whether the guild is undergoing a deep historical indexing operation.
    pub doing_deep_historical_index: bool,
    /// The number of documents that have been indexed during the current index operation, if any.
    pub documents_indexed: Option<u32>,
    /// The total number of results that match the query.
    pub total_results: u32,
    /// The messages that match the query.
    #[serde(deserialize_with = "de_flatten_vec")]
    pub messages: Vec<Message>,
    /// The threads that contain returned messages (if any).
    pub threads: Option<Vec<GuildThread>>,
    /// A thread member object for each returned thread the current user has joined.
    pub members: Option<Vec<ThreadMember>>,
}
