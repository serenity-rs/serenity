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
    application::interaction::application_command::*,
    application::interaction::message_component::*,
    application::interaction::modal::*,
    application::interaction::ping::*,
    application::interaction::{MessageFlags as InteractionMessageFlags, *},
    application::oauth::*,
    application::*,
    channel::{MessageFlags, *},
    colour::*,
    connection::*,
    // We have to explicitly mention EventType here for some reason or importing it won't work
    event::{EventType, *},
    gateway::*,
    guild::audit_log::*,
    guild::automod::{EventType as AutomodEventType, *},
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
