use internal::RwLockExt;
use model::prelude::*;

#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;
#[cfg(feature = "model")]
use builder::{
    CreateMessage,
    EditChannel,
    EditMessage,
    GetMessages
};
#[cfg(all(feature = "cache", feature = "model"))]
use CACHE;
#[cfg(all(feature = "cache", feature = "model"))]
use Cache;
#[cfg(feature = "model")]
use http::{self, AttachmentType};
#[cfg(feature = "model")]
use utils;

#[cfg(feature = "model")]
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
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[inline]
    pub fn broadcast_typing(&self) -> Result<()> { http::broadcast_typing(self.0) }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// [`GuildChannel::create_permission`]: ../channel/struct.GuildChannel.html#method.create_permission
    /// [`Member`]: ../guild/struct.Member.html
    /// [`PermissionOverwrite`]: ../channel/struct.PermissionOverwrite.html
    /// [`Role`]: ../guild/struct.Role.html
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    pub fn create_permission(&self, target: &PermissionOverwrite) -> Result<()> {
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
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: ../channel/struct.Message.html
    /// [`Message::react`]: ../channel/struct.Message.html#method.react
    /// [Add Reactions]: ../permissions/struct.Permissions.html#associatedconstant.ADD_REACTIONS
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R) -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self._create_reaction(message_id.into(), &reaction_type.into())
    }

    fn _create_reaction(
        self,
        message_id: MessageId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        http::create_reaction(self.0, message_id.0, reaction_type)
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    #[inline]
    pub fn delete(&self) -> Result<Channel> { http::delete_channel(self.0) }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    /// [`Message::delete`]: ../channel/struct.Message.html#method.delete
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn delete_message<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self._delete_message(message_id.into())
    }

    fn _delete_message(self, message_id: MessageId) -> Result<()> {
        http::delete_message(self.0, message_id.0)
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
    ///
    /// Refer to the documentation for [`Channel::delete_messages`] for more
    /// information.
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
    /// [`Channel::delete_messages`]: ../channel/enum.Channel.html#method.delete_messages
    /// [`ModelError::BulkDeleteAmount`]: ../error/enum.Error.html#variant.BulkDeleteAmount
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    pub fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item=T>>(&self, message_ids: It) -> Result<()> {
        let ids = message_ids
            .into_iter()
            .map(|message_id| message_id.as_ref().0)
            .collect::<Vec<u64>>();

        self._delete_messages(&ids)
    }

    fn _delete_messages(self, ids: &[u64]) -> Result<()> {
        let len = ids.len();

        if len == 0 || len > 100 {
            Err(Error::Model(ModelError::BulkDeleteAmount))
        } else if ids.len() == 1 {
            self.delete_message(ids[0])
        } else {
            let map = json!({ "messages": ids });

            http::delete_messages(self.0, &map)
        }
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        http::delete_permission(
            self.0,
            match permission_type {
                PermissionOverwriteType::Member(id) => id.0,
                PermissionOverwriteType::Role(id) => id.0,
            },
        )
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: ../channel/struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn delete_reaction<M, R>(&self,
                                 message_id: M,
                                 user_id: Option<UserId>,
                                 reaction_type: R)
                                 -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self._delete_reaction(
            message_id.into(),
            user_id,
            &reaction_type.into(),
        )
    }

    fn _delete_reaction(
        self,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        http::delete_reaction(
            self.0,
            message_id.0,
            user_id.map(|uid| uid.0),
            reaction_type,
        )
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
    /// [`Channel`]: ../channel/enum.Channel.html
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "utils")]
    #[inline]
    pub fn edit<F: FnOnce(EditChannel) -> EditChannel>(&self, f: F) -> Result<GuildChannel> {
        let map = utils::vecmap_to_json_map(f(EditChannel::default()).0);

        http::edit_channel(self.0, &map)
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
    /// [`Message`]: ../channel/struct.Message.html
    /// [`the limit`]: ../../builder/struct.EditMessage.html#method.content
    #[cfg(feature = "utils")]
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(EditMessage) -> EditMessage, M: Into<MessageId> {
        self._edit_message(message_id.into(), f)
    }

    fn _edit_message<F>(self, message_id: MessageId, f: F) -> Result<Message>
        where F: FnOnce(EditMessage) -> EditMessage {
        let msg = f(EditMessage::default());

        if let Some(content) = msg.0.get(&"content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        let map = utils::vecmap_to_json_map(msg.0);

        http::edit_message(self.0, message_id.0, &Value::Object(map))
    }

    /// Search the cache for the channel with the Id.
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.5.8", note = "Use the `to_channel_cached`-method instead.")]
    pub fn find(&self) -> Option<Channel> { self.to_channel_cached() }

    /// Attempts to find a [`Channel`] by its Id in the cache.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    #[cfg(feature = "cache")]
    #[inline]
    pub fn to_channel_cached(self) -> Option<Channel> {
        self._to_channel_cached(&CACHE)
    }

    /// To allow testing pass their own cache instead of using the globale one.
    #[cfg(feature = "cache")]
    #[inline]
    pub(crate) fn _to_channel_cached(self, cache: &RwLock<Cache>) -> Option<Channel> {
        cache.read().channel(self)
    }


    /// Search the cache for the channel. If it can't be found, the channel is
    /// requested over REST.
    #[deprecated(since = "0.5.8", note = "Use the `to_channel`-method instead.")]
    pub fn get(&self) -> Result<Channel> {
        self.to_channel()
    }

    /// First attempts to find a [`Channel`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the cache is not enabled,
    /// REST API will be used only.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    #[inline]
    pub fn to_channel(self) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(channel) = CACHE.read().channel(self) {
                return Ok(channel);
            }
        }

        http::get_channel(self.0)
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub fn invites(&self) -> Result<Vec<RichInvite>> { http::get_channel_invites(self.0) }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self._message(message_id.into())
    }

    fn _message(self, message_id: MessageId) -> Result<Message> {
        http::get_message(self.0, message_id.0).map(|mut msg| {
            msg.transform_content();

            msg
        })
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::messages`]: ../channel/enum.Channel.html#method.messages
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        let mut map = f(GetMessages::default()).0;
        let mut query = format!("?limit={}", map.remove(&"limit").unwrap_or(50));

        if let Some(after) = map.remove(&"after") {
            write!(query, "&after={}", after)?;
        } else if let Some(around) = map.remove(&"around") {
            write!(query, "&around={}", around)?;
        } else if let Some(before) = map.remove(&"before") {
            write!(query, "&before={}", before)?;
        }

        http::get_messages(self.0, &query).map(|msgs| {
            msgs.into_iter()
                .map(|mut msg| {
                    msg.transform_content();

                    msg
                })
                .collect::<Vec<Message>>()
        })
    }

    /// Returns the name of whatever channel this id holds.
    #[cfg(feature = "model")]
    pub fn name(&self) -> Option<String> {
        use self::Channel::*;

        let finding = feature_cache! {{
            Some(self.to_channel_cached())
        } else {
            None
        }};

        let channel = if let Some(Some(c)) = finding {
            c
        } else {
            return None;
        };

        Some(match channel {
            Guild(channel) => channel.read().name().to_string(),
            Group(channel) => match channel.read().name() {
                Cow::Borrowed(name) => name.to_string(),
                Cow::Owned(name) => name,
            },
            Category(category) => category.read().name().to_string(),
            Private(channel) => channel.read().name(),
        })
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self._pin(message_id.into())
    }

    fn _pin(self, message_id: MessageId) -> Result<()> {
        http::pin_message(self.0, message_id.0)
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// [`Message`]: ../channel/struct.Message.html
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> { http::get_pins(self.0) }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::reaction_users`]: ../channel/enum.Channel.html#method.reaction_users
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: ../channel/struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    pub fn reaction_users<M, R, U>(&self,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>> where M: Into<MessageId>,
                                 R: Into<ReactionType>,
                                 U: Into<Option<UserId>> {
        self._reaction_users(
            message_id.into(),
            &reaction_type.into(),
            limit,
            after.into(),
        )
    }

    fn _reaction_users(
        self,
        message_id: MessageId,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http::get_reaction_users(
            self.0,
            message_id.0,
            reaction_type,
            limit,
            after.map(|x| x.0),
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
    /// [`ChannelId`]: struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    #[inline]
    pub fn say<D: ::std::fmt::Display>(&self, content: D) -> Result<Message> {
        self.send_message(|m| m.content(content))
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Message contents may be passed by using the [`CreateMessage::content`]
    /// method.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Examples
    ///
    /// Send files with the paths `/path/to/file.jpg` and `/path/to/file2.jpg`:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ChannelId;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let paths = vec!["/path/to/file.jpg", "path/to/file2.jpg"];
    ///
    /// let _ = channel_id.send_files(paths, |m| m.content("a file"));
    /// ```
    ///
    /// Send files using `File`:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ChannelId;
    /// use std::fs::File;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let f1 = File::open("my_file.jpg").unwrap();
    /// let f2 = File::open("my_file2.jpg").unwrap();
    ///
    /// let files = vec![(&f1, "my_file.jpg"), (&f2, "my_file2.jpg")];
    ///
    /// let _ = channel_id.send_files(files, |m| m.content("a file"));
    /// ```
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// Returns an
    /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    /// if the file is too large to send.
    ///
    /// [`ClientError::MessageTooLong`]: ../../client/enum.ClientError.html#variant.MessageTooLong
    /// [`HttpError::InvalidRequest`]: ../../http/enum.HttpError.html#variant.InvalidRequest
    /// [`CreateMessage::content`]: ../../builder/struct.CreateMessage.html#method.content
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [Attach Files]: ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "utils")]
    pub fn send_files<'a, F, T, It: IntoIterator<Item=T>>(&self, files: It, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, T: Into<AttachmentType<'a>> {
        let mut msg = f(CreateMessage::default());

        if let Some(content) = msg.0.get(&"content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        if let Some(e) = msg.0.remove(&"embed") {
            msg.0.insert("payload_json", json!({ "embed": e }));
        }

        let map = utils::vecmap_to_json_map(msg.0);
        http::send_files(self.0, files, map)
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
    /// [`Channel`]: ../channel/enum.Channel.html
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../../builder/struct.CreateMessage.html
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "utils")]
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        let msg = f(CreateMessage::default());
        let map = utils::vecmap_to_json_map(msg.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        let message = http::send_message(self.0, &Value::Object(map))?;

        if let Some(reactions) = msg.1 {
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
    /// [`Message`]: ../channel/struct.Message.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self._unpin(message_id.into())
    }

    fn _unpin(self, message_id: MessageId) -> Result<()> {
        http::unpin_message(self.0, message_id.0)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> { http::get_channel_webhooks(self.0) }
}

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.with(|g| g.channel_id),
            Channel::Guild(ch) => ch.with(|c| c.id),
            Channel::Private(ch) => ch.with(|c| c.id),
            Channel::Category(ch) => ch.with(|c| c.id),
        }
    }
}

impl<'a> From<&'a Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: &Channel) -> ChannelId {
        match *channel {
            Channel::Group(ref group) => group.with(|g| g.channel_id),
            Channel::Guild(ref ch) => ch.with(|c| c.id),
            Channel::Private(ref ch) => ch.with(|c| c.id),
            Channel::Category(ref ch) => ch.with(|c| c.id),
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId { private_channel.id }
}

impl<'a> From<&'a PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId { private_channel.id }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId { public_channel.id }
}
impl<'a> From<&'a GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId { public_channel.id }
}
