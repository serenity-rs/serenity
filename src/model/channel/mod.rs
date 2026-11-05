//! Models relating to channels and types within channels.

mod attachment;
mod channel_id;
mod embed;
mod followed_channel;
mod guild_channel;
mod interaction_channel;
mod message;
mod private_channel;
mod reaction;
mod thread;

use std::borrow::Cow;
use std::fmt;

use serde::de::Error as DeError;
use serde::ser::SerializeMap as _;
use serde_json::value::RawValue;

pub use self::attachment::*;
#[cfg(feature = "model")]
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::followed_channel::*;
pub use self::guild_channel::*;
pub use self::interaction_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::thread::*;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;

impl From<ThreadId> for GenericChannelId {
    fn from(val: ThreadId) -> Self {
        Self::new(val.get())
    }
}

impl From<ChannelId> for GenericChannelId {
    fn from(val: ChannelId) -> Self {
        Self::new(val.get())
    }
}

impl GenericChannelId {
    /// Copies this ID into a [`ChannelId`] and a [`ThreadId`].
    ///
    /// It is only correct to use this when you use both returned values,
    /// otherwise use [`Self::expect_channel`] or [`Self::expect_thread`].
    #[must_use]
    pub fn split(self) -> (ChannelId, ThreadId) {
        (self.expect_channel(), self.expect_thread())
    }

    /// Converts the type of this Id to [`ChannelId`].
    ///
    /// This converts the type without changing the inner value, therefore,
    /// is only correct when you have knowledge which is not in the type system.
    ///
    /// This should be used as rarely as [`Option::expect`].
    #[must_use]
    pub fn expect_channel(self) -> ChannelId {
        ChannelId::new(self.get())
    }

    /// Converts the type of this Id to [`ThreadId`].
    ///
    /// This converts the type without changing the inner value, therefore,
    /// is only correct when you have knowledge which is not in the type system.
    ///
    /// This should be used as rarely as [`Option::expect`].
    #[must_use]
    pub fn expect_thread(self) -> ThreadId {
        ThreadId::new(self.get())
    }
}

/// A container for a reference to any Guild channel.
#[derive(Clone, Copy, Debug)]
// purposefully missing non-exhaustive, as discord considers new channel types like threads to be
// breaking (see the difference between API v8/v9).
pub enum GenericGuildChannelRef<'a> {
    Channel(GuildChannelRef<'a>),
    Thread(&'a GuildThread),
}

impl<'a> GenericGuildChannelRef<'a> {
    /// Returns the [`GenericChannelId`] of the [`GenericGuildChannelRef`].
    #[must_use]
    pub fn id(self) -> GenericChannelId {
        match self {
            Self::Channel(ch) => match ch {
                GuildChannelRef::Viewable(gc) => gc.id.widen(),
                GuildChannelRef::Obfuscated(oc) => oc.id.widen(),
            },
            Self::Thread(th) => th.id.widen(),
        }
    }

    /// Returns the shared fields between a [`GuildChannel`] and a [`GuildThread`],
    /// or an obfuscated version if [`GenericGuildChannelRef`] is an [`ObfuscatedChannel`].
    #[must_use]
    pub fn base(self) -> Cow<'a, BaseGuildChannel> {
        match self {
            Self::Channel(ch) => match ch {
                GuildChannelRef::Viewable(gc) => Cow::Borrowed(&gc.base),
                GuildChannelRef::Obfuscated(oc) => Cow::Owned(oc.into()),
            },
            Self::Thread(th) => Cow::Borrowed(&th.base),
        }
    }
}

/// A container for a reference to a non-thread guild channel.
#[derive(Clone, Copy, Debug)]
pub enum GuildChannelRef<'a> {
    Viewable(&'a GuildChannel),
    Obfuscated(&'a ObfuscatedChannel),
}

impl<'a> GuildChannelRef<'a> {
    /// Returns the [`GenericChannelId`] of the [`GuildChannelRef`].
    #[must_use]
    pub fn id(self) -> GenericChannelId {
        match self {
            GuildChannelRef::Viewable(gc) => gc.id.widen(),
            GuildChannelRef::Obfuscated(oc) => oc.id.widen(),
        }
    }

    /// Returns the [`ChannelType`] of the [`GuildChannelRef`].
    #[must_use]
    pub fn kind(self) -> ChannelType {
        match self {
            GuildChannelRef::Viewable(gc) => gc.base.kind,
            GuildChannelRef::Obfuscated(oc) => oc.kind,
        }
    }

    /// Returns the name of the [`GuildChannelRef`].
    ///
    /// **Note**: If the channel is obfuscated, this will return `___hidden___`.
    #[must_use]
    pub fn name(self) -> &'a str {
        match self {
            GuildChannelRef::Viewable(gc) => gc.base.name.as_str(),
            GuildChannelRef::Obfuscated(_) => "___hidden___",
        }
    }

    /// Returns the [`ChannelId`] of the parent category [`GuildChannelRef`] belongs to.
    ///
    /// **Note**: If the channel does not belong to a category, this will return `None`.
    #[must_use]
    pub fn parent_id(self) -> Option<ChannelId> {
        match self {
            GuildChannelRef::Viewable(gc) => gc.parent_id,
            GuildChannelRef::Obfuscated(oc) => oc.parent_id,
        }
    }

    /// Returns the position of the [`GuildChannelRef`].
    #[must_use]
    pub fn position(self) -> u16 {
        match self {
            GuildChannelRef::Viewable(gc) => gc.position,
            GuildChannelRef::Obfuscated(oc) => oc.position,
        }
    }

    /// Returns the [`GuildId`] of the [`GuildChannelRef`].
    #[must_use]
    pub fn guild_id(self) -> GuildId {
        match self {
            GuildChannelRef::Viewable(gc) => gc.base.guild_id,
            GuildChannelRef::Obfuscated(oc) => oc.guild_id,
        }
    }
}

impl<'a> From<&'a GuildChannel> for GuildChannelRef<'a> {
    fn from(guild_channel: &'a GuildChannel) -> Self {
        Self::Viewable(guild_channel)
    }
}

impl<'a> From<&'a ObfuscatedChannel> for GuildChannelRef<'a> {
    fn from(obfuscated_channel: &'a ObfuscatedChannel) -> Self {
        Self::Obfuscated(obfuscated_channel)
    }
}

/// A container for any channel.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Channel {
    /// A viewable channel within a [`Guild`].
    GuildViewable(GuildChannel),
    /// An obfuscated channel within a [`Guild`].
    GuildObfuscated(ObfuscatedChannel),
    /// A thread inside a [`Guild`].
    GuildThread(GuildThread),
    /// A private channel to another [`User`] (Direct Message). No other users may access the
    /// channel.
    Private(PrivateChannel),
}

#[cfg(feature = "model")]
impl Channel {
    /// If this is a viewable guild channel, returns it.
    #[must_use]
    pub fn guild(self) -> Option<GuildChannel> {
        match self {
            Self::GuildViewable(channel) => Some(channel),
            _ => None,
        }
    }
    /// If this is an obfuscated guild channel, returns it.
    #[must_use]
    pub fn obfuscated(self) -> Option<ObfuscatedChannel> {
        match self {
            Self::GuildObfuscated(channel) => Some(channel),
            _ => None,
        }
    }

    /// If this is a guild thread, returns it.
    #[must_use]
    pub fn thread(self) -> Option<GuildThread> {
        match self {
            Self::GuildThread(thread) => Some(thread),
            _ => None,
        }
    }

    /// If this is a private channel, returns it.
    #[must_use]
    pub fn private(self) -> Option<PrivateChannel> {
        match self {
            Self::Private(channel) => Some(channel),
            _ => None,
        }
    }

    /// If this is a category channel, returns it.
    #[must_use]
    pub fn category(self) -> Option<MaybeObfuscated> {
        match self {
            Self::GuildViewable(gc) if gc.base.kind == ChannelType::Category => {
                Some(MaybeObfuscated::Viewable(gc))
            },
            Self::GuildObfuscated(oc) if oc.kind == ChannelType::Category => {
                Some(MaybeObfuscated::Obfuscated(oc))
            },
            _ => None,
        }
    }

    /// Deletes the inner channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or
    /// [`ModelError::InvalidChannelType`] if the channel is obfuscated.
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        match self {
            Self::GuildViewable(public_channel) => {
                public_channel.delete(http, reason).await?;
            },
            Self::GuildObfuscated(_) => {
                return Err(Error::Model(ModelError::InvalidChannelType));
            },
            Self::GuildThread(thread) => {
                thread.delete(http, reason).await?;
            },
            Self::Private(private_channel) => {
                private_channel.delete(http).await?;
            },
        }

        Ok(())
    }

    /// Retrieves the inner Id.
    #[must_use]
    pub fn id(&self) -> GenericChannelId {
        match self {
            Self::GuildViewable(ch) => ch.id.widen(),
            Self::GuildObfuscated(ch) => ch.id.widen(),
            Self::GuildThread(ch) => ch.id.widen(),
            Self::Private(ch) => ch.id.widen(),
        }
    }

    /// If this is a guild channel or guild thread, returns the corresponding guild's Id.
    #[must_use]
    pub fn guild_id(&self) -> Option<GuildId> {
        match self {
            Channel::GuildThread(thread) => Some(thread.base.guild_id),
            Channel::GuildViewable(channel) => Some(channel.base.guild_id),
            Channel::GuildObfuscated(channel) => Some(channel.guild_id),
            Channel::Private(_) => None,
        }
    }
}

fn extract_type<'de, D>(deserializer: D) -> StdResult<(u64, bool, &'de RawValue), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct ChannelRaw {
        #[serde(rename = "type")]
        kind: u64,
        #[serde(default)]
        flags: Option<ChannelFlags>,
    }

    let raw_data = <&RawValue>::deserialize(deserializer)?;
    let raw = ChannelRaw::deserialize(raw_data).map_err(DeError::custom)?;
    let is_obfuscated =
        raw.flags.is_some_and(|flags| flags.contains(ChannelFlags::CHANNEL_OBFUSCATED));

    Ok((raw.kind, is_obfuscated, raw_data))
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let (kind, is_obfuscated, raw_data) = extract_type(deserializer)?;

        match kind {
            0 | 2 | 4 | 5 | 13 | 14 | 15 => {
                if is_obfuscated {
                    Deserialize::deserialize(raw_data).map(Channel::GuildObfuscated)
                } else {
                    Deserialize::deserialize(raw_data).map(Channel::GuildViewable)
                }
            },
            10..=12 => Deserialize::deserialize(raw_data).map(Channel::GuildThread),
            1 => Deserialize::deserialize(raw_data).map(Channel::Private),
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
            Self::GuildViewable(ch) => fmt::Display::fmt(&ch.id.widen().mention(), f),
            Self::GuildObfuscated(ch) => fmt::Display::fmt(&ch.id.widen().mention(), f),
            Self::GuildThread(ch) => fmt::Display::fmt(&ch.id.widen().mention(), f),
            Self::Private(ch) => fmt::Display::fmt(&ch.recipient.name, f),
        }
    }
}

enum_number! {
    /// A representation of a type of channel.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-channel-types).
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

/// [Discord docs](https://docs.discord.com/developers/resources/channel#overwrite-object).
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
/// [Discord docs](https://docs.discord.com/developers/resources/channel#overwrite-object).
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
/// This is for use with methods such as [`ChannelId::create_permission`].
///
/// If you would like to modify the default permissions of a channel, you can get its [`RoleId`]
/// from [`GuildId::everyone_role`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#overwrite-object-overwrite-structure) (field `type`).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-video-quality-modes).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/stage-instance#stage-instance-object-privacy-level).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#thread-metadata-object)
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
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

/// [Discord docs](https://docs.discord.com/developers/resources/stage-instance#stage-instance-object).
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

/// A response to getting several threads channels.
///
/// Discord docs: defined [multiple times](https://docs.discord.com/developers/topics/threads#enumerating-threads):
/// [1](https://docs.discord.com/developers/resources/guild#list-active-guild-threads-response-body),
/// [2](https://docs.discord.com/developers/resources/channel#list-private-archived-threads-response-body),
/// [3](https://docs.discord.com/developers/resources/channel#list-public-archived-threads-response-body),
/// [4](https://docs.discord.com/developers/resources/channel#list-joined-private-archived-threads-response-body)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadsData {
    /// The threads channels.
    pub threads: FixedArray<GuildThread>,
    /// A thread member for each returned thread the current user has joined.
    pub members: FixedArray<ThreadMember>,
    /// Whether there are potentially more threads that could be returned on a subsequent call.
    #[serde(default)]
    pub has_more: bool,
}

/// An object that specifies the emoji to use for Forum related emoji parameters.
///
/// See [Discord](https://docs.discord.com/developers/resources/channel#default-reaction-object)
/// [docs]()
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, PartialEq)]
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
/// See [Discord docs](https://docs.discord.com/developers/resources/channel#forum-tag-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-sort-order-types).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-channel-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ChannelFlags: u32 {
        /// This thread is pinned to the top of its parent `GUILD_FORUM` channel.
        const PINNED = 1 << 1;
        /// Whether a tag is required to be specified when creating a thread in a `GUILD_FORUM`
        /// channel. Tags are specified in the `applied_tags` field.
        const REQUIRE_TAG = 1 << 4;
        /// This channel's metadata has been obfuscated because the current user cannot view it.
        ///
        /// Only ever set on channels received over the Gateway; the HTTP API never sets this flag.
        /// See [Obfuscated Channels].
        ///
        /// [Obfuscated Channels]: https://docs.discord.com/developers/resources/channel#channel-object-obfuscated-channels
        const CHANNEL_OBFUSCATED = 1 << 17;
        /// Whether the channel is a spoiler channel. Can be set on text-based guild channels and
        /// voice channels. Cannot be set if `nsfw` is true.
        const IS_SPOILER_CHANNEL = 1 << 21;
    }
}

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#channel-info-channel-info-channel-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChannelInfoChannel {
    pub id: ChannelId,
    #[serde(default)]
    pub status: Option<FixedString<u16>>,
    #[serde(default)]
    pub voice_start_time: Option<i64>,
}

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#request-channel-info-request-channel-info-structure).
#[derive(Debug)]
pub enum ChannelInfoField {
    All,
    Status,
    VoiceStartTime,
}

impl From<ChannelInfoField> for Vec<String> {
    fn from(field: ChannelInfoField) -> Self {
        match field {
            ChannelInfoField::All => {
                Vec::from([String::from("status"), String::from("voice_start_time")])
            },
            ChannelInfoField::Status => Vec::from([String::from("status")]),
            ChannelInfoField::VoiceStartTime => Vec::from([String::from("voice_start_time")]),
        }
    }
}
