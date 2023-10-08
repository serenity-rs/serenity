//! A collection of newtypes defining type-strong IDs.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::{NonZeroI64, NonZeroU64};

use serde::Deserializer;

use super::Timestamp;

mod sealed {
    use super::*;

    pub trait IdMarker {}
    macro_rules! implement_marker {
        ($($marker:ident,)*) => {
            $(
                impl IdMarker for $marker {}
            )*
        }
    }

    implement_marker! {
        ApplicationMarker,
        ChannelMarker,
        EmojiMarker,
        GenericMarker,
        GuildMarker,
        IntegrationMarker,
        MessageMarker,
        RoleMarker,
        RuleMarker,
        ScheduledEventMarker,
        UserMarker,
        WebhookMarker,
        AuditLogEntryMarker,
        AttachmentMarker,
        StickerMarker,
        StickerPackMarker,
        StickerPackBannerMarker,
        SkuMarker,
        InteractionMarker,
        CommandMarker,
        CommandPermissionMarker,
        CommandVersionMarker,
        TargetMarker,
        StageInstanceMarker,
        ForumTagMarker,
    }
}

#[repr(packed)]
pub struct Id<Marker: sealed::IdMarker> {
    inner: NonZeroU64,
    _phantom: PhantomData<Marker>,
}

impl<Marker: sealed::IdMarker> Id<Marker> {
    /// Creates a new Id from a u64
    ///
    /// # Panics
    /// Panics if the id is zero.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn new(id: u64) -> Self {
        match NonZeroU64::new(id) {
            Some(inner) => Self::new_inner(inner),
            None => panic!("Attempted to call Id::new with invalid (0) value"),
        }
    }

    const fn new_inner(inner: NonZeroU64) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
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

    /// Casts the Id to another Id type, without
    /// changing the underlying integer value.
    #[must_use]
    pub fn cast<NewMarker: sealed::IdMarker>(self) -> Id<NewMarker> {
        Id::new_inner(self.inner)
    }
}

impl<Marker: sealed::IdMarker> Copy for Id<Marker> {}
impl<Marker: sealed::IdMarker> Clone for Id<Marker> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Marker: sealed::IdMarker> fmt::Debug for Id<Marker> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner;
        write!(f, "{inner:?}")
    }
}

impl<Marker: sealed::IdMarker> PartialEq for Id<Marker> {
    fn eq(&self, other: &Self) -> bool {
        let inner = self.inner;
        let other_inner = other.inner;
        inner == other_inner
    }
}

impl<Marker: sealed::IdMarker> Eq for Id<Marker> {}

impl<Marker: sealed::IdMarker> Hash for Id<Marker> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let inner = self.inner;
        inner.hash(state);
    }
}

impl<Marker: sealed::IdMarker> PartialOrd for Id<Marker> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Marker: sealed::IdMarker> Ord for Id<Marker> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let inner = self.inner;
        let other_inner = other.inner;
        inner.cmp(&other_inner)
    }
}

impl<'de, Marker: sealed::IdMarker> serde::de::Deserialize<'de> for Id<Marker> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SnowflakeVisitor;

        impl<'de> serde::de::Visitor<'de> for SnowflakeVisitor {
            type Value = NonZeroU64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a non-zero string or integer snowflake")
            }

            // Called by formats like TOML.
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                self.visit_u64(u64::try_from(value).map_err(E::custom)?)
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                NonZeroU64::new(value).ok_or_else(|| E::custom("invalid value, expected non-zero"))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(SnowflakeVisitor).map(Id::new_inner)
    }
}

impl<Marker: sealed::IdMarker> serde::ser::Serialize for Id<Marker> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<Marker: sealed::IdMarker> Default for Id<Marker> {
    fn default() -> Self {
        Self::new(1)
    }
}

// This is a hack so functions can accept iterators that either:
// 1. return the id itself (e.g: `MessageId`)
// 2. return a reference to it (`&MessageId`).
impl<Marker: sealed::IdMarker> AsRef<Id<Marker>> for Id<Marker> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<Marker: sealed::IdMarker> From<u64> for Id<Marker> {
    fn from(id: u64) -> Id<Marker> {
        Id::new(id)
    }
}

impl<Marker: sealed::IdMarker> From<NonZeroU64> for Id<Marker> {
    fn from(id: NonZeroU64) -> Id<Marker> {
        Self::new_inner(id)
    }
}

impl<Marker: sealed::IdMarker> PartialEq<u64> for Id<Marker> {
    fn eq(&self, u: &u64) -> bool {
        self.get() == *u
    }
}

impl<Marker: sealed::IdMarker> fmt::Display for Id<Marker> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner;
        fmt::Display::fmt(&inner, f)
    }
}

impl<Marker: sealed::IdMarker> From<Id<Marker>> for NonZeroU64 {
    fn from(id: Id<Marker>) -> NonZeroU64 {
        id.inner
    }
}

impl<Marker: sealed::IdMarker> From<Id<Marker>> for NonZeroI64 {
    fn from(id: Id<Marker>) -> NonZeroI64 {
        unsafe { NonZeroI64::new_unchecked(id.get() as i64) }
    }
}

impl<Marker: sealed::IdMarker> From<Id<Marker>> for u64 {
    fn from(id: Id<Marker>) -> u64 {
        id.get()
    }
}

impl<Marker: sealed::IdMarker> From<Id<Marker>> for i64 {
    fn from(id: Id<Marker>) -> i64 {
        id.get() as i64
    }
}

impl<Marker: sealed::IdMarker> std::str::FromStr for Id<Marker> {
    type Err = <u64 as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new_inner(s.parse()?))
    }
}

#[doc(hidden)]
pub struct ApplicationMarker;
/// An identifier for an Application.
pub type ApplicationId = Id<ApplicationMarker>;

#[doc(hidden)]
pub struct ChannelMarker;
/// An identifier for a Channel.
pub type ChannelId = Id<ChannelMarker>;

#[doc(hidden)]
pub struct EmojiMarker;
/// An identifier for an Emoji.
pub type EmojiId = Id<EmojiMarker>;

#[doc(hidden)]
pub struct GenericMarker;
/// An identifier for an unspecific entity.
pub type GenericId = Id<GenericMarker>;

#[doc(hidden)]
pub struct GuildMarker;
/// An identifier for a Guild.
pub type GuildId = Id<GuildMarker>;

#[doc(hidden)]
pub struct IntegrationMarker;
/// An identifier for an Integration.
pub type IntegrationId = Id<IntegrationMarker>;

#[doc(hidden)]
pub struct MessageMarker;
/// An identifier for a Message.
pub type MessageId = Id<MessageMarker>;

#[doc(hidden)]
pub struct RoleMarker;
/// An identifier for a Role.
pub type RoleId = Id<RoleMarker>;

#[doc(hidden)]
pub struct RuleMarker;
/// An identifier for an auto moderation rule.
pub type RuleId = Id<RuleMarker>;

#[doc(hidden)]
pub struct ScheduledEventMarker;
/// An identifier for a Scheduled Event.
pub type ScheduledEventId = Id<ScheduledEventMarker>;

#[doc(hidden)]
pub struct UserMarker;
/// An identifier for a User.
pub type UserId = Id<UserMarker>;

#[doc(hidden)]
pub struct WebhookMarker;
/// An identifier for a [`Webhook`][super::webhook::Webhook].
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

#[doc(hidden)]
pub struct CommandPermissionMarker;
/// An identifier for a slash command permission Id. Can contain
/// a [`RoleId`] or [`UserId`].
pub type CommandPermissionId = Id<CommandPermissionMarker>;

#[doc(hidden)]
pub struct CommandVersionMarker;
/// An identifier for a slash command version Id.
pub type CommandVersionId = Id<CommandVersionMarker>;

#[doc(hidden)]
pub struct TargetMarker;
/// An identifier for a slash command target Id. Can contain
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ShardId(pub u32);

impl fmt::Display for ShardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
