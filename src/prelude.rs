//! A set of exports which can be helpful to use.
//!
//! Note that the `SerenityError` re-export is equivalent to
//! [`serenity::Error`], although is re-exported as a separate name to remove
//! likely ambiguity with other crate error enums.
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

pub use crate::error::Error as SerenityError;
pub use crate::model::misc::Mentionable;
pub use tokio::sync::{Mutex, RwLock};

#[cfg(all(feature = "client", feature = "gateway"))]
pub use crate::client::{Client, ClientError, EventHandler, RawEventHandler};
#[cfg(feature = "client")]
pub use crate::client::Context;
#[cfg(feature = "client")]
pub use typemap_rev::{TypeMap, TypeMapKey};
#[cfg(feature = "gateway")]
pub use crate::gateway::GatewayError;
#[cfg(feature = "http")]
pub use crate::http::HttpError;
#[cfg(feature = "model")]
pub use crate::model::ModelError;
