#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;
#[cfg(feature = "model")]
use std::sync::Arc;

use futures::stream::Stream;

#[cfg(feature = "model")]
use crate::builder::{CreateInvite, CreateMessage, EditChannel, EditMessage, GetMessages};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectReaction,
    CollectReply,
    MessageCollectorBuilder,
    ReactionCollectorBuilder,
};
#[cfg(feature = "model")]
use crate::http::AttachmentType;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, Typing};
#[cfg(feature = "model")]
use crate::json::json;
use crate::model::prelude::*;
#[cfg(all(feature = "model", feature = "utils"))]
use crate::utils;

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
    /// ```rust,no_run
    /// use serenity::model::id::ChannelId;
    ///
    /// # async fn run() {
    /// # let http = serenity::http::Http::default();
    /// let _successful = ChannelId(7).broadcast_typing(&http).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks
    /// permission to send messages to this channel.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[inline]
    pub async fn broadcast_typing(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().broadcast_typing(self.0).await
    }

    /// Creates an invite leading to the given channel.
    ///
    /// **Note**: Requres the [Create Invite] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Create Invite]: Permissions::CREATE_INVITE
    #[cfg(feature = "utils")]
    pub async fn create_invite<F>(&self, http: impl AsRef<Http>, f: F) -> Result<RichInvite>
    where
        F: FnOnce(&mut CreateInvite) -> &mut CreateInvite,
    {
        let mut invite = CreateInvite::default();
        f(&mut invite);

        let map = utils::hashmap_to_json_map(invite.0);

        http.as_ref().create_invite(self.0, &map).await
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is set.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_permission(
        self,
        http: impl AsRef<Http>,
        target: &PermissionOverwrite,
    ) -> Result<()> {
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

        http.as_ref().create_permission(self.0, id, &map).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Add Reactions]: Permissions::ADD_REACTIONS
    #[inline]
    pub async fn create_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref().create_reaction(self.0, message_id.into().0, &reaction_type.into()).await
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn delete(self, http: impl AsRef<Http>) -> Result<Channel> {
        http.as_ref().delete_channel(self.0).await
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// delete the message.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_message(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        http.as_ref().delete_message(self.0, message_id.into().0).await
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
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
    /// Also will return [`Error::Http`] if the current user lacks permission
    /// to delete messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_messages<T, It>(self, http: impl AsRef<Http>, message_ids: It) -> Result<()>
    where
        T: AsRef<MessageId>,
        It: IntoIterator<Item = T>,
    {
        let ids =
            message_ids.into_iter().map(|message_id| message_id.as_ref().0).collect::<Vec<u64>>();

        let len = ids.len();

        if len == 0 || len > 100 {
            return Err(Error::Model(ModelError::BulkDeleteAmount));
        }

        if ids.len() == 1 {
            self.delete_message(http, ids[0]).await
        } else {
            let map = json!({ "messages": ids });

            http.as_ref().delete_messages(self.0, &map).await
        }
    }

    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    pub async fn delete_permission(
        self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        http.as_ref()
            .delete_permission(self.0, match permission_type {
                PermissionOverwriteType::Member(id) => id.0,
                PermissionOverwriteType::Role(id) => id.0,
            })
            .await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user did not perform the reaction,
    /// and lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_reaction(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref()
            .delete_reaction(
                self.0,
                message_id.into().0,
                user_id.map(|uid| uid.0),
                &reaction_type.into(),
            )
            .await
    }

    /// Deletes all [`Reaction`]s of the given emoji to a message within the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn delete_reaction_emoji(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.as_ref()
            .delete_message_reaction_emoji(self.0, message_id.into().0, &reaction_type.into())
            .await
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
    /// ```rust,no_run
    /// // assuming a `channel_id` has been bound
    ///
    /// # async fn run() {
    /// #     use serenity::http::Http;
    /// #     use serenity::model::id::ChannelId;
    /// #     let http = Http::default();
    /// #     let channel_id = ChannelId(1234);
    /// channel_id.edit(&http, |c| c.name("test").bitrate(64000)).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is set.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    #[cfg(feature = "utils")]
    #[inline]
    pub async fn edit<F>(self, http: impl AsRef<Http>, f: F) -> Result<GuildChannel>
    where
        F: FnOnce(&mut EditChannel) -> &mut EditChannel,
    {
        let mut channel = EditChannel::default();
        f(&mut channel);

        let map = utils::hashmap_to_json_map(channel.0);

        http.as_ref().edit_channel(self.0, &map).await
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
    /// [`EditMessage`]: crate::builder::EditMessage
    /// [`the limit`]: crate::builder::EditMessage::content
    #[cfg(feature = "utils")]
    #[inline]
    pub async fn edit_message<F>(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
    {
        let mut msg = EditMessage::default();
        f(&mut msg);

        if let Some(content) = msg.0.get(&"content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        let map = utils::hashmap_to_json_map(msg.0);

        http.as_ref().edit_message(self.0, message_id.into().0, &Value::from(map)).await
    }

    /// Attempts to find a [`Channel`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn to_channel_cached(self, cache: impl AsRef<Cache>) -> Option<Channel> {
        cache.as_ref().channel(self).await
    }

    /// First attempts to find a [`Channel`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon
    /// owning the required permissions the HTTP-request will be issued.
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn to_channel(self, cache_http: impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.channel(self).await {
                    return Ok(channel);
                }
            }
        }

        cache_http.http().get_channel(self.0).await
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        http.as_ref().get_channel_invites(self.0).await
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn message(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        http.as_ref().get_message(self.0, message_id.into().0).await.map(|mut msg| {
            msg.transform_content();

            msg
        })
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`GetMessages`] for more information on how to use `builder`.
    ///
    /// **Note**: Returns an empty `Vec` if the current user
    /// does not have the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have
    /// permission to view the channel.
    ///
    /// [`GetMessages`]: crate::builder::GetMessages
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn messages<F>(self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where
        F: FnOnce(&mut GetMessages) -> &mut GetMessages,
    {
        let mut get_messages = GetMessages::default();
        builder(&mut get_messages);
        let mut map = get_messages.0;
        let mut query = format!("?limit={}", map.remove(&"limit").unwrap_or(50));

        if let Some(after) = map.remove(&"after") {
            write!(query, "&after={}", after)?;
        } else if let Some(around) = map.remove(&"around") {
            write!(query, "&around={}", around)?;
        } else if let Some(before) = map.remove(&"before") {
            write!(query, "&before={}", before)?;
        }

        http.as_ref().get_messages(self.0, &query).await.map(|msgs| {
            msgs.into_iter()
                .map(|mut msg| {
                    msg.transform_content();

                    msg
                })
                .collect::<Vec<Message>>()
        })
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`messages`].
    /// A buffer of at most 100 messages is used to reduce the number of calls.
    /// necessary.
    ///
    /// The stream returns the newest message first, followed by older messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let channel_id = ChannelId::default();
    /// # let ctx = Http::default();
    /// use serenity::model::channel::MessagesIter;
    /// use serenity::futures::StreamExt;
    ///
    /// let mut messages = channel_id.messages_iter(&ctx).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!(
    ///             "{} said \"{}\".",
    ///             message.author.name,
    ///             message.content,
    ///         ),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// [`messages`]: Self::messages
    pub fn messages_iter<H: AsRef<Http>>(self, http: H) -> impl Stream<Item = Result<Message>> {
        MessagesIter::<H>::stream(http, self)
    }

    /// Returns the name of whatever channel this id holds.
    #[cfg(feature = "cache")]
    pub async fn name(self, cache: impl AsRef<Cache>) -> Option<String> {
        let channel = self.to_channel_cached(cache).await?;

        Some(match channel {
            Channel::Guild(channel) => channel.name().to_string(),
            Channel::Category(category) => category.name().to_string(),
            Channel::Private(channel) => channel.name(),
        })
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if the channel has too many pinned messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn pin(self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        http.as_ref().pin_message(self.0, message_id.into().0).await
    }

    /// Crossposts a [`Message`].
    ///
    /// Requires either to be the message author or to have manage [Manage Messages] permissions on this channel.
    ///
    /// **Note**: Only available on announcements channels.
    ///
    /// # Errors
    // Returns [`Error::Http`] if the current user lacks permission,
    // and if the user is not the author of the message.
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn crosspost(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        http.as_ref().crosspost_message(self.0, message_id.into().0).await
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// **Note**: Returns an empty `Vec` if the current user does not
    /// have the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn pins(self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        http.as_ref().get_pins(self.0).await
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to read messages in the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn reaction_users(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http.as_ref()
            .get_reaction_users(
                self.0,
                message_id.into().0,
                &reaction_type.into(),
                limit,
                after.into().map(|x| x.0),
            )
            .await
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
        self,
        http: impl AsRef<Http>,
        content: impl std::fmt::Display,
    ) -> Result<Message> {
        self.send_message(&http, |m| m.content(content)).await
    }

    /// Sends file(s) along with optional message contents. The filename _must_
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
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::id::ChannelId;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let paths = vec!["/path/to/file.jpg", "path/to/file2.jpg"];
    ///
    /// let _ = channel_id.send_files(&http, paths, |m| {
    ///     m.content("a file")
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// Send files using `File`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::id::ChannelId;
    /// use tokio::fs::File;
    ///
    /// let channel_id = ChannelId(7);
    ///
    /// let f1 = File::open("my_file.jpg").await?;
    /// let f2 = File::open("my_file2.jpg").await?;
    ///
    /// let files = vec![(&f1, "my_file.jpg"), (&f2, "my_file2.jpg")];
    ///
    /// let _ = channel_id.send_files(&http, files, |m| {
    ///     m.content("a file")
    /// })
    /// .await;
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ModelError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(ErrorResponse)`][`HttpError::UnsuccessfulRequest`]
    /// if the file(s) are too large to send.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    /// [`CreateMessage::content`]: crate::builder::CreateMessage::content
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[cfg(feature = "utils")]
    pub async fn send_files<'a, F, T, It>(
        self,
        http: impl AsRef<Http>,
        files: It,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
        T: Into<AttachmentType<'a>>,
        It: IntoIterator<Item = T>,
    {
        let mut create_message = CreateMessage::default();
        let msg = f(&mut create_message);

        let map = utils::hashmap_to_json_map(msg.0.clone());

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().send_files(self.0, files, map).await
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
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// send a message in this channel.
    ///
    /// [`CreateMessage`]: crate::builder::CreateMessage
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[cfg(feature = "utils")]
    pub async fn send_message<'a, F>(self, http: impl AsRef<Http>, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        let mut create_message = CreateMessage::default();
        let msg = f(&mut create_message);

        let map = utils::hashmap_to_json_map(msg.0.clone());

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        let message = if msg.2.is_empty() {
            http.as_ref().send_message(self.0, &Value::from(map)).await?
        } else {
            http.as_ref().send_files(self.0, msg.2.clone(), map).await?
        };

        if let Some(reactions) = msg.1.clone() {
            for reaction in reactions {
                self.create_reaction(&http, message.id, reaction).await?;
            }
        }

        Ok(message)
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
    /// # use serenity::{http::{Http, Typing}, Result, model::id::ChannelId};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # fn main() -> Result<()> {
    /// # let http = Arc::new(Http::default());
    /// // Initiate typing (assuming http is `Arc<Http>`)
    /// let typing = ChannelId(7).start_typing(&http)?;
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
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to send messages in this channel.
    pub fn start_typing(self, http: &Arc<Http>) -> Result<Typing> {
        http.start_typing(self.0)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    #[inline]
    pub async fn unpin(
        self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        http.as_ref().unpin_message(self.0, message_id.into().0).await
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    #[inline]
    pub async fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        http.as_ref().get_channel_webhooks(self.0).await
    }

    /// Returns a future that will await one message sent in this channel.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reply<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).channel_id(self.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages in this channel.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_replies<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).channel_id(self.0)
    }

    /// Await a single reaction in this guild.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reaction<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).channel_id(self.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent in this channel.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reactions<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).channel_id(self.0)
    }
}

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        channel.id()
    }
}

impl<'a> From<&'a Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: &Channel) -> ChannelId {
        channel.id()
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

/// A helper class returned by [`ChannelId::messages_iter`]
#[derive(Clone, Debug)]
#[cfg(feature = "model")]
pub struct MessagesIter<H: AsRef<Http>> {
    http: H,
    channel_id: ChannelId,
    buffer: Vec<Message>,
    before: Option<MessageId>,
    tried_fetch: bool,
}

#[cfg(feature = "model")]
impl<H: AsRef<Http>> MessagesIter<H> {
    fn new(http: H, channel_id: ChannelId) -> MessagesIter<H> {
        MessagesIter {
            http,
            channel_id,
            buffer: Vec::new(),
            before: None,
            tried_fetch: false,
        }
    }

    /// Fills the `self.buffer` cache with [`Message`]s.
    ///
    /// This drops any messages that were currently in the buffer. Ideally, it
    /// should only be called when `self.buffer` is empty. Additionally, this updates
    /// `self.before` so that the next call does not return duplicate items.
    ///
    /// If there are no more messages to be fetched, then this sets `self.before`
    /// as `None`, indicating that no more calls ought to be made.
    ///
    /// If this method is called with `self.before` as None, the last 100
    /// (or lower) messages sent in the channel are added in the buffer.
    ///
    /// The messages are sorted such that the newest message is the first
    /// element of the buffer and the newest message is the last.
    ///
    /// [`Message`]: crate::model::channel::Message
    async fn refresh(&mut self) -> Result<()> {
        // Number of messages to fetch.
        let grab_size = 100;

        // If `self.before` is not set yet, we can use `.messages` to fetch
        // the last message after very first fetch from last.
        self.buffer = self
            .channel_id
            .messages(&self.http, |b| {
                if let Some(before) = self.before {
                    b.before(before);
                }

                b.limit(grab_size)
            })
            .await?;

        self.buffer.reverse();

        self.before = self.buffer.first().map(|m| m.id);

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`messages`].
    /// A buffer of at most 100 messages is used to reduce the number of calls.
    /// necessary.
    ///
    /// The stream returns the newest message first, followed by older messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let channel_id = ChannelId::default();
    /// # let ctx = Http::default();
    /// use serenity::model::channel::MessagesIter;
    /// use serenity::futures::StreamExt;
    ///
    /// let mut messages = MessagesIter::<Http>::stream(&ctx, channel_id).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!(
    ///             "{} said \"{}\"",
    ///             message.author.name,
    ///             message.content,
    ///         ),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// [`messages`]: ChannelId::messages
    pub fn stream(
        http: impl AsRef<Http>,
        channel_id: ChannelId,
    ) -> impl Stream<Item = Result<Message>> {
        let init_state = MessagesIter::new(http, channel_id);

        futures::stream::unfold(init_state, |mut state| async {
            if state.buffer.is_empty() && state.before.is_some() || !state.tried_fetch {
                if let Err(error) = state.refresh().await {
                    return Some((Err(error), state));
                }
            }

            // the resultant stream goes from newest to oldest.
            state.buffer.pop().map(|entry| (Ok(entry), state))
        })
    }
}
