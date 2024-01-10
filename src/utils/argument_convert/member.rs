use std::fmt;

use super::ArgumentConvert;
use crate::model::prelude::*;
use crate::prelude::*;

/// Error that can be returned from [`Member::convert`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemberParseError {
    /// Parser was invoked outside a guild.
    OutsideGuild,
    /// The guild in which the parser was invoked is not in cache.
    GuildNotInCache,
    /// The provided member string failed to parse, or the parsed result cannot be found in the
    /// guild cache data.
    NotFoundOrMalformed,
}

impl std::error::Error for MemberParseError {}

impl fmt::Display for MemberParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutsideGuild => f.write_str("Tried to find member outside a guild"),
            Self::GuildNotInCache => f.write_str("Guild is not in cache"),
            Self::NotFoundOrMalformed => f.write_str("Member not found or unknown format"),
        }
    }
}

/// Look up a guild member by a string case-insensitively.
///
/// Requires the cache feature to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. [Lookup by mention](`crate::utils::parse_user_mention`).
/// 3. [Lookup by name#discrim](`crate::utils::parse_user_tag`).
/// 4. Lookup by name
/// 5. Lookup by nickname
#[async_trait::async_trait]
impl ArgumentConvert for Member {
    type Err = MemberParseError;

    async fn convert(
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let guild_id = guild_id.ok_or(MemberParseError::OutsideGuild)?;

        // DON'T use guild.members: it's only populated when guild presences intent is enabled!

        // If string is a raw user ID or a mention
        if let Some(user_id) = s.parse().ok().or_else(|| crate::utils::parse_user_mention(s)) {
            if let Ok(member) = guild_id.member(&ctx, user_id).await {
                return Ok(member);
            }
        }

        // Following code is inspired by discord.py's MemberConvert::query_member_named

        // If string is a username+discriminator
        let limit = nonmax::NonMaxU16::new(100);
        if let Some((name, discrim)) = crate::utils::parse_user_tag(s) {
            if let Ok(member_results) = guild_id.search_members(ctx.http(), name, limit).await {
                if let Some(member) = member_results.into_iter().find(|m| {
                    m.user.name.eq_ignore_ascii_case(name) && m.user.discriminator == discrim
                }) {
                    return Ok(member);
                }
            }
        }

        // If string is username or nickname
        if let Ok(member_results) = guild_id.search_members(ctx.http(), s, limit).await {
            if let Some(member) = member_results.into_iter().find(|m| {
                m.user.name.eq_ignore_ascii_case(s)
                    || m.nick.as_ref().is_some_and(|nick| nick.eq_ignore_ascii_case(s))
            }) {
                return Ok(member);
            }
        }

        Err(MemberParseError::NotFoundOrMalformed)
    }
}
