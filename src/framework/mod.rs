//! The framework is a customizable method of separating commands.
//!
//! This is used in combination with [`Client::with_framework`].
//!
//! The framework has a number of configurations, and can have any number of
//! commands bound to it. The primary purpose of it is to offer the utility of
//! not needing to manually match message content strings to determine if a
//! message is a command.
//!
//! Additionally, "checks" can be added to commands, to ensure that a certain
//! condition is met prior to calling a command; this could be a check that the
//! user who posted a message owns the bot, for example.
//!
//! Each command has a given name, and an associated function/closure. For
//! example, you might have two commands: `"ping"` and `"weather"`. These each
//! have an associated function that are called if the framework determines
//! that a message is of that command.
//!
//! Assuming a command prefix of `"~"`, then the following would occur with the
//! two previous commands:
//!
//! ```ignore
//! ~ping // calls the ping command's function
//! ~pin // does not
//! ~ ping // _does_ call it _if_ the `allow_whitespace` option is enabled
//! ~~ping // does not
//! ```
//!
//! # Examples
//!
//! Configuring a Client with a framework, which has a prefix of `"~"` and a
//! ping and about command:
//!
//! ```rust,ignore
//! use serenity::client::{Client, Context};
//! use serenity::model::Message;
//! use std::env;
//!
//! let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap());
//!
//! client.with_framework(|f| f
//!     .configure(|c| c.prefix("~"))
//!     .on("about", |_, msg, _| {
//!         msg.channel_id.say("A simple test bot")?;
//!
//!         // The `command!` macro implicitly puts an `Ok(())` at the end of each command definiton;
//!         // signifying successful execution.
//!         //
//!         // However since we're using `on` that requires a function with the definition `Fn(Context, Message, Args) -> Result<(), _>`,
//!         // we need to manually put the `Ok` ourselves.
//!         Ok(())
//!     })
//!     .cmd("ping", ping));
//!
//!
//! command!(ping(context, message) {
//!     message.channel_id.say(&context.http, "Pong!")?;
//! });
//! ```
//!
//! [`Client::with_framework`]: ../client/struct.Client.html#method.with_framework

#[cfg(feature = "standard_framework")]
pub mod standard;

#[cfg(feature = "standard_framework")]
pub use self::standard::StandardFramework;

use crate::client::Context;
use crate::model::channel::Message;
use threadpool::ThreadPool;
use std::sync::Arc;

/// A trait for defining your own framework for serenity to use.
///
/// Should you implement this trait, or define a `message` handler, depends on you.
/// However, using this will benefit you by abstracting the `EventHandler` away,
/// and providing a reference to serenity's threadpool,
/// so that you may run your commands in separate threads.
pub trait Framework {
    fn dispatch(&mut self, _: Context, _: Message, _: &ThreadPool);
}

impl<F: Framework + ?Sized> Framework for Box<F> {
     #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool) {
        (**self).dispatch(ctx, msg, threadpool);
    }
}

impl<T: Framework + ?Sized> Framework for Arc<T> {
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &threadpool::ThreadPool) {
        Arc::get_mut(self).map(|s| (*s).dispatch(ctx, msg, threadpool));
    }
}

impl<'a, F: Framework + ?Sized> Framework for &'a mut F {
     #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool) {
        (**self).dispatch(ctx, msg, threadpool);
    }
}
