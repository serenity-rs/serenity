use super::ArgumentConvert;
use crate::{model::prelude::*, prelude::*};

/// Error that can be returned from [`Member::convert`].
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

impl std::fmt::Display for MemberParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutsideGuild => write!(f, "Tried to find member outside a guild"),
            Self::GuildNotInCache => write!(f, "Guild is not in cache"),
            Self::NotFoundOrMalformed => write!(f, "Member not found or unknown format"),
        }
    }
}

/// Look up a guild member by a string case-insensitively.
///
/// Requires the cache feature to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. [Lookup by mention](`super::parse_username`).
/// 3. [Lookup by name#discrim](`parse_user_tag`).
/// 4. Lookup by name
/// 5. Lookup by nickname
#[cfg(feature = "cache")]
#[async_trait::async_trait]
impl ArgumentConvert for Member {
    type Err = MemberParseError;

    async fn convert(
        ctx: &Context,
        guild_id: Option<GuildId>,
        _channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let guild = guild_id
            .ok_or(MemberParseError::OutsideGuild)?
            .to_guild_cached(ctx)
            .await
            .ok_or(MemberParseError::GuildNotInCache)?;

        let lookup_by_id = || guild.members.get(&UserId(s.parse().ok()?));

        let lookup_by_mention = || guild.members.get(&UserId(crate::utils::parse_username(s)?));

        let lookup_by_name_and_discrim = || {
            let (name, discrim) = crate::utils::parse_user_tag(s)?;
            guild.members.values().find(|member| {
                member.user.discriminator == discrim && member.user.name.eq_ignore_ascii_case(name)
            })
        };

        let lookup_by_name = || guild.members.values().find(|member| member.user.name == s);

        let lookup_by_nickname = || {
            guild.members.values().find(|member| match &member.nick {
                Some(nick) => nick.eq_ignore_ascii_case(s),
                None => false,
            })
        };

        lookup_by_id()
            .or_else(lookup_by_mention)
            .or_else(lookup_by_name_and_discrim)
            .or_else(lookup_by_name)
            .or_else(lookup_by_nickname)
            .cloned()
            .ok_or(MemberParseError::NotFoundOrMalformed)
    }
}
