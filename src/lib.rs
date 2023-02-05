//! Serenity is a Rust library for the Discord API.
//!
//! View the [examples] on how to make and structure a bot.
//!
//! Serenity supports bot user authentication via the use of [`Client::builder`].
//!
//! Once logged in, you may add handlers to your client to dispatch [`Event`]s, such as
//! [`EventHandler::message`]. This will cause your handler to be called when a
//! [`Event::MessageCreate`] is received. Each handler is given a [`Context`], giving information
//! about the event. See the [client's module-level documentation].
//!
//! The [`Shard`] is transparently handled by the library, removing unnecessary complexity. Sharded
//! connections are automatically handled for you. See the [gateway's documentation][gateway docs]
//! for more information.
//!
//! A [`Cache`] is also provided for you. This will be updated automatically for you as data is
//! received from the Discord API via events. When calling a method on a [`Context`], the cache
//! will first be searched for relevant data to avoid unnecessary HTTP requests to the Discord API.
//! For more information, see the [cache's module-level documentation][cache docs].
//!
//! Note that, although this documentation will try to be as up-to-date and accurate as possible,
//! Discord hosts [official documentation][docs]. If you need to be sure that some information
//! piece is sanctioned by Discord, refer to their own documentation.
//!
//! ### Full Examples
//!
//! Full examples, detailing and explaining usage of the basic functionality of the library, can be
//! found in the [`examples`] directory.
//!
//! # Installation
//!
//! Add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! serenity = "0.11"
//! ```
//!
//! [`Cache`]: crate::cache::Cache
//! [`Context`]: crate::client::Context
//! [`EventHandler::message`]: crate::client::EventHandler::message
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
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    unused,
    rust_2018_idioms,
    clippy::unwrap_used,
    clippy::non_ascii_literal,
    clippy::fallible_impl_from,
    clippy::let_underscore_must_use,
    clippy::pedantic
)]
#![allow(
    // Allowed to avoid breaking changes.
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::unused_self,
    // Allowed as they are too pedantic
    clippy::cast_possible_truncation,
    clippy::unreadable_literal,
    clippy::cast_possible_wrap,
    clippy::wildcard_imports,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    // Covered by other lints
    clippy::missing_panics_doc, // clippy::unwrap_used
)]
#![cfg_attr(test, allow(clippy::unwrap_used))]
#![type_length_limit = "3294819"] // needed so ShardRunner::run compiles with instrument.

#[macro_use]
extern crate serde;

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
pub mod collector;
#[cfg(feature = "framework")]
pub mod framework;
#[cfg(feature = "gateway")]
pub mod gateway;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "interactions_endpoint")]
pub mod interactions_endpoint;
#[cfg(feature = "utils")]
pub mod utils;

mod error;

// For the procedural macros in `command_attr`.
pub use async_trait::async_trait;
pub use futures;
pub use futures::future::FutureExt;
#[cfg(feature = "standard_framework")]
#[doc(hidden)]
pub use static_assertions;

#[cfg(all(feature = "client", feature = "gateway"))]
pub use crate::client::Client;
pub use crate::error::{Error, Result};

#[cfg(feature = "absolute_ratelimits")]
compile_error!(
    "The absolute_ratelimits feature has been removed.\n\
    Configure absolute ratelimits via Ratelimiter::set_absolute_ratelimits.\n\
    You can set the Ratelimiter of Http via HttpBuilder::ratelimiter."
);

/// Special module that re-exports most public items from this crate.
///
/// Useful, because you don't have to remember the full paths of serenity items.
///
/// Not exported:
/// - [`crate::json`]: it's a general-purpose JSON wrapper, not intrinsic to serenity
/// - [`crate::framework::standard`]: has many standard_framework-specific items that may collide
/// with items from the rest of serenity
pub mod all {
    #[cfg(feature = "builder")]
    #[doc(no_inline)]
    pub use crate::builder::*;
    #[cfg(feature = "cache")]
    #[doc(no_inline)]
    pub use crate::cache::*;
    #[cfg(feature = "voice")]
    #[doc(no_inline)]
    pub use crate::client::bridge::voice::*;
    #[cfg(feature = "client")]
    #[doc(no_inline)]
    pub use crate::client::{
        bridge::gateway::{event::*, *},
        *,
    };
    #[cfg(feature = "collector")]
    #[doc(no_inline)]
    pub use crate::collector::*;
    #[doc(no_inline)]
    pub use crate::constants::{close_codes::*, *};
    #[cfg(feature = "framework")]
    #[doc(no_inline)]
    pub use crate::framework::*;
    #[cfg(feature = "http")]
    #[doc(no_inline)]
    pub use crate::http::*;
    #[cfg(feature = "interactions_endpoint")]
    #[doc(no_inline)]
    pub use crate::interactions_endpoint::*;
    #[cfg(feature = "utils")]
    #[doc(no_inline)]
    pub use crate::utils::{
        token::{validate as validate_token, InvalidToken},
        *,
    };
    // #[doc(no_inline)]
    // pub use crate::*;
    #[doc(no_inline)]
    pub use crate::{
        // Need to re-export this manually or it can't be accessed for some reason
        async_trait,
        model::prelude::*,
        *,
    };
}
