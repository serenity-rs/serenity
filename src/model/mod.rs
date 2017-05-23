//! Mappings of objects received from the API, with optional helper methods for
//! ease of use.
//!
//! Models can optionally have additional helper methods compiled, by enabling
//! the `model` feature.
//!
//! Methods like [`Message::delete`] or [`Webhook::execute`] are provided with
//! this feature, which can be shorthands for operations that are otherwise in
//! the [`Context`], or the much lower-level [`http`] module.
//!
//! [`Context`]: ../client/struct.Context.html
//! [`Message::delete`]: struct.Message.html#method.delete
//! [`Webhook::execute`]: struct.Webhook.html#method.execute
//! [`http`]: ../http/index.html

#[macro_use]
mod utils;

pub mod event;
pub mod permissions;

mod channel;
mod error;
mod gateway;
mod guild;
mod invite;
mod misc;
mod user;
mod voice;
mod webhook;

pub use self::channel::*;
pub use self::error::Error as ModelError;
pub use self::gateway::*;
pub use self::guild::*;
pub use self::invite::*;
pub use self::misc::*;
pub use self::permissions::Permissions;
pub use self::user::*;
pub use self::voice::*;
pub use self::webhook::*;

use self::utils::*;
use serde::de::Visitor;
use std::collections::HashMap;
use std::fmt::{Formatter, Result as FmtResult};
use std::sync::{Arc, RwLock};
use time::Timespec;
use ::internal::prelude::*;

#[cfg(feature="utils")]
use ::utils::Colour;

fn default_true() -> bool { true }

macro_rules! id {
    ($(#[$attr:meta] $name:ident;)*) => {
        $(
            #[$attr]
            #[derive(Copy, Clone, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
            #[allow(derive_hash_xor_eq)]
            pub struct $name(pub u64);

            impl $name {
                /// Retrieves the time that the Id was created at.
                pub fn created_at(&self) -> Timespec {
                    let offset = (self.0 >> 22) / 1000;

                    Timespec::new(1420070400 + offset as i64, 0)
                }
            }

            impl From<u64> for $name {
                fn from(id_as_u64: u64) -> $name {
                    $name(id_as_u64)
                }
            }

            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }

            impl PartialEq<u64> for $name {
                fn eq(&self, u: &u64) -> bool {
                    self.0 == *u
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
                    deserializer.deserialize_u64(U64Visitor).map($name)
                }
            }
        )*
    }
}

id! {
    /// An identifier for a Channel
    ChannelId;
    /// An identifier for an Emoji
    EmojiId;
    /// An identifier for a Guild
    GuildId;
    /// An identifier for an Integration
    IntegrationId;
    /// An identifier for a Message
    MessageId;
    /// An identifier for a Role
    RoleId;
    /// An identifier for a User
    UserId;
    /// An identifier for a [`Webhook`](struct.Webhook.html).
    WebhookId;
}

/// A container for guilds.
///
/// This is used to differentiate whether a guild itself can be used or whether
/// a guild needs to be retrieved from the cache.
#[allow(large_enum_variant)]
#[derive(Clone, Debug)]
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(PartialGuild),
    /// A guild's id, which can be used to search the cache for a guild.
    Id(GuildId),
}

/// Information about a user's application. An application does not necessarily
/// have an associated bot user.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationInfo {
    /// The bot user associated with the application. See [`BotApplication`] for
    /// more information.
    ///
    /// [`BotApplication`]: struct.BotApplication.html
    pub bot: Option<BotApplication>,
    /// Indicator of whether the bot is public.
    ///
    /// If a bot is public, anyone may invite it to their [`Guild`]. While a bot
    /// is private, only the owner may add it to a guild.
    ///
    /// [`Guild`]: struct.Guild.html
    #[serde(default="default_true")]
    pub bot_public: bool,
    /// Indicator of whether the bot requires an OAuth2 code grant.
    pub bot_require_code_grant: bool,
    /// A description of the application, assigned by the application owner.
    pub description: String,
    /// A set of bitflags assigned to the application, which represent gated
    /// feature flags that have been enabled for the application.
    pub flags: Option<u64>,
    /// A hash pointing to the application's icon.
    ///
    /// This is not necessarily equivalent to the bot user's avatar.
    pub icon: Option<String>,
    /// The unique numeric Id of the application.
    pub id: UserId,
    /// The name assigned to the application by the application owner.
    pub name: String,
    /// A list of redirect URIs assigned to the application.
    pub redirect_uris: Vec<String>,
    /// A list of RPC Origins assigned to the application.
    pub rpc_origins: Vec<String>,
    /// The given secret to the application.
    ///
    /// This is not equivalent to the application's bot user's token.
    pub secret: String,
}

/// Information about an application with an application's bot user.
#[derive(Clone, Debug, Deserialize)]
pub struct BotApplication {
    /// The unique Id of the bot user.
    pub id: UserId,
    /// A hash of the avatar, if one is assigned.
    ///
    /// Can be used to generate a full URL to the avatar.
    pub avatar: Option<String>,
    /// Indicator of whether it is a bot.
    #[serde(default)]
    pub bot: bool,
    /// The discriminator assigned to the bot user.
    ///
    /// While discriminators are not unique, the `username#discriminator` pair
    /// is.
    pub discriminator: u16,
    /// The bot user's username.
    pub name: String,
    /// The token used to authenticate as the bot user.
    ///
    /// **Note**: Keep this information private, as untrusted sources can use it
    /// to perform any action with a bot user.
    pub token: String,
}

/// Information about the current application and its owner.
#[derive(Clone, Debug, Deserialize)]
pub struct CurrentApplicationInfo {
    pub description: String,
    pub icon: Option<String>,
    pub id: UserId,
    pub name: String,
    pub owner: User,
    #[serde(default)]
    pub rpc_origins: Vec<String>,
}

/// The name of a region that a voice server can be located in.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum Region {
    #[serde(rename="amsterdam")]
    Amsterdam,
    #[serde(rename="brazil")]
    Brazil,
    #[serde(rename="eu-central")]
    EuCentral,
    #[serde(rename="eu-west")]
    EuWest,
    #[serde(rename="frankfurt")]
    Frankfurt,
    #[serde(rename="london")]
    London,
    #[serde(rename="sydney")]
    Sydney,
    #[serde(rename="us-central")]
    UsCentral,
    #[serde(rename="us-east")]
    UsEast,
    #[serde(rename="us-south")]
    UsSouth,
    #[serde(rename="us-west")]
    UsWest,
    #[serde(rename="vip-amsterdam")]
    VipAmsterdam,
    #[serde(rename="vip-us-east")]
    VipUsEast,
    #[serde(rename="vip-us-west")]
    VipUsWest,
}

impl Region {
    pub fn name(&self) -> &str {
        match *self {
            Region::Amsterdam => "amsterdam",
            Region::Brazil => "brazil",
            Region::EuCentral => "eu-central",
            Region::EuWest => "eu-west",
            Region::Frankfurt => "frankfurt",
            Region::London => "london",
            Region::Sydney => "sydney",
            Region::UsCentral => "us-central",
            Region::UsEast => "us-east",
            Region::UsSouth => "us-south",
            Region::UsWest => "us-west",
            Region::VipAmsterdam => "vip-amsterdam",
            Region::VipUsEast => "vip-us-east",
            Region::VipUsWest => "vip-us-west",
        }
    }
}

use serde::{Deserialize, Deserializer};
use std::result::Result as StdResult;

fn deserialize_sync_user<'de, D: Deserializer<'de>>(deserializer: D)
    -> StdResult<Arc<RwLock<User>>, D::Error> {
    Ok(Arc::new(RwLock::new(User::deserialize(deserializer)?)))
}
