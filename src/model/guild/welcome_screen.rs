use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::id::{ChannelId, EmojiId};

/// Information relating to a guild's welcome screen.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#welcome-screen-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildWelcomeScreen {
    /// The server description shown in the welcome screen.
    pub description: Option<String>,
    /// The channels shown in the welcome screen.
    ///
    /// **Note**: There can only be only up to 5 channels.
    pub welcome_channels: Vec<GuildWelcomeChannel>,
}

/// A channel shown in the [`GuildWelcomeScreen`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#welcome-screen-object-welcome-screen-channel-structure).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct GuildWelcomeChannel {
    /// The channel Id.
    pub channel_id: ChannelId,
    /// The description shown for the channel.
    pub description: String,
    /// The emoji shown, if there is one.
    pub emoji: Option<GuildWelcomeChannelEmoji>,
}

impl<'de> Deserialize<'de> for GuildWelcomeChannel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            channel_id: ChannelId,
            description: String,
            emoji_id: Option<EmojiId>,
            emoji_name: Option<String>,
        }
        let Helper {
            channel_id,
            description,
            emoji_id,
            emoji_name,
        } = Helper::deserialize(deserializer)?;

        let emoji = match (emoji_id, emoji_name) {
            (Some(id), Some(name)) => Some(GuildWelcomeChannelEmoji::Custom {
                id,
                name,
            }),
            (None, Some(name)) => Some(GuildWelcomeChannelEmoji::Unicode(name)),
            _ => None,
        };

        Ok(Self {
            channel_id,
            description,
            emoji,
        })
    }
}

impl Serialize for GuildWelcomeChannel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("GuildWelcomeChannel", 4)?;
        s.serialize_field("channel_id", &self.channel_id)?;
        s.serialize_field("description", &self.description)?;
        let (emoji_id, emoji_name) = match &self.emoji {
            Some(GuildWelcomeChannelEmoji::Custom {
                id,
                name,
            }) => (Some(id), Some(name)),
            Some(GuildWelcomeChannelEmoji::Unicode(name)) => (None, Some(name)),
            None => (None, None),
        };
        s.serialize_field("emoji_id", &emoji_id)?;
        s.serialize_field("emoji_name", &emoji_name)?;
        s.end()
    }
}

/// A [`GuildWelcomeScreen`] emoji.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#welcome-screen-object-welcome-screen-channel-structure).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum GuildWelcomeChannelEmoji {
    /// A custom emoji.
    Custom { id: EmojiId, name: String },
    /// A unicode emoji.
    Unicode(String),
}
