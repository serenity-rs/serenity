pub mod permissions;

#[macro_use]
mod utils;

mod channel;
mod gateway;
mod guild;
mod id;
mod misc;
mod user;
mod voice;

#[cfg(feature = "methods")]
mod invite;
#[cfg(feature = "methods")]
mod webhook;

pub use self::channel::*;
pub use self::gateway::*;
pub use self::guild::*;
pub use self::id::*;
pub use self::misc::*;
pub use self::permissions::Permissions;
pub use self::user::*;
pub use self::voice::*;

#[cfg(feature = "methods")]
pub use self::invite::*;
#[cfg(feature = "methods")]
pub use self::webhook::*;

use self::utils::*;
use std::collections::HashMap;
use std::fmt;
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
            #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
            pub struct $name(pub u64);

            impl $name {
                fn decode(value: Value) -> Result<Self> {
                    decode_id(value).map($name)
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            impl From<u64> for $name {
                fn from(id_as_u64: u64) -> $name {
                    $name(id_as_u64)
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
#[derive(Debug, Clone)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Group),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    Private(PrivateChannel),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Public(PublicChannel),
}

/// A container for guilds.
///
/// This is used to differentiate whether a guild itself can be used or whether
/// a guild needs to be retrieved from the state.
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(Guild),
    /// A guild's id, which can be used to search the state for a guild.
    Id(GuildId),
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as `Context::create_permission`.
///
/// [`Context::create_permission`]: ../client/
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

/// A guild which may or may not currently be available.
#[derive(Debug, Clone)]
pub enum PossibleGuild<T> {
    /// An indicator that a guild is currently unavailable for at least one of
    /// a variety of reasons.
    Offline(GuildId),
    /// An indicator that a guild is currently available.
    Online(T),
}
