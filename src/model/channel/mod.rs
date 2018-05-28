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

pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::group::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::channel_category::*;

use model::prelude::*;
use serde::de::Error as DeError;
use serde::ser::{SerializeStruct, Serialize, Serializer};
use serde_json;
use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;
use super::utils::deserialize_u64;

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Rc<RefCell<Group>>),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(Rc<RefCell<GuildChannel>>),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: struct.User.html
    Private(Rc<RefCell<PrivateChannel>>),
    /// A category of [`GuildChannel`]s
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Category(Rc<RefCell<ChannelCategory>>),
}

impl Channel {
    /// Converts from `Channel` to `Option<Arc<RwLock<Group>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<Group>>>`, consuming `self`,
    /// and discarding a GuildChannel, PrivateChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
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
    /// # }
    /// ```
    pub fn group(self) -> Option<Rc<RefCell<Group>>> {
        match self {
            Channel::Group(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<GuildChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<GuildChannel>>>`, consuming `self`,
    /// and discarding a Group, PrivateChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// let channel = ChannelId(0).get().unwrap();
    /// match channel.guild() {
    ///     Some(guild_lock) => {
    ///         println!("It's a guild named {}!", guild_lock.read().name);
    ///     },
    ///     None => { println!("It's not a guild!"); },
    /// }
    /// # }
    /// ```
    pub fn guild(self) -> Option<Rc<RefCell<GuildChannel>>> {
        match self {
            Channel::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<PrivateChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<PrivateChannel>>>`, consuming `self`,
    /// and discarding a Group, GuildChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
    /// match channel.private() {
    ///     Some(private_lock) => {
    ///         let private = private_lock.read();
    ///         let recipient_lock = &private.recipient;
    ///         let recipient = recipient_lock.read();
    ///         println!("It's a private channel with {}!", recipient.name);
    ///     },
    ///     None => { println!("It's not a private channel!"); },
    /// }
    /// # }
    /// ```
    pub fn private(self) -> Option<Rc<RefCell<PrivateChannel>>> {
        match self {
            Channel::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<ChannelCategory>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<ChannelCategory>>>`, consuming `self`,
    /// and discarding a Group, GuildChannel, or PrivateChannel, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
    /// match channel.category() {
    ///     Some(category_lock) => {
    ///         println!("It's a category named {}!", category_lock.read().name);
    ///     },
    ///     None => { println!("It's not a category!"); },
    /// }
    /// # }
    /// ```
    pub fn category(self) -> Option<Rc<RefCell<ChannelCategory>>> {
        match self {
            Channel::Category(lock) => Some(lock),
            _ => None,
        }
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// [`utils::is_nsfw`]: ../../utils/fn.is_nsfw.html
    #[cfg(all(feature = "model", feature = "utils"))]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        match *self {
            Channel::Guild(ref channel) => channel.borrow().is_nsfw(),
            Channel::Category(ref category) => category.borrow().is_nsfw(),
            Channel::Group(_) | Channel::Private(_) => false,
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
            Channel::Group(ref group) => group.borrow().channel_id,
            Channel::Guild(ref ch) => ch.borrow().id,
            Channel::Private(ref ch) => ch.borrow().id,
            Channel::Category(ref category) => category.borrow().id,
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
                .map(|x| Channel::Guild(Rc::new(RefCell::new(x))))
                .map_err(DeError::custom),
            1 => serde_json::from_value::<PrivateChannel>(Value::Object(v))
                .map(|x| Channel::Private(Rc::new(RefCell::new(x))))
                .map_err(DeError::custom),
            3 => serde_json::from_value::<Group>(Value::Object(v))
                .map(|x| Channel::Group(Rc::new(RefCell::new(x))))
                .map_err(DeError::custom),
            4 => serde_json::from_value::<ChannelCategory>(Value::Object(v))
                .map(|x| Channel::Category(Rc::new(RefCell::new(x))))
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
                ChannelCategory::serialize(&*c.borrow(), serializer)
            },
            Channel::Group(ref c) => {
                Group::serialize(&*c.borrow(), serializer)
            },
            Channel::Guild(ref c) => {
                GuildChannel::serialize(&*c.borrow(), serializer)
            },
            Channel::Private(ref c) => {
                PrivateChannel::serialize(&*c.borrow(), serializer)
            },
        }
    }
}

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
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Channel::Group(ref group) => Display::fmt(&group.borrow().name(), f),
            Channel::Guild(ref ch) => Display::fmt(&ch.borrow().id.mention(), f),
            Channel::Private(ref ch) => {
                let channel = ch.borrow();
                let recipient = channel.recipient.borrow();

                Display::fmt(&recipient.name, f)
            },
            Channel::Category(ref category) => Display::fmt(&category.borrow().name, f),
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
}

enum_number!(
    ChannelType {
        Text,
        Private,
        Voice,
        Group,
        Category,
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
        }
    }

    pub fn num(&self) -> u64 {
        match *self {
            ChannelType::Text => 0,
            ChannelType::Private => 1,
            ChannelType::Voice => 2,
            ChannelType::Group => 3,
            ChannelType::Category => 4,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(deserialize_with = "deserialize_u64")] id: u64,
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
}
