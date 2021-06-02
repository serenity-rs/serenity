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
pub trait Parse: Sized {
    /// The associated error which can be returned from parsing.
    type Err;

    /// Parses a string `s` as a command parameter of this type.
    async fn parse(ctx: &Context, msg: &Message, s: &str) -> Result<Self, Self::Err>;
}

#[async_trait::async_trait]
impl<T: std::str::FromStr> Parse for T {
    type Err = <T as std::str::FromStr>::Err;

    async fn parse(_: &Context, _: &Message, s: &str) -> Result<Self, Self::Err> {
        T::from_str(s)
    }
}

/// Error that can be returned from [`Member::parse`].
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum MemberParseError {
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
			Self::GuildNotInCache => write!(f, "Guild is not in cache"),
			Self::NotFoundOrMalformed => write!(f, "Provided member was not found or provided string did not adhere to any known guild member format"),
		}
    }
}

/// Look up a guild member by a string case-insensitively.
///
/// Requires the cache feature to be enabled.
///
/// The lookup strategy is as follows (in order):
/// 1. Lookup by ID.
/// 2. Lookup by mention.
/// 3. Lookup by name#discrim
/// 4. Lookup by name
/// 5. Lookup by nickname
#[cfg(feature = "cache")]
#[async_trait::async_trait]
impl Parse for Member {
    type Err = MemberParseError;

    async fn parse(ctx: &Context, msg: &Message, s: &str) -> Result<Self, Self::Err> {
        let guild = msg.guild(&ctx.cache).await.ok_or(MemberParseError::GuildNotInCache)?;

        let lookup_by_id = || guild.members.get(&UserId(s.parse().ok()?));

        let lookup_by_mention = || {
            guild.members.get(&UserId(
                s.strip_prefix("<@")?.trim_start_matches('!').strip_suffix('>')?.parse().ok()?,
            ))
        };

        let lookup_by_name_and_discrim = || {
            let pound_sign = s.find('#')?;
            let name = &s[..pound_sign];
            let discrim = s[(pound_sign + 1)..].parse::<u16>().ok()?;
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

/// Error that can be returned from [`Message::parse`].
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
/// 1. Lookup by "{channel ID}-{message ID}" (retrieved by shift-clicking on "Copy ID")
/// 2. Lookup by message ID (the message must be in the context channel)
/// 3. Lookup by message URL
#[async_trait::async_trait]
impl Parse for Message {
    type Err = MessageParseError;

    async fn parse(ctx: &Context, msg: &Message, s: &str) -> Result<Self, Self::Err> {
        let extract_from_id_pair = || {
            let mut parts = s.splitn(2, '-');
            let channel_id = ChannelId(parts.next()?.parse().ok()?);
            let message_id = MessageId(parts.next()?.parse().ok()?);
            Some((channel_id, message_id))
        };

        let extract_from_message_id = || Some((msg.channel_id, MessageId(s.parse().ok()?)));

        let extract_from_message_url = || {
            let mut parts = s.strip_prefix("https://discord.com/channels/")?.splitn(3, '/');
            let _guild_id = GuildId(parts.next()?.parse().ok()?);
            let channel_id = ChannelId(parts.next()?.parse().ok()?);
            let message_id = MessageId(parts.next()?.parse().ok()?);
            Some((channel_id, message_id))
        };

        let (channel_id, message_id) = extract_from_id_pair()
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
