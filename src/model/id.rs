//! A collection of newtypes defining type-strong IDs.

use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, TimeZone, Utc};
use serde::de::{Deserialize, Deserializer};

use super::utils::U64Visitor;
use crate::internal::prelude::*;

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Retrieves the time that the Id was created at.
                pub fn created_at(&self) -> DateTime<Utc> {
                    const DISCORD_EPOCH: u64 = 1_420_070_400_000;
                    Utc.timestamp_millis(((self.0 >> 22) + DISCORD_EPOCH) as i64)
                }

                /// Immutably borrow inner Id.
                #[inline]
                pub fn as_u64(&self) -> &u64 {
                    &self.0
                }

                /// Mutably borrow inner Id.
                #[inline]
                pub fn as_mut_u64(&mut self) -> &mut u64 {
                    &mut self.0
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

            impl PartialEq<u64> for $name {
                fn eq(&self, u: &u64) -> bool {
                    self.0 == *u
                }
            }

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    Display::fmt(&self.0, f)
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
                    deserializer.deserialize_any(U64Visitor).map($name)
                }
            }

            impl From<$name> for u64 {
                fn from(id: $name) -> u64 {
                    id.0 as u64
                }
            }

            impl From<$name> for i64 {
                fn from(id: $name) -> i64 {
                    id.0 as i64
                }
            }
        )*
    }
}

/// An identifier for an Application.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct ApplicationId(pub u64);

/// An identifier for a Channel
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct ChannelId(pub u64);

/// An identifier for an Emoji
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct EmojiId(pub u64);

/// An identifier for a Guild
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct GuildId(pub u64);

/// An identifier for an Integration
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct IntegrationId(pub u64);

/// An identifier for a Message
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct MessageId(pub u64);

/// An identifier for a Role
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct RoleId(pub u64);

/// An identifier for a User
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct UserId(pub u64);

/// An identifier for a [`Webhook`][super::webhook::Webhook]
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct WebhookId(pub u64);

/// An identifier for an audit log entry.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct AuditLogEntryId(pub u64);

/// An identifier for an attachment.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct AttachmentId(u64);

/// An identifier for a sticker.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct StickerId(pub u64);

/// An identifier for a sticker pack.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct StickerPackId(pub u64);

/// An identifier for an interaction.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct InteractionId(pub u64);

/// An identifier for a slash command.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct CommandId(pub u64);

/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct CommandPermissionId(pub u64);

/// An identifier for a slash command version Id.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct CommandVersionId(pub u64);

/// An identifier for a slash command target Id. Can contain
/// a [`UserId`] or [`MessageId`].
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct TargetId(pub u64);

/// An identifier for a stage channel instance.
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct StageInstanceId(pub u64);

id_u64! {
    AttachmentId;
    ApplicationId;
    ChannelId;
    EmojiId;
    GuildId;
    IntegrationId;
    MessageId;
    RoleId;
    StickerId;
    StickerPackId;
    UserId;
    WebhookId;
    AuditLogEntryId;
    InteractionId;
    CommandId;
    CommandPermissionId;
    CommandVersionId;
    TargetId;
    StageInstanceId;
}

#[cfg(test)]
mod tests {
    use super::GuildId;

    #[test]
    fn test_created_at() {
        // The id is from discord's snowflake docs
        assert_eq!(
            GuildId(175928847299117063).created_at().to_rfc3339(),
            "2016-04-30T11:18:25.796+00:00"
        );
    }
}
