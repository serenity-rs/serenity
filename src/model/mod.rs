//! Mappings of objects received from the API, with optional helper methods for ease of use.
//!
//! Models can optionally have additional helper methods compiled, by enabling the `model` feature.
//!
//! Normally you can import models through the sub-modules:
//!
//! ```rust,no_run
//! use serenity::model::{ChannelId, ChannelType, GuildChannel, GuildId, Message, User};
//! ```
//!
//! This can get a bit tedious - especially with a large number of imports - so this can be
//! simplified by simply glob importing everything from the prelude:
//!
//! ```rust,no_run
//! use serenity::model::*;
//! ```

#[macro_use]
mod utils;

pub mod application;
pub mod channel;
pub mod colour;
pub mod connection;
pub mod error;
pub mod event;
pub mod gateway;
pub mod guild;
pub mod id;
pub mod invite;
pub mod mention;
pub mod misc;
pub mod permissions;
// Soft-deprecated - you can import from serenity::model directly now
pub mod prelude {
    #[doc(no_inline)]
    pub use super::*;
}
pub mod sticker;
pub mod timestamp;
pub mod user;
pub mod voice;
pub mod webhook;

use std::collections::HashMap;
use std::result::Result as StdResult;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
#[cfg(feature = "voice-model")]
pub use serenity_voice_model as voice_gateway;

#[doc(inline)]
pub use self::{
    application::*,
    channel::*,
    colour::*,
    connection::*,
    error::{ModelError, *},
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
    timestamp::*,
    user::*,
    voice::*,
    webhook::*,
};
use crate::internal::prelude::*;
pub type Color = Colour;
