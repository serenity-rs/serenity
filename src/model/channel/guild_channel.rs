use std::fmt;

use nonmax::{NonMaxU16, NonMaxU32};

#[cfg(feature = "model")]
use crate::builder::{
    CreateMessage,
    CreateStageInstance,
    CreateWebhook,
    EditChannel,
    EditStageInstance,
    EditVoiceState,
};
#[cfg(feature = "cache")]
use crate::cache::{self, Cache};
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;

/// Represents the shared fields between [`GuildChannel`] and [`GuildThread`].
///
/// [Discord docs](https://docs.discord.com/developers/topics/threads#thread-fields)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseGuildChannel {
    /// The Id of the guild the channel is located in.
    #[serde(default)]
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the channel. (1-100 characters)
    pub name: FixedString<u16>,
    /// The Id of the last message sent in the channel.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    pub last_pin_timestamp: Option<Timestamp>,
    /// A rate limit that applies per user and excludes bots.
    ///
    /// **Note**: This is only available for text channels excluding news channels.
    #[doc(alias = "slowmode")]
    #[serde(default)]
    pub rate_limit_per_user: Option<NonMaxU16>,
}

#[cfg(feature = "model")]
impl BaseGuildChannel {
    /// Attempts to find this channel's guild in the Cache.
    #[cfg(feature = "cache")]
    pub fn guild<'a>(&self, cache: &'a Cache) -> Option<cache::GuildRef<'a>> {
        cache.guild(self.guild_id)
    }
}

impl From<&ObfuscatedChannel> for BaseGuildChannel {
    fn from(obfuscated_channel: &ObfuscatedChannel) -> Self {
        Self {
            guild_id: obfuscated_channel.guild_id,
            kind: obfuscated_channel.kind,
            name: FixedString::from_static_trunc("___hidden___"),
            last_message_id: None,
            last_pin_timestamp: None,
            rate_limit_per_user: None,
        }
    }
}

/// Represents a channel in a [`Guild`], excluding thread information.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildChannel {
    /// The shared fields between [`GuildChannel`] and [`GuildThread`].
    #[serde(flatten)]
    pub base: BaseGuildChannel,
    /// The unique ID of the channel.
    pub id: ChannelId,
    /// The Id of the parent category for a channel.
    ///
    /// **Note**: This is only available for channels in a category.
    // Technically shared, but for different purposes.
    pub parent_id: Option<ChannelId>,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice and stage channels.
    pub bitrate: Option<NonMaxU32>,
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
    /// The region override.
    ///
    /// **Note**: This is only available for voice and stage channels. [`None`] for voice and stage
    /// channels means automatic region selection.
    pub rtc_region: Option<FixedString<u8>>,
    /// The video quality mode for a voice channel.
    pub video_quality_mode: Option<VideoQualityMode>,
    /// Default duration for newly created threads, in minutes, to automatically archive the thread
    /// after recent activity.
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    /// Extra information about the channel.
    #[serde(default)]
    pub flags: ChannelFlags,
    /// The set of available tags.
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub available_tags: FixedArray<ForumTag>,
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
    /// **Note**: This is only available in voice channels and will only be [`Some`] when the cache
    /// is enabled. To manually retrieve the value and trigger population in the cache, see
    /// [`Context::channel_info`].
    ///
    /// [`Context::channel_info`]: crate::gateway::client::Context::channel_info
    pub status: Option<FixedString<u16>>,
    /// Unix timestamp (in seconds) of when a voice session started.
    ///
    /// **Note**: This is only available in voice channels and will only be [`Some`] when the cache
    /// is enabled. To manually retrieve the value and trigger population in the cache, see
    /// [`Context::channel_info`].
    ///
    /// [`Context::channel_info`]: crate::gateway::client::Context::channel_info
    pub voice_start_time: Option<i64>,
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
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-forum-layout-types).
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
            self.base.kind,
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
        let channel = self.id.widen().delete(http, reason).await?;
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
    ///     guild.viewable_channels.get(&channel_id).ok_or(ModelError::ItemMissing)?.clone()
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
        if self.base.kind != ChannelType::Stage {
            return Err(Error::from(ModelError::InvalidChannelType));
        }

        builder.execute(http, self.base.guild_id, self.id, Some(user_id)).await
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
    ///     guild.viewable_channels.get(&channel_id).ok_or(ModelError::ItemMissing)?.clone()
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
        builder.execute(http, self.base.guild_id, self.id, None).await
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
        let mut message = self.id.widen().send_message(http, builder).await?;
        message.guild_id = Some(self.base.guild_id);
        Ok(message)
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
        let guild = cache.guild(self.base.guild_id).ok_or(ModelError::GuildNotFound)?;

        match self.base.kind {
            ChannelType::Voice | ChannelType::Stage => Ok(guild
                .voice_states
                .iter()
                .filter_map(|v| {
                    v.channel_id.and_then(|c| {
                        if c == self.id { guild.members.get(&v.user_id).cloned() } else { None }
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
        if !self.is_text_based() && self.base.kind != ChannelType::Forum {
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
        if self.base.kind != ChannelType::Stage {
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
        if self.base.kind != ChannelType::Stage {
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
        if self.base.kind != ChannelType::Stage {
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
        if self.base.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }

        self.id.delete_stage_instance(http, reason).await
    }
}

impl fmt::Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl ExtractKey<ChannelId> for GuildChannel {
    fn extract_key(&self) -> &ChannelId {
        &self.id
    }
}

impl From<ObfuscatedChannel> for GuildChannel {
    fn from(obfuscated_channel: ObfuscatedChannel) -> Self {
        Self {
            base: BaseGuildChannel {
                guild_id: obfuscated_channel.guild_id,
                kind: obfuscated_channel.kind,
                name: FixedString::from_static_trunc("___hidden___"),
                ..Default::default()
            },
            id: obfuscated_channel.id,
            parent_id: obfuscated_channel.parent_id,
            permission_overwrites: FixedArray::from_vec_trunc(vec![PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                kind: PermissionOverwriteType::Role(RoleId::new(obfuscated_channel.guild_id.get())),
            }]),
            position: obfuscated_channel.position,
            flags: ChannelFlags::CHANNEL_OBFUSCATED,
            ..Default::default()
        }
    }
}

/// Represents an obfuscated channel in a [`Guild`].
///
/// Only includes data guaranteed to be available; obfuscated metadata is omitted.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-obfuscated-channels).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ObfuscatedChannel {
    /// The Id of the guild the channel is located in.
    #[serde(default)]
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The Id of the parent category the channel belongs to.
    ///
    /// **Note**: This is only available for channels in a category.
    pub parent_id: Option<ChannelId>,
    /// The position of the channel.
    #[serde(default)]
    pub position: u16,
}

impl fmt::Display for ObfuscatedChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl ExtractKey<ChannelId> for ObfuscatedChannel {
    fn extract_key(&self) -> &ChannelId {
        &self.id
    }
}

impl From<GuildChannel> for ObfuscatedChannel {
    fn from(guild_channel: GuildChannel) -> Self {
        Self {
            guild_id: guild_channel.base.guild_id,
            kind: guild_channel.base.kind,
            id: guild_channel.id,
            parent_id: guild_channel.parent_id,
            position: guild_channel.position,
        }
    }
}

impl From<&GuildChannel> for ObfuscatedChannel {
    fn from(guild_channel: &GuildChannel) -> Self {
        Self {
            guild_id: guild_channel.base.guild_id,
            kind: guild_channel.base.kind,
            id: guild_channel.id,
            parent_id: guild_channel.parent_id,
            position: guild_channel.position,
        }
    }
}

/// A container for a guild channel that might be obfuscated.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MaybeObfuscated {
    Viewable(GuildChannel),
    Obfuscated(ObfuscatedChannel),
}

impl MaybeObfuscated {
    /// Returns the [`ChannelId`] of [`MaybeObfuscated`].
    #[must_use]
    pub fn id(&self) -> ChannelId {
        match self {
            MaybeObfuscated::Viewable(gc) => gc.id,
            MaybeObfuscated::Obfuscated(oc) => oc.id,
        }
    }

    /// Returns the name of [`MaybeObfuscated`].
    ///
    /// **Note**: If the channel is obfuscated, this will return `___hidden___`.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            MaybeObfuscated::Viewable(gc) => &gc.base.name,
            MaybeObfuscated::Obfuscated(_) => "___hidden___",
        }
    }

    /// Returns the [`ChannelId`] of the parent category [`MaybeObfuscated`] belongs to.
    ///
    /// **Note**: If the channel does not belong to a category, this will return `None`.
    #[must_use]
    pub fn parent_id(&self) -> Option<ChannelId> {
        match self {
            MaybeObfuscated::Viewable(gc) => gc.parent_id,
            MaybeObfuscated::Obfuscated(oc) => oc.parent_id,
        }
    }

    /// Returns the position of [`MaybeObfuscated`].
    #[must_use]
    pub fn position(&self) -> u16 {
        match self {
            MaybeObfuscated::Viewable(gc) => gc.position,
            MaybeObfuscated::Obfuscated(oc) => oc.position,
        }
    }

    /// Returns the [`GuildId`] of [`MaybeObfuscated`].
    #[must_use]
    pub fn guild_id(&self) -> GuildId {
        match self {
            MaybeObfuscated::Viewable(gc) => gc.base.guild_id,
            MaybeObfuscated::Obfuscated(oc) => oc.guild_id,
        }
    }
}

impl fmt::Display for MaybeObfuscated {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl ExtractKey<ChannelId> for MaybeObfuscated {
    fn extract_key(&self) -> &ChannelId {
        match self {
            MaybeObfuscated::Viewable(gc) => &gc.id,
            MaybeObfuscated::Obfuscated(oc) => &oc.id,
        }
    }
}

impl From<GuildChannel> for MaybeObfuscated {
    fn from(guild_channel: GuildChannel) -> Self {
        if guild_channel.flags.contains(ChannelFlags::CHANNEL_OBFUSCATED) {
            Self::Obfuscated(guild_channel.into())
        } else {
            Self::Viewable(guild_channel)
        }
    }
}

impl From<ObfuscatedChannel> for MaybeObfuscated {
    fn from(obfuscated_channel: ObfuscatedChannel) -> Self {
        Self::Obfuscated(obfuscated_channel)
    }
}
