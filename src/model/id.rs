//! A collection of newtypes defining type-strong IDs.

use std::fmt;

use super::Timestamp;

macro_rules! id_u64 {
    ($($name:ident;)*) => {
        $(
            impl $name {
                /// Retrieves the time that the Id was created at.
                #[must_use]
                pub fn created_at(&self) -> Timestamp {
                    Timestamp::from_discord_id(self.0)
                }

                /// Immutably borrow inner Id.
                #[inline]
                #[must_use]
                pub fn as_u64(&self) -> &u64 {
                    &self.0
                }

                /// Mutably borrow inner Id.
                #[inline]
                #[must_use]
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

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Display::fmt(&self.0, f)
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
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct ApplicationId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a Channel
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct ChannelId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an Emoji
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct EmojiId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an unspecific entity.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
// TODO: replace occurences of `#[serde(with = "snowflake")] u64` in the codebase with GenericId
pub struct GenericId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a Guild
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct GuildId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an Integration
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct IntegrationId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a Message
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct MessageId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a Role
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct RoleId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an auto moderation rule
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct RuleId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a Scheduled Event
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct ScheduledEventId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a User
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct UserId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a [`Webhook`][super::webhook::Webhook]
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct WebhookId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an audit log entry.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct AuditLogEntryId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an attachment.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct AttachmentId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a sticker.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct StickerId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a sticker pack.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct StickerPackId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a sticker pack banner.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct StickerPackBannerId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a SKU.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct SkuId(#[serde(with = "snowflake")] pub u64);

/// An identifier for an interaction.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct InteractionId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a slash command.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct CommandId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct CommandPermissionId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a slash command version Id.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct CommandVersionId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a slash command target Id. Can contain
/// a [`UserId`] or [`MessageId`].
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct TargetId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a stage channel instance.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct StageInstanceId(#[serde(with = "snowflake")] pub u64);

/// An identifier for a forum tag.
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct ForumTagId(#[serde(with = "snowflake")] pub u64);

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

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        deserializer.deserialize_any(SnowflakeVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(id: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(id)
    }

    struct SnowflakeVisitor;

    impl<'de> Visitor<'de> for SnowflakeVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string or integer snowflake")
        }

        // Called by formats like TOML.
        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            u64::try_from(value).map_err(Error::custom)
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GuildId;

    #[test]
    fn test_created_at() {
        // The id is from discord's snowflake docs
        let id = GuildId(175928847299117063);
        assert_eq!(id.created_at().unix_timestamp(), 1462015105);
        assert_eq!(id.created_at().to_string(), "2016-04-30T11:18:25.796Z");
    }

    #[test]
    fn test_id_serde() {
        use serde::{Deserialize, Serialize};
        use serde_test::{assert_de_tokens, assert_tokens, Token};

        use super::snowflake;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct S {
            #[serde(with = "snowflake")]
            id: u64,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId(17_5928_8472_9911_7063);
        assert_tokens(&id, &[
            Token::NewtypeStruct {
                name: "GuildId",
            },
            Token::Str("175928847299117063"),
        ]);
        assert_de_tokens(&id, &[
            Token::NewtypeStruct {
                name: "GuildId",
            },
            Token::U64(17_5928_8472_9911_7063),
        ]);

        let s = S {
            id: 17_5928_8472_9911_7063,
        };
        assert_tokens(&s, &[
            Token::Struct {
                name: "S",
                len: 1,
            },
            Token::Str("id"),
            Token::Str("175928847299117063"),
            Token::StructEnd,
        ]);

        let s = Opt {
            id: Some(GuildId(17_5928_8472_9911_7063)),
        };
        assert_tokens(&s, &[
            Token::Struct {
                name: "Opt",
                len: 1,
            },
            Token::Str("id"),
            Token::Some,
            Token::NewtypeStruct {
                name: "GuildId",
            },
            Token::Str("175928847299117063"),
            Token::StructEnd,
        ]);
    }
}
