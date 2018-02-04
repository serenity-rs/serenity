use chrono::{DateTime, FixedOffset};
use client::Client;
use futures::{Future, future};
use model::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use ::FutureResult;

// #[cfg(feature = "model")]
use builder::{CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;

/// A group channel - potentially including other [`User`]s - separate from a
/// [`Guild`].
///
/// [`Guild`]: struct.Guild.html
/// [`User`]: struct.User.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    /// The Id of the group channel.
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    /// The optional icon of the group channel.
    pub icon: Option<String>,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the latest pinned message.
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// The name of the group channel.
    pub name: Option<String>,
    /// The Id of the group owner.
    pub owner_id: UserId,
    /// A map of the group's recipients.
    #[serde(deserialize_with = "deserialize_users",
            serialize_with = "serialize_users")]
    pub recipients: HashMap<UserId, Rc<RefCell<User>>>,
    #[serde(skip)]
    pub(crate) client: Option<Rc<Client>>,
}

impl Group {
    /// Adds the given user to the group. If the user is already in the group,
    /// then nothing is done.
    ///
    /// Refer to [`http::add_group_recipient`] for more information.
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    ///
    /// [`http::add_group_recipient`]: ../http/fn.add_group_recipient.html
    pub fn add_recipient<U: Into<UserId>>(&self, user: U) -> FutureResult<()> {
        let user = user.into();

        // If the group already contains the recipient, do nothing.
        if self.recipients.contains_key(&user) {
            return Box::new(future::ok(()));
        }

        ftryopt!(self.client)
            .http
            .add_group_recipient(self.channel_id.0, user.0)
    }

    /// Broadcasts that the current user is typing in the group.
    #[inline]
    pub fn broadcast_typing(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.broadcast_typing(self.channel_id.0)
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
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R)
        -> FutureResult<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(self.client).http.create_reaction(
            self.channel_id.0,
            message_id.into().0,
            &reaction_type.into(),
        )
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
        ftryopt!(self.client).http.delete_messages(
            self.channel_id.0,
            message_ids,
        )
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

        ftryopt!(self.client).http.delete_permission(
            self.channel_id.0,
            overwrite_id,
        )
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_reaction<M, R>(
        &self,
        message_id: M,
        user_id: Option<UserId>,
        reaction_type: R,
    ) -> FutureResult<()> where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(self.client).http.delete_reaction(
            self.channel_id.0,
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
            self.channel_id.0,
            message_id.into().0,
            f,
        )
    }

    /// Returns the formatted URI of the group's icon if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            format!(cdn!("/channel-icons/{}/{}.webp"), self.channel_id, icon)
        })
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// **Note**: This method is for consistency. This will always return
    /// `false`, due to groups not being considered NSFW.
    ///
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[inline]
    pub fn is_nsfw(&self) -> bool { false }

    /// Leaves the group.
    #[inline]
    pub fn leave(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.leave_group(self.channel_id.0)
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<Message> {
        ftryopt!(self.client).http.get_message(self.channel_id.0, message_id.into().0)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn messages<'a, F: FnOnce(GetMessages) -> GetMessages>(&'a self, f: F)
        -> Box<Future<Item = Vec<Message>, Error = Error> + 'a> {
        ftryopt!(self.client).http.get_messages(self.channel_id.0, f)
    }

    /// Generates a name for the group.
    ///
    /// If there are no recipients in the group, the name will be "Empty Group".
    /// Otherwise, the name is generated in a Comma Separated Value list, such
    /// as "person 1, person 2, person 3".
    pub fn name(&self) -> Cow<str> {
        match self.name {
            Some(ref name) => Cow::Borrowed(name),
            None => {
                let mut name = match self.recipients.values().nth(0) {
                    Some(recipient) => recipient.borrow().name.clone(),
                    None => return Cow::Borrowed("Empty Group"),
                };

                for recipient in self.recipients.values().skip(1) {
                    let recipient = recipient.borrow();

                    let _ = write!(name, ", {}", recipient.name);
                }

                Cow::Owned(name)
            },
        }
    }

    /// Retrieves the list of messages that have been pinned in the group.
    #[inline]
    pub fn pins(&self) -> FutureResult<Vec<Message>> {
        ftryopt!(self.client).http.get_pins(self.channel_id.0)
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
            self.channel_id.0,
            message_id.into().0,
            &reaction_type.into(),
            limit,
            after.into().map(|x| x.0),
        )
    }

    /// Removes a recipient from the group. If the recipient is already not in
    /// the group, then nothing is done.
    ///
    /// **Note**: This is only available to the group owner.
    pub fn remove_recipient<U>(&self, user: U) -> FutureResult<()>
        where U: Into<UserId> {
        let user = user.into();

        // If the group does not contain the recipient already, do nothing.
        if !self.recipients.contains_key(&user) {
            return Box::new(future::ok(()));
        }

        ftryopt!(self.client).http.remove_group_recipient(
            self.channel_id.0,
            user.0,
        )
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
    pub fn say(&self, content: &str) -> FutureResult<Message> {
        ftryopt!(self.client).http.send_message(self.channel_id.0, |f| f
            .content(content))
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
    //     ftryopt!(self.client).http.send_files(self.channel_id.0, files, f)
    // }

    /// Sends a message to the group with the given content.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[inline]
    pub fn send_message<F>(&self, f: F) -> FutureResult<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        ftryopt!(self.client).http.send_message(self.channel_id.0, f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<()> {
        ftryopt!(self.client)
            .http
            .unpin_message(self.channel_id.0, message_id.into().0)
    }
}
