//! Mappings of objects received from the API, with optional helper methods for
//! ease of use.
//!
//! Models can optionally have additional helper methods compiled, by enabling
//! the `model` feature.
//!
//! Normally you can import models through the sub-modules:
//!
//! ```rust,no_run
//! use serenity::model::channel::{ChannelType, GuildChannel, Message};
//! use serenity::model::id::{ChannelId, GuildId};
//! use serenity::model::user::User;
//! ```
//!
//! This can get a bit tedious - especially with a large number of imports - so
//! this can be simplified by simply glob importing everything from the prelude:
//!
//! ```rust,no_run
//! use serenity::model::prelude::*;
//! ```

#[macro_use]
mod utils;

pub mod application;
pub mod channel;
pub mod error;
pub mod event;
pub mod gateway;
pub mod guild;
pub mod id;
pub mod invite;
pub mod misc;
pub mod permissions;
pub mod prelude;
pub mod user;
pub mod voice;
pub mod webhook;

pub use self::error::Error as ModelError;
pub use self::permissions::Permissions;

use crate::internal::prelude::*;
use parking_lot::RwLock;
use self::utils::*;
use serde::de::Visitor;
use std::{
    collections::HashMap,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    },
    sync::Arc,
    result::Result as StdResult
};

#[cfg(feature = "utils")]
use crate::utils::Colour;

use serde::{Deserialize, Deserializer};
