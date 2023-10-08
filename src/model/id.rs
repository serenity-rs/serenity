//! A collection of newtypes defining type-strong IDs.
// These warns fire inside derivative
#![allow(
    clippy::non_canonical_partial_ord_impl,
    clippy::non_canonical_clone_impl,
    clippy::let_underscore_must_use
)]

use std::fmt;
use std::marker::PhantomData;
use std::num::{NonZeroI64, NonZeroU64};

use super::Timestamp;

#[derive(derivative::Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Ord(bound = ""),
    PartialEq(bound = ""),
    PartialOrd(bound = "")
)]
#[repr(packed)]
pub struct Id<Marker> {
    inner: NonZeroU64,
    #[derivative(Debug = "ignore")]
    phantom: PhantomData<Marker>,
}

impl<Marker> Id<Marker> {
    /// Creates a new Id from a u64
    ///
    /// # Panics
    /// Panics if the id is zero.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn new(id: u64) -> Self {
        match NonZeroU64::new(id) {
            Some(inner) => Self::new_nonzero(inner),
            None => panic!("Attempted to call Id::new with invalid (0) value"),
        }
    }

    /// Retrieves the inner ID as u64
    #[inline]
    #[must_use]
    pub const fn get(self) -> u64 {
        self.inner.get()
    }

    /// Retrieves the time that the Id was created at.
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        Timestamp::from_discord_id(self.get())
    }

    #[must_use]
    pub(crate) const fn new_nonzero(inner: NonZeroU64) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub const fn cast<DstMarker>(self) -> Id<DstMarker> {
        Id::new_nonzero(self.inner)
    }
}

impl<Marker> std::str::FromStr for Id<Marker> {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self::new_nonzero)
    }
}

impl<Marker> serde::ser::Serialize for Id<Marker> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inner = self.inner;
        serializer.collect_str(&inner)
    }
}

impl<'de, Marker> serde::de::Deserialize<'de> for Id<Marker> {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SnowflakeVisitor;

        impl<'de> serde::de::Visitor<'de> for SnowflakeVisitor {
            type Value = NonZeroU64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a non-zero string or integer snowflake")
            }

            // Called by formats like TOML.
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                self.visit_u64(u64::try_from(value).map_err(serde::de::Error::custom)?)
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                NonZeroU64::new(value)
                    .ok_or_else(|| serde::de::Error::custom("invalid value, expected non-zero"))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                value.parse().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_any(SnowflakeVisitor).map(Self::new_nonzero)
    }
}

impl<Marker> Default for Id<Marker> {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<Marker> From<u64> for Id<Marker> {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl<Marker> From<NonZeroU64> for Id<Marker> {
    fn from(id: NonZeroU64) -> Self {
        Self::new_nonzero(id)
    }
}

impl<Marker> PartialEq<u64> for Id<Marker> {
    fn eq(&self, u: &u64) -> bool {
        self.get() == *u
    }
}

impl<Marker> fmt::Display for Id<Marker> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner;
        fmt::Display::fmt(&inner, f)
    }
}

impl<Marker> From<Id<Marker>> for NonZeroU64 {
    fn from(id: Id<Marker>) -> Self {
        id.inner
    }
}

impl<Marker> From<Id<Marker>> for NonZeroI64 {
    fn from(id: Id<Marker>) -> Self {
        unsafe { NonZeroI64::new_unchecked(id.inner.get() as i64) }
    }
}

impl<Marker> From<Id<Marker>> for u64 {
    fn from(id: Id<Marker>) -> Self {
        id.get()
    }
}

impl<Marker> From<Id<Marker>> for i64 {
    fn from(id: Id<Marker>) -> Self {
        id.get() as i64
    }
}

#[doc(hidden)]
pub struct GenericMarker;
/// An identifier for a generic Discord object.
pub type GenericId = Id<GenericMarker>;

#[doc(hidden)]
pub struct ApplicationMarker;
/// An identifier for an Application.
pub type ApplicationId = Id<ApplicationMarker>;

#[doc(hidden)]
pub struct ChannelMarker;
/// An identifier for a Channel
pub type ChannelId = Id<ChannelMarker>;

#[doc(hidden)]
pub struct EmojiMarker;
/// An identifier for an Emoji
pub type EmojiId = Id<EmojiMarker>;

#[doc(hidden)]
pub struct GuildMarker;
/// An identifier for a Guild
pub type GuildId = Id<GuildMarker>;

#[doc(hidden)]
pub struct IntegrationMarker;
/// An identifier for an Integration
pub type IntegrationId = Id<IntegrationMarker>;

#[doc(hidden)]
pub struct MessageMarker;
/// An identifier for a Message
pub type MessageId = Id<MessageMarker>;

#[doc(hidden)]
pub struct RoleMarker;
/// An identifier for a Role
pub type RoleId = Id<RoleMarker>;

#[doc(hidden)]
pub struct RuleMarker;
/// An identifier for an Auto Moderation rule
pub type RuleId = Id<RuleMarker>;

#[doc(hidden)]
pub struct ScheduledEventMarker;
/// An identifier for a Scheduled Event
pub type ScheduledEventId = Id<ScheduledEventMarker>;

#[doc(hidden)]
pub struct UserMarker;
/// An identifier for a User
pub type UserId = Id<UserMarker>;

#[doc(hidden)]
pub struct WebhookMarker;
/// An identifier for a Webhook
pub type WebhookId = Id<WebhookMarker>;

#[doc(hidden)]
pub struct AuditLogEntryMarker;
/// An identifier for an audit log entry.
pub type AuditLogEntryId = Id<AuditLogEntryMarker>;

#[doc(hidden)]
pub struct AttachmentMarker;
/// An identifier for an attachment.
pub type AttachmentId = Id<AttachmentMarker>;

#[doc(hidden)]
pub struct StickerMarker;
/// An identifier for a sticker.
pub type StickerId = Id<StickerMarker>;

#[doc(hidden)]
pub struct StickerPackMarker;
/// An identifier for a sticker pack.
pub type StickerPackId = Id<StickerPackMarker>;

#[doc(hidden)]
pub struct StickerPackBannerMarker;
/// An identifier for a sticker pack banner.
pub type StickerPackBannerId = Id<StickerPackBannerMarker>;

#[doc(hidden)]
pub struct SkuMarker;
/// An identifier for a SKU.
pub type SkuId = Id<SkuMarker>;

#[doc(hidden)]
pub struct InteractionMarker;
/// An identifier for an interaction.
pub type InteractionId = Id<InteractionMarker>;

#[doc(hidden)]
pub struct CommandMarker;
/// An identifier for a slash command.
pub type CommandId = Id<CommandMarker>;

/// An identifier for a slash command permission Id. Can contain
#[doc(hidden)]
pub struct CommandPermissionMarker;
/// a [`RoleId`] or [`UserId`].
pub type CommandPermissionId = Id<CommandPermissionMarker>;

#[doc(hidden)]
pub struct CommandVersionMarker;
/// An identifier for a slash command version Id.
pub type CommandVersionId = Id<CommandVersionMarker>;

/// An identifier for a slash command target Id. Can contain
#[doc(hidden)]
pub struct TargetMarker;
/// a [`UserId`] or [`MessageId`].
pub type TargetId = Id<TargetMarker>;

#[doc(hidden)]
pub struct StageInstanceMarker;
/// An identifier for a stage channel instance.
pub type StageInstanceId = Id<StageInstanceMarker>;

#[doc(hidden)]
pub struct ForumTagMarker;
/// An identifier for a forum tag.
pub type ForumTagId = Id<ForumTagMarker>;

/// An identifier for a Shard.
///
/// This identifier is special, it simply models internal IDs for type safety,
/// and therefore cannot be [`Serialize`]d or [`Deserialize`]d.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(packed)]
pub struct ShardId(u32);

impl ShardId {
    /// Creates a new ShardId from a u32
    ///
    /// Due to this type not representing an actual Discord model,
    /// it is stored using a u32 to save space, and therefore cannot
    /// panic due to a 0 ID.
    #[inline]
    #[must_use]
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Retrieves the inner ID as u32
    #[inline]
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ShardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

#[cfg(test)]
mod tests {
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

        use crate::json::{assert_json, json};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Opt {
            id: Option<GuildId>,
        }

        let id = GuildId::new(17_5928_8472_9911_7063);
        assert_json(&id, json!("175928847299117063"));

        let s = Opt {
            id: Some(GuildId::new(17_5928_8472_9911_7063)),
        };
        assert_json(&s, json!({"id": "175928847299117063"}));
    }
}
