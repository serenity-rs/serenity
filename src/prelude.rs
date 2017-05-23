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
//! [`serenity::Error`]: ../enum.Error.html

pub use ::error::{Error as SerenityError};
pub use ::model::Mentionable;

#[cfg(feature="client")]
pub use ::client::{Client, ClientError as ClientError};
