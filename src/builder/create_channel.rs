#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for creating a new [`GuildChannel`] in a [`Guild`].
///
/// Except [`Self::name`], all fields are optional.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#create-guild-channel).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateChannel<'a> {
    name: String,
    #[serde(rename = "type")]
    kind: ChannelType,

    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    permission_overwrites: Vec<PermissionOverwriteData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rtc_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_quality_mode: Option<VideoQualityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_auto_archive_duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_reaction_emoji: Option<DefaultReaction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    available_tags: Vec<ForumTag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_sort_order: Option<SortOrder>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateChannel<'a> {
    /// Creates a builder with the given name, setting [`Self::kind`] to [`ChannelType::Text`] and
    /// leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nsfw: None,
            topic: None,
            bitrate: None,
            position: None,
            parent_id: None,
            user_limit: None,
            rate_limit_per_user: None,
            kind: ChannelType::Text,
            permission_overwrites: Vec::new(),
            audit_log_reason: None,
            rtc_region: None,
            video_quality_mode: None,
            default_auto_archive_duration: None,
            default_reaction_emoji: None,
            available_tags: Vec::new(),
            default_sort_order: None,
        }
    }

    /// Specify how to call this new channel, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Specify what type the channel is, whether it's a text, voice, category or news channel.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = kind;
        self
    }

    /// Specify the category, the "parent" of this channel.
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::Voice`], [`ChannelType::News`],
    /// [`ChannelType::Stage`], [`ChannelType::Forum`]
    #[doc(alias = "parent_id")]
    pub fn category(mut self, id: impl Into<ChannelId>) -> Self {
        self.parent_id = Some(id.into());
        self
    }

    /// Channel topic (0-1024 characters)
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::News`], [`ChannelType::Forum`]
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Specify if this channel is NSFW (18+)
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::Voice`], [`ChannelType::News`],
    /// [`ChannelType::Stage`], [`ChannelType::Forum`]
    pub fn nsfw(mut self, b: bool) -> Self {
        self.nsfw = Some(b);
        self
    }

    /// The bitrate (in bits) of the voice or stage channel; min 8000
    ///
    /// For voice channels, normal servers can set bitrate up to 96000, servers with Boost level 1
    /// can set up to 128000, servers with Boost level 2 can set up to 256000, and servers with
    /// Boost level 3 or the VIP_REGIONS guild feature can set up to 384000. For stage channels,
    /// bitrate can be set up to 64000.
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn bitrate(mut self, rate: u32) -> Self {
        self.bitrate = Some(rate);
        self
    }

    /// Set how many users may occupy this voice channel
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn user_limit(mut self, limit: u32) -> Self {
        self.user_limit = Some(limit);
        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are
    /// exempt from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    ///
    /// [`MANAGE_MESSAGES`]: crate::model::permissions::Permissions::MANAGE_MESSAGES
    /// [`MANAGE_CHANNELS`]: crate::model::permissions::Permissions::MANAGE_CHANNELS
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(mut self, seconds: u64) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// Specify where the channel should be located.
    pub fn position(mut self, pos: u32) -> Self {
        self.position = Some(pos);
        self
    }

    /// A set of overwrites defining what a user or a user carrying a certain role can and cannot
    /// do.
    ///
    /// # Example
    ///
    /// Inheriting permissions from an existing channel:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::guild::Guild};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut guild: Guild = unimplemented!();
    /// use serenity::builder::CreateChannel;
    /// use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::id::UserId;
    /// use serenity::model::permissions::Permissions;
    ///
    /// // Assuming a guild has already been bound.
    /// let permissions = vec![PermissionOverwrite {
    ///     allow: Permissions::VIEW_CHANNEL,
    ///     deny: Permissions::SEND_TTS_MESSAGES,
    ///     kind: PermissionOverwriteType::Member(UserId::new(1234)),
    /// }];
    ///
    /// let builder = CreateChannel::new("my_new_cool_channel").permissions(permissions);
    /// guild.create_channel(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn permissions(mut self, perms: impl IntoIterator<Item = PermissionOverwrite>) -> Self {
        self.permission_overwrites = perms.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Channel voice region id of the voice or stage channel, automatic when not set
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn rtc_region(mut self, rtc_region: String) -> Self {
        self.rtc_region = Some(rtc_region);
        self
    }

    /// The camera video quality mode of the voice channel
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn video_quality_mode(mut self, video_quality_mode: VideoQualityMode) -> Self {
        self.video_quality_mode = Some(video_quality_mode);
        self
    }

    /// The default duration that the clients use (not the API) for newly created threads in the
    /// channel, in minutes, to automatically archive the thread after recent activity
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::News`], [`ChannelType::Forum`]
    pub fn default_auto_archive_duration(mut self, default_auto_archive_duration: u32) -> Self {
        self.default_auto_archive_duration = Some(default_auto_archive_duration);
        self
    }

    /// Emoji to show in the add reaction button on a thread in a forum
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn default_reaction_emoji(mut self, default_reaction_emoji: DefaultReaction) -> Self {
        self.default_reaction_emoji = Some(default_reaction_emoji);
        self
    }

    /// Set of tags that can be used in a forum channel
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn available_tags(mut self, available_tags: impl IntoIterator<Item = ForumTag>) -> Self {
        self.available_tags = available_tags.into_iter().collect();
        self
    }

    /// The default sort order type used to order posts in forum channels
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn default_sort_order(mut self, default_sort_order: SortOrder) -> Self {
        self.default_sort_order = Some(default_sort_order);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for CreateChannel<'a> {
    type Context<'ctx> = GuildId;
    type Built = GuildChannel;

    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, ctx, Permissions::MANAGE_CHANNELS).await?;

        cache_http.http().create_channel(ctx, &self, self.audit_log_reason).await
    }
}
