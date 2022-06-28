use crate::model::channel::{PermissionOverwrite, PermissionOverwriteData, VideoQualityMode};
use crate::model::id::ChannelId;

/// A builder to edit a [`GuildChannel`] for use via [`GuildChannel::edit`]
///
/// Defaults are not directly provided by the builder itself.
///
/// # Examples
///
/// Edit a channel, providing a new name and topic:
///
/// ```rust,no_run
/// # use serenity::{http::Http, model::id::ChannelId};
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// #     let http = Http::new("token");
/// #     let mut channel = ChannelId::new(1);
/// // assuming a channel has already been bound
/// if let Err(why) = channel.edit(&http, |c| c.name("new name").topic("a test topic"), None).await
/// {
///     // properly handle the error
/// }
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`GuildChannel::edit`]: crate::model::channel::GuildChannel::edit
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditChannel {
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
}

impl EditChannel {
    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn bitrate(&mut self, bitrate: u32) -> &mut Self {
        self.bitrate = Some(bitrate);
        self
    }

    /// The camera video quality mode of the channel.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn video_quality_mode(&mut self, quality: VideoQualityMode) -> &mut Self {
        self.video_quality_mode = Some(quality);
        self
    }

    /// The voice region of the channel.
    /// It is automatic when `None`.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn voice_region(&mut self, id: Option<String>) -> &mut Self {
        self.rtc_region = id;
        self
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// The position of the channel in the channel list.
    pub fn position(&mut self, position: u32) -> &mut Self {
        self.position = Some(position);
        self
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    pub fn topic(&mut self, topic: impl Into<String>) -> &mut Self {
        self.topic = Some(topic.into());
        self
    }

    /// Is the channel inappropriate for work?
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.nsfw = Some(nsfw);
        self
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn user_limit(&mut self, user_limit: u32) -> &mut Self {
        self.user_limit = Some(user_limit);
        self
    }

    /// The parent category of the channel.
    ///
    /// This is for [text] and [voice] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    /// [voice]: crate::model::channel::ChannelType::Voice
    #[inline]
    pub fn category<C: Into<Option<ChannelId>>>(&mut self, category: C) -> &mut Self {
        self.parent_id = category.into();
        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are exempt
    /// from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    ///
    /// [`MANAGE_MESSAGES`]: crate::model::permissions::Permissions::MANAGE_MESSAGES
    /// [`MANAGE_CHANNELS`]: crate::model::permissions::Permissions::MANAGE_CHANNELS
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(&mut self, seconds: u64) -> &mut Self {
        self.rate_limit_per_user = Some(seconds);

        self
    }

    /// A set of overwrites defining what a user or a user carrying a certain role can
    /// and cannot do.
    ///
    /// # Example
    ///
    /// Inheriting permissions from an existing channel:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::ChannelId};
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
    /// channel.edit(http, |c| c.name("my_edited_cool_channel").permissions(permissions), None).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn permissions<I>(&mut self, perms: I) -> &mut Self
    where
        I: IntoIterator<Item = PermissionOverwrite>,
    {
        let overwrites = perms.into_iter().map(Into::into).collect::<Vec<_>>();

        self.permission_overwrites = Some(overwrites);
        self
    }
}
