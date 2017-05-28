mod attachment;
mod channel_id;
mod embed;
mod group;
mod guild_channel;
mod message;
mod private_channel;
mod reaction;

pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::group::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;

use serde::de::Error as DeError;
use serde_json;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;
use super::utils::deserialize_u64;
use ::model::*;

#[cfg(feature="model")]
use ::builder::{CreateMessage, GetMessages};
#[cfg(feature="model")]
use ::http::AttachmentType;

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Arc<RwLock<Group>>),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(Arc<RwLock<GuildChannel>>),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: struct.User.html
    Private(Arc<RwLock<PrivateChannel>>),
}

#[cfg(feature="model")]
impl Channel {
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
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        self.id().create_reaction(message_id, reaction_type)
    }

    /// Deletes the inner channel.
    ///
    /// **Note**: There is no real function as _deleting_ a [`Group`]. The
    /// closest functionality is leaving it.
    ///
    /// [`Group`]: struct.Group.html
    pub fn delete(&self) -> Result<()> {
        match *self {
            Channel::Group(ref group) => {
                let _ = group.read().unwrap().leave()?;
            },
            Channel::Guild(ref public_channel) => {
                let _ = public_channel.read().unwrap().delete()?;
            },
            Channel::Private(ref private_channel) => {
                let _ = private_channel.read().unwrap().delete()?;
            },
        }

        Ok(())
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
    #[inline]
    pub fn delete_message<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id().delete_message(message_id)
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: This uses bulk delete endpoint which is not available
    /// for user accounts.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        self.id().delete_messages(message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id().delete_permission(permission_type)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_reaction<M, R>(&self, message_id: M, user_id: Option<UserId>, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        self.id().delete_reaction(message_id, user_id, reaction_type)
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
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
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [`Message`]: struct.Message.html
    /// [`the limit`]: ../builder/struct.CreateMessage.html#method.content
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, M: Into<MessageId> {
        self.id().edit_message(message_id, f)
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[cfg(feature="utils")]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        match *self {
            Channel::Guild(ref channel) => channel.read().unwrap().is_nsfw(),
            Channel::Group(_) | Channel::Private(_) => false,
        }
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id().message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::MessageId;
    ///
    /// let id = MessageId(81392407232380928);
    ///
    /// // Maximum is 100.
    /// let _messages = channel.get_messages(|g| g.after(id).limit(100));
    /// ```
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id().messages(f)
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
    #[inline]
    pub fn reaction_users<M, R, U>(&self,
                                   message_id: M,
                                   reaction_type: R,
                                   limit: Option<u8>,
                                   after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id().reaction_users(message_id, reaction_type, limit, after)
    }

    /// Retrieves the Id of the inner [`Group`], [`GuildChannel`], or
    /// [`PrivateChannel`].
    ///
    /// [`Group`]: struct.Group.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    pub fn id(&self) -> ChannelId {
        match *self {
            Channel::Group(ref group) => group.read().unwrap().channel_id,
            Channel::Guild(ref channel) => channel.read().unwrap().id,
            Channel::Private(ref channel) => channel.read().unwrap().id,
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
    #[inline]
    pub fn say(&self, content: &str) -> Result<Message> {
        self.id().say(content)
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Refer to [`ChannelId::send_file`] for examples and more information.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    /// if the file is too large to send.
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ModelError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ChannelId::send_file`]: struct.ChannelId.html#method.send_file
    /// [`HttpError::InvalidRequest`]: ../http/enum.HttpError.html#variant.InvalidRequest
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[deprecated(since="0.2.0", note="Please use `send_files` instead.")]
    #[allow(deprecated)]
    pub fn send_file<F, R>(&self, file: R, filename: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, R: Read {
        self.id().send_file(file, filename, f)
    }

    /// Sends (a) file(s) along with optional message contents.
    ///
    /// Refer to [`ChannelId::send_file`] for examples and more information.
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
    /// [`ChannelId::send_file`]: struct.ChannelId.html#method.send_file
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[inline]
    pub fn send_files<F, T: Into<AttachmentType>>(&self, files: Vec<T>, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        self.id().send_files(files, f)
    }

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
    #[inline]
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        self.id().send_message(f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id().unpin(message_id)
    }

    /// Alias of [`message`].
    ///
    /// [`message`]: #method.message
    #[deprecated(since="0.1.5", note="Use `message` instead.")]
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.message(message_id)
    }

    /// Alias of [`messages`].
    ///
    /// [`messages`]: #method.messages
    #[deprecated(since="0.1.5", note="Use `messages` instead.")]
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.messages(f)
    }

    /// Alias of [`reaction_users`].
    ///
    /// [`reaction_users`]: #method.reaction_users
    #[deprecated(since="0.1.5", note="Use `reaction_users` instead.")]
    #[inline]
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.reaction_users(message_id, reaction_type, limit, after)
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
            _ => Err(DeError::custom("Unknown channel type")),
        }
    }
}

#[cfg(feature="model")]
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
            Channel::Group(ref group) => {
                Display::fmt(&group.read().unwrap().name(), f)
            },
            Channel::Guild(ref ch) => {
                Display::fmt(&ch.read().unwrap().id.mention(), f)
            },
            Channel::Private(ref ch) => {
                let channel = ch.read().unwrap();
                let recipient = channel.recipient.read().unwrap();

                Display::fmt(&recipient.name, f)
            },
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
    }
);

impl ChannelType {
    pub fn name(&self) -> &str {
        match *self {
            ChannelType::Group => "group",
            ChannelType::Private => "private",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
        }
    }
}

#[derive(Deserialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(deserialize_with="deserialize_u64")]
    id: u64,
    #[serde(rename="type")]
    kind: String,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<PermissionOverwrite, D::Error> {
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
