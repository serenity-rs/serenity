//! A collection of bridged support between the [`client`](../index.html) module and other
//! modules.
//!
//! **Warning**: You likely _do not_ need to mess with anything in here. Beware.
//! This is lower-level functionality abstracted by the [`Client`].
//!
//! [`Client`]: ../struct.Client.html
//! [`client`]: ../

pub mod gateway;

#[cfg(feature = "voice")]
pub mod voice;
