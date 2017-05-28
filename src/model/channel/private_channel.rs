use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;
use super::deserialize_single_recipient;
use ::model::*;

#[cfg(feature="model")]
use ::builder::{CreateMessage, GetMessages};
#[cfg(feature="model")]
use ::http::AttachmentType;

/// A Direct Message text channel with another user.
#[derive(Clone, Debug, Deserialize)]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the last time a [`Message`] was pinned.
    ///
    /// [`Message`]: struct.Message.html
    pub last_pin_timestamp: Option<String>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    ///
    /// [`ChannelType::Private`]: enum.ChannelType.html#variant.Private
    #[serde(rename="type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(deserialize_with="deserialize_single_recipient", rename="recipients")]
    pub recipient: Arc<RwLock<User>>,
}

#[cfg(feature="model")]
impl PrivateChannel {
    /// Broadcasts that the current user is typing to the recipient.
    pub fn broadcast_typing(&self) -> Result<()> {
        self.id.broadcast_typing()
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
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        self.id.create_reaction(message_id, reaction_type)
    }

    /// Deletes the channel. This does not delete the contents of the channel,
    /// and is equivalent to closing a private channel on the client, which can
    /// be re-opened.
    #[inline]
    pub fn delete(&self) -> Result<Channel> {
        self.id.delete()
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// Refer to [`Channel::delete_messages`] for more information.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: This uses bulk delete endpoint which is not available
    /// for user accounts.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// [`Channel::delete_messages`]: enum.Channel.html#method.delete_messages
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        self.id.delete_messages(message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(permission_type)
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
        self.id.delete_reaction(message_id, user_id, reaction_type)
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
        self.id.edit_message(message_id, f)
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// **Note**: This method is for consistency. This will always return
    /// `false`, due to DMs not being considered NSFW.
    ///
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        false
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id.message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id.messages(f)
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::get_reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_reaction_users`]: enum.Channel.html#variant.get_reaction_users
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
        self.id.reaction_users(message_id, reaction_type, limit, after)
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id.pin(message_id)
    }

    /// Retrieves the list of messages that have been pinned in the private
    /// channel.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        self.id.pins()
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    #[inline]
    pub fn say(&self, content: &str) -> Result<Message> {
        self.id.say(content)
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
        self.id.send_file(file, filename, f)
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
        self.id.send_files(files, f)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F) -> Result<Message> {
        self.id.send_message(f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id.unpin(message_id)
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

impl Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.recipient.read().unwrap().name)
    }
}
