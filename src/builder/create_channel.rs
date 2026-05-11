use std::borrow::Cow;

use nonmax::NonMaxU16;

use super::CreateForumTag;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for creating a new [`GuildChannel`] in a [`Guild`].
///
/// Except [`Self::name`], all fields are optional.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#create-guild-channel).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateChannel<'a> {
    name: Cow<'a, str>,
    #[serde(rename = "type")]
    kind: ChannelType,

    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<NonMaxU16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<NonMaxU16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u16>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    permission_overwrites: Cow<'a, [PermissionOverwrite]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rtc_region: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_quality_mode: Option<VideoQualityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_auto_archive_duration: Option<AutoArchiveDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_reaction_emoji: Option<ForumEmoji>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    available_tags: Cow<'a, [CreateForumTag<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_sort_order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_forum_layout: Option<ForumLayoutType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_thread_rate_limit_per_user: Option<NonMaxU16>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateChannel<'a> {
    /// Creates a builder with the given name, setting [`Self::kind`] to [`ChannelType::Text`] and
    /// leaving all other fields empty.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
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
            permission_overwrites: Cow::default(),
            audit_log_reason: None,
            rtc_region: None,
            video_quality_mode: None,
            default_auto_archive_duration: None,
            default_reaction_emoji: None,
            available_tags: Cow::default(),
            default_sort_order: None,
            default_forum_layout: None,
            default_thread_rate_limit_per_user: None,
        }
    }

    /// Specify the channel name (1-100 characters).
    ///
    /// Replaces the current value set in [`Self::new`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Specify the type of channel.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = kind;
        self
    }

    /// Specify the parent category for the channel.
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::Voice`], [`ChannelType::News`],
    /// [`ChannelType::Stage`], [`ChannelType::Forum`]
    #[doc(alias = "parent_id")]
    pub fn category(mut self, id: ChannelId) -> Self {
        self.parent_id = Some(id);
        self
    }

    /// Specify the channel topic (0-1024 characters).
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::News`], [`ChannelType::Forum`]
    pub fn topic(mut self, topic: impl Into<Cow<'a, str>>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Specify whether the channel is age-restricted (18+).
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::Voice`], [`ChannelType::News`],
    /// [`ChannelType::Stage`], [`ChannelType::Forum`]
    pub fn nsfw(mut self, b: bool) -> Self {
        self.nsfw = Some(b);
        self
    }

    /// Specify the bitrate (in bits per second; minimum 8000) of the voice or stage channel.
    ///
    /// For voice channels, normal servers can set bitrate up to 96000, servers with Boost level 1
    /// can set up to 128000, servers with Boost level 2 can set up to 256000, and servers with
    /// Boost level 3 or the `VIP_REGIONS` guild feature can set up to 384000. For stage channels,
    /// bitrate can be set up to 64000.
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn bitrate(mut self, rate: u32) -> Self {
        self.bitrate = Some(rate);
        self
    }

    /// Specify the user limit of the voice channel.
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn user_limit(mut self, limit: NonMaxU16) -> Self {
        self.user_limit = Some(limit);
        self
    }

    /// Specify the amount of seconds a user has to wait before sending another message.
    ///
    /// Bots and users with the [`BYPASS_SLOWMODE`] permission are unaffected.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes, or 6 hours).
    ///
    /// [`BYPASS_SLOWMODE`]: crate::model::permissions::Permissions::BYPASS_SLOWMODE
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(mut self, seconds: NonMaxU16) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// Specify the sorting position of the channel.
    ///
    /// Channels with the same position are sorted by id.
    pub fn position(mut self, pos: u16) -> Self {
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
    /// # use serenity::{http::Http, model::id::GuildId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut guild_id: GuildId = unimplemented!();
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
    /// guild_id.create_channel(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn permissions(mut self, perms: impl Into<Cow<'a, [PermissionOverwrite]>>) -> Self {
        self.permission_overwrites = perms.into();
        self
    }

    /// Specify the channel voice region id of the voice or stage channel.
    ///
    /// Automatic when left unset.
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn rtc_region(mut self, rtc_region: Cow<'a, str>) -> Self {
        self.rtc_region = Some(rtc_region);
        self
    }

    /// Specify the camera video quality mode of the voice channel.
    ///
    /// Only for [`ChannelType::Voice`] and [`ChannelType::Stage`]
    pub fn video_quality_mode(mut self, video_quality_mode: VideoQualityMode) -> Self {
        self.video_quality_mode = Some(video_quality_mode);
        self
    }

    /// Specify the default duration, in minutes, that the clients use (not the API) for newly
    /// created threads in the channel to automatically archive the thread after recent activity.
    ///
    /// Only for [`ChannelType::Text`], [`ChannelType::News`], [`ChannelType::Forum`]
    pub fn default_auto_archive_duration(
        mut self,
        default_auto_archive_duration: AutoArchiveDuration,
    ) -> Self {
        self.default_auto_archive_duration = Some(default_auto_archive_duration);
        self
    }

    /// Specify the emoji to show in the add reaction button on a thread in a forum channel.
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn default_reaction_emoji(mut self, default_reaction_emoji: ForumEmoji) -> Self {
        self.default_reaction_emoji = Some(default_reaction_emoji);
        self
    }

    /// Specify the set of tags that can be used in a forum channel.
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn available_tags(
        mut self,
        available_tags: impl Into<Cow<'a, [CreateForumTag<'a>]>>,
    ) -> Self {
        self.available_tags = available_tags.into();
        self
    }

    /// Specify the default sort order type used to order posts in forum channels.
    ///
    /// Only for [`ChannelType::Forum`]
    pub fn default_sort_order(mut self, default_sort_order: SortOrder) -> Self {
        self.default_sort_order = Some(default_sort_order);
        self
    }

    /// Specify the default forum layout view used to display posts in forum channels.
    pub fn default_forum_layout(mut self, default_forum_layout: ForumLayoutType) -> Self {
        self.default_forum_layout = Some(default_forum_layout);
        self
    }

    /// Specify the initial `rate_limit_per_user` to set on newly created threads in a channel.
    /// This field is copied to the thread at creation time and does not live update.
    pub fn default_thread_rate_limit_per_user(
        mut self,
        default_thread_rate_limit_per_user: NonMaxU16,
    ) -> Self {
        self.default_thread_rate_limit_per_user = Some(default_thread_rate_limit_per_user);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission. If setting permission overwrites, only
    /// permissions your bot has in the guild can be allowed/denied. Setting [Manage Roles]
    /// permission in channels is only possible for guild administrators.
    ///
    /// Fires a [`ChannelCreateEvent`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<GuildChannel> {
        http.create_channel(guild_id, &self, self.audit_log_reason).await
    }
}
