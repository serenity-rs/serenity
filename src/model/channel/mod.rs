//! Models relating to channels and types within channels.

mod attachment;
mod channel_id;
mod embed;
mod group;
mod guild_channel;
mod message;
mod private_channel;
mod reaction;
mod channel_category;

#[cfg(feature = "http")]
use crate::http::CacheHttp;
pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::group::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::channel_category::*;

use crate::{internal::RwLockExt, model::prelude::*};
use serde::de::Error as DeError;
use serde::ser::{SerializeStruct, Serialize, Serializer};
use serde_json;
use super::utils::deserialize_u64;

#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::model::misc::ChannelParseError;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_channel;
#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;
#[cfg(feature = "cache")]
use std::sync::Arc;
#[cfg(feature = "cache")]
use parking_lot::RwLock;

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Arc<RwLock<Group>>),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(Arc<RwLock<GuildChannel>>),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: ../user/struct.User.html
    Private(Arc<RwLock<PrivateChannel>>),
    /// A category of [`GuildChannel`]s
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Category(Arc<RwLock<ChannelCategory>>),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Channel {
    /// Converts from `Channel` to `Option<Arc<RwLock<Group>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<Group>>>`, consuming `self`,
    /// and discarding a `GuildChannel`, `PrivateChannel`, or `ChannelCategory`,
    /// if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::id::ChannelId};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #     let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #     let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
    /// match channel.group() {
    ///     Some(group_lock) => {
    ///         if let Some(ref name) = group_lock.read().name {
    ///             println!("It's a group named {}!", name);
    ///         } else {
    ///              println!("It's an unnamed group!");
    ///         }
    ///     },
    ///     None => { println!("It's not a group!"); },
    /// }
    /// #
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "model", feature = "cache")))]
    /// fn main() {}
    /// ```
    pub fn group(self) -> Option<Arc<RwLock<Group>>> {
        match self {
            Channel::Group(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<GuildChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<GuildChannel>>>`, consuming
    /// `self`, and discarding a `Group`, `PrivateChannel`, or
    /// `ChannelCategory`, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::id::ChannelId};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
    /// match channel.guild() {
    ///     Some(guild_lock) => {
    ///         println!("It's a guild named {}!", guild_lock.read().name);
    ///     },
    ///     None => { println!("It's not a guild!"); },
    /// }
    /// #
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "model", feature = "cache")))]
    /// fn main() {}
    /// ```
    pub fn guild(self) -> Option<Arc<RwLock<GuildChannel>>> {
        match self {
            Channel::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<PrivateChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<PrivateChannel>>>`, consuming
    /// `self`, and discarding a `Group`, `GuildChannel`, or `ChannelCategory`,
    /// if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::id::ChannelId};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
    /// match channel.private() {
    ///     Some(private_lock) => {
    ///         let private = private_lock.read();
    ///         let recipient_lock = &private.recipient;
    ///         let recipient = recipient_lock.read();
    ///         println!("It's a private channel with {}!", recipient.name);
    ///     },
    ///     None => { println!("It's not a private channel!"); },
    /// }
    /// #
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "model", feature = "cache")))]
    /// fn main() {}
    /// ```
    pub fn private(self) -> Option<Arc<RwLock<PrivateChannel>>> {
        match self {
            Channel::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<ChannelCategory>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<ChannelCategory>>>`,
    /// consuming `self`, and discarding a `Group`, `GuildChannel`, or
    /// `PrivateChannel`, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "model", feature = "cache"))]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::id::ChannelId};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #   let channel = ChannelId(0).to_channel_cached(&cache).unwrap();
    /// #
    /// match channel.category() {
    ///     Some(category_lock) => {
    ///         println!("It's a category named {}!", category_lock.read().name);
    ///     },
    ///     None => { println!("It's not a category!"); },
    /// }
    /// #
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "model", feature = "cache")))]
    /// fn main() {}
    /// ```
    pub fn category(self) -> Option<Arc<RwLock<ChannelCategory>>> {
        match self {
            Channel::Category(lock) => Some(lock),
            _ => None,
        }
    }

    /// Deletes the inner channel.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon
    /// owning the required permissions the HTTP-request will be issued.
    ///
    /// **Note**: There is no real function as _deleting_ a [`Group`]. The
    /// closest functionality is leaving it.
    ///
    /// [`Group`]: struct.Group.html
    #[cfg(all(feature = "model", feature = "http"))]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        match *self {
            Channel::Group(ref group) => {
                let _ = group.read().leave(cache_http.http())?;
            },
            Channel::Guild(ref public_channel) => {
                let _ = public_channel.read().delete(cache_http)?;
            },
            Channel::Private(ref private_channel) => {
                let _ = private_channel.read().delete(cache_http.http())?;
            },
            Channel::Category(ref category) => {
                category.read().delete(cache_http)?;
            },
            Channel::__Nonexhaustive => unreachable!(),
        }

        Ok(())
    }

    /// Determines if the channel is NSFW.
    #[cfg(feature = "model")]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        match *self {
            Channel::Guild(ref channel) => channel.with(|c| c.is_nsfw()),
            Channel::Category(ref category) => category.with(|c| c.is_nsfw()),
            Channel::Group(_) | Channel::Private(_) => false,
            Channel::__Nonexhaustive => unreachable!(),
        }
    }

    /// Retrieves the Id of the inner [`Group`], [`GuildChannel`], or
    /// [`PrivateChannel`].
    ///
    /// [`Group`]: struct.Group.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    pub fn id(&self) -> ChannelId {
        match *self {
            Channel::Group(ref group) => group.with(|g| g.channel_id),
            Channel::Guild(ref ch) => ch.with(|c| c.id),
            Channel::Private(ref ch) => ch.with(|c| c.id),
            Channel::Category(ref category) => category.with(|c| c.id),
            Channel::__Nonexhaustive => unreachable!(),
        }
    }

    /// Retrieves the position of the inner [`GuildChannel`] or
    /// [`ChannelCategory`].
    ///
    /// If other channel types are used it will return None.
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`CatagoryChannel`]: struct.ChannelCategory.html
    pub fn position(&self) -> Option<i64> {
        match *self {
            Channel::Guild(ref channel) => Some(channel.with(|c| c.position)),
            Channel::Category(ref catagory) => Some(catagory.with(|c| c.position)),
            _ => None
        }
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let v = JsonMap::deserialize(deserializer)?;
        let kind = {
            let kind = v.get("type").ok_or_else(|| DeError::missing_field("type"))?;

            kind.as_u64().unwrap()
        };

        match kind {
            0 | 2 => serde_json::from_value::<GuildChannel>(Value::Object(v))
                .map(|x| Channel::Guild(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            1 => serde_json::from_value::<PrivateChannel>(Value::Object(v))
                .map(|x| Channel::Private(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            3 => serde_json::from_value::<Group>(Value::Object(v))
                .map(|x| Channel::Group(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            4 => serde_json::from_value::<ChannelCategory>(Value::Object(v))
                .map(|x| Channel::Category(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown channel type")),
        }
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        match *self {
            Channel::Category(ref c) => {
                ChannelCategory::serialize(&*c.read(), serializer)
            },
            Channel::Group(ref c) => {
                Group::serialize(&*c.read(), serializer)
            },
            Channel::Guild(ref c) => {
                GuildChannel::serialize(&*c.read(), serializer)
            },
            Channel::Private(ref c) => {
                PrivateChannel::serialize(&*c.read(), serializer)
            },
            Channel::__Nonexhaustive => unreachable!(),
        }
    }
}

#[cfg(feature = "model")]
impl Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`Group`]s: the generated name retrievable via [`Group::name`];
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    ///
    /// [`Group`]: struct.Group.html
    /// [`Group::name`]: struct.Group.html#method.name
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Channel::Group(ref group) => Display::fmt(&group.read().name(), f),
            Channel::Guild(ref ch) => Display::fmt(&ch.read().id.mention(), f),
            Channel::Private(ref ch) => {
                let channel = ch.read();
                let recipient = channel.recipient.read();

                Display::fmt(&recipient.name, f)
            },
            Channel::Category(ref category) => Display::fmt(&category.read().name, f),
            Channel::__Nonexhaustive => unreachable!(),
        }
    }
}

/// A representation of a type of channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ChannelType {
    /// An indicator that the channel is a text [`GuildChannel`].
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Text = 0,
    /// An indicator that the channel is a [`PrivateChannel`].
    ///
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    Private = 1,
    /// An indicator that the channel is a voice [`GuildChannel`].
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Voice = 2,
    /// An indicator that the channel is the channel of a [`Group`].
    ///
    /// [`Group`]: struct.Group.html
    Group = 3,
    /// An indicator that the channel is the channel of a [`ChannelCategory`].
    ///
    /// [`ChannelCategory`]: struct.ChannelCategory.html
    Category = 4,
    /// An indicator that the channel is a [`NewsChannel`].
    ///
    /// [`NewsChannel`]: struct.NewsChannel.html
    News = 5,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    ChannelType {
        Text,
        Private,
        Voice,
        Group,
        Category,
        News,
    }
);

impl ChannelType {
    pub fn name(&self) -> &str {
        match *self {
            ChannelType::Group => "group",
            ChannelType::Private => "private",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Category => "category",
            ChannelType::News => "news",
            ChannelType::__Nonexhaustive => unreachable!(),
        }
    }

    pub fn num(self) -> u64 {
        match self {
            ChannelType::Text => 0,
            ChannelType::Private => 1,
            ChannelType::Voice => 2,
            ChannelType::Group => 3,
            ChannelType::Category => 4,
            ChannelType::News => 5,
            ChannelType::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(serialize_with = "serialize_u64", deserialize_with = "deserialize_u64")] id: u64,
    #[serde(rename = "type")] kind: String,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D)
                                         -> StdResult<PermissionOverwrite, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let kind = match &data.kind[..] {
            "member" => PermissionOverwriteType::Member(UserId(data.id)),
            "role" => PermissionOverwriteType::Role(RoleId(data.id)),
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
        where S: Serializer {
        let (id, kind) = match self.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
            PermissionOverwriteType::__Nonexhaustive => unreachable!(),
        };

        let mut state = serializer.serialize_struct("PermissionOverwrite", 4)?;
        state.serialize_field("allow", &self.allow.bits())?;
        state.serialize_field("deny", &self.deny.bits())?;
        state.serialize_field("id", &id)?;
        state.serialize_field("type", kind)?;

        state.end()
    }
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as `GuildChannel::create_permission`.
///
/// [`GuildChannel::create_permission`]: struct.GuildChannel.html#method.create_permission
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
    #[doc(hidden)]
    __Nonexhaustive,
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "model", feature = "utils"))]
    mod model_utils {
        use crate::model::prelude::*;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        fn group() -> Group {
            Group {
                channel_id: ChannelId(1),
                icon: None,
                last_message_id: None,
                last_pin_timestamp: None,
                name: None,
                owner_id: UserId(2),
                recipients: HashMap::new(),
                _nonexhaustive: (),
            }
        }

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
                _nonexhaustive: (),
            }
        }

        fn private_channel() -> PrivateChannel {
            PrivateChannel {
                id: ChannelId(1),
                last_message_id: None,
                last_pin_timestamp: None,
                kind: ChannelType::Private,
                recipient: Arc::new(RwLock::new(User {
                    id: UserId(2),
                    avatar: None,
                    bot: false,
                    discriminator: 1,
                    name: "ab".to_string(),
                    _nonexhaustive: (),
                })),
                _nonexhaustive: (),
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

            let channel = Channel::Guild(Arc::new(RwLock::new(channel)));
            assert!(!channel.is_nsfw());

            let group = group();
            assert!(!group.is_nsfw());

            let private_channel = private_channel();
            assert!(!private_channel.is_nsfw());
        }
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStrAndCache for Channel {
    type Err = ChannelParseError;

    fn from_str(cache: impl AsRef<CacheRwLock>, s: &str) -> StdResult<Self, Self::Err> {
        match parse_channel(s) {
            Some(x) => match ChannelId(x).to_channel_cached(&cache) {
                Some(channel) => Ok(channel),
                _ => Err(ChannelParseError::NotPresentInCache),
            },
            _ => Err(ChannelParseError::InvalidChannel),
        }
    }
}
