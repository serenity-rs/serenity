//! The HTTP module which provides functions for performing requests to
//! endpoints in Discord's API.
//!
//! An important function of the REST API is ratelimiting. Requests to endpoints
//! are ratelimited to prevent spam, and once ratelimited Discord will stop
//! performing requests. The library implements protection to pre-emptively
//! ratelimit, to ensure that no wasted requests are made.
//!
//! The HTTP module comprises of two types of requests:
//!
//! - REST API requests, which require an authorization token;
//! - Other requests, which do not require an authorization token.
//!
//! The former require a [`Client`] to have logged in, while the latter may be
//! made regardless of any other usage of the library.
//!
//! If a request spuriously fails, it will be retried once.
//!
//! Note that you may want to perform requests through a [model]s'
//! instance methods where possible, as they each offer different
//! levels of a high-level interface to the HTTP module.
//!
//! [`Client`]: crate::Client
//! [model]: crate::model

pub mod client;
pub mod error;
pub mod ratelimiting;
pub mod request;
pub mod routing;
pub mod typing;
pub mod utils;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::Arc,
};

use reqwest::Method;
pub use reqwest::StatusCode;
use tokio::fs::File;

pub use self::client::*;
pub use self::error::Error as HttpError;
use self::request::Request;
pub use self::typing::*;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "client")]
use crate::client::Context;
use crate::model::prelude::*;
#[cfg(feature = "client")]
use crate::CacheAndHttp;

/// This trait will be required by functions that need [`Http`] and can
/// optionally use a [`Cache`] to potentially avoid REST-requests.
///
/// The types [`Context`], [`Cache`], and [`Http`] implement this trait
/// and thus passing these to functions expecting `impl CacheHttp` is possible.
///
/// In a situation where you have the `cache`-feature enabled but you do not
/// pass a cache, the function will behave as if no `cache`-feature is active.
///
/// If you are calling a function that expects `impl CacheHttp` as argument
/// and you wish to utilise the `cache`-feature but you got no access to a
/// [`Context`], you can pass a tuple of `(CacheRwLock, Http)`.
pub trait CacheHttp: Send + Sync {
    fn http(&self) -> &Http;
    #[cfg(feature = "cache")]
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

#[cfg(feature = "client")]
impl CacheHttp for Context {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for CacheAndHttp {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for Arc<CacheAndHttp> {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

#[cfg(feature = "cache")]
impl CacheHttp for (&Arc<Cache>, &Http) {
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.0)
    }
    fn http(&self) -> &Http {
        &self.1
    }
}

impl CacheHttp for Arc<Http> {
    fn http(&self) -> &Http {
        &*self
    }
}

#[cfg(feature = "cache")]
impl AsRef<Cache> for (&Arc<Cache>, &Http) {
    fn as_ref(&self) -> &Cache {
        &**self.0
    }
}

#[cfg(feature = "cache")]
impl AsRef<Http> for (&Arc<Cache>, &Http) {
    fn as_ref(&self) -> &Http {
        self.1
    }
}

/// An method used for ratelimiting special routes.
///
/// This is needed because [`reqwest`]'s [`Method`] enum does not derive Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    pub fn reqwest_method(self) -> Method {
        match self {
            LightMethod::Delete => Method::DELETE,
            LightMethod::Get => Method::GET,
            LightMethod::Patch => Method::PATCH,
            LightMethod::Post => Method::POST,
            LightMethod::Put => Method::PUT,
        }
    }
}

/// Enum that allows a user to pass a [`Path`] or a [`File`] type to [`send_files`]
///
/// [`send_files`]: crate::model::id::ChannelId::send_files
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AttachmentType<'a> {
    /// Indicates that the [`AttachmentType`] is a byte slice with a filename.
    Bytes { data: Cow<'a, [u8]>, filename: String },
    /// Indicates that the [`AttachmentType`] is a [`File`]
    File { file: &'a File, filename: String },
    /// Indicates that the [`AttachmentType`] is a [`Path`]
    Path(&'a Path),
    /// Indicates that the [`AttachmentType`] is an image URL.
    Image(&'a str),
}

impl<'a> From<(&'a [u8], &str)> for AttachmentType<'a> {
    fn from(params: (&'a [u8], &str)) -> AttachmentType<'a> {
        AttachmentType::Bytes {
            data: Cow::Borrowed(params.0),
            filename: params.1.to_string(),
        }
    }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    /// Constructs an [`AttachmentType`] from a string.
    /// This string may refer to the path of a file on disk, or the http url to an image on the internet.
    fn from(s: &'a str) -> AttachmentType<'_> {
        if s.starts_with("http://") || s.starts_with("https://") {
            AttachmentType::Image(s)
        } else {
            AttachmentType::Path(Path::new(s))
        }
    }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType<'_> {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType<'_> {
        AttachmentType::Path(pathbuf.as_path())
    }
}

impl<'a> From<(&'a File, &str)> for AttachmentType<'a> {
    fn from(f: (&'a File, &str)) -> AttachmentType<'a> {
        AttachmentType::File {
            file: f.0,
            filename: f.1.to_string(),
        }
    }
}

/// Representation of the method of a query to send for the [`get_guilds`]
/// function.
///
/// [`get_guilds`]: Http::get_guilds
#[non_exhaustive]
pub enum GuildPagination {
    /// The Id to get the guilds after.
    After(GuildId),
    /// The Id to get the guilds before.
    Before(GuildId),
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::AttachmentType;

    #[test]
    fn test_attachment_type() {
        assert!(matches!(
            AttachmentType::from(Path::new("./dogs/corgis/kona.png")),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from(Path::new("./cats/copycat.png")),
            AttachmentType::Path(_)
        ));
    }
}
