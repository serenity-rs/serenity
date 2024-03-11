use std::borrow::Cow;
use std::fmt;
#[cfg(feature = "model")]
use std::sync::Arc;

#[cfg(feature = "model")]
use crate::builder::{CreateAttachment, CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "model")]
use crate::http::CacheHttp;
#[cfg(feature = "model")]
use crate::http::{Http, Typing};
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::single_recipient;

/// A Direct Message text channel with another user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the last time a [`Message`] was pinned.
    pub last_pin_timestamp: Option<Timestamp>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(with = "single_recipient", rename = "recipients")]
    pub recipient: User,
}

#[cfg(feature = "model")]
impl PrivateChannel {
    /// Broadcasts that the current user is typing to the recipient.
    ///
    /// See [ChannelId::broadcast_typing] for more details.
    #[allow(clippy::missing_errors_doc)]
    pub async fn broadcast_typing(&self, http: &Http) -> Result<()> {
        self.id.broadcast_typing(http).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most cases.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction cannot be added, or if a message with that Id does
    /// not exist.
    pub async fn create_reaction(
        &self,
        http: &Http,
        message_id: MessageId,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.create_reaction(http, message_id, reaction_type).await
    }

    /// Deletes the channel. This does not delete the contents of the channel, and is equivalent to
    /// closing a private channel on the client, which can be re-opened.
    #[allow(clippy::missing_errors_doc)]
    pub async fn delete(&self, http: &Http) -> Result<PrivateChannel> {
        self.id.delete(http).await?.private().ok_or(Error::Model(ModelError::InvalidChannelType))
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using this method.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::TooSmall`] or [`ModelError::TooLarge`] if an attempt was made to
    /// delete either 0 or more than 100 messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_messages(&self, http: &Http, message_ids: &[MessageId]) -> Result<()> {
        self.id.delete_messages(http, message_ids).await
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    #[allow(clippy::missing_errors_doc)]
    pub async fn delete_permission(
        &self,
        http: &Http,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        self.id.delete_permission(http, permission_type).await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: In private channels, the current user may only delete it's own reactions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction is not from the current user.
    pub async fn delete_reaction(
        &self,
        http: &Http,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.delete_reaction(http, message_id, user_id, reaction_type).await
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data, with some exceptions for embeds and
    /// attachments.
    ///
    /// **Note**: In most cases requires that the current user be the author of the message.
    ///
    /// Refer to the documentation for [`EditMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See [`EditMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    ///
    /// [`EditMessage::execute`]: ../../builder/struct.EditMessage.html#method.execute
    pub async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
        builder: EditMessage<'_>,
    ) -> Result<Message> {
        self.id.edit_message(cache_http, message_id, builder).await
    }

    /// Gets a message from the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with that Id does not exist in this channel.
    pub async fn message(
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
    ) -> Result<Message> {
        self.id.message(cache_http, message_id).await
    }

    /// Gets messages from the channel.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns an empty
    /// [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn messages(&self, http: &Http, builder: GetMessages) -> Result<Vec<Message>> {
        self.id.messages(http, builder).await
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different maximum number of
    /// users. The maximum that may be retrieve at a time is `100`, if a greater number is provided
    /// then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain user. This is
    /// useful for pagination.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with the given Id does not exist in the channel.
    pub async fn reaction_users(
        &self,
        http: &Http,
        message_id: MessageId,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        self.id.reaction_users(http, message_id, reaction_type, limit, after).await
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the number of pinned messages would exceed the 50 message limit.
    pub async fn pin(&self, http: &Http, message_id: MessageId) -> Result<()> {
        self.id.pin(http, message_id).await
    }

    /// Retrieves the list of messages that have been pinned in the private channel.
    #[allow(clippy::missing_errors_doc)]
    pub async fn pins(&self, http: &Http) -> Result<Vec<Message>> {
        self.id.pins(http).await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// **Note**: Message content must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the content length is over the above limit. See
    /// [`CreateMessage::execute`] for more details.
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
    pub async fn say(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<Cow<'_, str>>,
    ) -> Result<Message> {
        self.id.say(cache_http, content).await
    }

    /// Sends file(s) along with optional message contents.
    ///
    /// Refer to [`ChannelId::send_files`] for examples and more information.
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
    pub async fn send_files<'a>(
        self,
        cache_http: impl CacheHttp,
        files: impl IntoIterator<Item = CreateAttachment<'a>>,
        builder: CreateMessage<'a>,
    ) -> Result<Message> {
        self.id.send_files(cache_http, files, builder).await
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
    pub async fn send_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
        self.id.send_message(cache_http, builder).await
    }

    /// Starts typing in the channel for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called on
    /// the returned struct to stop typing. Note that on some clients, typing may persist for a few
    /// seconds after [`Typing::stop`] is called. Typing is also stopped when the struct is
    /// dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief
    /// period of time and then resume again until either [`Typing::stop`] is called or the struct
    /// is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a long-running
    /// command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() {
    /// # use serenity::{cache::Cache, http::Http, model::channel::PrivateChannel, Result};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel: PrivateChannel = unimplemented!();
    /// // Initiate typing (assuming http is `Arc<Http>` and `channel` is bound)
    /// let typing = channel.start_typing(&http);
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the current user cannot send a direct message to this user.
    pub fn start_typing(self, http: &Arc<Http>) -> Typing {
        http.start_typing(self.id)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, if the message was deleted,
    /// or if the channel already has the limit of 50 pinned messages.
    pub async fn unpin(&self, http: &Http, message_id: MessageId) -> Result<()> {
        self.id.unpin(http, message_id).await
    }
}

impl fmt::Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.recipient.name)
    }
}
