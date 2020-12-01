//! The framework is a customizable method of separating commands.
//!
//! This is used in combination with [`ClientBuilder::framework`].
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
//! ```rust,no_run
//! use serenity::client::{Client, Context, EventHandler};
//! use serenity::model::channel::Message;
//! use serenity::framework::standard::macros::{command, group};
//! use serenity::framework::standard::{StandardFramework, CommandResult};
//!
//! #[command]
//! async fn about(ctx: &Context, msg: &Message) -> CommandResult {
//!     msg.channel_id.say(&ctx.http, "A simple test bot").await?;
//!
//!     Ok(())
//! }
//!
//! #[command]
//! async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
//!     msg.channel_id.say(&ctx.http, "pong!").await?;
//!
//!     Ok(())
//! }
//!
//! #[group]
//! #[commands(about, ping)]
//! struct General;
//!
//! struct Handler;
//!
//! impl EventHandler for Handler {}
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let token = std::env::var("DISCORD_TOKEN")?;
//!
//! let framework = StandardFramework::new()
//!     .configure(|c| c.prefix("~"))
//!     // The `#[group]` (and similarly, `#[command]`) macro generates static instances
//!     // containing any options you gave it. For instance, the group `name` and its `commands`.
//!     // Their identifiers, names you can use to refer to these instances in code, are an
//!     // all-uppercased version of the `name` with a `_GROUP` suffix appended at the end.
//!     .group(&GENERAL_GROUP);
//!
//! let mut client = Client::builder(&token).event_handler(Handler).framework(framework).await?;
//! #     Ok(())
//! # }
//! ```
//!
//! [`ClientBuilder::framework`]: ../client/struct.ClientBuilder.html#method.framework

#[cfg(feature = "standard_framework")]
pub mod standard;

#[cfg(feature = "standard_framework")]
pub use self::standard::StandardFramework;

use crate::client::Context;
use crate::model::channel::Message;
use async_trait::async_trait;

/// A trait for defining your own framework for serenity to use.
///
/// Should you implement this trait, or define a `message` handler, depends on you.
/// However, using this will benefit you by abstracting the `EventHandler` away,
/// and providing a reference to serenity's threadpool,
/// so that you may run your commands in separate threads.
#[async_trait]
pub trait Framework: Send + Sync {
    async fn dispatch(&self, _: Context, _: Message);
}

#[async_trait]
impl<F> Framework for Box<F>
where F: Framework + ?Sized
{
    #[inline]
    async fn dispatch(&self, ctx: Context, msg: Message) {
        (**self).dispatch(ctx, msg).await;
    }
}

#[async_trait]
impl<'a, F> Framework for &'a mut F
where F: Framework + ?Sized
{
    #[inline]
    async fn dispatch(&self, ctx: Context, msg: Message) {
        (**self).dispatch(ctx, msg).await;
    }
}
