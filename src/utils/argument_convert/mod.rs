mod member;
pub use member::*;

mod message;
pub use message::*;

mod user;
pub use user::*;

mod channel;
pub use channel::*;

// From HTTP you can only get PartialGuild; for Guild you need gateway and cache
#[cfg(feature = "cache")]
mod guild;
#[cfg(feature = "cache")]
pub use guild::*;

mod role;
pub use role::*;

mod emoji;
pub use emoji::*;

use super::DOMAINS;
use crate::model::prelude::*;
use crate::prelude::*;

/// Parse a value from a string in context of a received message.
///
/// This trait is a superset of [`std::str::FromStr`]. The difference is that this trait aims to
/// support serenity-specific Discord types like [`Member`] or [`Message`].
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
        ctx: impl CacheHttp,
        guild_id: Option<GuildId>,
        channel_id: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err>;
}

#[async_trait::async_trait]
impl<T: std::str::FromStr> ArgumentConvert for T {
    type Err = <T as std::str::FromStr>::Err;

    async fn convert(
        _: impl CacheHttp,
        _: Option<GuildId>,
        _: Option<ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        T::from_str(s)
    }
}

// The following few parse_XXX methods are in here (parse.rs) because they need to be gated behind
// the model feature and it's just convenient to put them here for that

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
///     Some((ChannelId::new(673965002805477386), MessageId::new(842482646604972082))),
/// );
/// assert_eq!(
///     parse_message_id_pair("673965002805477386-842482646604972082-472029906943868929"),
///     None,
/// );
/// ```
#[must_use]
pub fn parse_message_id_pair(s: &str) -> Option<(ChannelId, MessageId)> {
    let mut parts = s.splitn(2, '-');
    let channel_id = parts.next()?.parse().ok()?;
    let message_id = parts.next()?.parse().ok()?;
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
///     Some((
///         GuildId::new(381880193251409931),
///         ChannelId::new(381880193700069377),
///         MessageId::new(806164913558781963),
///     )),
/// );
/// assert_eq!(
///     parse_message_url(
///         "https://canary.discord.com/channels/381880193251409931/381880193700069377/806164913558781963"
///     ),
///     Some((
///         GuildId::new(381880193251409931),
///         ChannelId::new(381880193700069377),
///         MessageId::new(806164913558781963),
///     )),
/// );
/// assert_eq!(parse_message_url("https://google.com"), None);
/// ```
#[must_use]
pub fn parse_message_url(s: &str) -> Option<(GuildId, ChannelId, MessageId)> {
    use aformat::{CapStr, aformat};

    for domain in DOMAINS {
        let prefix = aformat!("https://{}/channels/", CapStr::<MAX_DOMAIN_LEN>(domain));
        if let Some(parts) = s.strip_prefix(prefix.as_str()) {
            let mut parts = parts.splitn(3, '/');

            let guild_id = parts.next()?.parse().ok()?;
            let channel_id = parts.next()?.parse().ok()?;
            let message_id = parts.next()?.parse().ok()?;
            return Some((guild_id, channel_id, message_id));
        }
    }
    None
}

/// Retrieves guild, and channel ID from a channel URL.
///
/// If the URL is malformed, None is returned.
///
/// # Examples
/// ```rust
/// use serenity::model::prelude::*;
/// use serenity::utils::parse_channel_url;
///
/// assert_eq!(
///     parse_channel_url("https://discord.com/channels/381880193251409931/381880193700069377"),
///     Some((GuildId::new(381880193251409931), ChannelId::new(381880193700069377),)),
/// );
/// assert_eq!(
///     parse_channel_url(
///         "https://canary.discord.com/channels/381880193251409931/381880193700069377"
///     ),
///     Some((GuildId::new(381880193251409931), ChannelId::new(381880193700069377),)),
/// );
/// assert_eq!(parse_channel_url("https://google.com"), None);
/// ```
#[must_use]
pub fn parse_channel_url(s: &str) -> Option<(GuildId, ChannelId)> {
    use aformat::{CapStr, aformat};

    for domain in DOMAINS {
        let prefix = aformat!("https://{}/channels/", CapStr::<MAX_DOMAIN_LEN>(domain));
        if let Some(parts) = s.strip_prefix(prefix.as_str()) {
            let mut parts = parts.splitn(2, '/');

            let guild_id = parts.next()?.parse().ok()?;
            let channel_id = parts.next()?.parse().ok()?;
            return Some((guild_id, channel_id));
        }
    }
    None
}

const MAX_DOMAIN_LEN: usize = {
    let mut max_len = 0;
    let mut i = 0;

    while i < DOMAINS.len() {
        let cur_len = DOMAINS[i].len();
        if cur_len > max_len {
            max_len = cur_len;
        }

        i += 1;
    }

    max_len
};
