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

use futures::Future;
use model::prelude::*;
use serde::de::Error as DeError;
use serde::ser::{SerializeStruct, Serialize, Serializer};
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use super::utils::deserialize_u64;
use ::FutureResult;

#[cfg(feature = "model")]
use builder::{CreateMessage, EditMessage};
#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};

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

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`Message::react`]: struct.Message.html#method.react
    /// [Add Reactions]: permissions/constant.ADD_REACTIONS.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R)
        -> FutureResult<()> where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(ftry!(self.client())).http.create_reaction(
            self.id().0,
            message_id.into().0,
            &reaction_type.into(),
        )
    }

    /// Deletes the inner channel.
    ///
    /// **Note**: There is no real function as _deleting_ a [`Group`]. The
    /// closest functionality is leaving it.
    ///
    /// [`Group`]: struct.Group.html
    #[cfg(feature = "model")]
    pub fn delete(&self) -> FutureResult<()> {
        match *self {
            Channel::Group(ref group) => {
                Box::new(group.borrow().leave())
            },
            Channel::Guild(ref public_channel) => {
                Box::new(public_channel.borrow().delete().map(|_| ()))
            },
            Channel::Private(ref private_channel) => {
                Box::new(private_channel.borrow().delete().map(|_| ()))
            },
            Channel::Category(ref category) => {
                Box::new(category.borrow().delete())
            },
        }
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// [`Message`]: struct.Message.html
    /// [`Message::delete`]: struct.Message.html#method.delete
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn delete_message<M>(&self, message_id: M) -> FutureResult<()>
        where M: Into<MessageId> {
        ftryopt!(ftry!(self.client())).http.delete_message(
            self.id().0,
            message_id.into().0,
        )
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn delete_reaction<M, R>(
        &self,
        message_id: M,
        user_id: Option<UserId>,
        reaction_type: R,
    ) -> FutureResult<()> where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(ftry!(self.client())).http.delete_reaction(
            self.id().0,
            message_id.into().0,
            user_id.map(|x| x.0),
            &reaction_type.into(),
        )
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`EditMessage`]: ../builder/struct.EditMessage.html
    /// [`Message`]: struct.Message.html
    /// [`the limit`]: ../builder/struct.EditMessage.html#method.content
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F)
        -> FutureResult<Message>
        where F: FnOnce(EditMessage) -> EditMessage, M: Into<MessageId> {
        ftryopt!(ftry!(self.client())).http.edit_message(
            self.id().0,
            message_id.into().0,
            f,
        )
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[cfg(all(feature = "model", feature = "utils"))]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        match *self {
            Channel::Guild(ref channel) => channel.borrow().is_nsfw(),
            Channel::Category(ref category) => category.borrow().is_nsfw(),
            Channel::Group(_) | Channel::Private(_) => false,
        }
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn message<M>(&self, message_id: M) -> FutureResult<Message>
        where M: Into<MessageId> {
        ftryopt!(ftry!(self.client())).http.get_message(
            self.id().0,
            message_id.into().0,
        )
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn reaction_users<M, R, U>(&self,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> FutureResult<Vec<User>>
        where M: Into<MessageId>, R: Into<ReactionType>, U: Into<Option<UserId>> {
        ftryopt!(ftry!(self.client())).http.get_reaction_users(
            self.id().0,
            message_id.into().0,
            &reaction_type.into(),
            limit,
            after.into().map(|x| x.0),
        )
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

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn say(&self, content: &str) -> FutureResult<Message> {
        ftryopt!(ftry!(self.client())).http.send_message(self.id().0, |f|
            f.content(content))
    }

    /// Sends (a) file(s) along with optional message contents.
    ///
    /// Refer to [`ChannelId::send_files`] for examples and more information.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ChannelId::send_files`]: struct.ChannelId.html#method.send_files
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    // todo
    // #[cfg(feature = "model")]
    // #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    // #[inline]
    // pub fn send_files<'a, F, T, It>(&self, files: It, f: F)
    //     -> FutureResult<Message>
    //     where F: FnOnce(CreateMessage) -> CreateMessage,
    //           T: Into<AttachmentType<'a>>,
    //           It: IntoIterator<Item = T> {
    //     ftryopt!(ftry!(self.client())).http.send_files(self.id(), files, f)
    // }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// The [Send Messages] permission is required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn send_message<F>(&self, f: F) -> FutureResult<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        ftryopt!(ftry!(self.client())).http.send_message(self.id().0, f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<()> {
        let retrieve = ftry!(self.client());
        let obtained = ftryopt!(retrieve);

        obtained.http.unpin_message(self.id().0, message_id.into().0)
    }

    fn client(&self) -> Result<WrappedClient> {
        Ok(match *self {
            Channel::Category(ref c) => c.try_borrow()?.client.as_ref().map(Rc::clone),
            Channel::Group(ref c) => c.try_borrow()?.client.as_ref().map(Rc::clone),
            Channel::Guild(ref c) => c.try_borrow()?.client.as_ref().map(Rc::clone),
            Channel::Private(ref c) => c.try_borrow()?.client.as_ref().map(Rc::clone),
        })
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

enum_number!(
    /// A representation of a type of channel.
    ChannelType {
        #[doc="An indicator that the channel is a text [`GuildChannel`].

[`GuildChannel`]: struct.GuildChannel.html"]
        Text = 0,
        #[doc="An indicator that the channel is a [`PrivateChannel`].

[`PrivateChannel`]: struct.PrivateChannel.html"]
        Private = 1,
        #[doc="An indicator that the channel is a voice [`GuildChannel`].

[`GuildChannel`]: struct.GuildChannel.html"]
        Voice = 2,
        #[doc="An indicator that the channel is the channel of a [`Group`].

[`Group`]: struct.Group.html"]
        Group = 3,
        #[doc="An indicator that the channel is the channel of a [`ChannelCategory`].

[`ChannelCategory`]: struct.ChannelCategory.html"]
        Category = 4,
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
            kind: kind,
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
