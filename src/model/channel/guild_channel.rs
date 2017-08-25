use chrono::{DateTime, FixedOffset};
use model::*;

#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "model")]
use std::mem;
#[cfg(feature = "cache")]
use CACHE;
#[cfg(feature = "model")]
use builder::{CreateInvite, CreateMessage, EditChannel, GetMessages};
#[cfg(feature = "model")]
use http::{self, AttachmentType};
#[cfg(feature = "cache")]
use internal::prelude::*;
#[cfg(all(feature = "model", feature = "utils"))]
use utils as serenity_utils;

/// Represents a guild's text or voice channel. Some methods are available only
/// for voice channels and some are only available for text channels.
#[derive(Clone, Debug, Deserialize)]
pub struct GuildChannel {
    /// The unique Id of the channel.
    ///
    /// The default channel Id shares the Id of the guild and the default role.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub bitrate: Option<u64>,
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
    pub fn broadcast_typing(&self) -> Result<()> { self.id.broadcast_typing() }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(|i| i.max_uses(5));
    /// ```
    pub fn create_invite<F>(&self, f: F) -> Result<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        #[cfg(feature = "cache")]
        {
            let req = permissions::CREATE_INVITE;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        http::create_invite(self.id.0, &f(CreateInvite::default()).0)
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
    /// # use serenity::model::{ChannelId, UserId};
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::{
    ///     ModelError,
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    ///     permissions,
    /// };
    /// use serenity::CACHE;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read().unwrap();
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().unwrap().create_permission(&overwrite)?;
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
    /// # use serenity::model::{ChannelId, UserId};
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::{
    ///     ModelError,
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    ///     permissions,
    /// };
    /// use serenity::CACHE;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read().unwrap();
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().unwrap().create_permission(&overwrite)?;
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
    pub fn create_permission(&self, target: &PermissionOverwrite) -> Result<()> {
        self.id.create_permission(target)
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    pub fn delete(&self) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
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
    pub fn delete_reaction<M, R>(&self,
                                 message_id: M,
                                 user_id: Option<UserId>,
                                 reaction_type: R)
                                 -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
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
    /// channel.edit(|c| c.name("test").bitrate(86400));
    /// ```
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditChannel) -> EditChannel {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        let mut map = Map::new();
        map.insert(
            "name".to_owned(),
            Value::String(self.name.clone()),
        );
        map.insert(
            "position".to_owned(),
            Value::Number(Number::from(self.position)),
        );
        map.insert(
            "type".to_owned(),
            Value::String(self.kind.name().to_owned()),
        );

        let edited = f(EditChannel(map)).0;

        match http::edit_channel(self.id.0, &edited) {
            Ok(channel) => {
                mem::replace(self, channel);

                Ok(())
            },
            Err(why) => Err(why),
        }
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
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, M: Into<MessageId> {
        self.id.edit_message(message_id, f)
    }

    /// Attempts to find this channel's guild in the Cache.
    ///
    /// **Note**: Right now this performs a clone of the guild. This will be
    /// optimized in the future.
    #[cfg(feature = "cache")]
    pub fn guild(&self) -> Option<Arc<RwLock<Guild>>> { CACHE.read().unwrap().guild(self.guild_id) }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn invites(&self) -> Result<Vec<RichInvite>> { self.id.invites() }

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
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id.message(message_id)
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
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id.messages(f)
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
    /// use serenity::model::*;
    /// struct Handler;
    ///
    /// use serenity::CACHE;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, _: Context, msg: Message) {
    ///         let channel = match CACHE.read().unwrap().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let permissions = channel.read().unwrap().permissions_for(&msg.author).unwrap();
    ///
    ///         println!("The user's permissions: {:?}", permissions);
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// Check if the current user has the [Attach Files] and [Send Messages]
    /// permissions (note: serenity will automatically check this for; this is
    /// for demonstrative purposes):
    ///
    /// ```rust,no_run
    /// use serenity::prelude::*;
    /// use serenity::model::*;
    /// use serenity::CACHE;
    /// use serenity::model::permissions;
    /// use std::fs::File;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, _: Context, msg: Message) {
    ///         let channel = match CACHE.read().unwrap().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let current_user_id = CACHE.read().unwrap().user.id;
    /// let permissions =
    /// channel.read().unwrap().permissions_for(current_user_id).unwrap();
    ///
    /// if !permissions.contains(permissions::ATTACH_FILES |
    /// permissions::SEND_MESSAGES) {
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
    /// let _ = msg.channel_id.send_files(vec![(&file, "cat.png")], |m|
    /// m.content("here's a cat"));
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
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
    pub fn permissions_for<U: Into<UserId>>(&self, user_id: U) -> Result<Permissions> {
        self.guild()
            .ok_or_else(|| Error::Model(ModelError::GuildNotFound))
            .map(|g| g.read().unwrap().permissions_for(self.id, user_id))
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> { self.id.pin(message_id) }

    /// Gets all channel's pins.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> { self.id.pins() }

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
    pub fn reaction_users<M, R, U>(&self,
                                   message_id: M,
                                   reaction_type: R,
                                   limit: Option<u8>,
                                   after: Option<U>)
                                   -> Result<Vec<User>>
        where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id.reaction_users(
            message_id,
            reaction_type,
            limit,
            after,
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
    pub fn say(&self, content: &str) -> Result<Message> { self.id.say(content) }

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
    #[inline]
    pub fn send_files<'a, F, T>(&self, files: Vec<T>, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, T: Into<AttachmentType<'a>> {
        self.id.send_files(files, f)
    }

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
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F) -> Result<Message> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::SEND_MESSAGES;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.send_message(f)
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

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> { self.id.webhooks() }
}

#[cfg(feature = "model")]
impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.id.mention(), f) }
}
