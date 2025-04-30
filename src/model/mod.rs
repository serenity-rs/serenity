//! Mappings of objects received from the API, with optional helper methods for ease of use.
//!
//! Models can optionally have additional helper methods compiled, by enabling the `model` feature.
//!
//! Normally you can import models through the sub-modules:
//!
//! ```rust,no_run
//! use serenity::model::channel::{ChannelType, GuildChannel, Message};
//! use serenity::model::id::{ChannelId, GuildId};
//! use serenity::model::user::User;
//! ```
//!
//! This can get a bit tedious - especially with a large number of imports - so this can be
//! simplified by simply glob importing everything from the prelude:
//!
//! ```rust,no_run
//! use serenity::model::prelude::*;
//! ```

#[macro_use]
mod utils;
#[cfg(test)]
pub(crate) use utils::assert_json;

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
pub mod monetization;
pub mod permissions;
pub mod sticker;
pub mod timestamp;
pub mod user;
pub mod voice;
pub mod webhook;

#[cfg(feature = "voice_model")]
pub use serenity_voice_model as voice_gateway;

pub use self::colour::{Color, Colour};
pub use self::error::Error as ModelError;
pub use self::permissions::Permissions;
pub use self::timestamp::Timestamp;

/// The model prelude re-exports all types in the model sub-modules.
///
/// This allows for quick and easy access to all of the model types.
///
/// # Examples
///
/// Import all model types into scope:
///
/// ```rust,no_run
/// use serenity::model::prelude::*;
/// ```
pub mod prelude {
    pub(crate) use serde::{Deserialize, Deserializer};

    pub use super::guild::automod::EventType as AutomodEventType;
    #[doc(hidden)]
    pub use super::guild::automod::{
        Action,
        ActionExecution,
        ActionType,
        AutoModRule,
        KeywordPresetType,
        Trigger,
        TriggerMetadata,
        TriggerType,
    };
    #[doc(hidden)]
    pub use super::{
        ModelError,
        Timestamp,
        application::*,
        channel::*,
        colour::*,
        connection::*,
        event::*,
        gateway::*,
        guild::audit_log::*,
        guild::*,
        id::*,
        invite::*,
        mention::*,
        misc::*,
        monetization::*,
        permissions::*,
        sticker::*,
        user::*,
        voice::*,
        webhook::*,
    };
    pub(crate) use crate::internal::prelude::*;
}
