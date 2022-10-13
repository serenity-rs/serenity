//! A collection of newtypes defining type-strong IDs.

use std::fmt;
use std::num::{NonZeroI64, NonZeroU64};

use super::Timestamp;

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Creates a new Id from a u64
                ///
                /// # Panics
                /// Panics if the id is zero.
                #[inline]
                #[must_use]
                #[track_caller]
                pub const fn new(id: u64) -> Self {
                    match NonZeroU64::new(id) {
                        Some(inner) => Self(inner),
                        None => panic!("Attempted to call Id::new with invalid (0) value")
                    }
                }

                /// Retrieves the inner ID as u64
                #[inline]
                #[must_use]
                pub const fn get(self) -> u64 {
                    self.0.get()
                }

                /// Retrieves the time that the Id was created at.
                #[must_use]
                pub fn created_at(&self) -> Timestamp {
                    Timestamp::from_discord_id(self.get())
                }

                /// Immutably borrow inner Id.
                #[inline]
                #[must_use]
                pub const fn as_inner(&self) -> &NonZeroU64 {
                    &self.0
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    // Have the possible panic at compile time. `unwrap()` is not const-stable
                    const ONE: NonZeroU64 = match NonZeroU64::new(1) {
                        Some(x) => x,
                        None => unreachable!(),
                    };
                    Self(ONE)
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

            impl From<NonZeroU64> for $name {
                fn from(id: NonZeroU64) -> $name {
                    $name(id)
                }
            }

            impl PartialEq<u64> for $name {
                fn eq(&self, u: &u64) -> bool {
                    self.get() == *u
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Display::fmt(&self.0, f)
                }
            }

            impl From<$name> for NonZeroU64 {
                fn from(id: $name) -> NonZeroU64 {
                    id.0
                }
            }

            impl From<$name> for NonZeroI64 {
                fn from(id: $name) -> NonZeroI64 {
                    unsafe {NonZeroI64::new_unchecked(id.get() as i64)}
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
        )*
    }
}

/// An identifier for an Application.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ApplicationId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a Channel
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ChannelId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an Emoji
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct EmojiId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an unspecific entity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
// TODO: replace occurences of `#[serde(with = "snowflake")] u64` in the codebase with GenericId
pub struct GenericId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a Guild
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct GuildId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an Integration
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct IntegrationId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a Message
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct MessageId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a Role
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct RoleId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an auto moderation rule
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct RuleId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a Scheduled Event
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ScheduledEventId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a User
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UserId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a [`Webhook`][super::webhook::Webhook]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct WebhookId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an audit log entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AuditLogEntryId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AttachmentId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a sticker.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StickerId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a sticker pack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StickerPackId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a sticker pack banner.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StickerPackBannerId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a SKU.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SkuId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for an interaction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct InteractionId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a slash command.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CommandId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CommandPermissionId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a slash command version Id.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CommandVersionId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a slash command target Id. Can contain
/// a [`UserId`] or [`MessageId`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TargetId(#[serde(with = "snowflake")] pub NonZeroU64);

/// An identifier for a stage channel instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StageInstanceId(#[serde(with = "snowflake")] pub NonZeroU64);

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
}

/// Used with `#[serde(with|deserialize_with|serialize_with)]`
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Deserialize, Serialize)]
/// struct A {
///     #[serde(with = "snowflake")]
///     id: u64,
/// }
///
/// #[derive(Deserialize)]
/// struct B {
///     #[serde(deserialize_with = "snowflake::deserialize")]
///     id: u64,
/// }
///
/// #[derive(Serialize)]
/// struct C {
///     #[serde(serialize_with = "snowflake::serialize")]
///     id: u64,
/// }
/// ```
pub(crate) mod snowflake {
    use std::convert::TryFrom;
    use std::fmt;
    use std::num::NonZeroU64;

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NonZeroU64, D::Error> {
        deserializer.deserialize_any(SnowflakeVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(id: &NonZeroU64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&id.get())
    }

    struct SnowflakeVisitor;

    impl<'de> Visitor<'de> for SnowflakeVisitor {
        type Value = NonZeroU64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a non-zero string or integer snowflake")
        }

        // Called by formats like TOML.
        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            self.visit_u64(u64::try_from(value).map_err(Error::custom)?)
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            NonZeroU64::new(value).ok_or_else(|| Error::custom("invalid value, expected non-zero"))
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

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

        use super::snowflake;
        use crate::json::{assert_json, json};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct S {
            #[serde(with = "snowflake")]
            id: NonZeroU64,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId::new(17_5928_8472_9911_7063);
        assert_json(&id, json!("175928847299117063"));

        let s = S {
            id: NonZeroU64::new(17_5928_8472_9911_7063).unwrap(),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));

        let s = Opt {
            id: Some(GuildId::new(17_5928_8472_9911_7063)),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));
    }
}
