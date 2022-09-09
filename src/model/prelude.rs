//! The model prelude re-exports all types in the model sub-modules.
//!
//! This allows for quick and easy access to all of the model types.
//!
//! # Examples
//!
//! Import all model types into scope:
//!
//! ```rust,no_run
//! use serenity::model::prelude::*;
//! ```
#[allow(deprecated)]
#[doc(inline)]
pub use super::{
    application::*,
    channel::*,
    connection::*,
    event::*,
    gateway::*,
    guild::audit_log::*,
    guild::*,
    id::*,
    interactions::*,
    invite::*,
    mention::*,
    misc::*,
    oauth2::*,
    permissions::*,
    sticker::*,
    user::*,
    voice::*,
    webhook::*,
    *,
};
