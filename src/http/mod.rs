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
mod typing;

use std::sync::Arc;

use reqwest::Method;
pub use reqwest::StatusCode;

pub use self::client::*;
pub use self::error::*;
pub use self::multipart::*;
pub use self::ratelimiting::*;
pub use self::request::*;
pub use self::routing::*;
pub use self::typing::*;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "gateway")]
use crate::gateway::client::Context;
use crate::model::prelude::*;

/// This trait will be required by functions that need [`Http`] and can optionally use a [`Cache`]
/// to potentially avoid REST-requests.
///
/// The types [`Context`] and [`Http`] implement this trait and thus passing these to functions
/// expecting `impl CacheHttp` is possible. For the full list of implementations, see the
/// Implementors and Implementations on Foreign Types section in the generated docs.
///
/// In a situation where you have the `cache`-feature enabled but you do not pass a cache, the
/// function will behave as if no `cache`-feature is active.
///
/// If you are calling a function that expects `impl CacheHttp` as argument and you wish to utilise
/// the `cache`-feature but you got no access to a [`Context`], you can pass a tuple of
/// `(&Arc<Cache>, &Http)`.
pub trait CacheHttp: Send + Sync {
    fn http(&self) -> &Http;

    #[cfg(feature = "cache")]
    #[must_use]
    fn cache(&self) -> Option<&Arc<Cache>> {
        None
    }
}

impl<T> CacheHttp for &T
where
    T: CacheHttp,
{
    fn http(&self) -> &Http {
        (*self).http()
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        (*self).cache()
    }
}

impl<T> CacheHttp for Arc<T>
where
    T: CacheHttp,
{
    fn http(&self) -> &Http {
        (**self).http()
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        (**self).cache()
    }
}

#[cfg(feature = "gateway")]
impl CacheHttp for Context {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

#[cfg(feature = "cache")]
impl CacheHttp for (Option<&Arc<Cache>>, &Http) {
    fn cache(&self) -> Option<&Arc<Cache>> {
        self.0
    }

    fn http(&self) -> &Http {
        self.1
    }
}

impl CacheHttp for Http {
    fn http(&self) -> &Http {
        self
    }
}

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
