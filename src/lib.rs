//! Serenity is an ergonomic and high-level Rust library for the Discord API.
//!
//! View the [examples] on how to make and structure a bot.
//!
//! Serenity supports both bot and user login via the use of [`Client::login_bot`]
//! and [`Client::login_user`].
//!
//! You may also check your tokens prior to login via the use of
//! [`validate_token`].
//!
//! Once logged in, you may add handlers to your client to dispatch [`Event`]s,
//! such as [`Client::on_message`]. This will cause your handler to be called
//! when a [`Event::MessageCreate`] is received. Each handler is given a
//! [`Context`], giving information about the event. See the
//! [client's module-level documentation].
//!
//! The [`Connection`] is transparently handled by the library, removing
//! unnecessary complexity. Sharded connections are automatically handled for
//! you. See the [Connection's documentation][`Connection`] for more
//! information.
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
//! # Dependencies
//!
//! Serenity requires the following dependencies:
//!
//! - openssl
//!
//! # Example Bot
//!
//! A basic ping-pong bot looks like:
//!
//! ```rust,no_run
//! extern crate serenity;
//!
//! use serenity::Client;
//! use std::env;
//!
//! fn main() {
//!     // Login with a bot token from the environment
//!     let mut client = Client::login_bot(&env::var("DISCORD_TOKEN")
//!         .expect("token"));
//!     client.with_framework(|f| f
//!         .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
//!         .on("ping", |_context, message, _arguments| {
//!             let _ = message.reply("Pong!");
//!         }));
//!
//!     // start listening for events by starting a connection
//!     let _ = client.start();
//! }
//! ```
//!
//! [`Cache`]: ext/cache/struct.Cache.html
//! [`Client::login_bot`]: client/struct.Client.html#method.login_bot
//! [`Client::login_user`]: client/struct.Client.html#method.login_user
//! [`Client::on_message`]: client/struct.Client.html#method.on_message
//! [`Connection`]: client/struct.Connection.html
//! [`Context`]: client/struct.Context.html
//! [`Event`]: model/event/enum.Event.html
//! [`Event::MessageCreate`]: model/event/enum.Event.html#variant.MessageCreate
//! [cache docs]: ext/cache/index.html
//! [client's module-level documentation]: client/index.html
//! [docs]: https://discordapp.com/developers/docs/intro
//! [examples]: https://github.com/zeyla/serenity.rs/tree/master/examples
//! [`validate_token`]: client/fn.validate_token.html
#![allow(doc_markdown, inline_always, unknown_lints)]
#![doc(html_logo_url = "https://docs.austinhellyer.me/serenity.rs/docs_header.png")]
#![warn(enum_glob_use, if_not_else)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate base64;
extern crate byteorder;
extern crate flate2;
extern crate hyper;
extern crate multipart;
extern crate serde_json;
extern crate time;
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

mod constants;
mod error;
mod internal;

pub use client::Client;
pub use error::{Error, Result};
