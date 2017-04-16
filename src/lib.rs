//! Serenity is a Rust library for the Discord API.
//!
//! View the [examples] on how to make and structure a bot.
//!
//! Serenity supports bot user authentication via the use of [`Client::login`].
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
//! ```rust,ignore
//! #[macro_use] extern crate serenity;
//!
//! use serenity::client::Client;
//! use std::env;
//!
//! fn main() {
//!     // Login with a bot token from the environment
//!     let mut client = Client::login(&env::var("DISCORD_TOKEN").expect("token"));
//!     client.with_framework(|f| f
//!         .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
//!         .on("ping", ping));
//!
//!     // start listening for events by starting a single shard
//!     let _ = client.start();
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
//! serenity = "0.1"
//! ```
//!
//! and to the top of your `main.rs`:
//!
//! ```rs
//! #[macro_use] extern crate serenity;
//! ```
//!
//! [`Cache`]: ext/cache/struct.Cache.html
//! [`Client::login`]: client/struct.Client.html#method.login
//! [`Client::on_message`]: client/struct.Client.html#method.on_message
//! [`Context`]: client/struct.Context.html
//! [`Event`]: model/event/enum.Event.html
//! [`Event::MessageCreate`]: model/event/enum.Event.html#variant.MessageCreate
//! [`Shard`]: client/gateway/struct.Shard.html
//! [`examples`]: https://github.com/zeyla/serenity/blob/master/examples
//! [cache docs]: ext/cache/index.html
//! [client's module-level documentation]: client/index.html
//! [docs]: https://discordapp.com/developers/docs/intro
//! [examples]: https://github.com/zeyla/serenity/tree/master/examples
//! [gateway docs]: client/gateway/index.html
#![allow(doc_markdown, inline_always, unknown_lints)]
#![doc(html_logo_url="https://zey.moe/u/serenity$header.png")]
#![warn(enum_glob_use, if_not_else)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate base64;
extern crate byteorder;
extern crate flate2;
extern crate hyper;
extern crate multipart;
extern crate serde;
extern crate time;
extern crate typemap;
extern crate websocket;

#[cfg(feature="voice")]
extern crate opus;
#[cfg(feature="voice")]
extern crate sodiumoxide;

#[macro_use]
pub mod utils;

pub mod client;
pub mod ext;
pub mod model;
pub mod prelude;

#[macro_use]
mod internal;

mod constants;
mod error;

pub use client::Client;
pub use error::{Error, Result};
