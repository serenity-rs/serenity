use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Channel::convert`].
#[non_exhaustive]
#[derive(Debug)]
pub enum ChannelParseError {
    /// When channel retrieval via HTTP failed
    Http(SerenityError),
    /// The provided channel string failed to parse, or the parsed result cannot be found in the
    /// cache.
    NotFoundOrMalformed,
}

impl std::error::Error for ChannelParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::NotFoundOrMalformed => None,
        }
    }
}

impl fmt::Display for ChannelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(_) => f.write_str("Failed to request channel via HTTP"),
            Self::NotFoundOrMalformed => f.write_str("Channel not found or unknown format"),
        }
    }
}

fn channel_belongs_to_guild(channel: &Channel, guild: GuildId) -> bool {
    match channel {
        Channel::Guild(channel) => channel.guild_id == guild,
        Channel::Private(_channel) => false,
    }
}

async fn lookup_channel_global(
    ctx: impl CacheHttp,
    guild_id: Option<GuildId>,
    s: &str,
) -> Result<Channel, ChannelParseError> {
    if let Some(channel_id) = s.parse().ok().or_else(|| crate::utils::parse_channel_mention(s)) {
        return channel_id.to_channel(&ctx).await.map_err(ChannelParseError::Http);
    }

    let guild_id = guild_id.ok_or(ChannelParseError::NotFoundOrMalformed)?;

    #[cfg(feature = "cache")]
    if let Some(cache) = ctx.cache() {
        if let Some(guild) = cache.guild(guild_id) {
            let channel = guild.channels.iter().find(|c| c.name.eq_ignore_ascii_case(s));
            if let Some(channel) = channel {
                return Ok(Channel::Guild(channel.clone()));
            }
        }

        return Err(ChannelParseError::NotFoundOrMalformed);
    }

    let channels = ctx.http().get_channels(guild_id).await.map_err(ChannelParseError::Http)?;
    if let Some(channel) = channels.into_iter().find(|c| c.name.eq_ignore_ascii_case(s)) {
        Ok(Channel::Guild(channel))
    } else {
        Err(ChannelParseError::NotFoundOrMalformed)
    }
}

/// Look up a Channel by a string case-insensitively.
///
/// Lookup are done via local guild. If in DMs, the global cache is used instead.
///
/// The cache feature needs to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. [Lookup by mention](`crate::utils::parse_channel_mention`).
/// 3. Lookup by name.
#[async_trait::async_trait]
impl ArgumentConvert for Channel {
    type Err = ChannelParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let channel = lookup_channel_global(&ctx, guild_id, s).await?;

        // Don't yield for other guilds' channels
        if let Some(guild_id) = guild_id {
            if !channel_belongs_to_guild(&channel, guild_id) {
                return Err(ChannelParseError::NotFoundOrMalformed);
            }
        }

        Ok(channel)
    }
}

/// Error that can be returned from [`GuildChannel::convert`].
#[non_exhaustive]
#[derive(Debug)]
pub enum GuildChannelParseError {
    /// When channel retrieval via HTTP failed
    Http(SerenityError),
    /// The provided channel string failed to parse, or the parsed result cannot be found in the
    /// cache.
    NotFoundOrMalformed,
    /// When the referenced channel is not a guild channel
    NotAGuildChannel,
}

impl std::error::Error for GuildChannelParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::NotFoundOrMalformed | Self::NotAGuildChannel => None,
        }
    }
}

impl fmt::Display for GuildChannelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(_) => f.write_str("Failed to request channel via HTTP"),
            Self::NotFoundOrMalformed => f.write_str("Channel not found or unknown format"),
            Self::NotAGuildChannel => f.write_str("Channel is not a guild channel"),
        }
    }
}

/// Look up a GuildChannel by a string case-insensitively.
///
/// Lookup is done by the global cache, hence the cache feature needs to be enabled.
///
/// For more information, see the ArgumentConvert implementation for [`Channel`]
#[async_trait::async_trait]
impl ArgumentConvert for GuildChannel {
    type Err = GuildChannelParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        match Channel::convert(&ctx, guild_id, channel_id, s).await {
            Ok(Channel::Guild(channel)) => Ok(channel),
            Ok(_) => Err(GuildChannelParseError::NotAGuildChannel),
            Err(ChannelParseError::Http(e)) => Err(GuildChannelParseError::Http(e)),
            Err(ChannelParseError::NotFoundOrMalformed) => {
                Err(GuildChannelParseError::NotFoundOrMalformed)
            },
        }
    }
}
