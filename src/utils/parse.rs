use crate::model::prelude::*;
use crate::prelude::*;

/// Parse a value from a string in context of a received message.
///
/// This trait is a superset of [`std::str::FromStr`]. The
/// difference is that this trait aims to support serenity-specific Discord types like [`Member`]
/// or [`Message`].
///
/// Trait implementations may do network requests as part of their parsing procedure.
///
/// Useful for implementing argument parsing in command frameworks.
#[async_trait::async_trait]
pub trait ArgumentConvert: Sized {
    /// The associated error which can be returned from parsing.
    type Err;

    /// Parses a string `s` as a command parameter of this type.
    async fn convert(
        ctx: &Context,
        guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err>;
}

#[async_trait::async_trait]
impl<T: std::str::FromStr> ArgumentConvert for T {
    type Err = <T as std::str::FromStr>::Err;

    async fn convert(
        _: &Context,
        _: Option<GuildId>,
        _: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        T::from_str(s)
    }
}

// The following few parse_XXX methods are in here (parse.rs) because they need to be gated
// behind the model feature and it's just convenient to put them here then

/// Retrieves the username and discriminator out of a user tag (`name#discrim`).
///
/// If the user tag is invalid, None is returned.
///
/// # Examples
/// ```rust
/// use serenity::utils::parse_user_tag;
///
/// assert_eq!(parse_user_tag("kangalioo#9108"), Some(("kangalioo", 9108)));
/// assert_eq!(parse_user_tag("kangalioo#10108"), None);
/// ```
pub fn parse_user_tag(s: &str) -> Option<(&str, u16)> {
    let pound_sign = s.find('#')?;
    let name = &s[..pound_sign];
    let discrim = s[(pound_sign + 1)..].parse::<u16>().ok()?;
    if discrim > 9999 {
        return None;
    }
    Some((name, discrim))
}

/// Retrieves IDs from "{channel ID}-{message ID}" (retrieved by shift-clicking on "Copy ID").
///
/// If the string is invalid, None is returned.
///
/// # Examples
/// ```rust
/// use serenity::model::prelude::*;
/// use serenity::utils::parse_message_id_pair;
///
/// assert_eq!(
///     parse_message_id_pair("673965002805477386-842482646604972082"),
///     Some(ChannelId(673965002805477386), GuildId(842482646604972082)),
/// );
/// assert_eq!(
///     parse_message_id_pair("673965002805477386-842482646604972082-472029906943868929"),
///     None,
/// );
/// ```
pub fn parse_message_id_pair(s: &str) -> Option<(ChannelId, MessageId)> {
    let mut parts = s.splitn(2, '-');
    let channel_id = ChannelId(parts.next()?.parse().ok()?);
    let message_id = MessageId(parts.next()?.parse().ok()?);
    Some((channel_id, message_id))
}

/// Retrieves guild, channel, and message ID from a message URL.
///
/// If the URL is malformed, None is returned.
///
/// # Examples
/// ```rust
/// use serenity::model::prelude::*;
/// use serenity::utils::parse_message_url;
///
/// assert_eq!(
///     parse_message_url(
///         "https://discord.com/channels/381880193251409931/381880193700069377/806164913558781963"
///     ),
///     Some(
///         GuildId(381880193251409931),
///         ChannelId(381880193700069377),
///         MessageId(806164913558781963),
///     ),
/// );
/// assert_eq!(parse_message_url("https://google.com"), None);
/// ```
pub fn parse_message_url(s: &str) -> Option<(GuildId, ChannelId, MessageId)> {
    let mut parts = s.strip_prefix("https://discord.com/channels/")?.splitn(3, '/');
    let guild_id = GuildId(parts.next()?.parse().ok()?);
    let channel_id = ChannelId(parts.next()?.parse().ok()?);
    let message_id = MessageId(parts.next()?.parse().ok()?);
    Some((guild_id, channel_id, message_id))
}

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

        let lookup_by_mention = || guild.members.get(&UserId(super::parse_username(s)?));

        let lookup_by_name_and_discrim = || {
            let (name, discrim) = parse_user_tag(s)?;
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
            Self::Malformed => None,
            Self::Http(e) => Some(e),
            Self::HttpNotAvailable => None,
        }
    }
}

impl std::fmt::Display for MessageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => {
                write!(f, "Provided string did not adhere to any known guild message format")
            },
            Self::Http(e) => write!(f, "Failed to request message data via HTTP: {}", e),
            Self::HttpNotAvailable => write!(
                f,
                "Gateway feature is disabled and the required information was not in cache"
            ),
        }
    }
}

/// Look up a message by a string.
///
/// The lookup strategy is as follows (in order):
/// 1. [Lookup by "{channel ID}-{message ID}"](`parse_message_id_pair`) (retrieved by shift-clicking on "Copy ID")
/// 2. Lookup by message ID (the message must be in the context channel)
/// 3. [Lookup by message URL](`parse_message_url`)
#[async_trait::async_trait]
impl ArgumentConvert for Message {
    type Err = MessageParseError;

    async fn convert(
        ctx: &Context,
        _guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let extract_from_message_id = || Some((channel_id?, MessageId(s.parse().ok()?)));

        let extract_from_message_url = || {
            let (_guild_id, channel_id, message_id) = parse_message_url(s)?;
            Some((channel_id, message_id))
        };

        let (channel_id, message_id) = parse_message_id_pair(s)
            .or_else(extract_from_message_id)
            .or_else(extract_from_message_url)
            .ok_or(MessageParseError::Malformed)?;

        #[cfg(feature = "cache")]
        if let Some(msg) = ctx.cache.message(channel_id, message_id).await {
            return Ok(msg);
        }

        if cfg!(feature = "http") {
            ctx.http.get_message(channel_id.0, message_id.0).await.map_err(MessageParseError::Http)
        } else {
            Err(MessageParseError::HttpNotAvailable)
        }
    }
}
