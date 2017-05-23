//! A set of utilities to help with common use cases that are not required to
//! fully use the library.

mod colour;

mod message_builder;

pub use self::colour::Colour;

// Note: Here for BC purposes.
#[cfg(feature="builder")]
pub use super::builder;

use base64;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ::internal::prelude::*;
use ::model::{EmojiIdentifier, EmojiId};

pub use self::message_builder::MessageBuilder;

/// Determines if a name is NSFW.
///
/// This checks that the name is either `"nsfw"` or, for names longer than that,
/// is prefixed with `"nsfw"`.
///
/// **Note**: Whether a channel is NSFW is done client-side, as a field for the
/// NSFW-ness of a channel is not sent to clients. Discord's requirements for
/// defining a channel as NSFW can change at any time.
///
/// # Examples
///
/// Check that a channel named `"nsfw"` is in fact NSFW:
///
/// ```rust
/// use serenity::utils;
///
/// assert!(utils::is_nsfw("nsfw"));
/// ```
///
/// Check that a channel named `"cats"` is _not_ NSFW:
///
/// ```rust
/// use serenity::utils;
///
/// assert!(!utils::is_nsfw("cats"));
/// ```
///
/// Check that a channel named `"nsfw-stuff"` _is_ NSFW:
///
/// ```rust
/// use serenity::utils;
///
/// assert!(utils::is_nsfw("nsfw-stuff"));
/// ```
///
/// Channels prefixed with `"nsfw"` but not the hyphen (`'-'`) are _not_
/// considered NSFW:
///
/// ```rust
/// use serenity::utils;
///
/// assert!(!utils::is_nsfw("nsfwstuff"));
/// ```
pub fn is_nsfw(name: &str) -> bool {
    if name.len() == 4 {
        &name[..4] == "nsfw"
    } else if name.len() > 4 {
        &name[..5] == "nsfw-"
    } else {
        false
    }
}

/// Retrieves the "code" part of an [invite][`RichInvite`] out of a URL.
///
/// # Examples
///
/// Three formats of codes are supported:
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
/// 2. Retrieving the code from the URL `"http://discord.gg/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use serenity::utils;
///
/// let url = "http://discord.gg/0cDvIgU2voY8RSYL";
///
/// assert_eq!(utils::parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// 3. Retrieving the code from the URL `"discord.gg/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use serenity::utils;
///
/// let url = "discord.gg/0cDvIgU2voY8RSYL";
///
/// assert_eq!(utils::parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// [`RichInvite`]: ../model/struct.RichInvite.html
pub fn parse_invite(code: &str) -> &str {
    if code.starts_with("https://discord.gg/") {
        &code[19..]
    } else if code.starts_with("http://discord.gg/") {
        &code[18..]
    } else if code.starts_with("discord.gg/") {
        &code[11..]
    } else {
        code
    }
}

/// Retreives Id from a username mention.
pub fn parse_username(mention: &str) -> Option<u64> {
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

/// Retreives Id from a role mention.
pub fn parse_role(mention: &str) -> Option<u64> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else {
        None
    }
}

/// Retreives Id from a channel mention.
pub fn parse_channel(mention: &str) -> Option<u64> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<#") {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
    } else {
        None
    }
}

/// Retreives name and Id from an emoji mention.
pub fn parse_emoji(mention: &str) -> Option<EmojiIdentifier> {
    let len = mention.len();
    if len < 6 || len > 56 {
        return None;
    }

    if mention.starts_with("<:") {
        let mut name = String::default();
        let mut id = String::default();
        for (i, x) in mention[2..].chars().enumerate() {
            if x == ':' {
                let from = i + 3;
                for y in mention[from..].chars() {
                    if y == '>' {
                        break;
                    } else {
                        id.push(y);
                    }
                }
                break;
            } else {
                name.push(x);
            }
        }
        match id.parse::<u64>() {
            Ok(x) => Some(EmojiIdentifier {
                name: name,
                id: EmojiId(x)
            }),
            _ => None
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
/// let image = utils::read_image("./cat.png")
///     .expect("Failed to read image");
/// ```
///
/// [`EditProfile::avatar`]: ../builder/struct.EditProfile.html#method.avatar
pub fn read_image<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    let mut v = Vec::default();
    let mut f = File::open(path)?;
    let _ = f.read_to_end(&mut v);

    let b64 = base64::encode(&v);
    let ext = if path.extension() == Some(OsStr::new("png")) {
        "png"
    } else {
        "jpg"
    };

    Ok(format!("data:image/{};base64,{}", ext, b64))
}

/// Turns a string into a vector of string arguments, splitting by spaces, but
/// parsing content within quotes as one individual argument.
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
/// assert_eq!(utils::shard_id(81384788765712384, 17), 7);
/// ```
#[inline]
pub fn shard_id(guild_id: u64, shard_count: u64) -> u64 {
    (guild_id >> 22) % shard_count
}
