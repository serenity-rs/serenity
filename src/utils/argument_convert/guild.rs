use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Guild::convert`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GuildParseError {
    /// The provided guild string failed to parse, or the parsed result cannot be found in the
    /// cache.
    NotFoundOrMalformed,
    /// No cache, so no guild search could be done.
    NoCache,
}

impl std::error::Error for GuildParseError {}

impl fmt::Display for GuildParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFoundOrMalformed => f.write_str("Guild not found or unknown format"),
            Self::NoCache => f.write_str("No cached list of guilds was provided"),
        }
    }
}

/// Look up a Guild, either by ID or by a string case-insensitively.
///
/// Requires the cache feature to be enabled.
#[async_trait::async_trait]
impl ArgumentConvert for Guild {
    type Err = GuildParseError;

    async fn convert(
        ctx: impl CacheHttp,
        _guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let guilds = &ctx.cache().ok_or(GuildParseError::NoCache)?.guilds;

        let lookup_by_id = || guilds.get(&s.parse().ok()?).map(|g| g.clone());

        let lookup_by_name = || {
            guilds.iter().find_map(|m| {
                let guild = m.value();
                guild.name.eq_ignore_ascii_case(s).then(|| guild.clone())
            })
        };

        lookup_by_id().or_else(lookup_by_name).ok_or(GuildParseError::NotFoundOrMalformed)
    }
}
