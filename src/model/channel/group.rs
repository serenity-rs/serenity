use chrono::{DateTime, FixedOffset};
use model::prelude::*;

#[cfg(feature = "model")]
use builder::{
    CreateMessage,
    EditMessage,
    GetMessages
};
#[cfg(feature = "model")]
use http::{self, AttachmentType};
#[cfg(feature = "model")]
use internal::RwLockExt;
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;

/// A group channel - potentially including other [`User`]s - separate from a
/// [`Guild`].
///
/// [`Guild`]: ../guild/struct.Guild.html
/// [`User`]: ../user/struct.User.html
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
    pub recipients: HashMap<UserId, Arc<RwLock<User>>>,
}

#[cfg(feature = "model")]
impl Group {
    /// Adds the given user to the group. If the user is already in the group,
    /// then nothing is done.
    ///
    /// Refer to [`http::add_group_recipient`] for more information.
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    ///
    /// [`http::add_group_recipient`]: ../../http/fn.add_group_recipient.html
    #[inline]
    pub fn add_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        self._add_recipient(user.into())
    }

    fn _add_recipient(&self, user: UserId) -> Result<()> {
        // If the group already contains the recipient, do nothing.
        if self.recipients.contains_key(&user) {
            return Ok(());
        }

        http::add_group_recipient(self.channel_id.0, user.0)
    }

    /// Broadcasts that the current user is typing in the group.
    #[inline]
    pub fn broadcast_typing(&self) -> Result<()> { self.channel_id.broadcast_typing() }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`Message::react`]: struct.Message.html#method.react
    /// [Add Reactions]: ../permissions/struct.Permissions.html#associatedconstant.ADD_REACTIONS
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R) -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self.channel_id.create_reaction(message_id, reaction_type)
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
    /// [`ModelError::BulkDeleteAmount`]: ../error/enum.Error.html#variant.BulkDeleteAmount
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item=T>>(&self, message_ids: It) -> Result<()> {
        self.channel_id.delete_messages(message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        self.channel_id.delete_permission(permission_type)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn delete_reaction<M, R>(&self,
                                 message_id: M,
                                 user_id: Option<UserId>,
                                 reaction_type: R)
                                 -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self.channel_id
            .delete_reaction(message_id, user_id, reaction_type)
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
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`EditMessage`]: ../../builder/struct.EditMessage.html
    /// [`Message`]: struct.Message.html
    /// [`the limit`]: ../../builder/struct.EditMessage.html#method.content
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(EditMessage) -> EditMessage, M: Into<MessageId> {
        self.channel_id.edit_message(message_id, f)
    }

    /// Returns the formatted URI of the group's icon if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            format!(cdn!("/channel-icons/{}/{}.webp"), self.channel_id, icon)
        })
    }

    /// Determines if the channel is NSFW.
    ///
    /// **Note**: This method is for consistency. This will always return
    /// `false`, due to groups not being considered NSFW.
    #[inline]
    pub fn is_nsfw(&self) -> bool { false }

    /// Leaves the group.
    #[inline]
    pub fn leave(&self) -> Result<Group> { http::leave_group(self.channel_id.0) }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.channel_id.message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[inline]
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.channel_id.messages(f)
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
                    Some(recipient) => recipient.with(|c| c.name.clone()),
                    None => return Cow::Borrowed("Empty Group"),
                };

                for recipient in self.recipients.values().skip(1) {
                    let _ = write!(name, ", {}", recipient.with(|r| r.name.clone()));
                }

                Cow::Owned(name)
            },
        }
    }

    /// Retrieves the list of messages that have been pinned in the group.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> { self.channel_id.pins() }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::reaction_users`]: enum.Channel.html#method.reaction_users
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[inline]
    pub fn reaction_users<M, R, U>(
        &self,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>> where M: Into<MessageId>,
                                 R: Into<ReactionType>,
                                 U: Into<Option<UserId>> {
        self.channel_id.reaction_users(message_id, reaction_type, limit, after)
    }

    /// Removes a recipient from the group. If the recipient is already not in
    /// the group, then nothing is done.
    ///
    /// **Note**: This is only available to the group owner.
    #[inline]
    pub fn remove_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        self._remove_recipient(user.into())
    }

    fn _remove_recipient(&self, user: UserId) -> Result<()> {
        // If the group does not contain the recipient already, do nothing.
        if !self.recipients.contains_key(&user) {
            return Ok(());
        }

        http::remove_group_recipient(self.channel_id.0, user.0)
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: ../id/struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    #[inline]
    pub fn say(&self, content: &str) -> Result<Message> { self.channel_id.say(content) }

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
    /// [`ChannelId::send_files`]: ../id/struct.ChannelId.html#method.send_files
    /// [`ClientError::MessageTooLong`]: ../../client/enum.ClientError.html#variant.MessageTooLong
    /// [Attach Files]: ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[inline]
    pub fn send_files<'a, F, T, It: IntoIterator<Item=T>>(&self, files: It, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, T: Into<AttachmentType<'a>> {
        self.channel_id.send_files(files, f)
    }

    /// Sends a message to the group with the given content.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// [`CreateMessage`]: ../../builder/struct.CreateMessage.html
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[inline]
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F) -> Result<Message> {
        self.channel_id.send_message(f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.channel_id.unpin(message_id)
    }
}
