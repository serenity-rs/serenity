//! Mappings of objects received from the API, with optional helper methods for
//! ease of use.
//!
//! Models can optionally have additional helper methods compiled, by enabling
//! the `methods` feature.
//!
//! Methods like [`Message::delete`] or [`Webhook::execute`] are provided with
//! this feature, which can be shorthands for operations that are otherwise in
//! the [`Context`], or the much lower-level [`rest`] module.
//!
//! [`Context`]: ../client/struct.Context.html
//! [`Message::delete`]: struct.Message.html#method.delete
//! [`Webhook::execute`]: struct.Webhook.html#method.execute
//! [`rest`]: ../client/rest/index.html

#[macro_use]
mod utils;

pub mod event;
pub mod permissions;


mod channel;
mod gateway;
mod guild;
mod invite;
mod misc;
mod user;
mod webhook;

pub use self::channel::*;
pub use self::gateway::*;
pub use self::guild::*;
pub use self::invite::*;
pub use self::misc::*;
pub use self::permissions::Permissions;
pub use self::user::*;
pub use self::webhook::*;

use self::utils::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use time::Timespec;
use ::internal::prelude::*;
use ::utils::{Colour, decode_array};

// All of the enums and structs are imported here. These are built from the
// build script located at `./build.rs`.
//
// These use definitions located in `./definitions`, to map to structs and
// enums, each respectively located in their own folder.
//
// For structs, this will almost always include their decode method, although
// some require their own decoding due to many special fields.
//
// For enums, this will include the variants, and will automatically generate
// the number/string decoding methods where appropriate.
//
// As only the struct/enum itself and common mappings can be built, this leaves
// unique methods on each to be implemented here.
include!(concat!(env!("OUT_DIR"), "/models/built.rs"));

macro_rules! id {
    ($(#[$attr:meta] $name:ident;)*) => {
        $(
            #[$attr]
            #[derive(Copy, Clone, Debug, Eq, Hash, PartialOrd, Ord)]
            #[allow(derive_hash_xor_eq)]
            pub struct $name(pub u64);

            impl $name {
                fn decode(value: Value) -> Result<Self> {
                    decode_id(value).map($name)
                }

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

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Arc<RwLock<Group>>),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(Arc<RwLock<GuildChannel>>),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: struct.User.html
    Private(Arc<RwLock<PrivateChannel>>),
}

/// A container for guilds.
///
/// This is used to differentiate whether a guild itself can be used or whether
/// a guild needs to be retrieved from the cache.
#[allow(large_enum_variant)]
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(PartialGuild),
    /// A guild's id, which can be used to search the cache for a guild.
    Id(GuildId),
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as `Context::create_permission`.
///
/// [`Context::create_permission`]: ../client/
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

/// A guild which may or may not currently be available.
#[derive(Clone, Debug)]
pub enum PossibleGuild<T> {
    /// An indicator that a guild is currently unavailable for at least one of
    /// a variety of reasons.
    Offline(GuildId),
    /// An indicator that a guild is currently available.
    Online(T),
}

#[derive(Copy, Clone, Debug)]
pub enum SearchTarget {
    Channel(ChannelId),
    Guild(GuildId),
}

impl From<ChannelId> for SearchTarget {
    fn from(channel_id: ChannelId) -> SearchTarget {
        SearchTarget::Channel(channel_id)
    }
}

impl From<GuildId> for SearchTarget {
    fn from(guild_id: GuildId) -> SearchTarget {
        SearchTarget::Guild(guild_id)
    }
}
