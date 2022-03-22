use std::fmt;

use super::ArgumentConvert;
use crate::{model::prelude::*, prelude::*};

/// Error that can be returned from [`User::convert`].
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum UserParseError {
    /// The provided user string failed to parse, or the parsed result cannot be found in the
    /// guild cache data.
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
/// 2. [Lookup by mention](`crate::utils::parse_username`).
/// 3. [Lookup by name#discrim](`crate::utils::parse_user_tag`).
/// 4. Lookup by name
#[cfg(feature = "cache")]
#[async_trait::async_trait]
impl ArgumentConvert for User {
    type Err = UserParseError;

    async fn convert(
        ctx: &Context,
        guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let users = &ctx.cache.users;

        let lookup_by_id = || users.get(&UserId(s.parse().ok()?)).map(|u| u.clone());

        let lookup_by_mention =
            || users.get(&UserId(crate::utils::parse_username(s)?)).map(|u| u.clone());

        let lookup_by_name_and_discrim = || {
            let (name, discrim) = crate::utils::parse_user_tag(s)?;
            users.iter().find_map(|m| {
                let user = m.value();
                if user.discriminator == discrim && user.name.eq_ignore_ascii_case(name) {
                    Some(user.clone())
                } else {
                    None
                }
            })
        };

        let lookup_by_name = || {
            users.iter().find_map(|m| {
                let user = m.value();
                if user.name == s {
                    Some(user.clone())
                } else {
                    None
                }
            })
        };

        // Try to look up in global user cache via a variety of methods
        if let Some(user) = lookup_by_id()
            .or_else(lookup_by_mention)
            .or_else(lookup_by_name_and_discrim)
            .or_else(lookup_by_name)
        {
            return Ok(user);
        }

        // If not successful, convert as a Member which uses HTTP endpoints instead of cache
        if let Ok(member) = Member::convert(ctx, guild_id, channel_id, s).await {
            return Ok(member.user);
        }

        Err(UserParseError::NotFoundOrMalformed)
    }
}
