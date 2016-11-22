//! The set of extensions is functionality that is not required for a
//! [`Client`] and/or [`Connection`] to properly function.
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
//! [`Client`]: ../client/struct.Client.html
//! [`Connection`]: ../client/struct.Connection.html

#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "framework")]
pub mod framework;
#[cfg(feature = "voice")]
pub mod voice;
