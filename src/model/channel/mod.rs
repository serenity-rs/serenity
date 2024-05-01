//! Models relating to channels and types within channels.

mod attachment;
mod channel_id;
mod embed;
mod guild_channel;
mod message;
mod partial_channel;
mod private_channel;
mod reaction;

use std::fmt;

use serde::de::{Error as DeError, Unexpected};
use serde::ser::SerializeMap as _;
use serde_json::from_value;

pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::partial_channel::*;
pub use self::private_channel::*;
pub use self::reaction::*;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::is_false;

/// A container for any channel.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Channel {
    /// A channel within a [`Guild`].
    Guild(GuildChannel),
    /// A private channel to another [`User`] (Direct Message). No other users may access the
    /// channel.
    Private(PrivateChannel),
}

#[cfg(feature = "model")]
impl Channel {
    /// Converts from [`Channel`] to `Option<GuildChannel>`.
    ///
    /// Converts `self` into an `Option<GuildChannel>`, consuming `self`, and discarding a
    /// [`PrivateChannel`] if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use serenity::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// match channel.guild() {
    ///     Some(guild_channel) => {
    ///         println!("It's a guild channel named {}!", guild_channel.name);
    ///     },
    ///     None => {
    ///         println!("It's not in a guild!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn guild(self) -> Option<GuildChannel> {
        match self {
            Self::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from [`Channel`] to `Option<PrivateChannel>`.
    ///
    /// Converts `self` into an `Option<PrivateChannel>`, consuming `self`, and discarding a
    /// [`GuildChannel`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use serenity::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// #
    /// match channel.private() {
    ///     Some(private) => {
    ///         println!("It's a private channel with {}!", &private.recipient);
    ///     },
    ///     None => {
    ///         println!("It's not a private channel!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn private(self) -> Option<PrivateChannel> {
        match self {
            Self::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// If this is a category channel, returns it.
    #[must_use]
    pub fn category(self) -> Option<GuildChannel> {
        match self {
            Self::Guild(c) if c.kind == ChannelType::Category => Some(c),
            _ => None,
        }
    }

    /// Deletes the inner channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        match self {
            Self::Guild(public_channel) => {
                public_channel.delete(http, reason).await?;
            },
            Self::Private(private_channel) => {
                private_channel.delete(http).await?;
            },
        }

        Ok(())
    }

    /// Retrieves the Id of the inner [`GuildChannel`], or [`PrivateChannel`].
    #[must_use]
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::Guild(ch) => ch.id,
            Self::Private(ch) => ch.id,
        }
    }

    /// Retrieves the position of the inner [`GuildChannel`].
    ///
    /// In DMs (private channel) it will return None.
    #[must_use]
    pub const fn position(&self) -> Option<u16> {
        match self {
            Self::Guild(channel) => Some(channel.position),
            Self::Private(_) => None,
        }
    }
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = {
            let kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?;
            kind.as_u64().ok_or_else(|| {
                DeError::invalid_type(
                    Unexpected::Other("non-positive integer"),
                    &"a positive integer",
                )
            })?
        };

        let value = Value::from(map);
        match kind {
            0 | 2 | 4 | 5 | 10 | 11 | 12 | 13 | 14 | 15 => from_value(value).map(Channel::Guild),
            1 => from_value(value).map(Channel::Private),
            _ => return Err(DeError::custom("Unknown channel type")),
        }
        .map_err(DeError::custom)
    }
}

impl fmt::Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can see the channel can
    ///   click on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guild(ch) => fmt::Display::fmt(&ch.id.mention(), f),
            Self::Private(ch) => fmt::Display::fmt(&ch.recipient.name, f),
        }
    }
}

enum_number! {
    /// A representation of a type of channel.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ChannelType {
        /// An indicator that the channel is a text [`GuildChannel`].
        Text = 0,
        /// An indicator that the channel is a [`PrivateChannel`].
        Private = 1,
        /// An indicator that the channel is a voice [`GuildChannel`].
        Voice = 2,
        /// An indicator that the channel is a group DM.
        GroupDm = 3,
        /// An indicator that the channel is a channel category.
        Category = 4,
        /// An indicator that the channel is a `NewsChannel`.
        ///
        /// Note: `NewsChannel` is serialized into a [`GuildChannel`]
        News = 5,
        /// An indicator that the channel is a news thread [`GuildChannel`].
        NewsThread = 10,
        /// An indicator that the channel is a public thread [`GuildChannel`].
        PublicThread = 11,
        /// An indicator that the channel is a private thread [`GuildChannel`].
        PrivateThread = 12,
        /// An indicator that the channel is a stage [`GuildChannel`].
        Stage = 13,
        /// An indicator that the channel is a directory [`GuildChannel`] in a [hub].
        ///
        /// [hub]: https://support.discord.com/hc/en-us/articles/4406046651927-Discord-Student-Hubs-FAQ
        Directory = 14,
        /// An indicator that the channel is a forum [`GuildChannel`].
        Forum = 15,
        _ => Unknown(u8),
    } // Make sure to update [`GuildChannel::is_text_based`].
}

impl ChannelType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Text => "text",
            Self::Voice => "voice",
            Self::GroupDm => "group_dm",
            Self::Category => "category",
            Self::News => "news",
            Self::NewsThread => "news_thread",
            Self::PublicThread => "public_thread",
            Self::PrivateThread => "private_thread",
            Self::Stage => "stage",
            Self::Directory => "directory",
            Self::Forum => "forum",
            Self(_) => "unknown",
        }
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    id: TargetId,
    #[serde(rename = "type")]
    kind: u8,
}

pub(crate) struct InvalidPermissionOverwriteType(u8);

impl std::fmt::Display for InvalidPermissionOverwriteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Permission Overwrite Type: {}", self.0)
    }
}

impl std::convert::TryFrom<PermissionOverwriteData> for PermissionOverwrite {
    type Error = InvalidPermissionOverwriteType;

    fn try_from(data: PermissionOverwriteData) -> StdResult<Self, Self::Error> {
        let kind = match data.kind {
            0 => PermissionOverwriteType::Role(data.id.get().into()),
            1 => PermissionOverwriteType::Member(data.id.into()),
            raw => return Err(InvalidPermissionOverwriteType(raw)),
        };

        Ok(PermissionOverwrite {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl From<PermissionOverwrite> for PermissionOverwriteData {
    fn from(data: PermissionOverwrite) -> Self {
        let (kind, id) = match data.kind {
            PermissionOverwriteType::Role(id) => (0, id.get().into()),
            PermissionOverwriteType::Member(id) => (1, id.into()),
        };

        PermissionOverwriteData {
            allow: data.allow,
            deny: data.deny,
            kind,
            id,
        }
    }
}

/// A channel-specific permission overwrite for a member or role.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "PermissionOverwriteData", into = "PermissionOverwriteData")]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as [`GuildChannel::create_permission`].
///
/// If you would like to modify the default permissions of a channel, you can get its [`RoleId`]
/// from [`GuildId::everyone_role`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#overwrite-object-overwrite-structure) (field `type`).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

enum_number! {
    /// The video quality mode for a voice channel.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-video-quality-modes).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum VideoQualityMode {
        /// An indicator that the video quality is chosen by Discord for optimal
        /// performance.
        Auto = 1,
        /// An indicator that the video quality is 720p.
        Full = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// See [`StageInstance::privacy_level`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/stage-instance#stage-instance-object-privacy-level).
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    #[<default> = 2]
    pub enum StageInstancePrivacyLevel {
        /// The Stage instance is visible publicly. (deprecated)
        Public = 1,
        /// The Stage instance is visible to only guild members.
        GuildOnly = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// See [`ThreadMetadata::auto_archive_duration`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object)
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum AutoArchiveDuration {
        None = 0,
        OneHour = 60,
        OneDay = 1440,
        ThreeDays = 4320,
        OneWeek = 10080,
        _ => Unknown(u16),
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/stage-instance#stage-instance-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StageInstance {
    /// The Id of the stage instance.
    pub id: StageInstanceId,
    /// The guild Id of the associated stage channel.
    pub guild_id: GuildId,
    /// The Id of the associated stage channel.
    pub channel_id: ChannelId,
    /// The topic of the stage instance.
    pub topic: FixedString,
    /// The privacy level of the Stage instance.
    pub privacy_level: StageInstancePrivacyLevel,
    /// Whether or not Stage Discovery is disabled (deprecated).
    pub discoverable_disabled: bool,
    /// The id of the scheduled event for this Stage instance.
    pub guild_scheduled_event_id: Option<ScheduledEventId>,
}

/// A thread data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    pub auto_archive_duration: AutoArchiveDuration,
    /// The last time the thread's archive status was last changed; used for calculating recent
    /// activity.
    pub archive_timestamp: Option<Timestamp>,
    /// When a thread is locked, only users with `MANAGE_THREADS` permission can unarchive it.
    #[serde(default)]
    pub locked: bool,
    /// Timestamp when the thread was created.
    ///
    /// **Note**: only populated for threads created after 2022-01-09
    pub create_timestamp: Option<Timestamp>,
    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    #[serde(default, skip_serializing_if = "is_false")]
    pub invitable: bool,
}

/// A response to getting several threads channels.
///
/// Discord docs: defined [multiple times](https://discord.com/developers/docs/topics/threads#enumerating-threads):
/// [1](https://discord.com/developers/docs/resources/guild#list-active-guild-threads-response-body),
/// [2](https://discord.com/developers/docs/resources/channel#list-private-archived-threads-response-body),
/// [3](https://discord.com/developers/docs/resources/channel#list-public-archived-threads-response-body),
/// [4](https://discord.com/developers/docs/resources/channel#list-private-archived-threads-response-body)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadsData {
    /// The threads channels.
    pub threads: FixedArray<GuildChannel>,
    /// A thread member for each returned thread the current user has joined.
    pub members: FixedArray<ThreadMember>,
    /// Whether there are potentially more threads that could be returned on a subsequent call.
    #[serde(default)]
    pub has_more: bool,
}

/// An object that specifies the emoji to use for Forum related emoji parameters.
///
/// See [Discord](https://discord.com/developers/docs/resources/channel#default-reaction-object)
/// [docs]()
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ForumEmoji {
    /// The id of a guild's custom emoji.
    Id(EmojiId),
    /// The unicode character of the emoji.
    Name(FixedString),
}

#[derive(Deserialize)]
struct RawForumEmoji {
    emoji_id: Option<EmojiId>,
    emoji_name: Option<FixedString>,
}

impl serde::Serialize for ForumEmoji {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            Self::Id(id) => {
                map.serialize_entry("emoji_id", id)?;
                map.serialize_entry("emoji_name", &None::<()>)?;
            },
            Self::Name(name) => {
                map.serialize_entry("emoji_id", &None::<()>)?;
                map.serialize_entry("emoji_name", name)?;
            },
        }

        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for ForumEmoji {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let helper = RawForumEmoji::deserialize(deserializer)?;
        match (helper.emoji_id, helper.emoji_name) {
            (Some(id), None) => Ok(ForumEmoji::Id(id)),
            (None, Some(name)) => Ok(ForumEmoji::Name(name)),
            (None, None) => {
                Err(serde::de::Error::custom("expected emoji_name or emoji_id, found neither"))
            },
            (Some(_), Some(_)) => {
                Err(serde::de::Error::custom("expected emoji_name or emoji_id, found both"))
            },
        }
    }
}

/// An object that represents a tag able to be applied to a thread in a `GUILD_FORUM` channel.
///
/// See [Discord docs](https://discord.com/developers/docs/resources/channel#forum-tag-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForumTag {
    /// The id of the tag.
    pub id: ForumTagId,
    /// The name of the tag (0-20 characters).
    pub name: FixedString<u8>,
    /// Whether this tag can only be added to or removed from threads by a member with the
    /// MANAGE_THREADS permission.
    pub moderated: bool,
    /// The emoji to display next to the tag.
    #[serde(flatten)]
    pub emoji: Option<ForumEmoji>,
}

enum_number! {
    /// The sort order for threads in a forum.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-sort-order-types).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum SortOrder {
        /// Sort forum posts by activity.
        LatestActivity = 0,
        /// Sort forum posts by creation time (from most recent to oldest).
        CreationDate = 1,
        _ => Unknown(u8),
    }
}

bitflags! {
    /// Describes extra features of the channel.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ChannelFlags: u64 {
        /// This thread is pinned to the top of its parent GUILD_FORUM channel
        const PINNED = 1 << 1;
        /// Whether a tag is required to be specified when creating a
        /// thread in a GUILD_FORUM channel. Tags are specified in the applied_tags field.
        const REQUIRE_TAG = 1 << 4;
    }
}
