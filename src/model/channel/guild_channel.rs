use std::borrow::Cow;
use std::fmt;
#[cfg(feature = "model")]
use std::sync::Arc;

use nonmax::{NonMaxU16, NonMaxU32, NonMaxU8};

#[cfg(feature = "model")]
use crate::builder::{
    Builder,
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
    EditVoiceState,
    GetMessages,
};
#[cfg(feature = "cache")]
use crate::cache::{self, Cache};
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, Typing};
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::Timestamp;

/// Represents a guild's text, news, or voice channel. Some methods are available only for voice
/// channels and some are only available for text channels. News channels are a subset of text
/// channels and lack slow mode hence [`Self::rate_limit_per_user`] will be [`None`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildChannel {
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice and stage channels.
    pub bitrate: Option<NonMaxU32>,
    /// The Id of the parent category for a channel, or of the parent text channel for a thread.
    ///
    /// **Note**: This is only available for channels in a category and thread channels.
    pub parent_id: Option<ChannelId>,
    /// The Id of the guild the channel is located in.
    ///
    /// The original voice channel has an Id equal to the guild's Id, incremented by one.
    ///
    /// [`id`]: GuildChannel::id
    #[serde(default)]
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The Id of the user who created this channel
    ///
    /// **Note**: This is only available for threads and forum posts
    pub owner_id: Option<UserId>,
    /// The Id of the last message sent in the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    ///
    /// **Note**: This is only available for text channels.
    pub last_pin_timestamp: Option<Timestamp>,
    /// The name of the channel. (1-100 characters)
    pub name: FixedString<u16>,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    #[serde(default)]
    pub permission_overwrites: FixedArray<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `0`.
    #[serde(default)]
    pub position: u16,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text, forum and stage channels.
    pub topic: Option<FixedString<u16>>,
    /// The maximum number of members allowed in the channel.
    ///
    /// This is max 99 for voice channels and 10,000 for stage channels (0 refers to no limit).
    pub user_limit: Option<NonMaxU16>,
    /// Used to tell if the channel is not safe for work. Note however, it's recommended to use
    /// [`Self::is_nsfw`] as it's gonna be more accurate.
    // This field can or can not be present sometimes, but if it isn't default to `false`.
    #[serde(default)]
    pub nsfw: bool,
    /// A rate limit that applies per user and excludes bots.
    ///
    /// **Note**: This is only available for text channels excluding news channels.
    #[doc(alias = "slowmode")]
    #[serde(default)]
    pub rate_limit_per_user: Option<NonMaxU16>,
    /// The region override.
    ///
    /// **Note**: This is only available for voice and stage channels. [`None`] for voice and stage
    /// channels means automatic region selection.
    pub rtc_region: Option<FixedString<u8>>,
    /// The video quality mode for a voice channel.
    pub video_quality_mode: Option<VideoQualityMode>,
    /// An approximate count of messages in the thread.
    ///
    /// **Note**: This is only available on thread channels.
    pub message_count: Option<NonMaxU32>,
    /// An approximate count of users in a thread, stops counting at 50.
    ///
    /// **Note**: This is only available on thread channels.
    pub member_count: Option<NonMaxU8>,
    /// The thread metadata.
    ///
    /// **Note**: This is only available on thread channels.
    pub thread_metadata: Option<ThreadMetadata>,
    /// Thread member object for the current user, if they have joined the thread, only included on
    /// certain API endpoints.
    pub member: Option<PartialThreadMember>,
    /// Default duration for newly created threads, in minutes, to automatically archive the thread
    /// after recent activity.
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    /// Computed permissions for the invoking user in the channel, including overwrites.
    ///
    /// Only included inside [`CommandDataResolved`].
    pub permissions: Option<Permissions>,
    /// Extra information about the channel
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub flags: ChannelFlags,
    /// The number of messages ever sent in a thread, it's similar to `message_count` on message
    /// creation, but will not decrement the number when a message is deleted.
    pub total_message_sent: Option<NonMaxU32>,
    /// The set of available tags.
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub available_tags: FixedArray<ForumTag>,
    /// The set of applied tags.
    ///
    /// **Note**: This is only available in a thread in a forum.
    #[serde(default)]
    pub applied_tags: FixedArray<ForumTagId>,
    /// The emoji to show in the add reaction button
    ///
    /// **Note**: This is only available in a forum.
    pub default_reaction_emoji: Option<ForumEmoji>,
    /// The initial `rate_limit_per_user` to set on newly created threads in a channel. This field
    /// is copied to the thread at creation time and does not live update.
    ///
    /// **Note**: This is only available in a forum or text channel.
    pub default_thread_rate_limit_per_user: Option<NonMaxU16>,
    /// The status of a voice channel.
    ///
    /// **Note**: This is only available in voice channels.
    pub status: Option<FixedString<u16>>,
    /// The default sort order type used to order posts
    ///
    /// **Note**: This is only available in a forum.
    pub default_sort_order: Option<SortOrder>,
    /// The default forum layout view used to display posts in a forum. Defaults to 0, which
    /// indicates a layout view has not been set by a channel admin.
    ///
    /// **Note**: This is only available in a forum.
    pub default_forum_layout: Option<ForumLayoutType>,
}

enum_number! {
    /// See [`GuildChannel::default_forum_layout`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-forum-layout-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ForumLayoutType {
        /// No default has been set for forum channel.
        #[default]
        NotSet = 0,
        /// Display posts as a list.
        ListView = 1,
        /// Display posts as a collection of tiles.
        GalleryView = 2,
        _ => Unknown(u8),
    }
}

#[cfg(feature = "model")]
impl GuildChannel {
    /// Whether or not this channel is text-based, meaning that it is possible to send messages.
    #[must_use]
    pub fn is_text_based(&self) -> bool {
        matches!(
            self.kind,
            ChannelType::Text | ChannelType::News | ChannelType::Voice | ChannelType::Stage
        )
    }

    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have the required permissions.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    pub async fn broadcast_typing(&self, http: &Http) -> Result<()> {
        self.id.broadcast_typing(http).await
    }

    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let builder = CreateBuilder::default().max_uses(5);
    /// let invite = channel.create_invite(&context, builder).await;
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    #[cfg(feature = "utils")]
    pub async fn create_invite(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite> {
        builder.execute(cache_http, self.id).await
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a single [`Member`] or
    /// [`Role`] within a [`Channel`].
    ///
    /// Refer to the documentation for [`PermissionOverwrite`]s for more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Creating a permission overwrite for a member by specifying the
    /// [`PermissionOverwriteType::Member`] variant, allowing it the [Send Messages] permission,
    /// but denying the [Send TTS Messages] and [Attach Files] permissions:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// # use std::sync::Arc;
    /// #
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id) = (ChannelId::new(1), UserId::new(1));
    /// use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::{ModelError, Permissions};
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow,
    ///     deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, overwrite).await?;
    /// # Ok(())
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
    /// # use serenity::{cache::Cache, http::Http, model::id::{ChannelId, UserId, RoleId}};
    /// # use std::sync::Arc;
    /// #
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id, role_id) = (ChannelId::new(1), UserId::new(1), RoleId::new(1));
    /// use serenity::model::channel::{Channel, PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::{ModelError, Permissions};
    ///
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow,
    ///     deny,
    ///     kind: PermissionOverwriteType::Role(role_id),
    /// };
    ///
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, overwrite).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
    pub async fn create_permission(&self, http: &Http, target: PermissionOverwrite) -> Result<()> {
        self.id.create_permission(http, target).await
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission.
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, self.id, Permissions::MANAGE_CHANNELS)?;
            }
        }

        let channel = self.id.delete(cache_http.http()).await?;
        channel.guild().ok_or(Error::Model(ModelError::InvalidChannelType))
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    pub async fn delete_permission(
        &self,
        http: &Http,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        self.id.delete_permission(http, permission_type).await
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current user did not perform
    /// the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission. [Manage Messages]:
    /// Permissions::MANAGE_MESSAGES
    pub async fn delete_reaction(
        &self,
        http: &Http,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.delete_reaction(http, message_id, user_id, reaction_type).await
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
    pub async fn delete_reactions(&self, http: &Http, message_id: MessageId) -> Result<()> {
        self.id.delete_reactions(http, message_id).await
    }

    /// Edits the channel's settings.
    ///
    /// Refer to the documentation for [`EditChannel`] for a full list of methods.
    ///
    /// **Note**: Requires the [Manage Channels] permission. Modifying permissions via
    /// [`EditChannel::permissions`] also requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditChannel;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::ChannelId;
    /// # async fn run() {
    /// # let http: Http = unimplemented!();
    /// # let channel = ChannelId::new(1234);
    /// let builder = EditChannel::new().name("test").bitrate(86400);
    /// channel.edit(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn edit(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditChannel<'_>,
    ) -> Result<()> {
        let channel = builder.execute(cache_http, self.id).await?;
        *self = channel;
        Ok(())
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
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
        builder: EditMessage<'_>,
    ) -> Result<Message> {
        self.id.edit_message(cache_http, message_id, builder).await
    }

    /// Edits a thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_thread(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditThread<'_>,
    ) -> Result<()> {
        *self = self.id.edit_thread(cache_http, builder).await?;
        Ok(())
    }

    /// Edits the voice state of a given user in a stage channel.
    ///
    /// **Note**: Requires the [Request to Speak] permission. Also requires the [Mute Members]
    /// permission to suppress another user or unsuppress the current user. This is not required if
    /// suppressing the current user.
    ///
    /// # Example
    ///
    /// Invite a user to speak.
    ///
    /// ```rust
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// # use serenity::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id) = (ChannelId::new(1), UserId::new(1));
    /// use serenity::builder::EditVoiceState;
    /// use serenity::model::ModelError;
    ///
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// let builder = EditVoiceState::new().suppress(false);
    /// channel.edit_voice_state(&http, user_id, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidChannelType`] if the channel is
    /// not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    pub async fn edit_voice_state(
        &self,
        cache_http: impl CacheHttp,
        user_id: UserId,
        builder: EditVoiceState,
    ) -> Result<()> {
        builder.execute(cache_http, (self.guild_id, self.id, Some(user_id))).await
    }

    /// Edits the current user's voice state in a stage channel.
    ///
    /// **Note**: Requires the [Request to Speak] permission. The [Mute Members] permission is
    /// **not** required.
    ///
    /// # Example
    ///
    /// Send a request to speak, then clear the request.
    ///
    /// ```rust
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// # use serenity::{cache::Cache, http::Http, model::id::ChannelId};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel_id = ChannelId::new(1);
    /// use serenity::builder::EditVoiceState;
    /// use serenity::model::ModelError;
    ///
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// // Send a request to speak
    /// let builder = EditVoiceState::new().request_to_speak(true);
    /// channel.edit_own_voice_state(&http, builder.clone()).await?;
    ///
    /// // Clear own request to speak
    /// let builder = builder.request_to_speak(false);
    /// channel.edit_own_voice_state(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidChannelType`] if the channel is
    /// not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    pub async fn edit_own_voice_state(
        &self,
        cache_http: impl CacheHttp,
        builder: EditVoiceState,
    ) -> Result<()> {
        builder.execute(cache_http, (self.guild_id, self.id, None)).await
    }

    /// Follows the News Channel
    ///
    /// Requires [Manage Webhook] permissions on the target channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn follow(
        &self,
        http: &Http,
        target_channel_id: ChannelId,
    ) -> Result<FollowedChannel> {
        self.id.follow(http, target_channel_id).await
    }

    /// Attempts to find this channel's guild in the Cache.
    #[cfg(feature = "cache")]
    pub fn guild<'a>(&self, cache: &'a Cache) -> Option<cache::GuildRef<'a>> {
        cache.guild(self.guild_id)
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
    pub async fn invites(&self, http: &Http) -> Result<Vec<RichInvite>> {
        self.id.invites(http).await
    }

    /// Determines if the channel is NSFW.
    ///
    /// Only [text channels][`ChannelType::Text`] are taken into consideration as being NSFW.
    /// [voice channels][`ChannelType::Voice`] are never NSFW.
    #[must_use]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && self.nsfw
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a message with the
    /// given Id does not exist in the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
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
    pub async fn messages(
        &self,
        cache_http: impl CacheHttp,
        builder: GetMessages,
    ) -> Result<Vec<Message>> {
        self.id.messages(cache_http, builder).await
    }

    /// Returns the name of the guild channel.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Calculates the permissions of a member.
    ///
    /// The Id of the argument must be a [`Member`] of the [`Guild`] that the channel is in.
    ///
    /// # Examples
    ///
    /// Calculate the permissions of a [`User`] who posted a [`Message`] in a channel:
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: &Context, msg: &Message) {
    ///         let channel = match context.cache.channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         if let Ok(permissions) = channel.permissions_for_user(&context.cache, msg.author.id) {
    ///             println!("The user's permissions: {:?}", permissions);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the channel's guild could not be found in the
    /// [`Cache`].
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[cfg(feature = "cache")]
    pub fn permissions_for_user(&self, cache: &Cache, user_id: UserId) -> Result<Permissions> {
        let guild = self.guild(cache).ok_or(Error::Model(ModelError::GuildNotFound))?;
        let member = guild.members.get(&user_id).ok_or(Error::Model(ModelError::MemberNotFound))?;
        Ok(guild.user_permissions_in(self, member))
    }

    /// Pins a [`Message`] to the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the channel already has
    /// too many pinned messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn pin(&self, http: &Http, message_id: MessageId) -> Result<()> {
        self.id.pin(http, message_id).await
    }

    /// Gets all channel's pins.
    ///
    /// **Note**: If the current user lacks the [Read Message History] permission an empty [`Vec`]
    /// will be returned.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn pins(&self, http: &Http) -> Result<Vec<Message>> {
        self.id.pins(http).await
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
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
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

    /// Sends a message with just the given message content in the channel.
    ///
    /// **Note**: Message content must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the content length is over the above limit. See
    /// [`CreateMessage::execute`] for more details.
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
    pub async fn send_files<'a>(
        self,
        cache_http: impl CacheHttp,
        files: impl IntoIterator<Item = CreateAttachment<'a>>,
        builder: CreateMessage<'a>,
    ) -> Result<Message> {
        self.send_message(cache_http, builder.files(files)).await
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
    pub async fn send_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
        #[cfg(feature = "cache")]
        let msg = builder.execute(cache_http, (self.id, Some(self.guild_id))).await;
        #[cfg(not(feature = "cache"))]
        let msg = builder.execute(cache_http, (self.id,)).await;
        msg
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
    /// # use serenity::{cache::Cache, http::Http, model::channel::GuildChannel, Result};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel: GuildChannel = unimplemented!();
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
    #[allow(clippy::missing_errors_doc)]
    pub fn start_typing(&self, http: &Arc<Http>) -> Typing {
        http.start_typing(self.id)
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
    pub async fn unpin(&self, http: &Http, message_id: MessageId) -> Result<()> {
        self.id.unpin(http, message_id).await
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
    pub async fn webhooks(&self, http: &Http) -> Result<Vec<Webhook>> {
        self.id.webhooks(http).await
    }

    /// Retrieves [`Member`]s from the current channel.
    ///
    /// [`ChannelType::Voice`] and [`ChannelType::Stage`] returns [`Member`]s using the channel.
    ///
    /// [`ChannelType::Text`] and [`ChannelType::News`] return [`Member`]s that can read the
    /// channel.
    ///
    /// # Errors
    ///
    /// Other [`ChannelType`]s lack the concept of [`Member`]s and will return:
    /// [`ModelError::InvalidChannelType`].
    #[cfg(feature = "cache")]
    pub fn members(&self, cache: &Cache) -> Result<Vec<Member>> {
        let guild = cache.guild(self.guild_id).ok_or(ModelError::GuildNotFound)?;

        match self.kind {
            ChannelType::Voice | ChannelType::Stage => Ok(guild
                .voice_states
                .values()
                .filter_map(|v| {
                    v.channel_id.and_then(|c| {
                        if c == self.id {
                            guild.members.get(&v.user_id).cloned()
                        } else {
                            None
                        }
                    })
                })
                .collect()),
            ChannelType::News | ChannelType::Text => Ok(guild
                .members
                .iter()
                .filter(|(id, _)| {
                    self.permissions_for_user(cache, **id)
                        .map(|p| p.contains(Permissions::VIEW_CHANNEL))
                        .unwrap_or(false)
                })
                .map(|e| e.1.clone())
                .collect::<Vec<Member>>()),
            _ => Err(Error::from(ModelError::InvalidChannelType)),
        }
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages sent in
    /// this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_reply(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).channel_id(self.id)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a stream builder which can be awaited to obtain a reaction or stream of reactions
    /// sent by this guild channel.
    #[cfg(feature = "collector")]
    pub fn await_reaction(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).channel_id(self.id)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }

    /// Creates a webhook in the channel.
    ///
    /// # Errors
    ///
    /// See [`CreateWebhook::execute`] for a detailed list of possible errors.
    pub async fn create_webhook(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateWebhook<'_>,
    ) -> Result<Webhook> {
        self.id.create_webhook(cache_http, builder).await
    }

    /// Gets a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is no stage instance currently.
    pub async fn get_stage_instance(&self, http: &Http) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.get_stage_instance(http).await
    }

    /// Creates a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is already a stage instance currently.
    pub async fn create_stage_instance(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateStageInstance<'_>,
    ) -> Result<StageInstance> {
        self.id.create_stage_instance(cache_http, builder).await
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
        &self,
        cache_http: impl CacheHttp,
        builder: EditStageInstance<'_>,
    ) -> Result<StageInstance> {
        self.id.edit_stage_instance(cache_http, builder).await
    }

    /// Deletes a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is no stage instance currently.
    pub async fn delete_stage_instance(&self, http: &Http) -> Result<()> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.delete_stage_instance(http).await
    }

    /// Creates a public thread that is connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn create_thread_from_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_thread_from_message(cache_http, message_id, builder).await
    }

    /// Creates a thread that is not connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn create_thread(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_thread(cache_http, builder).await
    }

    /// Creates a post in a forum channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn create_forum_post(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateForumPost<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_forum_post(cache_http, builder).await
    }
}

impl fmt::Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

/// A partial guild channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset description](https://discord.com/developers/docs/topics/gateway#thread-delete)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialGuildChannel {
    /// The channel Id.
    pub id: ChannelId,
    /// The channel guild Id.
    pub guild_id: GuildId,
    /// The channel category Id,  or the parent text channel Id for a thread.
    pub parent_id: ChannelId,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
}
