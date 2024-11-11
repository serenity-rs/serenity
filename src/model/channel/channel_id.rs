#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::sync::Arc;

#[cfg(feature = "model")]
use futures::stream::Stream;

#[cfg(feature = "model")]
use crate::builder::{
    CreateAttachment,
    CreateForumPost,
    CreateInvite,
    CreateMessage,
    CreateStageInstance,
    CreateThread,
    CreateWebhook,
    EditChannel,
    EditMessage,
    EditStageInstance,
    EditThread,
    GetMessages,
};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, Typing};
use crate::model::prelude::*;

#[cfg(feature = "model")]
impl ChannelId {
    /// Broadcasts that the current user is typing to a channel for the next 5 seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting that the current
    /// user is typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for signifying that a
    /// long-running command is still being executed.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serenity::model::id::ChannelId;
    ///
    /// # async fn run() {
    /// # let http: serenity::http::Http = unimplemented!();
    /// let _successful = ChannelId::new(7).broadcast_typing(&http).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to send messages to this
    /// channel.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    pub async fn broadcast_typing(self, http: &Http) -> Result<()> {
        http.broadcast_typing(self).await
    }

    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    pub async fn create_invite(self, http: &Http, builder: CreateInvite<'_>) -> Result<RichInvite> {
        builder.execute(http, self).await
    }

    /// Creates a [permission overwrite] for either a single [`Member`] or [`Role`] within the
    /// channel.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an invalid value is
    /// set.
    ///
    /// [permission overwrite]: PermissionOverwrite
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_permission(
        self,
        http: &Http,
        target: PermissionOverwrite,
        reason: Option<&str>,
    ) -> Result<()> {
        let data: PermissionOverwriteData = target.into();
        http.create_permission(self, data.id, &data, reason).await
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the first user to perform
    /// a react with a certain emoji.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Add Reactions]: Permissions::ADD_REACTIONS
    pub async fn create_reaction(
        self,
        http: &Http,
        message_id: MessageId,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.create_reaction(self, message_id, &reaction_type.into()).await
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
    pub async fn delete(self, http: &Http, reason: Option<&str>) -> Result<Channel> {
        http.delete_channel(self, reason).await
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not the author of the
    /// message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete the message.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_message(
        self,
        http: &Http,
        message_id: MessageId,
        reason: Option<&str>,
    ) -> Result<()> {
        http.delete_message(self, message_id, reason).await
    }

    /// Deletes messages by Ids from the given vector in the given channel.
    ///
    /// The Discord API supports deleting between 2 and 100 messages at once. This function
    /// also handles the case of a single message by calling `delete_message` internally.
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
    /// Also will return [`Error::Http`] if the current user lacks permission to delete messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_messages(
        self,
        http: &Http,
        message_ids: &[MessageId],
        reason: Option<&str>,
    ) -> Result<()> {
        use crate::model::error::{Maximum, Minimum};

        #[derive(serde::Serialize)]
        struct DeleteMessages<'a> {
            messages: &'a [MessageId],
        }

        Minimum::BulkDeleteAmount.check_underflow(message_ids.len())?;
        Maximum::BulkDeleteAmount.check_overflow(message_ids.len())?;

        if message_ids.len() == 1 {
            self.delete_message(http, message_ids[0], reason).await
        } else {
            let req = DeleteMessages {
                messages: message_ids,
            };

            http.delete_messages(self, &req, reason).await
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
        http: &Http,
        permission_type: PermissionOverwriteType,
        reason: Option<&str>,
    ) -> Result<()> {
        let id = match permission_type {
            PermissionOverwriteType::Member(id) => id.into(),
            PermissionOverwriteType::Role(id) => id.get().into(),
        };
        http.delete_permission(self, id, reason).await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current user did not perform
    /// the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user did not perform the reaction, and lacks
    /// permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_reaction(
        self,
        http: &Http,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        let reaction_type = reaction_type.into();
        match user_id {
            Some(user_id) => http.delete_reaction(self, message_id, user_id, &reaction_type).await,
            None => http.delete_reaction_me(self, message_id, &reaction_type).await,
        }
    }
    /// Deletes all of the [`Reaction`]s associated with the provided message id.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_reactions(self, http: &Http, message_id: MessageId) -> Result<()> {
        http.delete_message_reactions(self, message_id).await
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
    pub async fn delete_reaction_emoji(
        self,
        http: &Http,
        message_id: MessageId,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.delete_message_reaction_emoji(self, message_id, &reaction_type.into()).await
    }

    /// Edits a channel's settings.
    ///
    /// Refer to the documentation for [`EditChannel`] for a full list of methods.
    ///
    /// **Note**: Requires the [Manage Channels] permission. Modifying permissions via
    /// [`EditChannel::permissions`] also requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channel's name and bitrate:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditChannel;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::ChannelId;
    /// # async fn run() {
    /// # let http: Http = unimplemented!();
    /// # let channel_id = ChannelId::new(1234);
    /// let builder = EditChannel::new().name("test").bitrate(64000);
    /// channel_id.edit(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn edit(self, http: &Http, builder: EditChannel<'_>) -> Result<GuildChannel> {
        builder.execute(http, self).await
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
    pub async fn edit_message(
        self,
        http: &Http,
        message_id: MessageId,
        builder: EditMessage<'_>,
    ) -> Result<Message> {
        builder.execute(http, self, message_id, None).await
    }

    /// Follows the News Channel
    ///
    /// Requires [Manage Webhook] permissions on the target channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission. [Manage Webhook]:
    /// Permissions::MANAGE_WEBHOOKS
    pub async fn follow(
        self,
        http: &Http,
        target_channel_id: ChannelId,
    ) -> Result<FollowedChannel> {
        #[derive(serde::Serialize)]
        struct FollowChannel {
            webhook_channel_id: ChannelId,
        }

        let map = FollowChannel {
            webhook_channel_id: target_channel_id,
        };

        http.follow_news_channel(self, &map).await
    }

    /// Attempts to retrieve the channel from the guild cache, otherwise from HTTP/temp cache.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel retrieval request failed.
    #[cfg_attr(not(feature = "cache"), allow(unused_variables))]
    pub async fn to_channel(
        self,
        cache_http: impl CacheHttp,
        guild_id: Option<GuildId>,
    ) -> Result<Channel> {
        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(guild_id) = guild_id {
                if let Some(guild) = cache.guild(guild_id) {
                    if let Some(channel) = guild.channels.get(&self) {
                        return Ok(Channel::Guild(channel.clone()));
                    }
                }
            }

            #[cfg(feature = "temp_cache")]
            if let Some(channel) = cache.temp_channels.get(&self) {
                return Ok(Channel::Guild(GuildChannel::clone(&*channel)));
            }
        }

        let channel = cache_http.http().get_channel(self).await?;

        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        {
            if let Some(cache) = cache_http.cache() {
                if let Channel::Guild(guild_channel) = &channel {
                    use crate::cache::MaybeOwnedArc;

                    let cached_channel = MaybeOwnedArc::new(guild_channel.clone());
                    cache.temp_channels.insert(cached_channel.id, cached_channel);
                }
            }
        }

        Ok(channel)
    }

    /// Fetches a channel from the cache, falling back to HTTP/temp cache.
    ///
    /// It is highly recommended to pass the `guild_id` parameter as otherwise this may perform many
    /// HTTP requests.
    ///
    /// # Errors
    ///
    /// Errors if the HTTP fallback fails, or if the channel does not come from the guild passed.
    pub async fn to_guild_channel(
        self,
        cache_http: impl CacheHttp,
        guild_id: Option<GuildId>,
    ) -> Result<GuildChannel> {
        let channel = self.to_channel(cache_http, guild_id).await?;
        let guild_channel = channel.guild().ok_or(ModelError::InvalidChannelType)?;

        if guild_id.is_some_and(|id| guild_channel.guild_id != id) {
            return Err(Error::Model(ModelError::ChannelNotFound));
        }

        Ok(guild_channel)
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
    pub async fn invites(self, http: &Http) -> Result<Vec<RichInvite>> {
        http.get_channel_invites(self).await
    }

    /// Gets a message from the channel.
    ///
    /// If the cache feature is enabled the cache will be checked first. If not found it will
    /// resort to an http request.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn message(
        self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
    ) -> Result<Message> {
        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(message) = cache.message(self, message_id) {
                return Ok(message.clone());
            }
        }

        let message = cache_http.http().get_message(self, message_id).await?;

        #[cfg(feature = "temp_cache")]
        if let Some(cache) = cache_http.cache() {
            use crate::cache::MaybeOwnedArc;

            let message = MaybeOwnedArc::new(message.clone());
            cache.temp_messages.insert(message_id, message);
        }

        Ok(message)
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
    pub async fn messages(
        self,
        cache_http: impl CacheHttp,
        builder: GetMessages,
    ) -> Result<Vec<Message>> {
        builder.execute(cache_http, self).await
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`Self::messages`]. A buffer of at
    /// most 100 messages is used to reduce the number of calls necessary.
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
    /// # let channel_id = ChannelId::new(1);
    /// # let ctx: Http = unimplemented!();
    /// use serenity::futures::StreamExt;
    /// use serenity::model::channel::MessagesIter;
    ///
    /// let mut messages = channel_id.messages_iter(&ctx).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!("{} said \"{}\".", message.author.name, message.content,),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn messages_iter(
        self,
        cache_http: &impl CacheHttp,
    ) -> impl Stream<Item = Result<Message>> + '_ {
        MessagesIter::stream(cache_http, self)
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the channel has too
    /// many pinned messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn pin(self, http: &Http, message_id: MessageId, reason: Option<&str>) -> Result<()> {
        http.pin_message(self, message_id, reason).await
    }

    /// Crossposts a [`Message`].
    ///
    /// Requires either to be the message author or to have manage [Manage Messages] permissions on
    /// this channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, and if the user is not the
    /// author of the message.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn crosspost(self, http: &Http, message_id: MessageId) -> Result<Message> {
        http.crosspost_message(self, message_id).await
    }

    /// Gets the list of [`Message`]s which are pinned to the channel.
    ///
    /// If the cache is enabled, this method will fill up the message cache for the channel, if the
    /// messages returned are newer than the existing cached messages or the cache is not full yet.
    ///
    /// **Note**: Returns an empty [`Vec`] if the current user does not have the [Read Message
    /// History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn pins(self, cache_http: impl CacheHttp) -> Result<Vec<Message>> {
        let messages = cache_http.http().get_pins(self).await?;

        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            cache.fill_message_cache(self, messages.iter().cloned());
        }

        Ok(messages)
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
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// **Note**: If the passed reaction_type is a custom guild emoji, it must contain the name.
    /// So, [`Emoji`] or [`EmojiIdentifier`] will always work, [`ReactionType`] only if
    /// [`ReactionType::Custom::name`] is Some, and **[`EmojiId`] will never work**.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to read messages in the
    /// channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn reaction_users(
        self,
        http: &Http,
        message_id: MessageId,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let limit = limit.map_or(50, |x| if x > 100 { 100 } else { x });

        http.get_reaction_users(self, message_id, &reaction_type.into(), limit, after).await
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// **Note**: Message content must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the content length is over the above limit. See
    /// [`CreateMessage::execute`] for more details.
    pub async fn say(self, http: &Http, content: impl Into<Cow<'_, str>>) -> Result<Message> {
        let builder = CreateMessage::new().content(content);
        self.send_message(http, builder).await
    }

    /// Sends file(s) along with optional message contents. The filename _must_ be specified.
    ///
    /// Message contents may be passed using the `builder` argument.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Examples
    ///
    /// Send files with the paths `/path/to/file.jpg` and `/path/to/file2.jpg`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), serenity::Error> {
    /// # let http: Arc<Http> = unimplemented!();
    /// use serenity::builder::{CreateAttachment, CreateMessage};
    /// use serenity::model::id::ChannelId;
    ///
    /// let channel_id = ChannelId::new(7);
    ///
    /// let paths = [
    ///     CreateAttachment::path("/path/to/file.jpg").await?,
    ///     CreateAttachment::path("path/to/file2.jpg").await?,
    /// ];
    ///
    /// let builder = CreateMessage::new().content("some files");
    /// channel_id.send_files(&http, paths, builder).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// Send files using [`File`]:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Arc<Http> = unimplemented!();
    /// use serenity::builder::{CreateAttachment, CreateMessage};
    /// use serenity::model::id::ChannelId;
    /// use tokio::fs::File;
    ///
    /// let channel_id = ChannelId::new(7);
    ///
    /// let f1 = File::open("my_file.jpg").await?;
    /// let f2 = File::open("my_file2.jpg").await?;
    ///
    /// let files = [
    ///     CreateAttachment::file(&f1, "my_file.jpg").await?,
    ///     CreateAttachment::file(&f2, "my_file2.jpg").await?,
    /// ];
    ///
    /// let builder = CreateMessage::new().content("some files");
    /// let _ = channel_id.send_files(&http, files, builder).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    ///
    /// [`File`]: tokio::fs::File
    pub async fn send_files<'a>(
        self,
        http: &Http,
        files: impl IntoIterator<Item = CreateAttachment<'a>>,
        builder: CreateMessage<'a>,
    ) -> Result<Message> {
        self.send_message(http, builder.files(files)).await
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
    pub async fn send_message(self, http: &Http, builder: CreateMessage<'_>) -> Result<Message> {
        builder.execute(http, self, None).await
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
    /// # use serenity::{http::Http, Result, model::id::ChannelId};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # fn main() {
    /// # let http: Arc<Http> = unimplemented!();
    /// // Initiate typing (assuming http is `Arc<Http>`)
    /// let typing = ChannelId::new(7).start_typing(http);
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
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to send messages in this channel.
    pub fn start_typing(self, http: Arc<Http>) -> Typing {
        Typing::start(http, self)
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
    pub async fn unpin(
        self,
        http: &Http,
        message_id: MessageId,
        reason: Option<&str>,
    ) -> Result<()> {
        http.unpin_message(self, message_id, reason).await
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
    pub async fn webhooks(self, http: &Http) -> Result<Vec<Webhook>> {
        http.get_channel_webhooks(self).await
    }

    /// Creates a webhook in the channel.
    ///
    /// # Errors
    ///
    /// See [`CreateWebhook::execute`] for a detailed list of possible errors.
    pub async fn create_webhook(self, http: &Http, builder: CreateWebhook<'_>) -> Result<Webhook> {
        builder.execute(http, self).await
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages in this
    /// channel.
    #[cfg(feature = "collector")]
    pub fn await_reply(self, shard_messenger: ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).channel_id(self)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(self, shard_messenger: ShardMessenger) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions sent in
    /// this channel.
    #[cfg(feature = "collector")]
    pub fn await_reaction(self, shard_messenger: ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).channel_id(self)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(self, shard_messenger: ShardMessenger) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }

    /// Gets a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or if there is no stage
    /// instance currently.
    pub async fn get_stage_instance(self, http: &Http) -> Result<StageInstance> {
        http.get_stage_instance(self).await
    }

    /// Creates a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is already a stage instance currently.
    pub async fn create_stage_instance(
        self,
        http: &Http,
        builder: CreateStageInstance<'_>,
    ) -> Result<StageInstance> {
        builder.execute(http, self).await
    }

    /// Edits the stage instance
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or there is no stage
    /// instance currently.
    pub async fn edit_stage_instance(
        self,
        http: &Http,
        builder: EditStageInstance<'_>,
    ) -> Result<StageInstance> {
        builder.execute(http, self).await
    }

    /// Edits a thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_thread(self, http: &Http, builder: EditThread<'_>) -> Result<GuildChannel> {
        builder.execute(http, self).await
    }

    /// Deletes a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or if there is no stage
    /// instance currently.
    pub async fn delete_stage_instance(self, http: &Http, reason: Option<&str>) -> Result<()> {
        http.delete_stage_instance(self, reason).await
    }

    /// Creates a public thread that is connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[doc(alias = "create_public_thread")]
    pub async fn create_thread_from_message(
        self,
        http: &Http,
        message_id: MessageId,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        builder.execute(http, self, Some(message_id)).await
    }

    /// Creates a thread that is not connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[doc(alias = "create_public_thread", alias = "create_private_thread")]
    pub async fn create_thread(
        self,
        http: &Http,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        builder.execute(http, self, None).await
    }

    /// Creates a post in a forum channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn create_forum_post(
        self,
        http: &Http,
        builder: CreateForumPost<'_>,
    ) -> Result<GuildChannel> {
        builder.execute(http, self).await
    }

    /// Gets the thread members, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn get_thread_members(self, http: &Http) -> Result<Vec<ThreadMember>> {
        http.get_channel_thread_members(self).await
    }

    /// Joins the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn join_thread(self, http: &Http) -> Result<()> {
        http.join_thread_channel(self).await
    }

    /// Leaves the thread, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn leave_thread(self, http: &Http) -> Result<()> {
        http.leave_thread_channel(self).await
    }

    /// Adds a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn add_thread_member(self, http: &Http, user_id: UserId) -> Result<()> {
        http.add_thread_channel_member(self, user_id).await
    }

    /// Removes a thread member, if this channel is a thread.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn remove_thread_member(self, http: &Http, user_id: UserId) -> Result<()> {
        http.remove_thread_channel_member(self, user_id).await
    }

    /// Gets a thread member, if this channel is a thread.
    ///
    /// `with_member` controls if ThreadMember::member should be `Some`
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the channel is not a thread channel
    pub async fn get_thread_member(
        self,
        http: &Http,
        user_id: UserId,
        with_member: bool,
    ) -> Result<ThreadMember> {
        http.get_thread_channel_member(self, user_id, with_member).await
    }

    /// Gets private archived threads of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the permission to get it.
    pub async fn get_archived_private_threads(
        self,
        http: &Http,
        before: Option<Timestamp>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.get_channel_archived_private_threads(self, before, limit).await
    }

    /// Gets public archived threads of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the permission to get it.
    pub async fn get_archived_public_threads(
        self,
        http: &Http,
        before: Option<Timestamp>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.get_channel_archived_public_threads(self, before, limit).await
    }

    /// Gets private archived threads joined by the current user of a channel.
    ///
    /// # Errors
    ///
    /// It may return an [`Error::Http`] if the bot doesn't have the permission to get it.
    pub async fn get_joined_archived_private_threads(
        self,
        http: &Http,
        before: Option<ChannelId>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        http.get_channel_joined_archived_private_threads(self, before, limit).await
    }

    /// Get a list of users that voted for this specific answer.
    ///
    /// # Errors
    ///
    /// If the message does not have a poll.
    pub async fn get_poll_answer_voters(
        self,
        http: &Http,
        message_id: MessageId,
        answer_id: AnswerId,
        after: Option<UserId>,
        limit: Option<u8>,
    ) -> Result<Vec<User>> {
        http.get_poll_answer_voters(self, message_id, answer_id, after, limit).await
    }

    /// Ends the [`Poll`] on a given [`MessageId`], if there is one.
    ///
    /// # Errors
    ///
    /// If the message does not have a poll, or if the poll was not created by the current user.
    pub async fn end_poll(self, http: &Http, message_id: MessageId) -> Result<Message> {
        http.expire_poll(self, message_id).await
    }
}

#[cfg(feature = "model")]
impl From<Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: Channel) -> ChannelId {
        channel.id()
    }
}

#[cfg(feature = "model")]
impl From<&Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
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

impl From<&PrivateChannel> for ChannelId {
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

impl From<&GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<WebhookChannel> for ChannelId {
    /// Gets the Id of a webhook channel.
    fn from(webhook_channel: WebhookChannel) -> ChannelId {
        webhook_channel.id
    }
}

impl From<&WebhookChannel> for ChannelId {
    /// Gets the Id of a webhook channel.
    fn from(webhook_channel: &WebhookChannel) -> ChannelId {
        webhook_channel.id
    }
}

/// A helper class returned by [`ChannelId::messages_iter`]
#[derive(Clone, Debug)]
#[cfg(feature = "model")]
pub struct MessagesIter<'a> {
    http: &'a Http,
    #[cfg(feature = "cache")]
    cache: Option<&'a Arc<Cache>>,
    channel_id: ChannelId,
    buffer: Vec<Message>,
    before: Option<MessageId>,
    tried_fetch: bool,
}

#[cfg(feature = "model")]
impl<'a> MessagesIter<'a> {
    fn new(cache_http: &'a impl CacheHttp, channel_id: ChannelId) -> MessagesIter<'a> {
        MessagesIter {
            http: cache_http.http(),
            #[cfg(feature = "cache")]
            cache: cache_http.cache(),
            channel_id,
            buffer: Vec::new(),
            before: None,
            tried_fetch: false,
        }
    }

    #[cfg(not(feature = "cache"))]
    fn cache_http(&self) -> impl CacheHttp + '_ {
        self.http
    }

    #[cfg(feature = "cache")]
    fn cache_http(&self) -> impl CacheHttp + '_ {
        (self.cache, self.http)
    }

    /// Fills the `self.buffer` cache with [`Message`]s.
    ///
    /// This drops any messages that were currently in the buffer. Ideally, it should only be
    /// called when `self.buffer` is empty. Additionally, this updates `self.before` so that the
    /// next call does not return duplicate items.
    ///
    /// If there are no more messages to be fetched, then this sets `self.before` as [`None`],
    /// indicating that no more calls ought to be made.
    ///
    /// If this method is called with `self.before` as None, the last 100 (or lower) messages sent
    /// in the channel are added in the buffer.
    ///
    /// The messages are sorted such that the newest message is the first element of the buffer and
    /// the newest message is the last.
    ///
    /// [`Message`]: crate::model::channel::Message
    async fn refresh(&mut self) -> Result<()> {
        // Number of messages to fetch.
        let grab_size = 100;

        // If `self.before` is not set yet, we can use `.messages` to fetch the last message after
        // very first fetch from last.
        let mut builder = GetMessages::new().limit(grab_size);
        if let Some(before) = self.before {
            builder = builder.before(before);
        }
        self.buffer = self.channel_id.messages(self.cache_http(), builder).await?;

        self.buffer.reverse();

        self.before = self.buffer.first().map(|m| m.id);

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the messages in a channel.
    ///
    /// This is accomplished and equivalent to repeated calls to [`ChannelId::messages`]. A buffer
    /// of at most 100 messages is used to reduce the number of calls necessary.
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
    /// # let channel_id = ChannelId::new(1);
    /// # let http: Http = unimplemented!();
    /// use serenity::futures::StreamExt;
    /// use serenity::model::channel::MessagesIter;
    ///
    /// let mut messages = MessagesIter::stream(&http, channel_id).boxed();
    /// while let Some(message_result) = messages.next().await {
    ///     match message_result {
    ///         Ok(message) => println!("{} said \"{}\"", message.author.name, message.content,),
    ///         Err(error) => eprintln!("Uh oh! Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream(
        cache_http: &'a impl CacheHttp,
        channel_id: ChannelId,
    ) -> impl Stream<Item = Result<Message>> + 'a {
        let init_state = MessagesIter::new(cache_http, channel_id);

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
