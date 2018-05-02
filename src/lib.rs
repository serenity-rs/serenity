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
//! [`examples`]: https://github.com/zeyla/serenity/blob/master/examples
//! [cache docs]: cache/index.html
//! [client's module-level documentation]: client/index.html
//! [docs]: https://discordapp.com/developers/docs/intro
//! [examples]: https://github.com/zeyla/serenity/tree/master/examples
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

extern crate chrono;
extern crate future_utils;
extern crate futures;
extern crate serde;
extern crate tokio_core;
extern crate tokio_timer;
extern crate tokio_tls;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

#[cfg(feature = "base64")]
extern crate base64;
#[cfg(feature = "byteorder")]
extern crate byteorder;
#[cfg(feature = "flate2")]
extern crate flate2;
#[cfg(feature = "hyper")]
extern crate hyper;
#[cfg(feature = "hyper-multipart-rfc7578")]
extern crate hyper_multipart_rfc7578;
#[cfg(feature = "hyper-tls")]
extern crate hyper_tls;
#[cfg(feature = "multipart")]
extern crate multipart;
#[cfg(feature = "native-tls")]
extern crate native_tls;
#[cfg(feature = "opus")]
extern crate opus;
#[cfg(feature = "rand")]
extern crate rand;
#[cfg(feature = "sodiumoxide")]
extern crate sodiumoxide;
#[cfg(feature = "threadpool")]
extern crate threadpool;
#[cfg(feature = "typemap")]
extern crate typemap;
#[cfg(feature = "evzht9h3nznqzwl")]
extern crate evzht9h3nznqzwl as websocket;

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

pub use error::{Error, FutureResult, Result};

#[cfg(feature = "client")]
pub use client::Client;
