//! The set of extensions is functionality that is not required for a
//! [`Client`] and/or [`Connection`] to properly function.
//!
//! These are flagged behind feature-gates and can be enabled and disabled.
//!
//! See each extension's module-level documentation for more information.
//!
//! [`Client`]: ../client/struct.Client.html
//! [`Connection`]: ../client/struct.Connection.html

pub mod framework;
pub mod state;
#[cfg(feature="voice")]
pub mod voice;
