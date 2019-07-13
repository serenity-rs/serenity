//! Serenity is a Rust library for the Discord API.
//!
//! View the [examples] on how to make and structure a bot.
//!
//! Serenity supports bot user authentication via the use of [`Client::new`].
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
//! serenity = "0.7"
//! ```
//!
//! [`Cache`]: cache/struct.Cache.html
//! [`Client::new`]: client/struct.Client.html#method.new
//! [`Client::on_message`]: client/struct.Client.html#method.on_message
//! [`Context`]: client/struct.Context.html
//! [`Event`]: model/event/enum.Event.html
//! [`Event::MessageCreate`]: model/event/enum.Event.html#variant.MessageCreate
//! [`Shard`]: gateway/struct.Shard.html
//! [`examples`]: https://github.com/serenity-rs/serenity/blob/current/examples
//! [cache docs]: cache/index.html
//! [client's module-level documentation]: client/index.html
//! [docs]: https://discordapp.com/developers/docs/intro
//! [examples]: https://github.com/serenity-rs/serenity/tree/current/examples
//! [gateway docs]: gateway/index.html
#![doc(html_root_url = "https://docs.rs/serenity/*")]
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate serde;

#[macro_use]
mod internal;

pub mod constants;
pub mod model;
pub mod prelude;

#[cfg(feature = "builder")]
pub mod builder;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "framework")]
pub mod framework;
#[cfg(feature = "gateway")]
pub mod gateway;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "voice")]
pub mod voice;

mod error;

pub use crate::error::{Error, Result};

#[cfg(feature = "client")]
pub use crate::client::Client;

#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;
#[cfg(feature = "cache")]
use std::time::Duration;
#[cfg(any(feature = "client", feature = "http"))]
use std::sync::Arc;
#[cfg(all(feature = "client", feature = "http"))]
use crate::http::Http;


#[cfg(feature = "client")]
#[derive(Default)]
pub struct CacheAndHttp {
    #[cfg(feature = "cache")]
    pub cache: CacheRwLock,
    #[cfg(feature = "cache")]
    pub update_cache_timeout: Option<Duration>,
    #[cfg(feature = "http")]
    pub http: Arc<Http>,
    __nonexhaustive: (),
}

// For the procedural macros defined in `command_attr`; do not remove!
#[allow(clippy::useless_attribute)]
#[allow(rust_2018_idioms)]
extern crate self as serenity;
