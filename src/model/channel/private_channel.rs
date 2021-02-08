use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "model")]
use std::sync::Arc;

use chrono::{DateTime, Utc};

#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "model")]
use crate::http::AttachmentType;
#[cfg(feature = "http")]
use crate::http::{Http, Typing};
use crate::model::prelude::*;

/// A Direct Message text channel with another user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the last time a [`Message`] was pinned.
    pub last_pin_timestamp: Option<DateTime<Utc>>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(
        deserialize_with = "deserialize_single_recipient",
        serialize_with = "serialize_single_recipient",
        rename = "recipients"
    )]
    pub recipient: User,
}

#[cfg(feature = "model")]
impl PrivateChannel {
    /// Broadcasts that the current user is typing to the recipient.
    ///
    /// See [ChannelId::broadcast_typing] for more details.
    ///
    /// [`ChannelId::broadcast_typing`]: crate::model::channel::ChannelId::broadcast_typing
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.broadcast_typing(&http).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction cannot be added,
    /// or if a message with that Id does not exist.
    #[inline]
    pub async fn create_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.create_reaction(&http, message_id, reaction_type).await
    }

    /// Deletes the channel. This does not delete the contents of the channel,
    /// and is equivalent to closing a private channel on the client, which can
    /// be re-opened.
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<Channel> {
        self.id.delete(&http).await
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
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
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item = T>>(
        &self,
        http: impl AsRef<Http>,
        message_ids: It,
    ) -> Result<()>
    where
        T: AsRef<MessageId>,
        It: IntoIterator<Item = T>,
    {
        self.id.delete_messages(&http, message_ids).await
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn delete_permission(
        &self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        self.id.delete_permission(&http, permission_type).await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: In private channels, the current user may only
    /// delete it's own reactions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction is not from
    /// the current user.
    #[inline]
    pub async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.delete_reaction(&http, message_id, user_id, reaction_type).await
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
    /// Returns [`Error::Http`] if the current user is not the owner of the message.
    ///
    /// [`EditMessage`]: crate::builder::EditMessage
    /// [`the limit`]: crate::builder::EditMessage::content
    #[inline]
    pub async fn edit_message<F>(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
    {
        self.id.edit_message(&http, message_id, f).await
    }

    /// Determines if the channel is NSFW.
    ///
    /// **Note**: This method is for consistency. This will always return
    /// `false`, due to DMs not being considered NSFW.
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        false
    }

    /// Gets a message from the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with that Id does not
    /// exist in this channel.
    #[inline]
    pub async fn message(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        self.id.message(&http, message_id).await
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`GetMessages`] for more information on how to use `builder`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if an invalid value is set in the builder.
    ///
    /// [`GetMessages`]: crate::builder::GetMessages
    #[inline]
    pub async fn messages<F>(&self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where
        F: FnOnce(&mut GetMessages) -> &mut GetMessages,
    {
        self.id.messages(&http, builder).await
    }

    /// Returns "DM with $username#discriminator".
    pub fn name(&self) -> String {
        format!("DM with {}", self.recipient.tag())
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with the given Id does not exist
    /// in the channel.
    #[inline]
    pub async fn reaction_users<M, R, U>(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        self.id.reaction_users(&http, message_id, reaction_type, limit, after).await
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the number of pinned messages
    /// would exceed the 50 message limit.
    #[inline]
    pub async fn pin(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        self.id.pin(&http, message_id).await
    }

    /// Retrieves the list of messages that have been pinned in the private
    /// channel.
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        self.id.pins(&http).await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    #[inline]
    pub async fn say(
        &self,
        http: impl AsRef<Http>,
        content: impl std::fmt::Display,
    ) -> Result<Message> {
        self.id.say(&http, content).await
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
    /// [`ModelError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[inline]
    pub async fn send_files<'a, F, T, It>(
        &self,
        http: impl AsRef<Http>,
        files: It,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
        T: Into<AttachmentType<'a>>,
        It: IntoIterator<Item = T>,
    {
        self.id.send_files(&http, files, f).await
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
    /// [`CreateMessage`]: crate::builder::CreateMessage
    #[inline]
    pub async fn send_message<'a, F>(&self, http: impl AsRef<Http>, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.id.send_message(&http, f).await
    }

    /// Starts typing in the channel for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called
    /// on the returned struct to stop typing. Note that on some clients, typing may persist
    /// for a few seconds after `stop` is called.
    /// Typing is also stopped when the struct is dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief period
    /// of time and then resume again until either `stop` is called or the struct is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a
    /// long-running command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::{
    /// #    cache::Cache,
    /// #    http::{Http, Typing},
    /// #    model::{ModelError, channel::PrivateChannel, id::ChannelId},
    /// #    Result,
    /// # };
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http = Arc::new(Http::default());
    /// # let cache = Cache::default();
    /// # let channel = cache.private_channel(ChannelId(7))
    /// #    .await
    /// #    .ok_or(ModelError::ItemMissing)?;
    /// // Initiate typing (assuming http is `Arc<Http>` and `channel` is bound)
    /// let typing = channel.start_typing(&http)?;
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the current user cannot send a direct message
    /// to this user.
    pub fn start_typing(self, http: &Arc<Http>) -> Result<Typing> {
        http.start_typing(self.id.0)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// if the message was deleted, or if the channel already has the limit of
    /// 50 pinned messages.
    #[inline]
    pub async fn unpin(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        self.id.unpin(&http, message_id).await
    }
}

impl Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.recipient.name)
    }
}
