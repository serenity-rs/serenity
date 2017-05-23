//! A set of extended functionality that is not required for a `Client` and/or
//! `Shard` to properly function.
//!
//! These are flagged behind feature-gates and can be enabled and disabled.
//!
//! See each extension's module-level documentation for more information.
//!
//! Note that the framework module requires the `framework` feature to be
//! enabled (enabled by default), the cache requires the `cache` feature to be
//! enabled (enabled by default), and voice support requires the `voice` feature
//! to be enabled (disabled by default).
//!
//! **Note**: This module exists for backwards compatibility purposes. Instead,
//! prefer to use the root modules directly.

#[cfg(feature="cache")]
pub use super::cache;
#[cfg(feature="framework")]
pub use super::framework;
#[cfg(feature="voice")]
pub use super::voice;
