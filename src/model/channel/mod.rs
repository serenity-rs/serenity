//! Models relating to channels and types within channels.

mod attachment;
mod attachment_type;
mod channel_category;
mod channel_id;
mod embed;
mod guild_channel;
mod message;
mod partial_channel;
mod private_channel;
mod reaction;

use std::fmt;

use serde::de::{Error as DeError, Unexpected};
use serde::ser::{Serialize, SerializeStruct, Serializer};

pub use self::attachment::*;
pub use self::attachment_type::*;
pub use self::channel_category::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::partial_channel::*;
pub use self::private_channel::*;
pub use self::reaction::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(feature = "model")]
use crate::http::CacheHttp;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::model::misc::ChannelParseError;
use crate::model::utils::is_false;
use crate::model::Timestamp;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_channel;
use crate::{json::prelude::*, model::prelude::*};

/// A container for any channel.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Channel {
    /// A [text], [voice], or [stage] channel within a [`Guild`].
    ///
    /// [text]: ChannelType::Text
    /// [voice]: ChannelType::Voice
    /// [stage]: ChannelType::Stage
    Guild(GuildChannel),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    Private(PrivateChannel),
    /// A category of [`GuildChannel`]s
    Category(ChannelCategory),
}

impl Channel {
    /// Converts from [`Channel`] to `Option<GuildChannel>`.
    ///
    /// Converts `self` into an `Option<GuildChannel>`, consuming
    /// `self`, and discarding a [`PrivateChannel`], or
    /// [`ChannelCategory`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # async fn run() {
    /// # use serenity::{cache::Cache, model::id::ChannelId};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache = Cache::default();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
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
    pub fn guild(self) -> Option<GuildChannel> {
        match self {
            Channel::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from [`Channel`] to `Option<PrivateChannel>`.
    ///
    /// Converts `self` into an `Option<PrivateChannel>`, consuming
    /// `self`, and discarding a [`GuildChannel`], or [`ChannelCategory`],
    /// if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # async fn run() {
    /// # use serenity::{cache::Cache, model::id::ChannelId};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache = Cache::default();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
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
    pub fn private(self) -> Option<PrivateChannel> {
        match self {
            Channel::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from [`Channel`] to `Option<ChannelCategory>`.
    ///
    /// Converts `self` into an `Option<ChannelCategory>`,
    /// consuming `self`, and discarding a [`GuildChannel`], or
    /// [`PrivateChannel`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # async fn run() {
    /// # use serenity::{cache::Cache, model::id::ChannelId};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache = Cache::default();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
    /// match channel.category() {
    ///     Some(category) => {
    ///         println!("It's a category named {}!", category.name);
    ///     },
    ///     None => {
    ///         println!("It's not a category!");
    ///     },
    /// }
    /// #
    /// # }
    /// ```
    pub fn category(self) -> Option<ChannelCategory> {
        match self {
            Channel::Category(lock) => Some(lock),
            _ => None,
        }
    }

    /// Deletes the inner channel.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`],
    /// if the current user lacks permission.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not
    /// have permission.
    #[cfg(feature = "http")]
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        match self {
            Channel::Guild(public_channel) => {
                public_channel.delete(cache_http).await?;
            },
            Channel::Private(private_channel) => {
                private_channel.delete(cache_http.http()).await?;
            },
            Channel::Category(category) => {
                category.delete(cache_http).await?;
            },
        }

        Ok(())
    }

    /// Determines if the channel is NSFW.
    #[inline]
    #[cfg(feature = "model")]
    pub fn is_nsfw(&self) -> bool {
        match self {
            Channel::Guild(channel) => channel.is_nsfw(),
            Channel::Category(category) => category.is_nsfw(),
            Channel::Private(_) => false,
        }
    }

    /// Retrieves the Id of the inner [`GuildChannel`], or
    /// [`PrivateChannel`].
    #[inline]
    pub fn id(&self) -> ChannelId {
        match self {
            Channel::Guild(ch) => ch.id,
            Channel::Private(ch) => ch.id,
            Channel::Category(ch) => ch.id,
        }
    }

    /// Retrieves the position of the inner [`GuildChannel`] or
    /// [`ChannelCategory`].
    ///
    /// If other channel types are used it will return None.
    #[inline]
    pub fn position(&self) -> Option<i64> {
        match self {
            Channel::Guild(channel) => Some(channel.position),
            Channel::Category(category) => Some(category.position),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let v = JsonMap::deserialize(deserializer)?;
        let kind = {
            let kind = v.get("type").ok_or_else(|| DeError::missing_field("type"))?;

            match kind.as_u64() {
                Some(kind) => kind,
                None => {
                    return Err(DeError::invalid_type(
                        Unexpected::Other("non-positive integer"),
                        &"a positive integer",
                    ));
                },
            }
        };

        match kind {
            0 | 2 | 5 | 6 | 10 | 11 | 12 | 13 | 15 => from_value::<GuildChannel>(Value::from(v))
                .map(Channel::Guild)
                .map_err(DeError::custom),
            1 => from_value::<PrivateChannel>(Value::from(v))
                .map(Channel::Private)
                .map_err(DeError::custom),
            4 => from_value::<ChannelCategory>(Value::from(v))
                .map(Channel::Category)
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown channel type")),
        }
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Channel::Category(c) => ChannelCategory::serialize(c, serializer),
            Channel::Guild(c) => GuildChannel::serialize(c, serializer),
            Channel::Private(c) => PrivateChannel::serialize(c, serializer),
        }
    }
}

impl fmt::Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Channel::Guild(ch) => fmt::Display::fmt(&ch.id.mention(), f),
            Channel::Private(ch) => fmt::Display::fmt(&ch.recipient.name, f),
            Channel::Category(ch) => fmt::Display::fmt(&ch.name, f),
        }
    }
}

/// A representation of a type of channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelType {
    /// An indicator that the channel is a text [`GuildChannel`].
    Text = 0,
    /// An indicator that the channel is a [`PrivateChannel`].
    Private = 1,
    /// An indicator that the channel is a voice [`GuildChannel`].
    Voice = 2,
    /// An indicator that the channel is the channel of a [`ChannelCategory`].
    Category = 4,
    /// An indicator that the channel is a `NewsChannel`.
    ///
    /// Note: `NewsChannel` is serialized into a [`GuildChannel`]
    News = 5,
    /// An indicator that the channel is a `StoreChannel`
    ///
    /// Note: `StoreChannel` is serialized into a [`GuildChannel`]
    Store = 6,
    /// An indicator that the channel is a news thread [`GuildChannel`].
    NewsThread = 10,
    /// An indicator that the channel is a public thread [`GuildChannel`].
    PublicThread = 11,
    /// An indicator that the channel is a private thread [`GuildChannel`].
    PrivateThread = 12,
    /// An indicator that the channel is a stage [`GuildChannel`].
    Stage = 13,
    /// An indicator that the channel is a forum [`GuildChannel`].
    #[cfg(feature = "unstable_discord_api")]
    Forum = 15,
    /// An indicator that the channel is of unknown type.
    Unknown = !0,
}

enum_number!(ChannelType {
    Text,
    Private,
    Voice,
    Category,
    News,
    Store,
    NewsThread,
    PublicThread,
    PrivateThread,
    Stage,
    #[cfg(feature = "unstable_discord_api")]
    Forum,
});

impl ChannelType {
    #[inline]
    pub fn name(&self) -> &str {
        match *self {
            ChannelType::Private => "private",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Category => "category",
            ChannelType::News => "news",
            ChannelType::Store => "store",
            ChannelType::NewsThread => "news_thread",
            ChannelType::PublicThread => "public_thread",
            ChannelType::PrivateThread => "private_thread",
            ChannelType::Stage => "stage",
            #[cfg(feature = "unstable_discord_api")]
            ChannelType::Forum => "forum",
            ChannelType::Unknown => "unknown",
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(with = "snowflake")]
    id: u64,
    #[serde(rename = "type")]
    kind: u8,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug, PartialEq)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> StdResult<PermissionOverwrite, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let kind = match &data.kind {
            0 => PermissionOverwriteType::Role(RoleId(data.id)),
            1 => PermissionOverwriteType::Member(UserId(data.id)),
            _ => return Err(DeError::custom("Unknown PermissionOverwriteType")),
        };

        Ok(PermissionOverwrite {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl Serialize for PermissionOverwrite {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (id, kind) = match self.kind {
            PermissionOverwriteType::Role(id) => (GenericId(id.0), 0),
            PermissionOverwriteType::Member(id) => (GenericId(id.0), 1),
        };

        let mut state = serializer.serialize_struct("PermissionOverwrite", 4)?;
        state.serialize_field("allow", &self.allow)?;
        state.serialize_field("deny", &self.deny)?;
        state.serialize_field("id", &id)?;
        state.serialize_field("type", &kind)?;

        state.end()
    }
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as [`GuildChannel::create_permission`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

/// The video quality mode for a voice channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum VideoQualityMode {
    /// An indicator that the video quality is chosen by Discord for optimal
    /// performance.
    Auto = 1,
    /// An indicator that the video quality is 720p.
    Full = 2,
    /// An indicator that video quality is of unknown type.
    Unknown = !0,
}

enum_number!(VideoQualityMode {
    Auto,
    Full
});

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
    pub topic: String,
}

/// A thread data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: It can currently only be set to 60, 1440, 4320, 10080.
    pub auto_archive_duration: Option<u64>,
    /// Timestamp when the thread's archive status was last changed, used for calculating recent activity.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThreadsData {
    /// The threads channels.
    pub threads: Vec<GuildChannel>,
    /// A thread member for each returned thread the current user has joined.
    pub members: Vec<ThreadMember>,
    /// Whether there are potentially additional threads that could be returned on a subsequent call.
    #[serde(default)]
    pub has_more: bool,
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "model", feature = "utils"))]
    mod model_utils {
        use crate::model::prelude::*;

        fn guild_channel() -> GuildChannel {
            GuildChannel {
                id: ChannelId(1),
                bitrate: None,
                parent_id: None,
                guild_id: GuildId(2),
                kind: ChannelType::Text,
                last_message_id: None,
                last_pin_timestamp: None,
                name: "nsfw-stuff".to_string(),
                permission_overwrites: vec![],
                position: 0,
                topic: None,
                user_limit: None,
                nsfw: false,
                rate_limit_per_user: Some(0),
                rtc_region: None,
                video_quality_mode: None,
                message_count: None,
                member_count: None,
                thread_metadata: None,
                member: None,
                default_auto_archive_duration: None,
            }
        }

        fn private_channel() -> PrivateChannel {
            PrivateChannel {
                id: ChannelId(1),
                last_message_id: None,
                last_pin_timestamp: None,
                kind: ChannelType::Private,
                recipient: User {
                    id: UserId(2),
                    avatar: None,
                    bot: false,
                    discriminator: 1,
                    name: "ab".to_string(),
                    public_flags: None,
                    banner: None,
                    accent_colour: None,
                },
            }
        }

        #[test]
        fn nsfw_checks() {
            let mut channel = guild_channel();
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());

            channel.kind = ChannelType::Text;
            channel.name = "nsfw-".to_string();
            assert!(!channel.is_nsfw());

            channel.name = "nsfw".to_string();
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Text;

            channel.name = "nsf".to_string();
            channel.nsfw = true;
            assert!(channel.is_nsfw());
            channel.nsfw = false;
            assert!(!channel.is_nsfw());

            let channel = Channel::Guild(channel);
            assert!(!channel.is_nsfw());

            let private_channel = private_channel();
            assert!(!private_channel.is_nsfw());
        }
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStrAndCache for Channel {
    type Err = ChannelParseError;

    fn from_str<C>(cache: C, s: &str) -> StdResult<Self, Self::Err>
    where
        C: AsRef<Cache> + Send + Sync,
    {
        match parse_channel(s) {
            Some(x) => match ChannelId(x).to_channel_cached(&cache) {
                Some(channel) => Ok(channel),
                _ => Err(ChannelParseError::NotPresentInCache),
            },
            _ => Err(ChannelParseError::InvalidChannel),
        }
    }
}
