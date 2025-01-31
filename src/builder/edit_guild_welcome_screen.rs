use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
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
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    welcome_channels: Cow<'a, [CreateGuildWelcomeChannel<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,

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
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_welcome_channel(mut self, channel: CreateGuildWelcomeChannel<'a>) -> Self {
        self.welcome_channels.to_mut().push(channel);
        self
    }

    /// Channels linked in the welcome screen and their display options
    pub fn set_welcome_channels(
        mut self,
        channels: impl Into<Cow<'a, [CreateGuildWelcomeChannel<'a>]>>,
    ) -> Self {
        self.welcome_channels = channels.into();
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
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
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<GuildWelcomeScreen> {
        http.edit_guild_welcome_screen(guild_id, &self, self.audit_log_reason).await
    }
}

/// A builder for creating a [`GuildWelcomeChannel`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#welcome-screen-object-welcome-screen-channel-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateGuildWelcomeChannel<'a> {
    channel_id: ChannelId,
    emoji_name: Option<String>,
    emoji_id: Option<EmojiId>,
    description: Cow<'a, str>,
}

impl<'a> CreateGuildWelcomeChannel<'a> {
    pub fn new(channel_id: ChannelId, description: impl Into<Cow<'a, str>>) -> Self {
        Self {
            channel_id,
            emoji_id: None,
            emoji_name: None,
            description: description.into(),
        }
    }

    /// The Id of the channel to show.
    pub fn id(mut self, id: ChannelId) -> Self {
        self.channel_id = id;
        self
    }

    /// The description shown for the channel.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// The emoji shown for the channel.
    pub fn emoji(mut self, emoji: GuildWelcomeChannelEmoji) -> Self {
        match emoji {
            GuildWelcomeChannelEmoji::Custom {
                id,
                name,
            } => {
                self.emoji_id = Some(id);
                self.emoji_name = Some(name.into());
            },
            GuildWelcomeChannelEmoji::Unicode(name) => {
                self.emoji_id = None;
                self.emoji_name = Some(name.into());
            },
        }

        self
    }
}
