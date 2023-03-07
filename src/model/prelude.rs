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
#[doc(inline)]
pub use super::{
    application::interaction::MessageFlags as InteractionApplicationCommandCallbackDataFlags,
    application::interaction::*,
    application::oauth::*,
    application::*,
    channel::MessageFlags,
    channel::*,
    connection::*,
    event::*,
    gateway::*,
    guild::audit_log::*,
    guild::*,
    id::*,
    invite::*,
    mention::*,
    misc::*,
    permissions::*,
    sticker::*,
    user::*,
    voice::*,
    webhook::*,
    *,
};
