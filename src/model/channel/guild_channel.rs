use std::fmt;

use nonmax::{NonMaxU16, NonMaxU32, NonMaxU8};

#[cfg(feature = "model")]
use crate::builder::{
    CreateMessage,
    CreateStageInstance,
    CreateWebhook,
    EditChannel,
    EditStageInstance,
    EditThread,
    EditVoiceState,
};
#[cfg(feature = "cache")]
use crate::cache::{self, Cache};
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;

/// Represents a guild's text, news, or voice channel.
///
/// Some methods are available only for voice channels and some are only available for text
/// channels. News channels are a subset of text channels and lack slow mode hence
/// [`Self::rate_limit_per_user`] will be [`None`].
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
    /// Used to tell if the channel is not safe for work.
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
    #[non_exhaustive]
    pub enum ForumLayoutType {
        /// No default has been set for forum channel.
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
            ChannelType::Text
                | ChannelType::News
                | ChannelType::Voice
                | ChannelType::Stage
                | ChannelType::PublicThread
                | ChannelType::PrivateThread
                | ChannelType::NewsThread
        )
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
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<GuildChannel> {
        let channel = self.id.delete(http, reason).await?;
        channel.guild().ok_or(Error::Model(ModelError::InvalidChannelType))
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
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn edit(&mut self, http: &Http, builder: EditChannel<'_>) -> Result<()> {
        let channel = builder.execute(http, self.id).await?;
        *self = channel;
        Ok(())
    }

    /// Edits a thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_thread(&mut self, http: &Http, builder: EditThread<'_>) -> Result<()> {
        *self = self.id.edit_thread(http, builder).await?;
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
    /// # use serenity::{cache::Cache, http::Http, model::id::{GuildId, ChannelId, UserId}};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (guild_id, channel_id, user_id) = (GuildId::new(1), ChannelId::new(1), UserId::new(1));
    /// use serenity::builder::EditVoiceState;
    /// use serenity::model::ModelError;
    ///
    /// let channel = {
    ///     let guild = cache.guild(guild_id).ok_or(ModelError::ItemMissing)?;
    ///     guild.channels.get(&channel_id).ok_or(ModelError::ItemMissing)?.clone()
    /// };
    ///
    /// let builder = EditVoiceState::new().suppress(false);
    /// channel.edit_voice_state(&http, user_id, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    pub async fn edit_voice_state(
        &self,
        http: &Http,
        user_id: UserId,
        builder: EditVoiceState,
    ) -> Result<()> {
        if self.kind != ChannelType::Stage {
            return Err(Error::from(ModelError::InvalidChannelType));
        }

        builder.execute(http, self.guild_id, self.id, Some(user_id)).await
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
    /// # use serenity::{cache::Cache, http::Http, model::id::{GuildId, ChannelId}};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (guild_id, channel_id) = (GuildId::new(1), ChannelId::new(1));
    /// use serenity::builder::EditVoiceState;
    /// use serenity::model::ModelError;
    ///
    /// let channel = {
    ///     let guild = cache.guild(guild_id).ok_or(ModelError::ItemMissing)?;
    ///     guild.channels.get(&channel_id).ok_or(ModelError::ItemMissing)?.clone()
    /// };
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
    pub async fn edit_own_voice_state(&self, http: &Http, builder: EditVoiceState) -> Result<()> {
        builder.execute(http, self.guild_id, self.id, None).await
    }

    /// Attempts to find this channel's guild in the Cache.
    #[cfg(feature = "cache")]
    pub fn guild<'a>(&self, cache: &'a Cache) -> Option<cache::GuildRef<'a>> {
        cache.guild(self.guild_id)
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
    pub async fn send_message(&self, http: &Http, builder: CreateMessage<'_>) -> Result<Message> {
        builder.execute(http, self.id, Some(self.guild_id)).await
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
                .iter()
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
                .filter(|member| {
                    guild.user_permissions_in(self, member).contains(Permissions::VIEW_CHANNEL)
                })
                .cloned()
                .collect::<Vec<Member>>()),
            _ => Err(Error::from(ModelError::InvalidChannelType)),
        }
    }

    /// Creates a webhook in the channel.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the corresponding channel is not of type
    /// [`ChannelType::Text`] or [`ChannelType::News`].
    ///
    /// See [`CreateWebhook::execute`] for a detailed list of other
    /// possible errors,
    pub async fn create_webhook(&self, http: &Http, builder: CreateWebhook<'_>) -> Result<Webhook> {
        // forum channels are not text-based, but webhooks can be created in them
        // and used to send messages in their posts
        if !self.is_text_based() && self.kind != ChannelType::Forum {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.create_webhook(http, builder).await
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
        http: &Http,
        builder: CreateStageInstance<'_>,
    ) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.create_stage_instance(http, builder).await
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
        http: &Http,
        builder: EditStageInstance<'_>,
    ) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.edit_stage_instance(http, builder).await
    }

    /// Deletes a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is no stage instance currently.
    pub async fn delete_stage_instance(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.delete_stage_instance(http, reason).await
    }
}

impl fmt::Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

impl ExtractKey<ChannelId> for GuildChannel {
    fn extract_key(&self) -> &ChannelId {
        &self.id
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
