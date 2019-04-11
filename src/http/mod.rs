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
//! [`Client`]: ../client/struct.Client.html
//! [model]: ../model/index.html

pub mod ratelimiting;
pub mod raw;
pub mod request;
pub mod routing;

mod error;

pub use reqwest::StatusCode;
pub use self::error::Error as HttpError;
pub use self::raw::*;

use reqwest::{
    Method,
};
use crate::model::prelude::*;
use self::{request::Request};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// An method used for ratelimiting special routes.
///
/// This is needed because `reqwest`'s `Method` enum does not derive Copy.
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

/// Enum that allows a user to pass a `Path` or a `File` type to `send_files`
#[derive(Clone, Debug)]
pub enum AttachmentType<'a> {
    /// Indicates that the `AttachmentType` is a byte slice with a filename.
    Bytes((&'a [u8], &'a str)),
    /// Indicates that the `AttachmentType` is a `File`
    File((&'a File, &'a str)),
    /// Indicates that the `AttachmentType` is a `Path`
    Path(&'a Path),
}

impl<'a> From<(&'a [u8], &'a str)> for AttachmentType<'a> {
    fn from(params: (&'a [u8], &'a str)) -> AttachmentType<'_> { AttachmentType::Bytes(params) }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    fn from(s: &'a str) -> AttachmentType<'_> { AttachmentType::Path(Path::new(s)) }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType<'_> {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType<'_> { AttachmentType::Path(pathbuf.as_path()) }
}

impl<'a> From<(&'a File, &'a str)> for AttachmentType<'a> {
    fn from(f: (&'a File, &'a str)) -> AttachmentType<'a> { AttachmentType::File((f.0, f.1)) }
}

/// Representation of the method of a query to send for the [`get_guilds`]
/// function.
///
/// [`get_guilds`]: fn.get_guilds.html
pub enum GuildPagination {
    /// The Id to get the guilds after.
    After(GuildId),
    /// The Id to get the guilds before.
    Before(GuildId),
}

#[cfg(test)]
mod test {
    use super::AttachmentType;
    use std::path::Path;

    #[test]
    fn test_attachment_type() {
        assert!(match AttachmentType::from(Path::new("./dogs/corgis/kona.png")) {
            AttachmentType::Path(_) => true,
            _ => false,
        });
        assert!(match AttachmentType::from("./cats/copycat.png") {
            AttachmentType::Path(_) => true,
            _ => false,
        });
    }
}
