//! A set of utilities to help with common use cases that are not required to
//! fully use the library.

#[macro_use]
pub mod macros;

pub mod builder;

mod colour;

mod message_builder;

pub use self::colour::Colour;

use base64;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ::internal::prelude::*;
use ::model::{EmojiIdentifier, EmojiId};

pub use self::message_builder::MessageBuilder;

#[doc(hidden)]
pub fn decode_array<T, F: Fn(Value) -> Result<T>>(value: Value, f: F) -> Result<Vec<T>> {
    into_array(value).and_then(|x| x.into_iter().map(f).collect())
}

#[doc(hidden)]
pub fn into_array(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(v) => Ok(v),
        value => Err(Error::Decode("Expected array", value)),
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
