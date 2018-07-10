//! A collection of newtypes defining type-strong IDs.

use chrono::NaiveDateTime;
use internal::prelude::*;
use serde::de::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter, Result as FmtResult};
use super::utils::U64Visitor;

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Retrieves the time that the Id was created at.
                pub fn created_at(&self) -> NaiveDateTime {
                    let offset = (self.0 >> 22) / 1000;

                    NaiveDateTime::from_timestamp(1_420_070_400 + offset as i64, 0)
                }
            }

            // This is a hack so functions can accept iterators that either:
            // 1. return the id itself (e.g: `MessageId`)
            // 2. return a reference to it (`&MessageId`).
            impl AsRef<$name> for $name {
                fn as_ref(&self) -> &Self {
                    self
                }
            }

            impl<'a> From<&'a $name> for $name {
                fn from(id: &'a $name) -> $name {
                    id.clone()
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

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter) -> FmtResult {
                    Display::fmt(&self.0, f)
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
                    deserializer.deserialize_any(U64Visitor).map($name)
                }
            }
        )*
    }
}

/// An identifier for an Application.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct ApplicationId(pub u64);

/// An identifier for a Channel
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct ChannelId(pub u64);

/// An identifier for an Emoji
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct EmojiId(pub u64);

/// An identifier for a Guild
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct GuildId(pub u64);

/// An identifier for an Integration
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct IntegrationId(pub u64);

/// An identifier for a Message
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct MessageId(pub u64);

/// An identifier for a Role
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct RoleId(pub u64);

/// An identifier for a User
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct UserId(pub u64);

/// An identifier for a [`Webhook`](struct.Webhook.html).
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct WebhookId(pub u64);

/// An identifier for an audit log entry.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialOrd, Ord, Serialize)]
#[allow(derive_hash_xor_eq)]
pub struct AuditLogEntryId(pub u64);

id_u64! {
    ApplicationId;
    ChannelId;
    EmojiId;
    GuildId;
    IntegrationId;
    MessageId;
    RoleId;
    UserId;
    WebhookId;
    AuditLogEntryId;
}
