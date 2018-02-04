use chrono::{DateTime, FixedOffset};
use futures::{Future, future};
use model::prelude::*;
use std::cell::RefCell;
use super::super::WrappedClient;
use ::FutureResult;

#[cfg(feature = "model")]
use builder::{CreateInvite, CreateMessage, EditChannel, EditMessage, GetMessages};
#[cfg(all(feature = "cache", feature = "model"))]
use internal::prelude::*;
#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(all(feature = "model", feature = "utils"))]
use utils as serenity_utils;

/// Represents a guild's text or voice channel. Some methods are available only
/// for voice channels and some are only available for text channels.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildChannel {
    /// The unique Id of the channel.
    ///
    /// The default channel Id shares the Id of the guild and the default role.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub bitrate: Option<u64>,
    /// Whether this guild channel belongs in a category.
    #[serde(rename = "parent_id")]
    pub category_id: Option<ChannelId>,
    /// The Id of the guild the channel is located in.
    ///
    /// If this matches with the [`id`], then this is the default text channel.
    ///
    /// The original voice channel has an Id equal to the guild's Id,
    /// incremented by one.
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The Id of the last message sent in the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    ///
    /// **Note**: This is only available for text channels.
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// The name of the channel.
    pub name: String,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    ///
    /// [`Member`]: struct.Member.html
    /// [`Role`]: struct.Role.html
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `-1` or
    /// `0`.
    pub position: i64,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub topic: Option<String>,
    /// The maximum number of members allowed in the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub user_limit: Option<u64>,
    /// Used to tell if the channel is not safe for work.
    /// Note however, it's recommended to use [`is_nsfw`] as it's gonna be more accurate.
    ///
    /// [`is_nsfw`]: struct.GuildChannel.html#method.is_nsfw
    // This field can or can not be present sometimes, but if it isn't
    // default to `false`.
    #[serde(default)]
    pub nsfw: bool,
    #[serde(skip)]
    pub(crate) client: WrappedClient,
}

#[cfg(feature = "model")]
impl GuildChannel {
    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn broadcast_typing(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.broadcast_typing(self.id.0)
    }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(|i| i.max_uses(5));
    /// ```
    #[cfg(feature = "utils")]
    pub fn create_invite<F>(&self, f: F) -> FutureResult<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::CREATE_INVITE;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        client.http.create_invite(self.id.0, f)
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
    /// ```rust,no_run
    /// # use serenity::model::id::{ChannelId, UserId};
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::channel::{
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    /// };
    /// use serenity::model::{ModelError, Permissions};
    /// use serenity::CACHE;
    ///
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read();
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().create_permission(&overwrite)?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwrite::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::{ChannelId, UserId};
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::channel::{
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    /// };
    /// use serenity::model::{ModelError, Permissions};
    /// use serenity::CACHE;
    ///
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read();
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().create_permission(&overwrite)?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Member`]: struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverwrite.html
    /// [`PermissionOverwrite::Member`]: struct.PermissionOverwrite.html#variant.Member
    /// [`PermissionOverwrite::Role`]: struct.PermissionOverwrite.html#variant.Role
    /// [`Role`]: struct.Role.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    /// [Send TTS Messages]: permissions/constant.SEND_TTS_MESSAGES.html
    #[inline]
    pub fn create_permission(&self, target: &PermissionOverwrite)
        -> FutureResult<()> {
        ftryopt!(self.client).http.create_permission(self.id.0, target)
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    pub fn delete(&self) -> FutureResult<Channel> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        client.http.delete_channel(self.id.0)
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
    pub fn delete_messages<T, It>(&self, message_ids: It) -> FutureResult<()>
        where T: AsRef<MessageId>, It: IntoIterator<Item=T> {
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
    pub fn delete_reaction<M, R>(
        &self,
        message_id: M,
        user_id: Option<UserId>,
        reaction_type: R,
    ) -> FutureResult<()> where M: Into<MessageId>, R: Into<ReactionType> {
        ftryopt!(self.client).http.delete_reaction(
            self.id.0,
            message_id.into().0,
            user_id.map(|x| x.0),
            &reaction_type.into(),
        )
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
    /// channel.edit(|c| c.name("test").bitrate(86400));
    /// ```
    #[cfg(feature = "utils")]
    pub fn edit<F: FnOnce(EditChannel) -> EditChannel>(&self, f: F)
        -> FutureResult<GuildChannel> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::MANAGE_CHANNELS;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        client.http.edit_channel(self.id.0, f)
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

    /// Attempts to find this channel's guild in the Cache.
    ///
    /// **Note**: Right now this performs a clone of the guild. This will be
    /// optimized in the future.
    #[cfg(feature = "cache")]
    pub fn guild(&self) -> Option<Rc<RefCell<Guild>>> {
        self.client.as_ref()?.cache.try_borrow().ok().and_then(|cache| {
            cache.guild(self.guild_id)
        })
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn invites(&self) -> FutureResult<Vec<RichInvite>> {
        ftryopt!(self.client).http.get_channel_invites(self.id.0)
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// Only [text channels][`ChannelType::Text`] are taken into consideration
    /// as being NSFW. [voice channels][`ChannelType::Voice`] are never NSFW.
    ///
    /// [`ChannelType::Text`]: enum.ChannelType.html#variant.Text
    /// [`ChannelType::Voice`]: enum.ChannelType.html#variant.Voice
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[cfg(feature = "utils")]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && (self.nsfw || serenity_utils::is_nsfw(&self.name))
    }

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

    /// Returns the name of the guild channel.
    pub fn name(&self) -> &str { &self.name }

    /// Calculates the permissions of a member.
    ///
    /// The Id of the argument must be a [`Member`] of the [`Guild`] that the
    /// channel is in.
    ///
    /// # Examples
    ///
    /// Calculate the permissions of a [`User`] who posted a [`Message`] in a
    /// channel:
    ///
    /// ```rust,no_run
    /// use serenity::prelude::*;
    /// use serenity::model::prelude::*;
    /// struct Handler;
    ///
    /// use serenity::CACHE;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         let channel = match CACHE.read().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let permissions = channel.read().permissions_for(&msg.author).unwrap();
    ///
    ///         println!("The user's permissions: {:?}", permissions);
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// Check if the current user has the [Attach Files] and [Send Messages]
    /// permissions (note: serenity will automatically check this for; this is
    /// for demonstrative purposes):
    ///
    /// ```rust,no_run
    /// use serenity::CACHE;
    /// use serenity::prelude::*;
    /// use serenity::model::prelude::*;
    /// use std::fs::File;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         let channel = match CACHE.read().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let current_user_id = CACHE.read().user.id;
    ///         let permissions =
    ///             channel.read().permissions_for(current_user_id).unwrap();
    ///
    ///         if !permissions.contains(Permissions::ATTACH_FILES | Permissions::SEND_MESSAGES) {
    ///             return;
    ///         }
    ///
    ///         let file = match File::open("./cat.png") {
    ///             Ok(file) => file,
    ///             Err(why) => {
    ///                 println!("Err opening file: {:?}", why);
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         let _ = msg.channel_id.send_files(vec![(&file, "cat.png")], |m|
    ///             m.content("here's a cat"));
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the channel's guild could
    /// not be found in the [`Cache`].
    ///
    /// [`Cache`]: ../cache/struct.Cache.html
    /// [`ModelError::GuildNotFound`]: enum.ModelError.html#variant.GuildNotFound
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[cfg(feature = "cache")]
    pub fn permissions_for<U: Into<UserId>>(&self, user_id: U)
        -> Result<Permissions> {
        self.guild()
            .ok_or_else(|| Error::Model(ModelError::GuildNotFound))
            .map(|g| g.borrow().permissions_in(self.id, user_id))
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> FutureResult<()> {
        ftryopt!(self.client).http.pin_message(self.id.0, message_id.into().0)
    }

    /// Gets all channel's pins.
    #[inline]
    pub fn pins(&self) -> FutureResult<Vec<Message>> {
        ftryopt!(self.client).http.get_pins(self.id.0)
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
    pub fn say(&self, content: &str) -> FutureResult<Message> {
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
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`Message`]: struct.Message.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F)
        -> FutureResult<Message> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let req = Permissions::SEND_MESSAGES;

            if !ftry!(client.cache.borrow().user_has_perms(self.id, req)) {
                return Box::new(future::err(Error::Model(
                    ModelError::InvalidPermissions(req),
                )));
            }
        }

        client.http.send_message(self.id.0, f)
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

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> FutureResult<Vec<Webhook>> {
        ftryopt!(self.client).http.get_channel_webhooks(self.id.0)
    }
}

#[cfg(feature = "model")]
impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.id.mention(), f) }
}
