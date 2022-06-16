use crate::model::guild::GuildWelcomeChannelEmoji;
use crate::model::id::{ChannelId, EmojiId};

/// A builder to specify the fields to edit in a [`GuildWelcomeScreen`].
///
/// [`GuildWelcomeScreen`]: crate::model::guild::GuildWelcomeScreen
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditGuildWelcomeScreen {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    welcome_channels: Vec<CreateGuildWelcomeChannel>,
}

impl EditGuildWelcomeScreen {
    /// Whether the welcome screen is enabled or not.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = Some(enabled);

        self
    }

    /// The server description shown in the welcome screen.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());

        self
    }

    pub fn create_welcome_channel<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateGuildWelcomeChannel) -> &mut CreateGuildWelcomeChannel,
    {
        let mut data = CreateGuildWelcomeChannel::default();
        f(&mut data);

        self.add_welcome_channel(data)
    }

    pub fn add_welcome_channel(&mut self, channel: CreateGuildWelcomeChannel) -> &mut Self {
        self.welcome_channels.push(channel);
        self
    }

    pub fn set_welcome_channels(&mut self, channels: Vec<CreateGuildWelcomeChannel>) -> &mut Self {
        self.welcome_channels = channels;
        self
    }
}

/// A builder for creating a [`GuildWelcomeChannel`].
///
/// [`GuildWelcomeChannel`]: crate::model::guild::GuildWelcomeChannel
#[derive(Clone, Debug, Default, Serialize)]
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
    pub fn id(&mut self, id: impl Into<ChannelId>) -> &mut Self {
        self.channel_id = Some(id.into());

        self
    }

    /// The description shown for the channel. It is required.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());

        self
    }

    /// The emoji shown for the channel.
    pub fn emoji(&mut self, emoji: GuildWelcomeChannelEmoji) -> &mut Self {
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
