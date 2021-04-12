//! Serenity is a Rust library for the Discord API.
//!
//! View the [examples] on how to make and structure a bot.
//!
//! Serenity supports bot user authentication via the use of [`Client::builder`].
//!
//! Once logged in, you may add handlers to your client to dispatch [`Event`]s,
//! such as [`Client::on_message`]. This will cause your handler to be called
//! when a [`Event::MessageCreate`] is received. Each handler is given a
//! [`Context`], giving information about the event. See the
//! [client's module-level documentation].
//!
//! The [`Shard`] is transparently handled by the library, removing
//! unnecessary complexity. Sharded connections are automatically handled for
//! you. See the [gateway's documentation][gateway docs] for more information.
//!
//! A [`Cache`] is also provided for you. This will be updated automatically for
//! you as data is received from the Discord API via events. When calling a
//! method on a [`Context`], the cache will first be searched for relevant data
//! to avoid unnecessary HTTP requests to the Discord API. For more information,
//! see the [cache's module-level documentation][cache docs].
//!
//! Note that, although this documentation will try to be as up-to-date and
//! accurate as possible, Discord hosts [official documentation][docs]. If you
//! need to be sure that some information piece is sanctioned by Discord, refer
//! to their own documentation.
//!
//! ### Full Examples
//!
//! Full examples, detailing and explaining usage of the basic functionality of the
//! library, can be found in the [`examples`] directory.
//!
//! # Installation
//!
//! Add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! serenity = "0.10"
//! ```
//!
//! [`Context`]: crate::client::Context
//! [`Client::on_message`]: client/struct.Client.html#method.on_message
//! [`Event`]: crate::model::event::Event
//! [`Event::MessageCreate`]: crate::model::event::Event::MessageCreate
//! [`Shard`]: crate::gateway::Shard
//! [`examples`]: https://github.com/serenity-rs/serenity/blob/current/examples
//! [cache docs]: crate::cache
//! [client's module-level documentation]: crate::client
//! [docs]: https://discord.com/developers/docs/intro
//! [examples]: https://github.com/serenity-rs/serenity/tree/current/examples
//! [gateway docs]: crate::gateway
#![doc(html_root_url = "https://docs.rs/serenity/*")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rust_2018_idioms)]
#![deny(broken_intra_doc_links)]
#![deny(
    clippy::unwrap_used,
    clippy::non_ascii_literal,
    clippy::missing_errors_doc,
    clippy::let_underscore_must_use
)]
#![type_length_limit = "3294819"] // needed so ShardRunner::run compiles with instrument.

#[macro_use]
extern crate serde;

#[cfg(all(feature = "tokio_compat", not(feature = "tokio")))]
extern crate tokio_compat as tokio;

#[cfg(all(feature = "reqwest_compat", not(feature = "reqwest")))]
extern crate reqwest_compat as reqwest;

#[cfg(all(feature = "async-tungstenite_compat", not(feature = "async-tungstenite")))]
extern crate async_tungstenite_compat as async_tungstenite;

#[cfg(all(feature = "bytes_compat", not(feature = "bytes")))]
extern crate bytes_compat as bytes;

#[macro_use]
mod internal;

pub mod constants;
pub mod json;
pub mod model;
pub mod prelude;

#[cfg(feature = "builder")]
pub mod builder;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "collector")]
#[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
pub mod collector;
#[cfg(feature = "framework")]
pub mod framework;
#[cfg(feature = "gateway")]
pub mod gateway;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "utils")]
pub mod utils;

mod error;

#[cfg(feature = "client")]
use std::sync::Arc;
#[cfg(all(feature = "client", feature = "cache"))]
use std::time::Duration;

#[cfg(all(feature = "client", feature = "cache"))]
use crate::cache::Cache;
#[cfg(all(feature = "client", feature = "gateway"))]
pub use crate::client::Client;
pub use crate::error::{Error, Result};
#[cfg(feature = "client")]
use crate::http::Http;

#[cfg(feature = "client")]
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct CacheAndHttp {
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    #[cfg(feature = "cache")]
    pub update_cache_timeout: Option<Duration>,
    pub http: Arc<Http>,
}

// For the procedural macros in `command_attr`.
pub use async_trait::async_trait;
pub use futures;
pub use futures::future::FutureExt;
#[cfg(feature = "standard_framework")]
#[doc(hidden)]
pub use static_assertions;
