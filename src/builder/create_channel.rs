#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for creating a new [`GuildChannel`] in a [`Guild`].
///
/// Except [`Self::name`], all fields are optional.
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`Guild`]: crate::model::guild::Guild
#[derive(Debug, Clone, Serialize)]
#[must_use]
pub struct CreateChannel {
    #[cfg(feature = "http")]
    #[serde(skip)]
    id: GuildId,
    kind: ChannelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u32>,
    permission_overwrites: Vec<PermissionOverwriteData>,
}

impl CreateChannel {
    /// Creates a builder with default values, setting [`Self::kind`] to [`ChannelType::Text`].
    pub fn new(#[cfg(feature = "http")] id: GuildId) -> Self {
        Self {
            #[cfg(feature = "http")]
            id,
            name: None,
            nsfw: None,
            topic: None,
            bitrate: None,
            position: None,
            parent_id: None,
            user_limit: None,
            rate_limit_per_user: None,
            kind: ChannelType::Text,
            permission_overwrites: Vec::new(),
        }
    }

    /// Specify how to call this new channel.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Specify what type the channel is, whether it's a text, voice, category or news channel.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = kind;
        self
    }

    /// Specify the category, the "parent" of this channel.
    pub fn category<I: Into<ChannelId>>(mut self, id: I) -> Self {
        self.parent_id = Some(id.into());
        self
    }

    /// Set an interesting topic.
    ///
    /// **Note**: Must be between 0 and 1000 characters long.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Specify if this channel will be inappropriate to browse while at work.
    pub fn nsfw(mut self, b: bool) -> Self {
        self.nsfw = Some(b);
        self
    }

    /// [Voice-only] Specify the bitrate at which sound plays in the voice channel.
    pub fn bitrate(mut self, rate: u32) -> Self {
        self.bitrate = Some(rate);
        self
    }

    /// [Voice-only] Set how many users may occupy this voice channel.
    pub fn user_limit(mut self, limit: u32) -> Self {
        self.user_limit = Some(limit);
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
    pub fn rate_limit_per_user(mut self, seconds: u64) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// Specify where the channel should be located.
    pub fn position(mut self, pos: u32) -> Self {
        self.position = Some(pos);
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
    /// # use serenity::{http::Http, model::id::GuildId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Arc::new(Http::new("token"));
    /// #     let mut guild = GuildId::new(1).to_partial_guild(&http).await?;
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
    /// guild.create_channel().name("my_cool_channel").permissions(permissions).execute(&http).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn permissions<I>(mut self, perms: I) -> Self
    where
        I: IntoIterator<Item = PermissionOverwrite>,
    {
        self.permission_overwrites = perms.into_iter().map(Into::into).collect();
        self
    }

    #[cfg(feature = "http")]
    #[inline]
    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to manage channels. Otherwise returns [`Error::Http`], as well as
    /// if invalid data was given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn execute(self, cache_http: impl CacheHttp) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(self.id) {
                    let req = Permissions::MANAGE_CHANNELS;

                    if !guild.has_perms(&cache_http, req).await {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http) -> Result<GuildChannel> {
        http.create_channel(self.id.into(), &self, None).await
    }
}
