#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to edit a [`GuildChannel`] for use via [`GuildChannel::edit`].
///
/// # Examples
///
/// Edit a channel, providing a new name and topic:
///
/// ```rust,no_run
/// # use serenity::builder::EditChannel;
/// # use serenity::http::Http;
/// # use serenity::model::id::ChannelId;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// #     let http = Http::new("token");
/// #     let mut channel = ChannelId::new(1);
/// let builder = EditChannel::new().name("new name").topic("a test topic");
/// if let Err(why) = channel.edit(&http, builder).await {
///     // properly handle the error
/// }
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditChannel<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_quality_mode: Option<VideoQualityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rtc_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission_overwrites: Option<Vec<PermissionOverwriteData>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditChannel<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the channel's settings.
    ///
    /// **Note**: Requires the [Manage Channels] permission. Modifying permissions via
    /// [`Self::permissions`] also requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
        #[cfg(feature = "cache")] guild_id: Option<GuildId>,
    ) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(
                    cache,
                    channel_id,
                    guild_id,
                    Permissions::MANAGE_CHANNELS,
                )?;
                if self.permission_overwrites.is_some() {
                    crate::utils::user_has_perms_cache(
                        cache,
                        channel_id,
                        guild_id,
                        Permissions::MANAGE_ROLES,
                    )?;
                }
            }
        }

        self._execute(cache_http.http(), channel_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, channel_id: ChannelId) -> Result<GuildChannel> {
        http.edit_channel(channel_id, &self, self.audit_log_reason).await
    }

    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ChannelType::Voice
    pub fn bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = Some(bitrate);
        self
    }

    /// The camera video quality mode of the channel.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ChannelType::Voice
    pub fn video_quality_mode(mut self, quality: VideoQualityMode) -> Self {
        self.video_quality_mode = Some(quality);
        self
    }

    /// The voice region of the channel. It is automatic when `None`.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ChannelType::Voice
    pub fn voice_region(mut self, id: Option<String>) -> Self {
        self.rtc_region = id;
        self
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The position of the channel in the channel list.
    pub fn position(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ChannelType::Text
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Is the channel inappropriate for work?
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ChannelType::Text
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = Some(nsfw);
        self
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ChannelType::Voice
    pub fn user_limit(mut self, user_limit: u32) -> Self {
        self.user_limit = Some(user_limit);
        self
    }

    /// The parent category of the channel.
    ///
    /// This is for [text] and [voice] channels only.
    ///
    /// [text]: ChannelType::Text
    /// [voice]: ChannelType::Voice
    #[inline]
    pub fn category<C: Into<Option<ChannelId>>>(mut self, category: C) -> Self {
        self.parent_id = category.into();
        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are
    /// exempt from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    ///
    /// [`MANAGE_MESSAGES`]: Permissions::MANAGE_MESSAGES
    /// [`MANAGE_CHANNELS`]: Permissions::MANAGE_CHANNELS
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(mut self, seconds: u64) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// A set of overwrites defining what a user or a user carrying a certain role can or can't do.
    ///
    /// # Example
    ///
    /// Inheriting permissions from an existing channel:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditChannel;
    /// # use serenity::http::Http;
    /// # use serenity::model::id::ChannelId;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Arc::new(Http::new("token"));
    /// #     let mut channel = ChannelId::new(1);
    /// use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::id::UserId;
    /// use serenity::model::permissions::Permissions;
    ///
    /// // Assuming a channel has already been bound.
    /// let permissions = vec![PermissionOverwrite {
    ///     allow: Permissions::VIEW_CHANNEL,
    ///     deny: Permissions::SEND_TTS_MESSAGES,
    ///     kind: PermissionOverwriteType::Member(UserId::new(1234)),
    /// }];
    ///
    /// let builder = EditChannel::new().name("my_edited_cool_channel").permissions(permissions);
    /// channel.edit(http, builder).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn permissions<I>(mut self, perms: I) -> Self
    where
        I: IntoIterator<Item = PermissionOverwrite>,
    {
        let overwrites = perms.into_iter().map(Into::into).collect::<Vec<_>>();

        self.permission_overwrites = Some(overwrites);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
