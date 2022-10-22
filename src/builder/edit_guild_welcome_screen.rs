#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to specify the fields to edit in a [`GuildWelcomeScreen`].
///
/// [`GuildWelcomeScreen`]: crate::model::guild::GuildWelcomeScreen
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildWelcomeScreen<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    welcome_channels: Vec<CreateGuildWelcomeChannel>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditGuildWelcomeScreen<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the guild's welcome screen.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        guild_id: GuildId,
    ) -> Result<GuildWelcomeScreen> {
        http.as_ref().edit_guild_welcome_screen(guild_id, &self, self.audit_log_reason).await
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

/// A builder for creating a [`GuildWelcomeChannel`].
///
/// [`GuildWelcomeChannel`]: crate::model::guild::GuildWelcomeChannel
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateGuildWelcomeChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<EmojiId>,
}

impl CreateGuildWelcomeChannel {
    /// The Id of the channel to show. It is required.
    pub fn id(mut self, id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    /// The description shown for the channel. It is required.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The emoji shown for the channel.
    pub fn emoji(mut self, emoji: GuildWelcomeChannelEmoji) -> Self {
        match emoji {
            GuildWelcomeChannelEmoji::Unicode(name) => {
                self.emoji_name = Some(name);
            },
            GuildWelcomeChannelEmoji::Custom {
                id,
                name,
            } => {
                self.emoji_id = Some(id);
                self.emoji_name = Some(name);
            },
        }

        self
    }
}
