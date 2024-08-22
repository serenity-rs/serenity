use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Message::convert`].
#[non_exhaustive]
#[derive(Debug)]
pub enum MessageParseError {
    /// When the provided string does not adhere to any known guild message format
    Malformed,
    /// When message data retrieval via HTTP failed
    Http(SerenityError),
    /// When the `gateway` feature is disabled and the required information was not in cache.
    HttpNotAvailable,
}

impl std::error::Error for MessageParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::HttpNotAvailable | Self::Malformed => None,
        }
    }
}

impl fmt::Display for MessageParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => {
                f.write_str("Provided string did not adhere to any known guild message format")
            },
            Self::Http(_) => f.write_str("Failed to request message data via HTTP"),
            Self::HttpNotAvailable => f.write_str(
                "Gateway feature is disabled and the required information was not in cache",
            ),
        }
    }
}

/// Look up a message by a string.
///
/// The lookup strategy is as follows (in order):
/// 1. [Lookup by "{channel ID}-{message ID}"](`crate::utils::parse_message_id_pair`) (retrieved by
///    shift-clicking on "Copy ID")
/// 2. Lookup by message ID (the message must be in the context channel)
/// 3. [Lookup by message URL](`crate::utils::parse_message_url`)
#[async_trait::async_trait]
impl ArgumentConvert for Message {
    type Err = MessageParseError;

    async fn convert(
        ctx: impl CacheHttp,
        _guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let extract_from_message_id = || Some((channel_id?, s.parse().ok()?));

        let extract_from_message_url = || {
            let (_guild_id, channel_id, message_id) = super::parse_message_url(s)?;
            Some((channel_id, message_id))
        };

        let (channel_id, message_id) = super::parse_message_id_pair(s)
            .or_else(extract_from_message_id)
            .or_else(extract_from_message_url)
            .ok_or(MessageParseError::Malformed)?;

        channel_id.message(ctx, message_id).await.map_err(MessageParseError::Http)
    }
}
