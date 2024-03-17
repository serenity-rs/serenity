use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Emoji::convert`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EmojiParseError {
    /// Parser was invoked outside a guild.
    OutsideGuild,
    /// Guild was not in cache, or guild HTTP request failed.
    FailedToRetrieveGuild,
    /// The provided emoji string failed to parse, or the parsed result cannot be found in the
    /// guild roles.
    NotFoundOrMalformed,
}

impl std::error::Error for EmojiParseError {}

impl fmt::Display for EmojiParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutsideGuild => f.write_str("Tried to find emoji outside a guild"),
            Self::FailedToRetrieveGuild => f.write_str("Could not retrieve guild data"),
            Self::NotFoundOrMalformed => f.write_str("Emoji not found or unknown format"),
        }
    }
}

/// Look up a [`Emoji`].
///
/// Requires the cache feature to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. [Lookup by extracting ID from the emoji](`crate::utils::parse_emoji`).
/// 3. Lookup by name.
#[async_trait::async_trait]
impl ArgumentConvert for Emoji {
    type Err = EmojiParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        // Get Guild or PartialGuild
        let guild_id = guild_id.ok_or(EmojiParseError::OutsideGuild)?;
        let guild = guild_id
            .to_partial_guild(&ctx)
            .await
            .map_err(|_| EmojiParseError::FailedToRetrieveGuild)?;

        let direct_id = s.parse().ok();
        let id_from_mention = crate::utils::parse_emoji(s).map(|e| e.id);

        if let Some(emoji_id) = direct_id.or(id_from_mention) {
            if let Some(emoji) = guild.emojis.get(&emoji_id).cloned() {
                return Ok(emoji);
            }
        }

        if let Some(emoji) =
            guild.emojis.iter().find(|emoji| emoji.name.eq_ignore_ascii_case(s)).cloned()
        {
            return Ok(emoji);
        }

        Err(EmojiParseError::NotFoundOrMalformed)
    }
}
