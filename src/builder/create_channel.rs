use crate::model::prelude::*;

/// A builder for creating a new [`GuildChannel`] in a [`Guild`].
///
/// Except [`Self::name`], all fields are optional.
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`Guild`]: crate::model::guild::Guild
#[derive(Debug, Clone, Serialize)]
pub struct CreateChannel {
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
    /// Specify how to call this new channel.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());

        self
    }
    /// Specify what type the channel is, whether it's a text, voice, category or news channel.
    pub fn kind(&mut self, kind: ChannelType) -> &mut Self {
        self.kind = kind;

        self
    }

    /// Specify the category, the "parent" of this channel.
    pub fn category<I: Into<ChannelId>>(&mut self, id: I) -> &mut Self {
        self.parent_id = Some(id.into());

        self
    }

    /// Set an interesting topic.
    ///
    /// **Note**: Must be between 0 and 1000 characters long.
    pub fn topic(&mut self, topic: impl Into<String>) -> &mut Self {
        self.topic = Some(topic.into());

        self
    }

    /// Specify if this channel will be inappropriate to browse while at work.
    pub fn nsfw(&mut self, b: bool) -> &mut Self {
        self.nsfw = Some(b);

        self
    }

    /// [Voice-only] Specify the bitrate at which sound plays in the voice channel.
    pub fn bitrate(&mut self, rate: u32) -> &mut Self {
        self.bitrate = Some(rate);

        self
    }

    /// [Voice-only] Set how many users may occupy this voice channel.
    pub fn user_limit(&mut self, limit: u32) -> &mut Self {
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
    pub fn rate_limit_per_user(&mut self, seconds: u64) -> &mut Self {
        self.rate_limit_per_user = Some(seconds);

        self
    }

    /// Specify where the channel should be located.
    pub fn position(&mut self, pos: u32) -> &mut Self {
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
    /// guild.create_channel(http, |c| c.name("my_new_cool_channel").permissions(permissions)).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn permissions<I>(&mut self, perms: I) -> &mut Self
    where
        I: IntoIterator<Item = PermissionOverwrite>,
    {
        self.permission_overwrites = perms.into_iter().map(Into::into).collect();

        self
    }
}

impl Default for CreateChannel {
    /// Creates a builder with default values, setting [`Self::kind`] to [`ChannelType::Text`].
    ///
    /// # Examples
    ///
    /// Create a default [`CreateChannel`] builder:
    ///
    /// ```rust
    /// use serenity::builder::CreateChannel;
    ///
    /// let channel_builder = CreateChannel::default();
    /// ```
    fn default() -> Self {
        CreateChannel {
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
}
