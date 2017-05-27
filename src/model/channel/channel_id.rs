use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use std::io::Read;
use ::model::*;

#[cfg(feature="model")]
use ::builder::{CreateMessage, EditChannel, GetMessages};
#[cfg(feature="cache")]
use ::CACHE;
#[cfg(feature="model")]
use ::http;

#[cfg(feature="model")]
impl ChannelId {
    /// Broadcasts that the current user is typing to a channel for the next 5
    /// seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting
    /// that the current user is typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for
    /// signifying that a long-running command is still being executed.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelId;
    ///
    /// let _successful = ChannelId(7).broadcast_typing();
    /// ```
    ///
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[inline]
    pub fn broadcast_typing(&self) -> Result<()> {
        http::broadcast_typing(self.0)
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// [`GuildChannel::create_permission`]: struct.GuildChannel.html#method.create_permission
    /// [`Member`]: struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverwrite.html
    /// [`Role`]: struct.Role.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    pub fn create_permission(&self, target: &PermissionOverwrite)
        -> Result<()> {
        let (id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };

        let map = json!({
            "allow": target.allow.bits(),
            "deny": target.deny.bits(),
            "id": id,
            "type": kind,
        });

        http::create_permission(self.0, id, &map)
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
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        http::create_reaction(self.0, message_id.into().0, &reaction_type.into())
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    #[inline]
    pub fn delete(&self) -> Result<Channel> {
        http::delete_channel(self.0)
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
        http::delete_message(self.0, message_id.into().0)
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
    ///
    /// Refer to the documentation for [`Channel::delete_messages`] for more
    /// information.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: This uses bulk delete endpoint which is not available
    /// for user accounts.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using this method.
    ///
    /// [`Channel::delete_messages`]: enum.Channel.html#method.delete_messages
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        let ids = message_ids.into_iter()
            .map(|message_id| message_id.0)
            .collect::<Vec<u64>>();

        let map = json!({
            "messages": ids
        });

        http::delete_messages(self.0, &map)
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        http::delete_permission(self.0, match permission_type {
            PermissionOverwriteType::Member(id) => id.0,
            PermissionOverwriteType::Role(id) => id.0,
        })
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reaction<M, R>(&self, message_id: M, user_id: Option<UserId>, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        http::delete_reaction(self.0,
                              message_id.into().0,
                              user_id.map(|uid| uid.0),
                              &reaction_type.into())
    }


    /// Edits the settings of a [`Channel`], optionally setting new values.
    ///
    /// Refer to `EditChannel`'s documentation for its methods.
    ///
    /// Requires the [Manage Channel] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channel's name and bitrate:
    ///
    /// ```rust,ignore
    /// // assuming a `channel_id` has been bound
    ///
    /// channel_id.edit(|c| c.name("test").bitrate(64000));
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn edit<F: FnOnce(EditChannel) -> EditChannel>(&self, f: F) -> Result<GuildChannel> {
        http::edit_channel(self.0, &f(EditChannel::default()).0)
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
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, M: Into<MessageId> {
        let map = f(CreateMessage::default()).0;

        if let Some(content) = map.get("content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        http::edit_message(self.0, message_id.into().0, &Value::Object(map))
    }

    /// Search the cache for the channel with the Id.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Channel> {
        CACHE.read().unwrap().channel(*self)
    }

    /// Search the cache for the channel. If it can't be found, the channel is
    /// requested over REST.
    pub fn get(&self) -> Result<Channel> {
        #[cfg(feature="cache")]
        {
            if let Some(channel) = CACHE.read().unwrap().channel(*self) {
                return Ok(channel);
            }
        }

        http::get_channel(self.0)
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn invites(&self) -> Result<Vec<RichInvite>> {
        http::get_channel_invites(self.0)
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        http::get_message(self.0, message_id.into().0)
            .map(|mut msg| {
                msg.transform_content();

                msg
            })
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        let mut map = f(GetMessages::default()).0;
        let mut query = format!("?limit={}", map.remove("limit").unwrap_or(50));

        if let Some(after) = map.remove("after") {
            write!(query, "&after={}", after)?;
        } else if let Some(around) = map.remove("around") {
            write!(query, "&around={}", around)?;
        } else if let Some(before) = map.remove("before") {
            write!(query, "&before={}", before)?;
        }

        http::get_messages(self.0, &query)
            .map(|msgs| msgs
                .into_iter()
                .map(|mut msg| {
                    msg.transform_content();

                    msg
                }).collect::<Vec<Message>>())
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        http::pin_message(self.0, message_id.into().0)
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        http::get_pins(self.0)
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
    pub fn reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http::get_reaction_users(self.0,
                                 message_id.into().0,
                                 &reaction_type.into(),
                                 limit,
                                 after.map(|u| u.into().0))
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
        self.send_message(|m| m.content(content))
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Message contents may be passed by using the [`CreateMessage::content`]
    /// method.
    ///
    /// An embed can _not_ be sent when sending a file. If you set one, it will
    /// be automatically removed.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Examples
    ///
    /// Send a file with the filename `my_file.jpg`:
    ///
    /// ```rust,no_run
    /// use serenity::model::ChannelId;
    /// use std::fs::File;
    ///
    /// let channel_id = ChannelId(7);
    /// let filename = "my_file.jpg";
    /// let file = File::open(filename).unwrap();
    ///
    /// let _ = channel_id.send_file(file, filename, |m| m.content("a file"));
    /// ```
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ModelError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// Returns an
    /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    /// if the file is too large to send.
    ///
    ///
    /// [`HttpError::InvalidRequest`]: ../http/enum.HttpError.html#variant.InvalidRequest
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`CreateMessage::content`]: ../builder/struct.CreateMessage.html#method.content
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_file<F, R>(&self, file: R, filename: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, R: Read {
        let mut map = f(CreateMessage::default()).0;

        if let Some(content) = map.get("content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        let _ = map.remove("embed");

        http::send_file(self.0, file, filename, map)
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// Requires the [Send Messages] permission.
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
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        let CreateMessage(map, reactions) = f(CreateMessage::default());

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        let message = http::send_message(self.0, &Value::Object(map))?;

        if let Some(reactions) = reactions {
            for reaction in reactions {
                self.create_reaction(message.id, reaction)?;
            }
        }

        Ok(message)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        http::unpin_message(self.0, message_id.into().0)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        http::get_channel_webhooks(self.0)
    }

    /// Alias of [`invites`].
    ///
    /// [`invites`]: #method.invites
    #[deprecated(since="0.1.5", note="Use `invites` instead.")]
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        self.invites()
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

    /// Alias of [`webhooks`].
    ///
    /// [`webhooks`]: #method.webhooks
    #[deprecated(since="0.1.5", note="Use `webhooks` instead.")]
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        self.webhooks()
    }
}

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.read().unwrap().channel_id,
            Channel::Guild(ch) => ch.read().unwrap().id,
            Channel::Private(ch) => ch.read().unwrap().id,
        }
    }
}

impl<'a> From<&'a Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: &Channel) -> ChannelId {
        match *channel {
            Channel::Group(ref group) => group.read().unwrap().channel_id,
            Channel::Guild(ref ch) => ch.read().unwrap().id,
            Channel::Private(ref ch) => ch.read().unwrap().id,
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl<'a> From<&'a PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}
impl<'a> From<&'a GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}
