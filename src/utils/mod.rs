//! A set of utilities to help with common use cases that are not required to
//! fully use the library.

#[macro_use]
pub mod macros;

pub mod builder;

mod colour;

#[cfg(feature = "extras")]
mod message_builder;

pub use self::colour::Colour;

use base64;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ::internal::prelude::*;

#[cfg(feature = "extras")]
pub use self::message_builder::MessageBuilder;

macro_rules! cdn_concat {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    }
}
macro_rules! api {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

macro_rules! api_concat {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    }
}
macro_rules! status_concat {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}

macro_rules! map_nums {
    ($item:ident; $($entry:ident $value:expr,)*) => {
        impl $item {
            #[allow(dead_code)]
            pub fn num(&self) -> u64 {
                match *self {
                    $($item::$entry => $value,)*
                }
            }

            #[allow(dead_code)]
            pub fn from_num(num: u64) -> Option<Self> {
                match num {
                    $($value => Some($item::$entry),)*
                    _ => None,
                }
            }

            #[allow(dead_code)]
            fn decode(value: Value) -> Result<Self> {
                value.as_u64().and_then(Self::from_num).ok_or(Error::Decode(
                    concat!("Expected valid ", stringify!($item)),
                    value
                ))
            }
        }
    }
}

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
/// let image = match utils::read_image("./cat.png") {
///     Ok(image) => image,
///     Err(why) => {
///         // properly handle the error
///
///         return;
///     },
/// };
/// ```
///
/// [`EditProfile::avatar`]: ../builder/struct.EditProfile.html#method.avatar
pub fn read_image<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    let mut v = Vec::default();
    let mut f = try!(File::open(path));
    let _ = f.read_to_end(&mut v);

    let b64 = base64::encode(&v);
    let ext = if path.extension() == Some(OsStr::new("png")) {
        "png"
    } else {
        "jpg"
    };

    Ok(format!("data:image/{};base64,{}", ext, b64))
}
