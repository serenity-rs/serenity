//! A set of utilities to help with common use cases that are not required to
//! fully use the library.

#[cfg(feature = "client")]
mod argument_convert;
pub(crate) mod backports;
mod colour;
#[cfg(feature = "cache")]
mod content_safe;
mod custom_message;
mod message_builder;

pub mod token;

#[cfg(feature = "client")]
pub use argument_convert::*;
#[cfg(feature = "cache")]
pub use content_safe::*;
use url::Url;

pub use self::colour::{colours, Colour};
pub use self::custom_message::CustomMessage;
pub use self::message_builder::{Content, ContentModifier, EmbedMessageBuilding, MessageBuilder};
#[doc(inline)]
pub use self::token::{parse as parse_token, validate as validate_token};
pub type Color = Colour;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::internal::prelude::*;
use crate::model::id::EmojiId;
use crate::model::misc::EmojiIdentifier;

#[cfg(feature = "model")]
pub(crate) fn encode_image(raw: &[u8]) -> String {
    let mut encoded = base64::encode(raw);
    encoded.insert_str(0, "data:image/png;base64,");
    encoded
}

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
#[must_use]
pub fn parse_user_tag(s: &str) -> Option<(&str, u16)> {
    let (name, discrim) = s.split_once('#')?;
    let discrim = discrim.parse().ok()?;
    if discrim > 9999 {
        return None;
    }
    Some((name, discrim))
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
/// use serenity::utils::parse_username;
///
/// // regular username mention
/// assert_eq!(parse_username("<@114941315417899012>"), Some(114941315417899012));
///
/// // nickname mention
/// assert_eq!(parse_username("<@!114941315417899012>"), Some(114941315417899012));
/// ```
///
/// Asserting that an invalid username or nickname mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_username;
///
/// assert!(parse_username("<@1149413154aa17899012").is_none());
/// assert!(parse_username("<@!11494131541789a90b1c2").is_none());
/// ```
///
/// [`User`]: crate::model::user::User
pub fn parse_username(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@!") {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if mention.starts_with("<@") {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
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
/// use serenity::utils::parse_role;
///
/// assert_eq!(parse_role("<@&136107769680887808>"), Some(136107769680887808));
/// ```
///
/// Asserting that an invalid role mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_role;
///
/// assert!(parse_role("<@&136107769680887808").is_none());
/// ```
///
/// [`Role`]: crate::model::guild::Role
pub fn parse_role(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
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
/// use serenity::utils::parse_channel;
///
/// assert_eq!(parse_channel("<#81384788765712384>"), Some(81384788765712384));
/// ```
///
/// Asserting that an invalid channel mention returns [`None`]:
///
/// ```rust
/// use serenity::utils::parse_channel;
///
/// assert!(parse_channel("<#!81384788765712384>").is_none());
/// assert!(parse_channel("<#81384788765712384").is_none());
/// ```
///
/// [`Channel`]: crate::model::channel::Channel
pub fn parse_channel(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<#") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
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
/// let expected = EmojiIdentifier {
///     animated: false,
///     id: EmojiId(302516740095606785),
///     name: "smugAnimeFace".to_string(),
/// };
///
/// assert_eq!(parse_emoji("<:smugAnimeFace:302516740095606785>").unwrap(), expected);
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
pub fn parse_emoji(mention: impl AsRef<str>) -> Option<EmojiIdentifier> {
    let mention = mention.as_ref();

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

        match id.parse::<u64>() {
            Ok(x) => Some(EmojiIdentifier {
                animated,
                name,
                id: EmojiId(x),
            }),
            _ => None,
        }
    } else {
        None
    }
}

/// Reads an image from a path and encodes it into base64.
///
/// This can be used for methods like [`EditProfile::avatar`].
///
/// # Examples
///
/// Reads an image located at `./cat.png` into a base64-encoded string:
///
/// ```rust,no_run
/// use serenity::utils;
///
/// let image = utils::read_image("./cat.png").expect("Failed to read image");
/// ```
///
/// # Errors
///
/// Returns an [`Error::Io`] if the path does not exist.
///
/// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
/// [`Error::Io`]: crate::error::Error::Io
#[inline]
pub fn read_image<P: AsRef<Path>>(path: P) -> Result<String> {
    _read_image(path.as_ref())
}

fn _read_image(path: &Path) -> Result<String> {
    let mut v = Vec::default();
    let mut f = File::open(path)?;

    // errors here are intentionally ignored
    drop(f.read_to_end(&mut v));

    let b64 = base64::encode(&v);
    let ext = if path.extension() == Some(OsStr::new("png")) { "png" } else { "jpg" };

    Ok(format!("data:image/{};base64,{}", ext, b64))
}

/// Turns a string into a vector of string arguments, splitting by spaces, but
/// parsing content within quotes as one individual argument.
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
pub fn parse_quotes(s: impl AsRef<str>) -> Vec<String> {
    let s = s.as_ref();
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

/// Parses the id and token from a webhook url. Expects a [`url::Url`] object rather than a [`&str`].
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
pub fn parse_webhook(url: &Url) -> Option<(u64, &str)> {
    let (webhook_id, token) = url.path().strip_prefix("/api/webhooks/")?.split_once('/')?;
    if !["http", "https"].contains(&url.scheme())
        || !["discord.com", "discordapp.com"].contains(&url.domain()?)
        || !(17..=20).contains(&webhook_id.len())
        || !(60..=68).contains(&token.len())
    {
        return None;
    }
    Some((webhook_id.parse().ok()?, token))
}

/// Calculates the Id of the shard responsible for a guild, given its Id and
/// total number of shards used.
///
/// # Examples
///
/// Retrieve the Id of the shard for a guild with Id `81384788765712384`, using
/// 17 shards:
///
/// ```rust
/// use serenity::utils;
///
/// assert_eq!(utils::shard_id(81384788765712384 as u64, 17), 7);
/// ```
#[inline]
pub fn shard_id(guild_id: impl Into<u64>, shard_count: u64) -> u64 {
    (guild_id.into() >> 22) % shard_count
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
        assert_eq!(parse_username("<@12345>").unwrap(), 12_345);
        assert_eq!(parse_username("<@!12345>").unwrap(), 12_345);
    }

    #[test]
    fn role_parser() {
        assert_eq!(parse_role("<@&12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_channel_parser() {
        assert_eq!(parse_channel("<#12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_emoji_parser() {
        let emoji = parse_emoji("<:name:12345>").unwrap();
        assert_eq!(emoji.name, "name");
        assert_eq!(emoji.id, 12_345);
    }

    #[test]
    fn test_quote_parser() {
        let parsed = parse_quotes("a \"b c\" d\"e f\"  g");
        assert_eq!(parsed, ["a", "b c", "d", "e f", "g"]);
    }

    #[test]
    fn test_webhook_parser() {
        let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV".parse().unwrap();
        let (id, token) = parse_webhook(&url).unwrap();
        assert_eq!(id, 245037420704169985);
        assert_eq!(token, "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV");
    }
}
