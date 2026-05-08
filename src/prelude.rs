//! A set of exports which can be helpful to use.
//!
//! Note that the `SerenityError` re-export is equivalent to [`serenity::Error`], although is
//! re-exported as a separate name to remove likely ambiguity with other crate error enums.
//!
//! # Examples
//!
//! Import all of the exports:
//!
//! ```rust
//! use serenity::prelude::*;
//! ```
//!
//! [`serenity::Error`]: crate::Error

pub use tokio::sync::{Mutex, RwLock};

pub use crate::error::Error as SerenityError;
#[cfg(feature = "gateway")]
pub use crate::gateway::GatewayError;
#[cfg(feature = "gateway")]
pub use crate::gateway::client::{Client, Context, EventHandler, RawEventHandler};
#[cfg(feature = "http")]
pub use crate::http::HttpError;
#[cfg(feature = "http")]
pub use crate::model::CacheHttp;
pub use crate::model::mention::Mentionable;
#[cfg(feature = "model")]
pub use crate::model::{ModelError, gateway::GatewayIntents};
pub use crate::secrets::Token;
