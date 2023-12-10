#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to edit the welcome screen of a guild
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#modify-guild-welcome-screen)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildWelcomeScreen<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    welcome_channels: Vec<CreateGuildWelcomeChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditGuildWelcomeScreen<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the welcome screen is enabled or not.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// The server description shown in the welcome screen.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_welcome_channel(mut self, channel: CreateGuildWelcomeChannel) -> Self {
        self.welcome_channels.push(channel);
        self
    }

    /// Channels linked in the welcome screen and their display options
    pub fn set_welcome_channels(mut self, channels: Vec<CreateGuildWelcomeChannel>) -> Self {
        self.welcome_channels = channels;
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditGuildWelcomeScreen<'_> {
    type Context<'ctx> = GuildId;
    type Built = GuildWelcomeScreen;

    /// Edits the guild's welcome screen.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().edit_guild_welcome_screen(ctx, &self, self.audit_log_reason).await
    }
}

/// A builder for creating a [`GuildWelcomeChannel`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#welcome-screen-object-welcome-screen-channel-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateGuildWelcomeChannel(GuildWelcomeChannel);

impl CreateGuildWelcomeChannel {
    pub fn new(channel_id: ChannelId, description: String) -> Self {
        Self(GuildWelcomeChannel {
            channel_id,
            emoji: None,
            description: description.into(),
        })
    }

    /// The Id of the channel to show.
    pub fn id(mut self, id: impl Into<ChannelId>) -> Self {
        self.0.channel_id = id.into();
        self
    }

    /// The description shown for the channel.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.0.description = description.into().into();
        self
    }

    /// The emoji shown for the channel.
    pub fn emoji(mut self, emoji: GuildWelcomeChannelEmoji) -> Self {
        self.0.emoji = Some(emoji);
        self
    }
}
