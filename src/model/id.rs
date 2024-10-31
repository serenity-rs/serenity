//! A collection of newtypes defining type-strong IDs.

use std::fmt;

use nonmax::NonMaxU64;
use serde::de::Error;
use to_arraystring::ToArrayString;

use super::prelude::*;

macro_rules! newtype_display_impl {
    ($name:ident, |$this:ident| $inner:expr) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&(|$this: $name| $inner)(*self), f)
            }
        }
    };
}

macro_rules! forward_fromstr_impl {
    ($name:ident, $wrapper:path) => {
        impl std::str::FromStr for $name {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map($wrapper).map(Self).map_err(ParseIdError)
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
    ($($name:ident: $doc:literal;)*) => {
        $(
            #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
            #[doc = $doc]
            pub struct $name(InnerId);

            impl $name {
                #[doc = concat!("Creates a new ", stringify!($name), " from a u64.")]
                /// # Panics
                /// Panics if `id` is u64::MAX.
                #[must_use]
                #[track_caller]
                pub const fn new(id: u64) -> Self {
                    match NonMaxU64::new(id) {
                        Some(inner) => Self(InnerId(inner)),
                        None => panic!(concat!("Attempted to call ", stringify!($name), "::new with invalid (u64::MAX) value"))
                    }
                }

                /// Retrieves the inner `id` as a [`u64`].
                #[must_use]
                pub const fn get(self) -> u64 {
                    // By wrapping `self.0.0` in a block, it forces a Copy, as NonMax::get takes &self.
                    // If removed, the compiler will auto-ref to `&self.0`, which is a
                    // reference to a packed field and therefore errors.
                    {self.0.0}.get()
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

            newtype_display_impl!($name, |this| this.0.0);
            forward_fromstr_impl!($name, InnerId);

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

/// The inner storage of an ID.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(packed)]
pub(crate) struct InnerId(NonMaxU64);

struct SnowflakeVisitor;

impl serde::de::Visitor<'_> for SnowflakeVisitor {
    type Value = InnerId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string or integer snowflake that is not u64::MAX")
    }

    // Called by formats like TOML.
    fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
        self.visit_u64(u64::try_from(value).map_err(Error::custom)?)
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
        NonMaxU64::new(value)
            .map(InnerId)
            .ok_or_else(|| Error::custom("invalid value, expected non-max"))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map(InnerId).map_err(Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for InnerId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<InnerId, D::Error> {
        deserializer.deserialize_any(SnowflakeVisitor)
    }
}

impl serde::Serialize for InnerId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&{ self.0 })
    }
}

id_u64! {
    AttachmentId: "An identifier for an attachment.";
    ApplicationId: "An identifier for an Application.";
    ChannelId: "An identifier for a Channel";
    EmojiId: "An identifier for an Emoji";
    GenericId: "An identifier for an unspecific entity.";
    GuildId: "An identifier for a Guild";
    IntegrationId: "An identifier for an Integration";
    MessageId: "An identifier for a Message";
    RoleId: "An identifier for a Role";
    ScheduledEventId: "An identifier for a Scheduled Event";
    StickerId: "An identifier for a sticker.";
    StickerPackId: "An identifier for a sticker pack.";
    StickerPackBannerId: "An identifier for a sticker pack banner.";
    SkuId: "An identifier for a SKU.";
    UserId: "An identifier for a User";
    WebhookId: "An identifier for a [`Webhook`]";
    AuditLogEntryId: "An identifier for an audit log entry.";
    InteractionId: "An identifier for an interaction.";
    CommandId: "An identifier for a slash command.";
    CommandPermissionId: "An identifier for a slash command permission Id.";
    CommandVersionId: "An identifier for a slash command version Id.";
    TargetId: "An identifier for a slash command target Id.";
    StageInstanceId: "An identifier for a stage channel instance.";
    RuleId: "An identifier for an auto moderation rule";
    ForumTagId: "An identifier for a forum tag.";
    EntitlementId: "An identifier for an entitlement.";
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

newtype_display_impl!(ShardId, |this| this.0);

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

newtype_display_impl!(AnswerId, |this| this.0);
forward_fromstr_impl!(AnswerId, std::convert::identity);

#[cfg(test)]
mod tests {
    use nonmax::NonMaxU64;

    use super::{GuildId, InnerId};

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

        use crate::model::utils::assert_json;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct S {
            id: InnerId,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId::new(17_5928_8472_9911_7063);
        assert_json(&id, json!("175928847299117063"));

        let s = S {
            id: InnerId(NonMaxU64::new(17_5928_8472_9911_7063).unwrap()),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));

        let s = Opt {
            id: Some(GuildId::new(17_5928_8472_9911_7063)),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));
    }
}
