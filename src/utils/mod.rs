//! A set of utilities to help with common use cases that are not required to fully use the
//! library.

#[cfg(feature = "client")]
mod argument_convert;
#[cfg(feature = "cache")]
mod content_safe;
mod custom_message;
mod formatted_timestamp;
mod message_builder;
#[cfg(feature = "collector")]
mod quick_modal;

pub mod token;

use std::num::NonZeroU16;

#[cfg(feature = "client")]
pub use argument_convert::*;
#[cfg(feature = "cache")]
pub use content_safe::*;
pub use formatted_timestamp::*;
#[cfg(feature = "collector")]
pub use quick_modal::*;
use tracing::warn;
use url::Url;

pub use self::custom_message::CustomMessage;
pub use self::message_builder::{Content, ContentModifier, EmbedMessageBuilding, MessageBuilder};
#[doc(inline)]
pub use self::token::validate as validate_token;
use crate::model::prelude::*;

/// Retrieves the "code" part of an invite out of a URL.
///
/// # Examples
///
/// Two formats of [invite][`RichInvite`] codes are supported, both regardless of protocol prefix.
/// Some examples:
///
/// 1. Retrieving the code from the URL `"https://discord.gg/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use serenity::utils;
///
/// let url = "https://discord.gg/0cDvIgU2voY8RSYL";
///
/// assert_eq!(utils::parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// 2. Retrieving the code from the URL `"http://discord.com/invite/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use serenity::utils;
///
/// let url = "http://discord.com/invite/0cDvIgU2voY8RSYL";
///
/// assert_eq!(utils::parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// [`RichInvite`]: crate::model::invite::RichInvite
#[must_use]
pub fn parse_invite(code: &str) -> &str {
    let code = code.trim_start_matches("http://").trim_start_matches("https://");
    let lower = code.to_lowercase();
    if lower.starts_with("discord.gg/") {
        &code[11..]
    } else if lower.starts_with("discord.com/invite/") {
        &code[19..]
    } else {
        code
    }
}

/// Retrieves the username and discriminator out of a user tag (`name#discrim`).
/// In order to accomodate next gen Discord usernames, this will also accept `name` style tags.
///
/// If the user tag is invalid, None is returned.
///
/// # Examples
/// ```rust
/// use std::num::NonZeroU16;
///
/// use serenity::utils::parse_user_tag;
///
/// assert_eq!(parse_user_tag("kangalioo#9108"), Some(("kangalioo", NonZeroU16::new(9108))));
/// assert_eq!(parse_user_tag("kangalioo#10108"), None);
/// assert_eq!(parse_user_tag("kangalioo"), Some(("kangalioo", None)));
/// ```
#[must_use]
pub fn parse_user_tag(s: &str) -> Option<(&str, Option<NonZeroU16>)> {
    if let Some((name, discrim)) = s.split_once('#') {
        let discrim: u16 = discrim.parse().ok()?;
        if discrim > 9999 {
            return None;
        }
        Some((name, NonZeroU16::new(discrim)))
    } else {
        Some((s, None))
    }
}

/// Retrieves an Id from a user mention.
///
/// If the mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`User`] mention:
///
/// ```rust
/// use serenity::model::id::UserId;
/// use serenity::utils::parse_user_mention;
///
/// // regular username mention
/// assert_eq!(parse_user_mention("<@114941315417899012>"), Some(UserId::new(114941315417899012)));
///
/// // nickname mention
/// assert_eq!(parse_user_mention("<@!114941315417899012>"), Some(UserId::new(114941315417899012)));
/// ```
///
/// Asserting that an invalid username or nickname mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_user_mention;
///
/// assert!(parse_user_mention("<@1149413154aa17899012").is_none());
/// assert!(parse_user_mention("<@!11494131541789a90b1c2").is_none());
/// ```
///
/// [`User`]: crate::model::user::User
#[must_use]
pub fn parse_user_mention(mention: &str) -> Option<UserId> {
    if mention.len() < 4 {
        return None;
    }

    let len = mention.len() - 1;
    if mention.starts_with("<@!") {
        mention[3..len].parse().ok()
    } else if mention.starts_with("<@") {
        mention[2..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves an Id from a role mention.
///
/// If the mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`Role`] mention:
///
/// ```rust
/// use serenity::model::id::RoleId;
/// use serenity::utils::parse_role_mention;
///
/// assert_eq!(parse_role_mention("<@&136107769680887808>"), Some(RoleId::new(136107769680887808)));
/// ```
///
/// Asserting that an invalid role mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_role_mention;
///
/// assert!(parse_role_mention("<@&136107769680887808").is_none());
/// ```
///
/// [`Role`]: crate::model::guild::Role
#[must_use]
pub fn parse_role_mention(mention: &str) -> Option<RoleId> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves an Id from a channel mention.
///
/// If the channel mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`Channel`] mention:
///
/// ```rust
/// use serenity::model::id::ChannelId;
/// use serenity::utils::parse_channel_mention;
///
/// assert_eq!(
///     parse_channel_mention("<#81384788765712384>"),
///     Some(ChannelId::new(81384788765712384))
/// );
/// ```
///
/// Asserting that an invalid channel mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_channel_mention;
///
/// assert!(parse_channel_mention("<#!81384788765712384>").is_none());
/// assert!(parse_channel_mention("<#81384788765712384").is_none());
/// ```
///
/// [`Channel`]: crate::model::channel::Channel
#[must_use]
pub fn parse_channel_mention(mention: &str) -> Option<ChannelId> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<#") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[2..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves the animated state, name and Id from an emoji mention, in the form of an
/// [`EmojiIdentifier`].
///
/// If the emoji usage is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Ensure that a valid [`Emoji`] usage is correctly parsed:
///
/// ```rust
/// use serenity::model::id::{EmojiId, GuildId};
/// use serenity::model::misc::EmojiIdentifier;
/// use serenity::utils::parse_emoji;
///
/// let emoji = parse_emoji("<:smugAnimeFace:302516740095606785>").unwrap();
/// assert_eq!(emoji.animated, false);
/// assert_eq!(emoji.id, EmojiId::new(302516740095606785));
/// assert_eq!(&*emoji.name, "smugAnimeFace");
/// ```
///
/// Asserting that an invalid emoji usage returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_emoji;
///
/// assert!(parse_emoji("<:smugAnimeFace:302516740095606785").is_none());
/// ```
///
/// [`Emoji`]: crate::model::guild::Emoji
#[must_use]
pub fn parse_emoji(mention: &str) -> Option<EmojiIdentifier> {
    let len = mention.len();
    if !(6..=56).contains(&len) {
        return None;
    }

    if (mention.starts_with("<:") || mention.starts_with("<a:")) && mention.ends_with('>') {
        let mut name = String::default();
        let mut id = String::default();
        let animated = &mention[1..3] == "a:";

        let start = if animated { 3 } else { 2 };

        for (i, x) in mention[start..].chars().enumerate() {
            if x == ':' {
                let from = i + start + 1;

                for y in mention[from..].chars() {
                    if y == '>' {
                        break;
                    }
                    id.push(y);
                }

                break;
            }
            name.push(x);
        }

        id.parse().ok().map(|id| EmojiIdentifier {
            name: name.trunc_into(),
            animated,
            id,
        })
    } else {
        None
    }
}

/// Turns a string into a vector of string arguments, splitting by spaces, but parsing content
/// within quotes as one individual argument.
///
/// # Examples
///
/// Parsing two quoted commands:
///
/// ```rust
/// use serenity::utils::parse_quotes;
///
/// let command = r#""this is the first" "this is the second""#;
/// let expected = vec!["this is the first".to_string(), "this is the second".to_string()];
///
/// assert_eq!(parse_quotes(command), expected);
/// ```
///
/// ```rust
/// use serenity::utils::parse_quotes;
///
/// let command = r#""this is a quoted command that doesn't have an ending quotation"#;
/// let expected =
///     vec!["this is a quoted command that doesn't have an ending quotation".to_string()];
///
/// assert_eq!(parse_quotes(command), expected);
/// ```
#[must_use]
pub fn parse_quotes(s: &str) -> Vec<String> {
    let mut args = vec![];
    let mut in_string = false;
    let mut escaping = false;
    let mut current_str = String::default();

    for x in s.chars() {
        if in_string {
            if x == '\\' && !escaping {
                escaping = true;
            } else if x == '"' && !escaping {
                if !current_str.is_empty() {
                    args.push(current_str);
                }

                current_str = String::default();
                in_string = false;
            } else {
                current_str.push(x);
                escaping = false;
            }
        } else if x == ' ' {
            if !current_str.is_empty() {
                args.push(current_str.clone());
            }

            current_str = String::default();
        } else if x == '"' {
            if !current_str.is_empty() {
                args.push(current_str.clone());
            }

            in_string = true;
            current_str = String::default();
        } else {
            current_str.push(x);
        }
    }

    if !current_str.is_empty() {
        args.push(current_str);
    }

    args
}

/// Discord's official domains. This is used in [`parse_webhook`] and in its corresponding test.
const DOMAINS: &[&str] = &[
    "discord.com",
    "canary.discord.com",
    "ptb.discord.com",
    "discordapp.com",
    "canary.discordapp.com",
    "ptb.discordapp.com",
];

/// Parses the id and token from a webhook url. Expects a [`url::Url`] rather than a [`&str`].
///
/// # Examples
///
/// ```rust
/// use serenity::utils;
///
/// let url_str = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let url = url_str.parse().unwrap();
/// let (id, token) = utils::parse_webhook(&url).unwrap();
///
/// assert_eq!(id, 245037420704169985);
/// assert_eq!(token, "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV");
/// ```
#[must_use]
pub fn parse_webhook(url: &Url) -> Option<(WebhookId, &str)> {
    let (webhook_id, token) = url.path().strip_prefix("/api/webhooks/")?.split_once('/')?;
    if !["http", "https"].contains(&url.scheme())
        || !DOMAINS.contains(&url.domain()?)
        || !(17..=20).contains(&webhook_id.len())
        || !(60..=68).contains(&token.len())
    {
        return None;
    }
    Some((webhook_id.parse().ok()?, token))
}

/// Calculates the Id of the shard responsible for a guild, given its Id and total number of shards
/// used.
///
/// # Examples
///
/// Retrieve the Id of the shard for a guild with Id `81384788765712384`, using 17 shards:
///
/// ```rust
/// use serenity::model::id::GuildId;
/// use serenity::utils;
///
/// let guild_id = GuildId::new(81384788765712384);
/// let shard_total = std::num::NonZeroU16::new(17).unwrap();
///
/// assert_eq!(utils::shard_id(guild_id, shard_total), 7);
/// ```
#[must_use]
pub fn shard_id(guild_id: GuildId, shard_count: NonZeroU16) -> u16 {
    ((guild_id.get() >> 22) % u64::from(shard_count.get())) as u16
}

pub(crate) fn check_shard_total(total_shards: u16) -> NonZeroU16 {
    NonZeroU16::new(total_shards).unwrap_or_else(|| {
        warn!("Invalid shard total provided ({total_shards}), defaulting to 1");
        NonZeroU16::MIN
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_invite_parser() {
        assert_eq!(parse_invite("https://discord.gg/abc"), "abc");
        assert_eq!(parse_invite("http://discord.gg/abc"), "abc");
        assert_eq!(parse_invite("discord.gg/abc"), "abc");
        assert_eq!(parse_invite("DISCORD.GG/ABC"), "ABC");
        assert_eq!(parse_invite("https://discord.com/invite/abc"), "abc");
        assert_eq!(parse_invite("http://discord.com/invite/abc"), "abc");
        assert_eq!(parse_invite("discord.com/invite/abc"), "abc");
    }

    #[test]
    fn test_username_parser() {
        assert_eq!(parse_user_mention("<@12345>").unwrap(), 12_345);
        assert_eq!(parse_user_mention("<@!12345>").unwrap(), 12_345);
    }

    #[test]
    fn role_parser() {
        assert_eq!(parse_role_mention("<@&12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_channel_parser() {
        assert_eq!(parse_channel_mention("<#12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_emoji_parser() {
        let emoji = parse_emoji("<:name:12345>").unwrap();
        assert_eq!(&*emoji.name, "name");
        assert_eq!(emoji.id, 12_345);
    }

    #[test]
    fn test_quote_parser() {
        let parsed = parse_quotes("a \"b c\" d\"e f\"  g");
        assert_eq!(parsed, ["a", "b c", "d", "e f", "g"]);
    }

    #[test]
    fn test_webhook_parser() {
        for domain in DOMAINS {
            let url = format!("https://{domain}/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV").parse().unwrap();
            let (id, token) = parse_webhook(&url).unwrap();
            assert_eq!(id, 245037420704169985);
            assert_eq!(
                token,
                "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV"
            );
        }
    }
}
