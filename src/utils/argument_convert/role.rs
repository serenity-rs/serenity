use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Role::convert`].
#[non_exhaustive]
#[derive(Debug)]
pub enum RoleParseError {
    /// When the operation was invoked outside a guild.
    NotInGuild,
    /// When the guild's roles were not found in cache.
    NotInCache,
    /// HTTP error while retrieving guild roles.
    Http(SerenityError),
    /// The provided channel string failed to parse, or the parsed result cannot be found in the
    /// cache.
    NotFoundOrMalformed,
}

impl std::error::Error for RoleParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::NotFoundOrMalformed | Self::NotInGuild | Self::NotInCache => None,
        }
    }
}

impl fmt::Display for RoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInGuild => f.write_str("Must invoke this operation in a guild"),
            Self::NotInCache => f.write_str("Guild's roles were not found in cache"),
            Self::Http(_) => f.write_str("Failed to retrieve roles via HTTP"),
            Self::NotFoundOrMalformed => f.write_str("Role not found or unknown format"),
        }
    }
}

/// Look up a [`Role`] by a string case-insensitively.
///
/// Requires the cache feature to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID
/// 2. [Lookup by mention](`crate::utils::parse_role_mention`).
/// 3. Lookup by name (case-insensitive)
#[async_trait::async_trait]
impl ArgumentConvert for Role {
    type Err = RoleParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let guild_id = guild_id.ok_or(RoleParseError::NotInGuild)?;

        #[cfg(feature = "cache")]
        let guild;

        #[cfg(feature = "cache")]
        let roles = {
            let cache = ctx.cache().ok_or(RoleParseError::NotInCache)?;
            guild = cache.guild(guild_id).ok_or(RoleParseError::NotInCache)?;
            &guild.roles
        };

        #[cfg(not(feature = "cache"))]
        let roles = ctx.http().get_guild_roles(guild_id).await.map_err(RoleParseError::Http)?;

        if let Some(role_id) = s.parse().ok().or_else(|| crate::utils::parse_role_mention(s)) {
            #[cfg(feature = "cache")]
            if let Some(role) = roles.get(&role_id) {
                return Ok(role.clone());
            }
            #[cfg(not(feature = "cache"))]
            if let Some(role) = roles.iter().find(|role| role.id == role_id) {
                return Ok(role.clone());
            }
        }

        #[cfg(feature = "cache")]
        if let Some(role) = roles.iter().find(|role| role.name.eq_ignore_ascii_case(s)) {
            return Ok(role.clone());
        }
        #[cfg(not(feature = "cache"))]
        if let Some(role) = roles.into_iter().find(|role| role.name.eq_ignore_ascii_case(s)) {
            return Ok(role);
        }

        Err(RoleParseError::NotFoundOrMalformed)
    }
}
