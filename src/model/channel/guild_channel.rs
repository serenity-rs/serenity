use chrono::{DateTime, Utc};
use crate::model::prelude::*;

use tokio::{
    io::AsyncReadExt,
    fs::File,
};
use reqwest::Url;
use bytes::buf::Buf;

#[cfg(feature = "cache")]
use futures::stream::StreamExt;
#[cfg(feature = "cache")]
use crate::cache::Cache;
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
#[cfg(feature = "model")]
use crate::builder::EditChannel;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectReaction, ReactionCollectorBuilder,
    CollectReply, MessageCollectorBuilder,
};
#[cfg(feature = "model")]
use crate::http::{Http, CacheHttp, Typing};
#[cfg(feature = "model")]
use std::sync::Arc;

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
    pub last_pin_timestamp: Option<DateTime<Utc>>,
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
    #[inline]
    pub async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.broadcast_typing(&http).await
    }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(&context, |i| i.max_uses(5)).await;
    /// ```
    #[inline]
    #[cfg(feature = "utils")]
    pub async fn create_invite<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<RichInvite>
    where
        F: FnOnce(&mut CreateInvite) -> &mut CreateInvite
    {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::CREATE_INVITE;

                if !utils::user_has_perms(&cache, self.id, Some(self.guild_id), req).await? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.create_invite(cache_http.http(), f).await
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
    /// [`PermissionOverwriteType::Member`] variant, allowing it the [Send Messages]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #     let http = Arc::new(Http::default());
    /// #     let cache = Cache::default();
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
    /// // assuming the cache has been unlocked
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .await
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, &overwrite).await?;
    /// #   Ok(())
    /// # }
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwriteType::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// #   let http = Arc::new(Http::default());
    /// #   let cache = Cache::default();
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
    /// let channel = cache
    ///     .guild_channel(channel_id)
    ///     .await
    ///     .ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, &overwrite).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Member`]: ../guild/struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverwrite.html
    /// [`PermissionOverwriteType::Member`]: struct.PermissionOverwriteType.html#variant.Member
    /// [`PermissionOverwriteType::Role`]: struct.PermissionOverwriteType.html#variant.Role
    /// [`Role`]: ../guild/struct.Role.html
    /// [Attach Files]:
    /// ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    /// [Send TTS Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_TTS_MESSAGES
    #[inline]
    pub async fn create_permission(&self, http: impl AsRef<Http>, target: &PermissionOverwrite) -> Result<()> {
        self.id.create_permission(&http, target).await
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon
    /// owning the required permissions the HTTP-request will be issued.
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_CHANNELS;

                if !utils::user_has_perms(&cache, self.id, Some(self.guild_id), req).await? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.delete(&cache_http.http()).await
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
    /// [`ModelError::BulkDeleteAmount`]: ../error/enum.Error.html#variant.BulkDeleteAmount
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub async fn delete_messages<T, It>(&self, http: impl AsRef<Http>, message_ids: It) -> Result<()>
    where T: AsRef<MessageId>, It: IntoIterator<Item=T>
    {
        self.id.delete_messages(&http, message_ids).await
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub async fn delete_permission(&self, http: impl AsRef<Http>, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(&http, permission_type).await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
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

    /// Modifies a channel's settings, such as its position or name.
    ///
    /// Refer to `EditChannel`s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,ignore
    /// channel.edit(&context, |c| c.name("test").bitrate(86400)).await;
    /// ```
    #[cfg(feature = "utils")]
    pub async fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
    where F: FnOnce(&mut EditChannel) -> &mut EditChannel
    {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_CHANNELS;

                if !utils::user_has_perms(&cache, self.id, Some(self.guild_id), req).await? {
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

        *self = cache_http.http().edit_channel(self.id.0, &edited).await?;

        Ok(())
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
    #[inline]
    pub async fn edit_message<F>(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        f: F
    ) -> Result<Message>
    where F: FnOnce(&mut EditMessage) -> &mut EditMessage
    {
        self.id.edit_message(&http, message_id, f).await
    }

    /// Attempts to find this channel's guild in the Cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn guild(&self, cache: impl AsRef<Cache>) -> Option<Guild> {
        cache.as_ref().guild(self.guild_id).await
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        self.id.invites(&http).await
    }

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
    #[inline]
    pub async fn message(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<Message> {
        self.id.message(&http, message_id).await
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
    pub async fn messages<F>(&self, http: impl AsRef<Http>, builder: F) -> Result<Vec<Message>>
    where F: FnOnce(&mut GetMessages) -> &mut GetMessages
    {
        self.id.messages(&http, builder).await
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
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         let channel = match context.cache.guild_channel(msg.channel_id).await {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         if let Ok(permissions) = channel.permissions_for_user(&context.cache, &msg.author).await {
    ///             println!("The user's permissions: {:?}", permissions);
    ///         }
    ///     }
    /// }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client =Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
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
    /// use tokio::fs::File;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, mut msg: Message) {
    ///         let channel = match context.cache.guild_channel(msg.channel_id).await {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         let current_user_id = context.cache.current_user().await.id;
    ///         if let Ok(permissions) = channel.permissions_for_user(&context.cache, current_user_id).await {
    ///
    ///             if !permissions.contains(Permissions::ATTACH_FILES | Permissions::SEND_MESSAGES) {
    ///                 return;
    ///             }
    ///
    ///             let file = match File::open("./cat.png").await {
    ///                 Ok(file) => file,
    ///                 Err(why) => {
    ///                     println!("Err opening file: {:?}", why);
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let _ = msg.channel_id.send_files(&context.http, vec![(&file, "cat.png")], |mut m| {
    ///                 m.content("here's a cat");
    ///                 m
    ///             })
    ///             .await;
    ///         }
    ///     }
    /// }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client =Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
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
    pub async fn permissions_for_user(&self, cache: impl AsRef<Cache>, user_id: impl Into<UserId>) -> Result<Permissions> {
        let guild = self.guild(&cache).await.ok_or(Error::Model(ModelError::GuildNotFound))?;
        Ok(guild.user_permissions_in(self.id, user_id))
    }

    /// Calculates the permissions of a role.
    ///
    /// The Id of the argument must be a [`Role`] of the [`Guild`] that the
    /// channel is in.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the channel's guild could
    /// not be found in the [`Cache`].
    ///
    /// Returns a [`ModelError::RoleNotFound`] if the given role could not
    /// be found in the [`Cache`].
    ///
    /// [`Cache`]: ../../cache/struct.Cache.html
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    /// [`ModelError::RoleNotFound`]: ../error/enum.Error.html#variant.RoleNotFound
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Role`]: ../guild/struct.Role.html
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn permissions_for_role(&self, cache: impl AsRef<Cache>, role_id: impl Into<RoleId>) -> Result<Permissions> {
        let guild = self.guild(&cache).await.ok_or(Error::Model(ModelError::GuildNotFound))?;
        guild.role_permissions_in(self.id, role_id).ok_or(Error::Model(ModelError::GuildNotFound))
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// [`Message`]: struct.Message.html
    #[inline]
    pub async fn pin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.pin(&http, message_id).await
    }

    /// Gets all channel's pins.
    #[inline]
    pub async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        self.id.pins(&http).await
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
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    pub async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        self.id.reaction_users(&http, message_id, reaction_type, limit, after).await
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
    #[inline]
    pub async fn say(&self, http: impl AsRef<Http>, content: impl std::fmt::Display) -> Result<Message> {
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
    /// [`ChannelId::send_files`]: ../id/struct.ChannelId.html#method.send_files
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [Attach Files]: ../permissions/struct.Permissions.html#associatedconstant.ATTACH_FILES
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[inline]
    pub async fn send_files<'a, F, T, It>(&self, http: impl AsRef<Http>, files: It, f: F) -> Result<Message>
        where for <'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
              T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T> {
        self.id.send_files(&http, files, f).await
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
    pub async fn send_message<'a, F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where for <'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::SEND_MESSAGES;

                if !utils::user_has_perms(&cache, self.id, Some(self.guild_id), req).await? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.send_message(&cache_http.http(), f).await
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
    /// #    model::{ModelError, channel::GuildChannel, id::ChannelId},
    /// #    Result,
    /// # };
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http = Arc::new(Http::default());
    /// # let cache = Cache::default();
    /// # let channel = cache
    /// #    .guild_channel(ChannelId(7))
    /// #    .await.ok_or(ModelError::ItemMissing)?;
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
    /// [`Typing`]: ../../http/typing/struct.Typing.html
    /// [`Typing::stop`]: ../../http/typing/struct.Typing.html#method.stop
    pub fn start_typing(self, http: &Arc<Http>) -> Result<Typing> {
        http.start_typing(self.id.0)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[inline]
    pub async fn unpin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.unpin(&http, message_id).await
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[inline]
    pub async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        self.id.webhooks(&http).await
    }

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
    pub async fn members(&self, cache: impl AsRef<Cache>) -> Result<Vec<Member>> {
        let cache = cache.as_ref();
        let guild = cache
            .guild(self.guild_id)
            .await
            .ok_or(ModelError::GuildNotFound)?;

        match self.kind {
            ChannelType::Voice => {
                Ok(guild
                .voice_states
                .values()
                .filter_map(|v| {
                    v.channel_id.and_then(
                        |c| {
                            if c == self.id {
                                guild.members.get(&v.user_id).cloned()
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect())
            },
            ChannelType::News | ChannelType::Text => Ok(futures::stream::iter(
                guild
                    .members
                    .iter())
                    .filter_map(|e| async move {
                        if self.permissions_for_user(cache, e.0)
                            .await
                            .map(|p| p.contains(Permissions::READ_MESSAGES)).unwrap_or(false)
                        {
                            Some(e.1.clone())
                        } else {
                            None
                        }
                    }).collect::<Vec<Member>>()
                    .await),
            _ => Err(Error::from(ModelError::InvalidChannelType)),
        }
    }

    /// Returns a future that will await one message by this guild.
    #[cfg(feature = "collector")]
    pub fn await_reply<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).guild_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages sent by this guild.
    #[cfg(feature = "collector")]
    pub fn await_replies<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).guild_id(self.id.0)
    }

    /// Await a single reaction by this guild.
    #[cfg(feature = "collector")]
    pub fn await_reaction<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).guild_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent by this guild.
    #[cfg(feature = "collector")]
    pub fn await_reactions<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).guild_id(self.id.0)
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::NameTooShort`] if the name of the webhook is
    /// under the limit of 2 characters.
    /// Returns a [`ModelError::NameTooLong`] if the name of the webhook is
    /// over the limit of 100 characters.
    /// Returns a [`ModelError::InvalidChannelType`] if the channel type is not text.
    ///
    /// [`ModelError::NameTooShort`]: ../error/enum.Error.html#variant.NameTooShort
    /// [`ModelError::NameTooLong`]: ../error/enum.Error.html#variant.NameTooLong
    /// [`ModelError::InvalidChannelType`]: ../error/enum.Error.html#variant.InvalidChannelType
    pub async fn create_webhook(&self, http: impl AsRef<Http>, name: impl std::fmt::Display) -> Result<Webhook> {
        let name = name.to_string();

        if name.len() < 2 {
            return Err(Error::Model(ModelError::NameTooShort))
        } else if name.len() > 100 {
            return Err(Error::Model(ModelError::NameTooLong))
        } else if self.kind.num() != 0 {
            return Err(Error::Model(ModelError::InvalidChannelType))
        }

        let map = serde_json::json!({
            "name": name,
        });

        http.as_ref().create_webhook(self.id.0, &map).await
    }

    /// Avatar must be a 128x128 image.
    pub async fn create_webhook_with_avatar<'a>(&self, http: impl AsRef<Http>, name: impl std::fmt::Display, avatar: impl Into<AttachmentType<'a>>) -> Result<Webhook> {
        let name = name.to_string();
        let avatar = avatar.into();

        if name.len() < 2 {
            return Err(Error::Model(ModelError::NameTooShort))
        } else if name.len() > 100 {
            return Err(Error::Model(ModelError::NameTooLong))
        } else if self.kind.num() != 0 {
            return Err(Error::Model(ModelError::InvalidChannelType))
        }

        let avatar = match avatar {
            AttachmentType::Bytes{ data, filename: _ } => {
                "data:image/png;base64,".to_string() + &base64::encode(&data.into_owned())
            },
            AttachmentType::File{ file, filename: _ } => {
                let mut buf = Vec::new();
                file.try_clone().await?.read_to_end(&mut buf).await?;

                "data:image/png;base64,".to_string() + &base64::encode(&buf)
            },
            AttachmentType::Path(path) => {
                let mut file = File::open(path).await?;
                let mut buf = vec![];
                file.read_to_end(&mut buf).await?;

                "data:image/png;base64,".to_string() + &base64::encode(&buf)
            },
            AttachmentType::Image(url) => {
                let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;
                let response = http.as_ref().client.get(url).send().await?;
                let mut bytes = response.bytes().await?;
                let mut picture: Vec<u8> = vec![0; bytes.len()];
                bytes.copy_to_slice(&mut picture[..]);

                "data:image/png;base64,".to_string() + &base64::encode(&picture)
            },
        };

        let map = serde_json::json!({
            "name": name,
            "avatar": avatar
        });

        http.as_ref().create_webhook(self.id.0, &map).await
    }
}

impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.id.mention(), f)
    }
}
