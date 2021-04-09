//! Models relating to channels and types within channels.

mod attachment;
mod channel_category;
mod channel_id;
mod embed;
mod guild_channel;
mod message;
mod private_channel;
mod reaction;
mod sticker;

#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use async_trait::async_trait;
use serde::de::{Error as DeError, Unexpected};
use serde::ser::{Serialize, SerializeStruct, Serializer};

pub use self::attachment::*;
pub use self::channel_category::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::sticker::*;
use super::utils::deserialize_u64;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(feature = "model")]
use crate::http::CacheHttp;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::model::misc::ChannelParseError;
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

#[cfg(feature = "model")]
impl Channel {
    /// Converts from `Channel` to `Option<GuildChannel>`.
    ///
    /// Converts `self` into an `Option<GuildChannel>`, consuming
    /// `self`, and discarding a `PrivateChannel`, or
    /// `ChannelCategory`, if any.
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
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).await.unwrap();
    /// #
    /// match channel.guild() {
    ///     Some(guild) => {
    ///         println!("It's a guild named {}!", guild.name);
    ///     },
    ///     None => { println!("It's not a guild!"); },
    /// }
    /// # }
    /// ```
    pub fn guild(self) -> Option<GuildChannel> {
        match self {
            Channel::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<PrivateChannel>`.
    ///
    /// Converts `self` into an `Option<PrivateChannel>`, consuming
    /// `self`, and discarding a `GuildChannel`, or `ChannelCategory`,
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
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).await.unwrap();
    /// #
    /// match channel.private() {
    ///     Some(private) => {
    ///         println!("It's a private channel with {}!", &private.recipient);
    ///     },
    ///     None => { println!("It's not a private channel!"); },
    /// }
    /// # }
    /// ```
    pub fn private(self) -> Option<PrivateChannel> {
        match self {
            Channel::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<ChannelCategory>`.
    ///
    /// Converts `self` into an `Option<ChannelCategory>`,
    /// consuming `self`, and discarding a `GuildChannel`, or
    /// `PrivateChannel`, if any.
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
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).await.unwrap();
    /// #
    /// match channel.category() {
    ///     Some(category) => {
    ///         println!("It's a category named {}!", category.name);
    ///     },
    ///     None => { println!("It's not a category!"); },
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
            Channel::Category(catagory) => Some(catagory.position),
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
            0 | 2 | 5 | 6 | 13 => from_value::<GuildChannel>(Value::from(v))
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

impl Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Channel::Guild(ch) => Display::fmt(&ch.id.mention(), f),
            Channel::Private(ch) => Display::fmt(&ch.recipient.name, f),
            Channel::Category(ch) => Display::fmt(&ch.name, f),
        }
    }
}

/// A representation of a type of channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
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
    /// An indicator that the channel is a stage [`GuildChannel`].
    Stage = 13,
}

enum_number!(ChannelType {
    Text,
    Private,
    Voice,
    Category,
    News,
    Store,
    Stage
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
            ChannelType::Stage => "stage",
        }
    }

    #[inline]
    pub fn num(self) -> u64 {
        match self {
            ChannelType::Text => 0,
            ChannelType::Private => 1,
            ChannelType::Voice => 2,
            ChannelType::Category => 4,
            ChannelType::News => 5,
            ChannelType::Store => 6,
            ChannelType::Stage => 13,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(serialize_with = "serialize_u64", deserialize_with = "deserialize_u64")]
    id: u64,
    #[serde(rename = "type")]
    kind: u8,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug)]
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
            PermissionOverwriteType::Role(id) => (id.0, 0),
            PermissionOverwriteType::Member(id) => (id.0, 1),
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
}

enum_number!(VideoQualityMode {
    Auto,
    Full
});

#[cfg(test)]
mod test {
    #[cfg(all(feature = "model", feature = "utils"))]
    mod model_utils {
        use crate::model::prelude::*;

        fn guild_channel() -> GuildChannel {
            GuildChannel {
                id: ChannelId(1),
                bitrate: None,
                category_id: None,
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
                slow_mode_rate: Some(0),
                rtc_region: None,
                video_quality_mode: None,
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
#[async_trait]
impl FromStrAndCache for Channel {
    type Err = ChannelParseError;

    async fn from_str<C>(cache: C, s: &str) -> StdResult<Self, Self::Err>
    where
        C: AsRef<Cache> + Send + Sync,
    {
        match parse_channel(s) {
            Some(x) => match ChannelId(x).to_channel_cached(&cache).await {
                Some(channel) => Ok(channel),
                _ => Err(ChannelParseError::NotPresentInCache),
            },
            _ => Err(ChannelParseError::InvalidChannel),
        }
    }
}
