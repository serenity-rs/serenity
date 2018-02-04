use chrono::{DateTime, FixedOffset};
use futures::Future;
use model::prelude::*;
use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;
use super::super::WrappedClient;
use super::deserialize_single_recipient;
use ::FutureResult;

#[cfg(feature = "model")]
use builder::{CreateMessage, EditMessage, GetMessages};

/// A Direct Message text channel with another user.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    ///
    /// [`ChannelType::Private`]: enum.ChannelType.html#variant.Private
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(deserialize_with = "deserialize_single_recipient",
            rename = "recipients",
            serialize_with = "serialize_user")]
    pub recipient: Rc<RefCell<User>>,
    #[serde(skip)]
    pub(crate) client: WrappedClient,
}

#[cfg(feature = "model")]
impl PrivateChannel {
    /// Broadcasts that the current user is typing to the recipient.
    pub fn broadcast_typing(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.broadcast_typing(self.id.0)
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
        -> FutureResult<()> where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(self.client).http.create_reaction(
            self.id.0,
            message_id.into().0,
            &reaction_type.into(),
        )
    }

    /// Deletes the channel. This does not delete the contents of the channel,
    /// and is equivalent to closing a private channel on the client, which can
    /// be re-opened.
    #[inline]
    pub fn delete(&self) -> FutureResult<Channel> {
        ftryopt!(self.client).http.delete_channel(self.id.0)
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// Refer to [`Channel::delete_messages`] for more information.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BulkDeleteAmount`] if an attempt was made to
    /// delete either 0 or more than 100 messages.
    ///
    /// [`Channel::delete_messages`]: enum.Channel.html#method.delete_messages
    /// [`ModelError::BulkDeleteAmount`]: ../enum.ModelError.html#variant.BulkDeleteAmount
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item=T>>(
        &self,
        message_ids: It,
    ) -> FutureResult<()> {
        ftryopt!(self.client).http.delete_messages(self.id.0, message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType)
        -> FutureResult<()> {
        let overwrite_id = match permission_type {
            PermissionOverwriteType::Member(id) => id.0,
            PermissionOverwriteType::Role(id) => id.0,
        };

        ftryopt!(self.client).http.delete_permission(self.id.0, overwrite_id)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_reaction<M: Into<MessageId>, R: Into<ReactionType>>(
        &self,
        message_id: M,
        user_id: Option<UserId>,
        reaction_type: R,
    ) -> FutureResult<()> {
        ftryopt!(self.client).http.delete_reaction(
            self.id.0,
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
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F)
        -> FutureResult<Message>
        where F: FnOnce(EditMessage) -> EditMessage, M: Into<MessageId> {
        ftryopt!(self.client).http.edit_message(
            self.id.0,
            message_id.into().0,
            f,
        )
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
    pub fn is_nsfw(&self) -> bool { false }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M)
        -> FutureResult<Message> {
        ftryopt!(self.client).http.get_message(self.id.0, message_id.into().0)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::messages`]: enum.Channel.html#method.messages
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn messages<'a, F: FnOnce(GetMessages) -> GetMessages>(&'a self, f: F)
        -> Box<Future<Item = Vec<Message>, Error = Error> + 'a> {
        ftryopt!(self.client).http.get_messages(self.id.0, f)
    }

    /// Returns "DM with $username#discriminator".
    pub fn name(&self) -> String {
        format!("DM with {}", self.recipient.borrow().tag())
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::reaction_users`]: enum.Channel.html#method.reaction_users
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn reaction_users<M, R, U>(
        &self,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> FutureResult<Vec<User>>
        where M: Into<MessageId>, R: Into<ReactionType>, U: Into<Option<UserId>> {
        ftryopt!(self.client).http.get_reaction_users(
            self.id.0,
            message_id.into().0,
            &reaction_type.into(),
            limit,
            after.into().map(|x| x.0),
        )
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<()> {
        ftryopt!(self.client).http.pin_message(self.id.0, message_id.into().0)
    }

    /// Retrieves the list of messages that have been pinned in the private
    /// channel.
    #[inline]
    pub fn pins(&self) -> FutureResult<Vec<Message>> {
        ftryopt!(self.client).http.get_pins(self.id.0)
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: ../model/id/struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    #[inline]
    pub fn say<D: ::std::fmt::Display>(&self, content: D)
        -> FutureResult<Message> {
        ftryopt!(self.client)
            .http
            .send_message(self.id.0, |m| m.content(content))
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
    // #[inline]
    // pub fn send_files<'a, F, T, It>(&self, files: It, f: F)
    //     -> FutureResult<Message>
    //     where F: FnOnce(CreateMessage) -> CreateMessage,
    //           T: Into<AttachmentType<'a>>,
    //           It: IntoIterator<Item=T> {
    //     ftryopt!(self.client).http.send_files(self.id.0, files, f)
    // }

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
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F)
        -> FutureResult<Message> {
        ftryopt!(self.client).http.send_message(self.id.0, f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<()> {
        ftryopt!(self.client).http.unpin_message(self.id.0, message_id.into().0)
    }
}

impl Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.recipient.borrow().name)
    }
}
