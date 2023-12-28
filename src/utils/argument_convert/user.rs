use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`User::convert`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UserParseError {
    /// The provided user string failed to parse, or the parsed result cannot be found in the guild
    /// cache data.
    NotFoundOrMalformed,
}

impl std::error::Error for UserParseError {}

impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFoundOrMalformed => f.write_str("User not found or unknown format"),
        }
    }
}

/// Look up a user by a string case-insensitively.
///
/// Requires the cache feature to be enabled. If a user is not in cache, they will not be found!
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. [Lookup by mention](`crate::utils::parse_user_mention`).
/// 3. [Lookup by name#discrim](`crate::utils::parse_user_tag`).
/// 4. Lookup by name
#[async_trait::async_trait]
impl ArgumentConvert for User {
    type Err = UserParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        // Convert as a Member which uses HTTP endpoints instead of cache
        if let Ok(member) = Member::convert(&ctx, guild_id, channel_id, s).await {
            return Ok(member.user);
        }

        // If string is a raw user ID or a mention
        if let Some(user_id) = s.parse().ok().or_else(|| crate::utils::parse_user_mention(s)) {
            // Now, we can still try UserId::to_user because it works for all users from all guilds
            // the bot is joined
            if let Ok(user) = user_id.to_user(&ctx).await {
                return Ok(user);
            }
        }

        Err(UserParseError::NotFoundOrMalformed)
    }
}
