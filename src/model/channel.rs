use hyper::Client as HyperClient;
use serde_json::builder::ObjectBuilder;
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::io::Read;
use std::mem;
use super::utils::{
    decode_id,
    into_map,
    into_string,
    opt,
    remove,
};
use super::*;
use ::client::rest;
use ::constants;
use ::internal::prelude::*;
use ::utils::builder::{
    CreateEmbed,
    CreateInvite,
    CreateMessage,
    EditChannel,
    GetMessages,
    Search
};
use ::utils::decode_array;

#[cfg(feature="cache")]
use super::utils;
#[cfg(feature="cache")]
use ::client::CACHE;
#[cfg(feature="cache")]
use ::ext::cache::ChannelRef;

impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height
    /// in pixels is returned.
    pub fn dimensions(&self) -> Option<(u64, u64)> {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            Some((width, height))
        } else {
            None
        }
    }

    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    /// use std::fs::File;
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// let token = env::var("DISCORD_TOKEN").expect("token in environment");
    /// let mut client = Client::login_bot(&token);
    ///
    /// client.on_message(|context, message| {
    ///     for attachment in message.attachments {
    ///         let content = match attachment.download() {
    ///             Ok(content) => content,
    ///             Err(why) => {
    ///                 println!("Error downloading attachment: {:?}", why);
    ///                 let _ = context.say("Error downloading attachment");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         let mut file = match File::create(&attachment.filename) {
    ///             Ok(file) => file,
    ///             Err(why) => {
    ///                 println!("Error creating file: {:?}", why);
    ///                 let _ = context.say("Error creating file");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         if let Err(why) = file.write(&content) {
    ///             println!("Error writing to file: {:?}", why);
    ///
    ///             return;
    ///         }
    ///
    ///         let _ = context.say(&format!("Saved {:?}", attachment.filename));
    ///     }
    /// });
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected!", ready.user.name);
    /// });
    ///
    /// let _ = client.start();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents
    /// of the HTTP response.
    ///
    /// Returns an [`Error::Hyper`] when there is a problem retrieving the
    /// attachment.
    ///
    /// [`Error::Hyper`]: ../enum.Error.html#variant.Hyper
    /// [`Error::Io`]: ../enum.Error.html#variant.Io
    /// [`Message`]: struct.Message.html
    pub fn download(&self) -> Result<Vec<u8>> {
        let hyper = HyperClient::new();
        let mut response = hyper.get(&self.url).send()?;

        let mut bytes = vec![];
        response.read_to_end(&mut bytes)?;

        Ok(bytes)
    }
}

impl Channel {
    /// Marks the channel as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id().ack(message_id)
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
        self.id().create_reaction(message_id, reaction_type)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Channel> {
        let map = into_map(value)?;
        match req!(map.get("type").and_then(|x| x.as_u64())) {
            0 | 2 => GuildChannel::decode(Value::Object(map))
                .map(Channel::Guild),
            1 => PrivateChannel::decode(Value::Object(map))
                .map(Channel::Private),
            3 => Group::decode(Value::Object(map))
                .map(Channel::Group),
            other => Err(Error::Decode("Expected value Channel type",
                                       Value::U64(other))),
        }
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
                let _ = group.leave()?;
            },
            Channel::Guild(ref public_channel) => {
                let _ = public_channel.delete()?;
            },
            Channel::Private(ref private_channel) => {
                let _ = private_channel.delete()?;
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
    /// (in practice, please do not do this)
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

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id().get_message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelId;
    ///
    /// let messages = channel.get_messages(|g| g
    ///     .before(20)
    ///     .after(100)); // Maximum is 100.
    /// ```
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id().get_messages(f)
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
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id().get_reaction_users(message_id, reaction_type, limit, after)
    }

    /// Retrieves the Id of the inner [`Group`], [`GuildChannel`], or
    /// [`PrivateChannel`].
    ///
    /// [`Group`]: struct.Group.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    pub fn id(&self) -> ChannelId {
        match *self {
            Channel::Group(ref group) => group.channel_id,
            Channel::Guild(ref channel) => channel.id,
            Channel::Private(ref channel) => channel.id,
        }
    }

    /// Performs a search request to the API for the inner channel's
    /// [`Message`]s.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    pub fn search<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id().search(f)
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
}

impl fmt::Display for Channel {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Channel::Group(ref group) => group.name().to_owned(),
            Channel::Guild(ref channel) => Cow::Owned(format!("{}", channel)),
            Channel::Private(ref channel) => Cow::Owned(channel.recipient.name.clone()),
        };

        fmt::Display::fmt(&out, f)
    }
}

impl ChannelId {
    /// Marks a [`Channel`] as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    #[inline]
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        rest::ack_message(self.0, message_id.into().0)
    }

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
        rest::broadcast_typing(self.0)
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
    /// [`PermissionOverwrite`]: struct.PermissionOverWrite.html
    /// [`Role`]: struct.Role.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    pub fn create_permission(&self, target: PermissionOverwrite)
        -> Result<()> {
        let (id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };

        let map = ObjectBuilder::new()
            .insert("allow", target.allow.bits())
            .insert("deny", target.deny.bits())
            .insert("id", id)
            .insert("type", kind)
            .build();

        rest::create_permission(self.0, id, map)
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
        rest::create_reaction(self.0, message_id.into().0, reaction_type.into())
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    #[inline]
    pub fn delete(&self) -> Result<Channel> {
        rest::delete_channel(self.0)
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// (in practice, please do not do this)
    ///
    /// [`Message`]: struct.Message.html
    /// [`Message::delete`]: struct.Message.html#method.delete
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_message<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        rest::delete_message(self.0, message_id.into().0)
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

        let map = ObjectBuilder::new().insert("messages", ids).build();

        rest::delete_messages(self.0, map)
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        rest::delete_permission(self.0, match permission_type {
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
        rest::delete_reaction(self.0,
                              message_id.into().0,
                              user_id.map(|uid| uid.0),
                              reaction_type.into())
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
    /// context.edit_channel(channel_id, |c| c
    ///     .name("test")
    ///     .bitrate(64000));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ClientError::NoChannelId`]: ../client/enum.ClientError.html#variant.NoChannelId
    #[inline]
    pub fn edit<F: FnOnce(EditChannel) -> EditChannel>(&self, f: F) -> Result<GuildChannel> {
        rest::edit_channel(self.0, f(EditChannel::default()).0.build())
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Pass an empty string (`""`) to `text` if you are editing a message with
    /// an embed or file but no content. Otherwise, `text` must be given.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: ../client/enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: struct.Message.html
    pub fn edit_message<F, M>(&self, message_id: M, text: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateEmbed) -> CreateEmbed, M: Into<MessageId> {
        let mut map = ObjectBuilder::new().insert("content", text);

        let embed = f(CreateEmbed::default()).0;

        if embed.len() > 1 {
            map = map.insert("embed", Value::Object(embed));
        }

        rest::edit_message(self.0, message_id.into().0, map.build())
    }

    /// Search the cache for the channel with the Id.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Channel> {
        CACHE.read().unwrap().get_channel(*self).map(|x| x.clone_inner())
    }

    /// Search the cache for the channel. If it can't be found, the channel is
    /// requested over REST.
    pub fn get(&self) -> Result<Channel> {
        #[cfg(feature="cache")]
        {
            if let Some(channel) = CACHE.read().unwrap().get_channel(*self) {
                return Ok(channel.clone_inner());
            }
        }

        rest::get_channel(self.0)
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        rest::get_channel_invites(self.0)
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        rest::get_message(self.0, message_id.into().0)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
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

        rest::get_messages(self.0, &query)
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
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        rest::get_reaction_users(self.0,
                                 message_id.into().0,
                                 reaction_type.into(),
                                 limit,
                                 after.map(|u| u.into().0))
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_channel_webhooks(self.0)
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        rest::pin_message(self.0, message_id.into().0)
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        rest::get_pins(self.0)
    }

    /// Searches the channel's messages by providing query parameters via the
    /// search builder.
    ///
    /// Refer to the documentation for the [`Search`] builder for restrictions
    /// and defaults parameters, as well as potentially advanced usage.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Examples
    ///
    /// Refer to the [`Search`] builder's documentation for examples,
    /// specifically the section on [searching a channel][search channel].
    ///
    /// [`Search`]: ../utils/builder/struct.Search.html
    #[inline]
    pub fn search<F: FnOnce(Search) -> Search>(&self, f: F) -> Result<SearchResult> {
        rest::search_channel_messages(self.0, f(Search::default()).0)
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
    /// Requires the [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [`CreateMessage::content`]: ../utils/builder/struct.CreateMessage.html#method.content
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_file<F, R>(&self, file: R, filename: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, R: Read {
        let mut map = f(CreateMessage::default()).0;

        if let Some(content) = map.get("content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Client(ClientError::MessageTooLong(length_over)));
                }
            }
        }

        let _ = map.remove("embed");

        rest::send_file(self.0, file, filename, map)
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// Requires the [Send Messages] permission is required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../utils/builder/struct.CreateMessage.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        let map = f(CreateMessage::default()).0;

        if let Some(content) = map.get(&"content".to_owned()) {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Client(ClientError::MessageTooLong(length_over)));
                }
            }
        }

        rest::send_message(self.0, Value::Object(map))
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        rest::unpin_message(self.0, message_id.into().0)
    }
}

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.channel_id,
            Channel::Guild(channel) => channel.id,
            Channel::Private(channel) => channel.id,
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Embed {
    /// Creates a fake Embed, giving back a `serde_json` map.
    ///
    /// This should only be useful in conjunction with [`Webhook::execute`].
    ///
    /// [`Webhook::execute`]: struct.Webhook.html
    #[inline]
    pub fn fake<F>(f: F) -> Value where F: FnOnce(CreateEmbed) -> CreateEmbed {
        Value::Object(f(CreateEmbed::default()).0)
    }
}

impl Group {
    /// Marks the group as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.channel_id.ack(message_id)
    }

    /// Adds the given user to the group. If the user is already in the group,
    /// then nothing is done.
    ///
    /// Refer to [`rest::add_group_recipient`] for more information.
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    ///
    /// [`rest::add_group_recipient`]: ../client/rest/fn.add_group_recipient.html
    pub fn add_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        let user = user.into();

        // If the group already contains the recipient, do nothing.
        if self.recipients.contains_key(&user) {
            return Ok(());
        }

        rest::add_group_recipient(self.channel_id.0, user.0)
    }

    /// Broadcasts that the current user is typing in the group.
    #[inline]
    pub fn broadcast_typing(&self) -> Result<()> {
        self.channel_id.broadcast_typing()
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
        self.channel_id.create_reaction(message_id, reaction_type)
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
        self.channel_id.delete_messages(message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
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
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_reaction<M, R>(&self, message_id: M, user_id: Option<UserId>, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        self.channel_id.delete_reaction(message_id, user_id, reaction_type)
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.channel_id.get_message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.channel_id.get_messages(f)
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
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.channel_id.get_reaction_users(message_id, reaction_type, limit, after)
    }

    /// Returns the formatted URI of the group's icon if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/channel-icons/{}/{}.webp"), self.channel_id, icon))
    }

    /// Leaves the group.
    #[inline]
    pub fn leave(&self) -> Result<Group> {
        rest::leave_group(self.channel_id.0)
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
                    Some(recipient) => recipient.name.clone(),
                    None => return Cow::Borrowed("Empty Group"),
                };

                for recipient in self.recipients.values().skip(1) {
                    let _ = write!(name, ", {}", recipient.name);
                }

                Cow::Owned(name)
            }
        }
    }

    /// Retrieves the list of messages that have been pinned in the group.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        self.channel_id.pins()
    }

    /// Removes a recipient from the group. If the recipient is already not in
    /// the group, then nothing is done.
    ///
    /// **Note**: This is only available to the group owner.
    pub fn remove_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        let user = user.into();

        // If the group does not contain the recipient already, do nothing.
        if !self.recipients.contains_key(&user) {
            return Ok(());
        }

        rest::remove_group_recipient(self.channel_id.0, user.0)
    }

    /// Performs a search request to the API for the group's channel's
    /// [`Message`]s.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    #[inline]
    pub fn search<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        self.channel_id.search(f)
    }

    /// Sends a message to the group with the given content.
    ///
    /// Note that an @everyone mention will not be applied.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[inline]
    pub fn send_message(&self, content: &str) -> Result<Message> {
        self.channel_id.send_message(|m| m.content(content))
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.channel_id.unpin(message_id)
    }
}

impl Message {
    /// Marks the [`Channel`] as being read up to the message.
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<M: Into<MessageId>>(&self) -> Result<()> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.channel_id.ack(self.id)
    }

    /// Deletes the message.
    ///
    /// **Note**: The logged in user must either be the author of the message or
    /// have the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete(&self) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_MESSAGES;
            let is_author = self.author.id == CACHE.read().unwrap().user.id;
            let has_perms = utils::user_has_perms(self.channel_id, req)?;

            if !is_author && !has_perms {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.channel_id.delete_message(self.id)
    }

    /// Deletes all of the [`Reaction`]s associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reactions(&self) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::delete_message_reactions(self.channel_id.0, self.id.0)
    }

    /// Edits this message, replacing the original content with new content.
    ///
    /// If editing a message and not using an embed, just return the embed
    /// builder directly, via:
    ///
    /// ```rust,ignore
    /// message.edit("new content", |f| f);
    /// ```
    ///
    /// **Note**: You must be the author of the message to be able to do this.
    ///
    /// **Note**: Messages must be at most 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidUser`] if the
    /// current user is not the author.
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    pub fn edit<F>(&mut self, new_content: &str, embed: F) -> Result<()>
        where F: FnOnce(CreateEmbed) -> CreateEmbed {
        if let Some(length_over) = Message::overflow_length(new_content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        #[cfg(feature="cache")]
        {
            if self.author.id != CACHE.read().unwrap().user.id {
                return Err(Error::Client(ClientError::InvalidUser));
            }
        }

        let mut map = ObjectBuilder::new().insert("content", new_content);

        let embed = embed(CreateEmbed::default()).0;

        if embed.len() > 1 {
            map = map.insert("embed", Value::Object(embed));
        }

        match rest::edit_message(self.channel_id.0, self.id.0, map.build()) {
            Ok(edited) => {
                mem::replace(self, edited);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Returns message content, but with user and role mentions replaced with
    /// names and everyone/here mentions cancelled.
    #[cfg(feature="cache")]
    pub fn content_safe(&self) -> String {
        let mut result = self.content.clone();

        // First replace all user mentions.
        for u in &self.mentions {
            result = result.replace(&u.mention(), &u.distinct());
        }

        // Then replace all role mentions.
        for id in &self.mention_roles {
            let mention = id.mention();

            if let Some(role) = id.find() {
                result = result.replace(&mention, &format!("@{}", role.name));
            } else {
                result = result.replace(&mention, "@deleted-role");
            }
        }

        // And finally replace everyone and here mentions.
        result.replace("@everyone", "@\u{200B}everyone")
              .replace("@here", "@\u{200B}here")
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
    pub fn get_reaction_users<R, U>(&self, reaction_type: R, limit: Option<u8>, after: Option<U>)
        -> Result<Vec<User>> where R: Into<ReactionType>, U: Into<UserId> {
        self.id.get_reaction_users(self.channel_id, reaction_type, limit, after)
    }

    /// Retrieves the Id of the guild that the message was sent in, if sent in
    /// one.
    ///
    /// Returns `None` if the channel data or guild data does not exist in the
    /// cache.
    #[cfg(feature="cache")]
    pub fn guild_id(&self) -> Option<GuildId> {
        match CACHE.read().unwrap().get_channel(self.channel_id) {
            Some(ChannelRef::Guild(channel)) => Some(channel.guild_id),
            _ => None,
        }
    }

    /// True if message was sent using direct messages.
    #[cfg(feature="cache")]
    pub fn is_private(&self) -> bool {
        match CACHE.read().unwrap().get_channel(self.channel_id) {
            Some(ChannelRef::Group(_)) | Some(ChannelRef::Private(_)) => true,
            _ => false,
        }
    }

    /// Checks the length of a string to ensure that it is within Discord's
    /// maximum message length limit.
    ///
    /// Returns `None` if the message is within the limit, otherwise returns
    /// `Some` with an inner value of how many unicode code points the message
    /// is over.
    pub fn overflow_length(content: &str) -> Option<u64> {
        // Check if the content is over the maximum number of unicode code
        // points.
        let count = content.chars().count() as i64;
        let diff = count - (constants::MESSAGE_CODE_LIMIT as i64);

        if diff > 0 {
            Some(diff as u64)
        } else {
            None
        }
    }

    /// Pins this message to its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn pin(&self) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.channel_id.pin(self.id.0)
    }

    /// React to the message with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [Add Reactions]: permissions/constant.ADD_REACTIONS.html
    /// [permissions]: permissions
    pub fn react<R: Into<ReactionType>>(&self, reaction_type: R) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::ADD_REACTIONS;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::create_reaction(self.channel_id.0,
                              self.id.0,
                              reaction_type.into())
    }

    /// Replies to the user, mentioning them prior to the content in the form
    /// of: `@<USER_ID>: YOUR_CONTENT`.
    ///
    /// User mentions are generally around 20 or 21 characters long.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn reply(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        #[cfg(feature="cache")]
        {
            let req = permissions::SEND_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let mut gen = self.author.mention();
        gen.push_str(": ");
        gen.push_str(content);

        let map = ObjectBuilder::new()
            .insert("content", gen)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        rest::send_message(self.channel_id.0, map)
    }

    /// Unpins the message from its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn unpin(&self) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::unpin_message(self.channel_id.0, self.id.0)
    }
}

impl MessageId {
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
    pub fn get_reaction_users<C, R, U>(&self,
                                       channel_id: C,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where C: Into<ChannelId>, R: Into<ReactionType>, U: Into<UserId> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        rest::get_reaction_users(channel_id.into().0,
                                 self.0,
                                 reaction_type.into(),
                                 limit,
                                 after.map(|u| u.into().0))
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: Message) -> MessageId {
        message.id
    }
}

impl PermissionOverwrite {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<PermissionOverwrite> {
        let mut map = into_map(value)?;
        let id = remove(&mut map, "id").and_then(decode_id)?;
        let kind = remove(&mut map, "type").and_then(into_string)?;
        let kind = match &*kind {
            "member" => PermissionOverwriteType::Member(UserId(id)),
            "role" => PermissionOverwriteType::Role(RoleId(id)),
            _ => return Err(Error::Decode("Expected valid PermissionOverwrite type", Value::String(kind))),
        };

        Ok(PermissionOverwrite {
            kind: kind,
            allow: remove(&mut map, "allow").and_then(Permissions::decode)?,
            deny: remove(&mut map, "deny").and_then(Permissions::decode)?,
        })
    }
}

impl PrivateChannel {
    /// Marks the channel as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.ack(message_id)
    }

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

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<PrivateChannel> {
        let mut map = into_map(value)?;
        let mut recipients = decode_array(remove(&mut map, "recipients")?,
                                  User::decode)?;

        Ok(PrivateChannel {
            id: remove(&mut map, "id").and_then(ChannelId::decode)?,
            kind: remove(&mut map, "type").and_then(ChannelType::decode)?,
            last_message_id: opt(&mut map, "last_message_id", MessageId::decode)?,
            last_pin_timestamp: opt(&mut map, "last_pin_timestamp", into_string)?,
            recipient: recipients.remove(0),
        })
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

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id.get_message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id.get_messages(f)
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
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id.get_reaction_users(message_id, reaction_type, limit, after)
    }

    /// Pins a [`Message`] to the channel.
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

    /// Performs a search request to the API for the channel's [`Message`]s.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    pub fn search<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search(f)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    pub fn send_message(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        self.id.send_message(|m| m.content(content))
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
}

impl fmt::Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.recipient.name)
    }
}

impl GuildChannel {
    /// Marks the channel as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot
    /// user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        rest::ack_message(self.id.0, message_id.into().0)
    }

    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have the
    /// required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Send Messages]: permissions/constants.SEND_MESSAGES.html
    pub fn broadcast_typing(&self) -> Result<()> {
        self.id.broadcast_typing()
    }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(|i| i
    ///     .max_uses(5));
    /// ```
    pub fn create_invite<F>(&self, f: F) -> Result<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        #[cfg(feature="cache")]
        {
            let req = permissions::CREATE_INVITE;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = f(CreateInvite::default()).0.build();

        rest::create_invite(self.id.0, map)
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within a [`Channel`].
    ///
    /// Refer to the documentation for [`PermissionOverwrite`]s for more
    /// information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Creating a permission overwrite for a member by specifying the
    /// [`PermissionOverwrite::Member`] variant, allowing it the [Send Messages]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// // assuming you are in a context
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let _result = context.create_permission(channel_id, overwrite);
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwrite::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// // assuming you are in a context
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let _result = context.create_permission(channel_id, overwrite);
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Member`]: struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverWrite.html
    /// [`PermissionOverwrite::Member`]: struct.PermissionOverwrite.html#variant.Member
    /// [`Role`]: struct.Role.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    /// [Send TTS Messages]: permissions/constant.SEND_TTS_MESSAGES.html
    #[inline]
    pub fn create_permission(&self, target: PermissionOverwrite) -> Result<()> {
        self.id.create_permission(target)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<GuildChannel> {
        let mut map = into_map(value)?;

        let id = remove(&mut map, "guild_id").and_then(GuildId::decode)?;

        GuildChannel::decode_guild(Value::Object(map), id)
    }

    #[doc(hidden)]
    pub fn decode_guild(value: Value, guild_id: GuildId) -> Result<GuildChannel> {
        let mut map = into_map(value)?;

        Ok(GuildChannel {
            id: remove(&mut map, "id").and_then(ChannelId::decode)?,
            name: remove(&mut map, "name").and_then(into_string)?,
            guild_id: guild_id,
            topic: opt(&mut map, "topic", into_string)?,
            position: req!(remove(&mut map, "position")?.as_i64()),
            kind: remove(&mut map, "type").and_then(ChannelType::decode)?,
            last_message_id: opt(&mut map, "last_message_id", MessageId::decode)?,
            permission_overwrites: decode_array(remove(&mut map, "permission_overwrites")?, PermissionOverwrite::decode)?,
            bitrate: remove(&mut map, "bitrate").ok().and_then(|v| v.as_u64()),
            user_limit: remove(&mut map, "user_limit").ok().and_then(|v| v.as_u64()),
            last_pin_timestamp: opt(&mut map, "last_pin_timestamp", into_string)?,
        })
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    pub fn delete(&self) -> Result<Channel> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

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

    /// Modifies a channel's settings, such as its position or name.
    ///
    /// Refer to `EditChannel`s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,ignore
    /// channel.edit(|c| c
    ///     .name("test")
    ///     .bitrate(86400));
    /// ```
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditChannel) -> EditChannel {

        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = ObjectBuilder::new()
            .insert("name", &self.name)
            .insert("position", self.position)
            .insert("type", self.kind.name());

        let edited = f(EditChannel(map)).0.build();

        match rest::edit_channel(self.id.0, edited) {
            Ok(channel) => {
                mem::replace(self, channel);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        self.id.get_invites()
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id.get_message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permission/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id.get_messages(f)
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
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id.get_reaction_users(message_id, reaction_type, limit, after)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        self.id.get_webhooks()
    }

    /// Attempts to find this channel's guild in the Cache.
    ///
    /// **Note**: Right now this performs a clone of the guild. This will be
    /// optimized in the future.
    #[cfg(feature="cache")]
    pub fn guild(&self) -> Option<Guild> {
        CACHE.read().unwrap().get_guild(self.guild_id).cloned()
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id.pin(message_id)
    }

    /// Gets all channel's pins.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        self.id.pins()
    }

    /// Performs a search request for the channel's [`Message`]s.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    pub fn search<F: FnOnce(Search) -> Search>(&self, f: F) -> Result<SearchResult> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search(f)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [`Message`]: struct.Message.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        #[cfg(feature="cache")]
        {
            let req = permissions::SEND_MESSAGES;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.send_message(|m| m.content(content))
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
}

impl fmt::Display for GuildChannel {
    /// Formas the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

impl Reaction {
    /// Deletes the reaction, but only if the current user is the user who made
    /// the reaction or has permission to.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    /// [permissions]: permissions
    pub fn delete(&self) -> Result<()> {
        let user_id = feature_cache! {{
            let user = if self.user_id == CACHE.read().unwrap().user.id {
                None
            } else {
                Some(self.user_id.0)
            };

            // If the reaction is one _not_ made by the current user, then ensure
            // that the current user has permission* to delete the reaction.
            //
            // Normally, users can only delete their own reactions.
            //
            // * The `Manage Messages` permission.
            if user.is_some() {
                let req = permissions::MANAGE_MESSAGES;

                if !utils::user_has_perms(self.channel_id, req).unwrap_or(true) {
                    return Err(Error::Client(ClientError::InvalidPermissions(req)));
                }
            }

            user
        } else {
            Some(self.user_id.0)
        }};

        rest::delete_reaction(self.channel_id.0,
                              self.message_id.0,
                              user_id,
                              self.emoji.clone())
    }

    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a
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
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    /// [permissions]: permissions
    pub fn users<R, U>(&self,
                       reaction_type: R,
                       limit: Option<u8>,
                       after: Option<U>)
                       -> Result<Vec<User>>
                       where R: Into<ReactionType>,
                             U: Into<UserId> {
        rest::get_reaction_users(self.channel_id.0,
                                 self.message_id.0,
                                 reaction_type.into(),
                                 limit.unwrap_or(50),
                                 after.map(|u| u.into().0))
    }
}

/// The type of a [`Reaction`] sent.
///
/// [`Reaction`]: struct.Reaction.html
#[derive(Clone, Debug)]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the
    /// guild.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Guild`]: struct.Guild.html
    Custom {
        /// The Id of the custom [`Emoji`].
        ///
        /// [`Emoji`]: struct.Emoji.html
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration
        /// and distinguishing the emoji client-side.
        name: String,
    },
    /// A reaction with a twemoji.
    Unicode(String),
}

impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom { id, ref name } => {
                format!("{}:{}", name, id)
            },
            ReactionType::Unicode(ref unicode) => unicode.clone(),
        }
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut map = into_map(value)?;
        let name = remove(&mut map, "name").and_then(into_string)?;

        // Only custom emoji reactions (`ReactionType::Custom`) have an Id.
        Ok(match opt(&mut map, "id", EmojiId::decode)? {
            Some(id) => ReactionType::Custom {
                id: id,
                name: name,
            },
            None => ReactionType::Unicode(name),
        })
    }
}

impl From<Emoji> for ReactionType {
    fn from(emoji: Emoji) -> ReactionType {
        ReactionType::Custom {
            id: emoji.id,
            name: emoji.name,
        }
    }
}

impl From<String> for ReactionType {
    fn from(unicode: String) -> ReactionType {
        ReactionType::Unicode(unicode)
    }
}

impl fmt::Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a
    /// way that clients can understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to
    /// the documentation for [emoji's formatter][`Emoji::fmt`] on how this is
    /// displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    ///
    /// [`Emoji::fmt`]: struct.Emoji.html#method.fmt
    /// [`ReactionType::Custom`]: enum.ReactionType.html#variant.Custom
    /// [`ReactionType::Unicode`]: enum.ReactionType.html#variant.Unicode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReactionType::Custom { id, ref name } => {
                f.write_char('<')?;
                f.write_char(':')?;
                f.write_str(name)?;
                f.write_char(':')?;
                fmt::Display::fmt(&id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
