#[cfg(feature = "http")]
use crate::http::CacheHttp;
use chrono::{DateTime, FixedOffset};
use crate::model::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(feature = "cache")]
use parking_lot::RwLock;
#[cfg(feature = "cache")]
use std::sync::Arc;
#[cfg(feature = "model")]
use crate::builder::{
    CreateInvite,
    CreateMessage,
    EditMessage,
    GetMessages
};
#[cfg(feature = "model")]
use crate::http::AttachmentType;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult
};
#[cfg(all(feature = "model", feature = "utils"))]
use crate::utils as serenity_utils;
#[cfg(all(feature = "model", feature = "builder"))]
use crate::builder::EditChannel;
#[cfg(feature = "http")]
use crate::http::Http;

/// Represents a guild's text, news, or voice channel. Some methods are available
/// only for voice channels and some are only available for text channels.
/// News channels are a subset of text channels and lack slow mode hence
/// `slow_mode_rate` will be `None`.
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
    /// [`Member`]: ../guild/struct.Member.html
    /// [`Role`]: ../guild/struct.Role.html
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
    /// A rate limit that applies per user and excludes bots.
    ///
    /// **Note**: This is only available for text channels excluding news
    /// channels.
    #[serde(default, rename = "rate_limit_per_user")]
    pub slow_mode_rate: Option<u64>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "http")]
    pub fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> { self.id.broadcast_typing(&http) }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(|i| i.max_uses(5));
    /// ```
    #[cfg(all(feature = "utils", feature = "client"))]
    pub fn create_invite<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<RichInvite>
        where F: FnOnce(&mut CreateInvite) -> &mut CreateInvite {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::CREATE_INVITE;

                if !utils::user_has_perms(&cache, self.id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        let mut invite = CreateInvite::default();
        f(&mut invite);

        let map = serenity_utils::hashmap_to_json_map(invite.0);

        cache_http.http().create_invite(self.id.0, &map)
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
    /// # use std::error::Error;
    /// #
    /// # #[cfg(feature = "cache")]
    /// # fn main() -> Result<(), Box<Error>> {
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::id::{ChannelId, UserId}};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #     let http = Arc::new(Http::default());
    /// #     let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #     let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::channel::{
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    /// };
    /// use serenity::model::{ModelError, Permissions};
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    /// # let cache = cache.read();
    /// // assuming the cache has been unlocked
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().create_permission(&http, &overwrite)?;
    /// # Ok(())
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() -> Result<(), Box<Error>> { Ok(()) }
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwrite::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,no_run
    /// # use std::error::Error;
    /// #
    /// # #[cfg(feature = "cache")]
    /// # fn main() -> Result<(), Box<Error>> {
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::id::{ChannelId, UserId}};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let http = Arc::new(Http::default());
    /// #   let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #   let (channel_id, user_id) = (ChannelId(0), UserId(0));
    /// #
    /// use serenity::model::channel::{
    ///     PermissionOverwrite,
    ///     PermissionOverwriteType,
    /// };
    /// use serenity::model::{ModelError, Permissions, channel::Channel};
    ///
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = cache.read();
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.read().create_permission(&http, &overwrite)?;
    /// #     Ok(())
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() -> Result<(), Box<Error>> { Ok(()) }
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Member`]: ../guild/struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverwrite.html
    /// [`PermissionOverwrite::Member`]: struct.PermissionOverwrite.html#variant.Member
    /// [`PermissionOverwrite::Role`]: struct.PermissionOverwrite.html#variant.Role
    /// [`Role`]: ../guild/struct.Role.html
    /// [Attach Files]:
    /// ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    /// [Send TTS Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_TTS_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_permission(&self, http: impl AsRef<Http>, target: &PermissionOverwrite) -> Result<()> {
        self.id.create_permission(&http, target)
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon
    /// owning the required permissions the HTTP-request will be issued.
    #[cfg(feature = "http")]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_CHANNELS;

                if !utils::user_has_perms(&cache, self.id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.delete(&cache_http.http())
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
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_messages<T: AsRef<MessageId>, It: IntoIterator<Item=T>>(&self, http: impl AsRef<Http>, message_ids: It) -> Result<()> {
        self.id.delete_messages(&http, message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_permission(&self, http: impl AsRef<Http>, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(&http, permission_type)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_reaction<M, R>(&self,
                                 http: impl AsRef<Http>,
                                 message_id: M,
                                 user_id: Option<UserId>,
                                 reaction_type: R)
                                 -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self.id.delete_reaction(&http, message_id, user_id, reaction_type)
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
    /// channel.edit(&context, |c| c.name("test").bitrate(86400));
    /// ```
    #[cfg(all(feature = "utils", feature = "client", feature = "builder"))]
    pub fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
        where F: FnOnce(&mut EditChannel) -> &mut EditChannel {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_CHANNELS;

                if !utils::user_has_perms(&cache, self.id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        let mut map = HashMap::new();
        map.insert("name", Value::String(self.name.clone()));
        map.insert("position", Value::Number(Number::from(self.position)));

        let mut edit_channel = EditChannel::default();
        f(&mut edit_channel);
        let edited = serenity_utils::hashmap_to_json_map(edit_channel.0);

        match cache_http.http().edit_channel(self.id.0, &edited) {
            Ok(channel) => {
                std::mem::replace(self, channel);

                Ok(())
            },
            Err(why) => Err(why),
        }
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
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_message<F, M>(&self, http: impl AsRef<Http>, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(&mut EditMessage) -> &mut EditMessage, M: Into<MessageId> {
        self.id.edit_message(&http, message_id, f)
    }

    /// Attempts to find this channel's guild in the Cache.
    ///
    /// **Note**: Right now this performs a clone of the guild. This will be
    /// optimized in the future.
    #[cfg(feature = "cache")]
    pub fn guild(&self, cache: impl AsRef<CacheRwLock>) -> Option<Arc<RwLock<Guild>>> { cache.as_ref().read().guild(self.guild_id) }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> { self.id.invites(&http) }

    /// Determines if the channel is NSFW.
    ///
    /// Only [text channels][`ChannelType::Text`] are taken into consideration
    /// as being NSFW. [voice channels][`ChannelType::Voice`] are never NSFW.
    ///
    /// [`ChannelType::Text`]: enum.ChannelType.html#variant.Text
    /// [`ChannelType::Voice`]: enum.ChannelType.html#variant.Voice
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && self.nsfw
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, http: impl AsRef<Http>, message_id: M) -> Result<Message> {
        self.id.message(&http, message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to the [`GetMessages`]-builder for more information on how to
    /// use `builder`.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`GetMessages`]: ../../builder/struct.GetMessages.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[inline]
    pub fn messages<F>(&self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
        where F: FnOnce(&mut GetMessages) -> &mut GetMessages {
        self.id.messages(&http, builder)
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
    /// impl EventHandler for Handler {
    ///     fn message(&self, context: Context, msg: Message) {
    ///         let channel = match context.cache.read().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let permissions = channel.read().permissions_for(&context.cache, &msg.author).unwrap();
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
    /// use serenity::prelude::*;
    /// use serenity::model::prelude::*;
    /// use serenity::model::channel::Channel;
    /// use std::fs::File;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, context: Context, mut msg: Message) {
    ///         let channel = match context.cache.read().guild_channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let current_user_id = context.cache.read().user.id;
    ///         let permissions =
    ///             channel.read().permissions_for(&context.cache, current_user_id).unwrap();
    ///
    ///             if !permissions.contains(Permissions::ATTACH_FILES | Permissions::SEND_MESSAGES) {
    ///                 return;
    ///             }
    ///
    ///             let file = match File::open("./cat.png") {
    ///                 Ok(file) => file,
    ///                 Err(why) => {
    ///                     println!("Err opening file: {:?}", why);
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///         let _ = msg.channel_id.send_files(&context.http, vec![(&file, "cat.png")], |mut m| {
    ///             m.content("here's a cat");
    ///
    ///             m
    ///         });
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
    /// [`Cache`]: ../../cache/struct.Cache.html
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: ../guild/struct.Member.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Attach Files]: ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "cache")]
    #[inline]
    pub fn permissions_for<U: Into<UserId>>(&self, cache: impl AsRef<CacheRwLock>, user_id: U) -> Result<Permissions> {
        self._permissions_for(&cache, user_id.into())
    }

    #[cfg(feature = "cache")]
    fn _permissions_for(&self, cache: impl AsRef<CacheRwLock>, user_id: UserId) -> Result<Permissions> {
        self.guild(&cache)
            .ok_or_else(|| Error::Model(ModelError::GuildNotFound))
            .map(|g| g.read().permissions_in(self.id, user_id))
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, http: impl AsRef<Http>, message_id: M) -> Result<()> { self.id.pin(&http, message_id) }

    /// Gets all channel's pins.
    #[cfg(feature = "http")]
    #[inline]
    pub fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> { self.id.pins(&http) }

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
    #[cfg(feature = "http")]
    pub fn reaction_users<M, R, U>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>> where M: Into<MessageId>,
                                 R: Into<ReactionType>,
                                 U: Into<Option<UserId>> {
        self.id.reaction_users(&http, message_id, reaction_type, limit, after)
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
    #[cfg(feature = "http")]
    #[inline]
    pub fn say(&self, http: impl AsRef<Http>, content: impl std::fmt::Display) -> Result<Message> { self.id.say(&http, content) }

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
    #[cfg(feature = "http")]
    #[inline]
    pub fn send_files<'a, F, T, It>(&self, http: impl AsRef<Http>, files: It, f: F) -> Result<Message>
        where for <'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
              T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T> {
        self.id.send_files(&http, files, f)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`Message`]: struct.Message.html
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "client")]
    pub fn send_message<'a, F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where for <'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::SEND_MESSAGES;

                if !utils::user_has_perms(&cache, self.id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.send_message(&cache_http.http(), f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, http: impl AsRef<Http>, message_id: M) -> Result<()> {
        self.id.unpin(&http, message_id)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[cfg(feature = "http")]
    #[inline]
    pub fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> { self.id.webhooks(&http) }

    /// Retrieves [`Member`]s from the current channel.
    ///
    /// [`ChannelType::Voice`] returns [`Member`]s using the channel.
    /// [`ChannelType::Text`] and [`ChannelType::News`] return [`Member`]s
    /// that can read the channel.
    ///
    /// Other [`ChannelType`]s lack the concept of [`Member`]s and
    /// will return: [`ModelError::InvalidChannelType`].
    ///
    /// [`Member`]: ../guild/struct.Member.html
    /// [`ChannelType`]: enum.ChannelType.html
    /// [`ChannelType::Voice`]: enum.ChannelType.html#variant.Voice
    /// [`ChannelType::Text`]: enum.ChannelType.html#variant.Text
    /// [`ChannelType::News`]: enum.ChannelType.html#variant.News
    /// [`ModelError::InvalidChannelType`]: ../error/enum.Error.html#variant.InvalidChannelType
    #[cfg(feature = "cache")]
    #[inline]
    pub fn members(&self, cache: impl AsRef<CacheRwLock>) -> Result<Vec<Member>> {
        let cache = cache.as_ref();
        let guild = cache.read().guild(self.guild_id).unwrap();

        match self.kind {
            ChannelType::Voice => Ok(guild
                .read()
                .voice_states
                .values()
                .filter_map(|v| {
                    v.channel_id.and_then(
                        |c| {
                            if c == self.id {
                                guild.read().members.get(&v.user_id).cloned()
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect()),
            ChannelType::News | ChannelType::Text => Ok(guild
                    .read()
                    .members
                    .iter()
                    .filter_map(|e|
                        if self.permissions_for(&cache, e.0).map(|p| p.contains(Permissions::READ_MESSAGES)).unwrap_or(false) {
                            Some(e.1.clone())
                        } else {
                            None
                        }
                    ).collect()),
            _ => Err(Error::from(ModelError::InvalidChannelType)),
        }
    }
}

#[cfg(feature = "model")]
impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { Display::fmt(&self.id.mention(), f) }
}
