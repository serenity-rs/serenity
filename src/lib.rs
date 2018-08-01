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
//! # Example Bot
//!
//! A basic ping-pong bot looks like:
//!
//! ```rust,no_run
//! #[macro_use] extern crate serenity;
//!
//! use serenity::client::Client;
//! use serenity::prelude::EventHandler;
//! use serenity::framework::standard::StandardFramework;
//! use std::env;
//!
//! struct Handler;
//!
//! impl EventHandler for Handler {}
//!
//! fn main() {
//!     // Login with a bot token from the environment
//!     let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
//!         .expect("Error creating client");
//!     client.with_framework(StandardFramework::new()
//!         .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
//!         .cmd("ping", ping));
//!
//!     // start listening for events by starting a single shard
//!     if let Err(why) = client.start() {
//!         println!("An error occurred while running the client: {:?}", why);
//!     }
//! }
//!
//! command!(ping(_context, message) {
//!     let _ = message.reply("Pong!");
//! });
//! ```
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
//! serenity = "0.5"
//! ```
//!
//! and to the top of your `main.rs`:
//!
//! ```rs
//! #[macro_use] extern crate serenity;
//! ```
//!
//! [`Cache`]: cache/struct.Cache.html
//! [`Client::new`]: client/struct.Client.html#method.new
//! [`Client::on_message`]: client/struct.Client.html#method.on_message
//! [`Context`]: client/struct.Context.html
//! [`Event`]: model/event/enum.Event.html
//! [`Event::MessageCreate`]: model/event/enum.Event.html#variant.MessageCreate
//! [`Shard`]: gateway/struct.Shard.html
//! [`examples`]: https://github.com/serenity-rs/serenity/blob/master/examples
//! [cache docs]: cache/index.html
//! [client's module-level documentation]: client/index.html
//! [docs]: https://discordapp.com/developers/docs/intro
//! [examples]: https://github.com/serenity-rs/serenity/tree/master/examples
//! [gateway docs]: gateway/index.html
#![doc(html_root_url = "https://docs.rs/serenity/*")]
#![allow(unknown_lints)]
#![allow(doc_markdown, inline_always)]
#![warn(enum_glob_use, if_not_else)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[cfg(feature = "lazy_static")]
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate parking_lot;
extern crate serde;

#[cfg(feature = "base64")]
extern crate base64;
#[cfg(feature = "byteorder")]
extern crate byteorder;
#[cfg(feature = "flate2")]
extern crate flate2;
#[cfg(feature = "hyper")]
extern crate hyper;
#[cfg(feature = "hyper-native-tls")]
extern crate hyper_native_tls;
#[cfg(feature = "multipart")]
extern crate multipart;
#[cfg(feature = "native-tls")]
extern crate native_tls;
#[cfg(feature = "opus")]
extern crate opus;
#[cfg(feature = "sodiumoxide")]
extern crate sodiumoxide;
#[cfg(feature = "threadpool")]
extern crate threadpool;
#[cfg(feature = "typemap")]
extern crate typemap;
#[cfg(feature = "evzht9h3nznqzwl")]
extern crate evzht9h3nznqzwl as websocket;

#[cfg(test)]
#[macro_use]
extern crate matches;

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

pub use error::{Error, Result};

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "cache")]
use cache::Cache;
#[cfg(feature = "cache")]
use parking_lot::RwLock;

#[cfg(feature = "cache")]
lazy_static! {
    /// A mutable and lazily-initialized static binding. It can be accessed
    /// across any function and in any context.
    ///
    /// This [`Cache`] instance is updated for every event received, so you do
    /// not need to maintain your own cache.
    ///
    /// See the [cache module documentation] for more details.
    ///
    /// The Cache itself is wrapped within an `RwLock`, which allows for
    /// multiple readers or at most one writer at a time across threads. This
    /// means that you may have multiple commands reading from the Cache
    /// concurrently.
    ///
    /// # Examples
    ///
    /// Retrieve the [current user][`CurrentUser`]'s Id, by opening a Read
    /// guard:
    ///
    /// ```rust,ignore
    /// use serenity::CACHE;
    ///
    /// println!("{}", CACHE.read().user.id);
    /// ```
    ///
    /// Update the cache's settings to enable caching of channels' messages:
    ///
    /// ```rust
    /// use serenity::CACHE;
    ///
    /// // Cache up to the 10 most recent messages per channel.
    /// CACHE.write().settings_mut().max_messages(10);
    /// ```
    ///
    /// [`CurrentUser`]: model/struct.CurrentUser.html
    /// [`Cache`]: cache/struct.Cache.html
    /// [cache module documentation]: cache/index.html
    pub static ref CACHE: RwLock<Cache> = RwLock::new(Cache::default());
}
