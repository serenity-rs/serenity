//! A collection of newtypes defining type-strong IDs.

use std::fmt;

use nonmax::NonMaxU64;

use super::prelude::*;

macro_rules! newtype_display_impl {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // See comment in Self::get for block.
                fmt::Display::fmt(&{ self.0 }, f)
            }
        }
    };
}

macro_rules! forward_fromstr_impl {
    ($name:ident) => {
        impl std::str::FromStr for $name {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map(Self).map_err(ParseIdError)
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseIdError(nonmax::ParseIntError);

impl std::error::Error for ParseIdError {}
impl std::fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                #[doc = concat!("Creates a new ", stringify!($name), " from a u64.")]
                /// # Panics
                /// Panics if `id` is u64::MAX.
                #[must_use]
                #[track_caller]
                pub const fn new(id: u64) -> Self {
                    match NonMaxU64::new(id) {
                        Some(inner) => Self(inner),
                        None => panic!(concat!("Attempted to call ", stringify!($name), "::new with invalid (u64::MAX) value"))
                    }
                }

                /// Retrieves the inner `id` as a [`u64`].
                #[must_use]
                pub const fn get(self) -> u64 {
                    // By wrapping `self.0` in a block, it forces a Copy, as NonMax::get takes &self.
                    // If removed, the compiler will auto-ref to `&self.0`, which is a
                    // reference to a packed field and therefore errors.
                    {self.0}.get()
                }

                #[doc = concat!("Retrieves the time that the ", stringify!($name), " was created.")]
                #[must_use]
                pub fn created_at(&self) -> Timestamp {
                    Timestamp::from_discord_id(self.get())
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
                fn from(id: u64) -> $name {
                    $name::new(id)
                }
            }

            impl PartialEq<u64> for $name {
                fn eq(&self, u: &u64) -> bool {
                    self.get() == *u
                }
            }

            impl From<$name> for u64 {
                fn from(id: $name) -> u64 {
                    id.get()
                }
            }

            impl From<$name> for i64 {
                fn from(id: $name) -> i64 {
                    id.get() as i64
                }
            }

            newtype_display_impl!($name);
            forward_fromstr_impl!($name);

            impl ToArrayString for $name {
                type ArrayString = <u64 as ToArrayString>::ArrayString;
                const MAX_LENGTH: usize = <u64 as ToArrayString>::MAX_LENGTH;

                fn to_arraystring(self) -> Self::ArrayString {
                    self.get().to_arraystring()
                }
            }

            #[cfg(feature = "typesize")]
            impl typesize::TypeSize for $name {}
        )*
    }
}

/// An identifier for an Application.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct ApplicationId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a Channel
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct ChannelId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an Emoji
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct EmojiId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an unspecific entity.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct GenericId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a Guild
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct GuildId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an Integration
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct IntegrationId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a Message
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct MessageId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a Role
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct RoleId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an auto moderation rule
#[repr(packed)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct RuleId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a Scheduled Event
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct ScheduledEventId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a User
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct UserId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a [`Webhook`]
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct WebhookId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an audit log entry.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct AuditLogEntryId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an attachment.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct AttachmentId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a sticker.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct StickerId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a sticker pack.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct StickerPackId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a sticker pack banner.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct StickerPackBannerId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a SKU.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct SkuId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an interaction.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct InteractionId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a slash command.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct CommandId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct CommandPermissionId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a slash command version Id.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct CommandVersionId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a slash command target Id. Can contain
/// a [`UserId`] or [`MessageId`].
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct TargetId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a stage channel instance.
#[repr(packed)]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
pub struct StageInstanceId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for a forum tag.
#[repr(packed)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ForumTagId(#[serde(with = "snowflake")] NonMaxU64);

/// An identifier for an entitlement.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct EntitlementId(#[serde(with = "snowflake")] pub NonMaxU64);

id_u64! {
    AttachmentId;
    ApplicationId;
    ChannelId;
    EmojiId;
    GenericId;
    GuildId;
    IntegrationId;
    MessageId;
    RoleId;
    ScheduledEventId;
    StickerId;
    StickerPackId;
    StickerPackBannerId;
    SkuId;
    UserId;
    WebhookId;
    AuditLogEntryId;
    InteractionId;
    CommandId;
    CommandPermissionId;
    CommandVersionId;
    TargetId;
    StageInstanceId;
    RuleId;
    ForumTagId;
    EntitlementId;
}

/// An identifier for a Shard.
///
/// This identifier is special, it simply models internal IDs for type safety and therefore cannot
/// be deserialized
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct ShardId(pub u16);

impl ShardId {
    /// Retrieves the value as a [`u16`].
    ///
    /// This is not a [`u64`] as [`ShardId`]s are not a discord concept and are simply used for
    /// internal type safety.
    #[must_use]
    pub fn get(self) -> u16 {
        self.0
    }
}

newtype_display_impl!(ShardId);

/// An identifier for a [`Poll Answer`](super::channel::PollAnswer).
///
/// This is identifier is special as it is not a snowflake.
///
/// The specific algorithm used is currently just a sequential index but this is subject to change.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
#[repr(packed)]
pub struct AnswerId(nonmax::NonMaxU8);

impl AnswerId {
    /// Retrieves the value as a [`u64`].
    ///
    /// Keep in mind that this is **not a snowflake** and the values are subject to change.
    #[must_use]
    pub fn get(self) -> u64 {
        { self.0 }.get().into()
    }
}

newtype_display_impl!(AnswerId);
forward_fromstr_impl!(AnswerId);

mod snowflake {
    use std::fmt;

    use nonmax::NonMaxU64;
    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NonMaxU64, D::Error> {
        deserializer.deserialize_any(SnowflakeVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(id: &NonMaxU64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&id.get())
    }

    struct SnowflakeVisitor;

    impl Visitor<'_> for SnowflakeVisitor {
        type Value = NonMaxU64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or integer snowflake that is not u64::MAX")
        }

        // Called by formats like TOML.
        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            self.visit_u64(u64::try_from(value).map_err(Error::custom)?)
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            NonMaxU64::new(value).ok_or_else(|| Error::custom("invalid value, expected non-max"))
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use nonmax::NonMaxU64;

    use super::GuildId;

    #[test]
    fn test_created_at() {
        // The id is from discord's snowflake docs
        let id = GuildId::new(175928847299117063);
        assert_eq!(id.created_at().unix_timestamp(), 1462015105);
        assert_eq!(id.created_at().to_string(), "2016-04-30T11:18:25.796Z");
    }

    #[test]
    fn test_id_serde() {
        use serde::{Deserialize, Serialize};
        use serde_json::json;

        use super::snowflake;
        use crate::model::utils::assert_json;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct S {
            #[serde(with = "snowflake")]
            id: NonMaxU64,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId::new(17_5928_8472_9911_7063);
        assert_json(&id, json!("175928847299117063"));

        let s = S {
            id: NonMaxU64::new(17_5928_8472_9911_7063).unwrap(),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));

        let s = Opt {
            id: Some(GuildId::new(17_5928_8472_9911_7063)),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));
    }
}
