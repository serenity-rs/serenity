//! Miscellaneous helper traits, enums, and structs for models.

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
use std::str::FromStr;

use super::prelude::*;
#[cfg(all(feature = "model", any(feature = "cache", feature = "utils")))]
use crate::utils;

macro_rules! impl_from_str {
    ($($id:ident, $err:ident, $parse_function:ident;)+) => {
        $(
            #[cfg(all(feature = "model", feature = "utils"))]
            #[derive(Debug)]
            pub enum $err {
                InvalidFormat,
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl fmt::Display for $err {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("invalid id format")
                }
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl StdError for $err {}

            #[cfg(all(feature = "model", feature = "utils"))]
            impl FromStr for $id {
                type Err = $err;

                fn from_str(s: &str) -> StdResult<Self, Self::Err> {
                    match utils::$parse_function(s) {
                        Some(id) => Ok(id),
                        None => s.parse().map($id::new).map_err(|_| $err::InvalidFormat),
                    }
                }
            }
        )*
    };
}

impl_from_str! {
    UserId, UserIdParseError, parse_username;
    RoleId, RoleIdParseError, parse_role;
    ChannelId, ChannelIdParseError, parse_channel;
}

/// Hides the implementation detail of ImageHash as an enum.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImageHashInner {
    Normal { hash: [u8; 16], is_animated: bool },
    Clyde,
}

/// An image hash returned from the Discord API.
///
/// This type can be constructed via it's [`FromStr`] implementation, and can be turned into it's
/// cannonical representation via [`std::fmt::Display`] or [`serde::Serialize`].
///
/// # Example
/// ```rust
/// use serenity::model::misc::ImageHash;
///
/// let image_hash: ImageHash = "f1eff024d9c85339c877985229ed8fec".parse().unwrap();
/// assert_eq!(image_hash.to_string(), String::from("f1eff024d9c85339c877985229ed8fec"));
/// ```

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageHash(ImageHashInner);

impl ImageHash {
    /// Returns if the linked image is animated, which means the hash starts with `a_`.
    ///
    /// # Example
    /// ```rust
    /// use serenity::model::misc::ImageHash;
    ///
    /// let animated_hash: ImageHash = "a_e3c0db7f38777778fb43081f8746ebc9".parse().unwrap();
    /// assert!(animated_hash.is_animated());
    /// ```
    #[must_use]
    pub fn is_animated(&self) -> bool {
        match &self.0 {
            ImageHashInner::Normal {
                is_animated, ..
            } => *is_animated,
            ImageHashInner::Clyde => true,
        }
    }
}

impl std::fmt::Debug for ImageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        <Self as std::fmt::Display>::fmt(self, f)?;
        f.write_str("\"")
    }
}

impl serde::Serialize for ImageHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ImageHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // TODO: Replace this with ArrayString<34>?
        let helper = String::deserialize(deserializer)?;
        Self::from_str(&helper).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ImageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ImageHashInner::Normal { hash, is_animated } = &self.0 else {
            return f.write_str("clyde")
        };

        if *is_animated {
            f.write_str("a_")?;
        }

        for byte in hash {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

/// An error returned when [`ImageHash`] is passed an erronous value.
#[derive(Debug, Clone)]
pub enum ImageHashParseError {
    /// The given hash was not a valid [`ImageHash`] length, containing the invalid length.
    InvalidLength(usize),
    /// The given hash was a valid length, but was not entirely parsable hex values.
    UnparsableBytes(std::num::ParseIntError),
}

impl std::error::Error for ImageHashParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::UnparsableBytes(source) = self {
            Some(source)
        } else {
            None
        }
    }
}

impl std::fmt::Display for ImageHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(f, "Invalid length {length}, expected 32 or 34 characters")
            },
            Self::UnparsableBytes(_) => write!(f, "Could not parse hex to ImageHash"),
        }
    }
}

impl std::str::FromStr for ImageHash {
    type Err = ImageHashParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let (hex, is_animated) = if s.len() == 34 && s.starts_with("a_") {
            (&s[2..], true)
        } else if s.len() == 32 {
            (s, false)
        } else if s == "clyde" {
            return Ok(Self(ImageHashInner::Clyde));
        } else {
            return Err(Self::Err::InvalidLength(s.len()));
        };

        let mut hash = [0u8; 16];
        for i in (0..hex.len()).step_by(2) {
            let hex_byte = &hex[i..i + 2];
            hash[i / 2] = u8::from_str_radix(hex_byte, 16).map_err(Self::Err::UnparsableBytes)?;
        }

        Ok(Self(ImageHashInner::Normal {
            is_animated,
            hash,
        }))
    }
}

/// A version of an emoji used only when solely the animated state, Id, and name are known.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-emoji).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct EmojiIdentifier {
    /// Whether the emoji is animated
    pub animated: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can only contain
    /// alphanumeric characters and underscores.
    pub name: String,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl EmojiIdentifier {
    /// Generates a URL to the emoji's image.
    #[must_use]
    pub fn url(&self) -> String {
        let ext = if self.animated { "gif" } else { "png" };

        cdn!("/emojis/{}.{}", self.id, ext)
    }
}

#[derive(Debug)]
#[cfg(all(feature = "model", feature = "utils"))]
pub struct EmojiIdentifierParseError {
    parsed_string: String,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for EmojiIdentifierParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is not a valid emoji identifier", self.parsed_string)
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for EmojiIdentifierParseError {}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for EmojiIdentifier {
    type Err = EmojiIdentifierParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        utils::parse_emoji(s).ok_or_else(|| EmojiIdentifierParseError {
            parsed_string: s.to_owned(),
        })
    }
}

/// An incident retrieved from the Discord status page.
///
/// This is not necessarily a representation of an ongoing incident.
///
/// [Discord docs](https://discordstatus.com/api) (see "Unresolved incident" example)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Incident {
    pub created_at: String,
    pub id: String,
    pub impact: String,
    pub incident_updates: Vec<IncidentUpdate>,
    pub monitoring_at: Option<String>,
    pub name: String,
    pub page_id: String,
    pub resolved_at: Option<String>,
    pub shortlink: String,
    pub status: String,
    pub updated_at: String,
}

/// An update to an incident from the Discord status page.
///
/// This will typically state what new information has been discovered about an incident.
///
/// [Discord docs](https://discordstatus.com/api) (see "Unresolved incident" example)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IncidentUpdate {
    pub body: String,
    pub created_at: String,
    pub display_at: String,
    pub id: String,
    pub incident_id: String,
    pub status: String,
    pub updated_at: String,
}

/// A Discord status maintenance message. This can be either for active maintenances or for
/// scheduled maintenances.
///
/// [Discord docs](https://discordstatus.com/api) (see "scheduled maintenances" examples)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Maintenance {
    pub created_at: String,
    pub id: String,
    pub impact: String,
    pub incident_updates: Vec<IncidentUpdate>,
    pub monitoring_at: Option<String>,
    pub name: String,
    pub page_id: String,
    pub resolved_at: Option<String>,
    pub scheduled_for: String,
    pub scheduled_until: String,
    pub shortlink: String,
    pub status: String,
    pub updated_at: String,
}

#[cfg(test)]
mod test {
    use crate::model::prelude::*;

    #[test]
    fn test_formatters() {
        assert_eq!(ChannelId::new(1).to_string(), "1");
        assert_eq!(EmojiId::new(2).to_string(), "2");
        assert_eq!(GuildId::new(3).to_string(), "3");
        assert_eq!(RoleId::new(4).to_string(), "4");
        assert_eq!(UserId::new(5).to_string(), "5");
    }

    #[cfg(feature = "utils")]
    mod utils {
        use crate::model::prelude::*;

        #[cfg(feature = "model")]
        #[test]
        fn parse_mentions() {
            assert_eq!("<@1234>".parse::<UserId>().unwrap(), UserId::new(1234));
            assert_eq!("<@&1234>".parse::<RoleId>().unwrap(), RoleId::new(1234));
            assert_eq!("<#1234>".parse::<ChannelId>().unwrap(), ChannelId::new(1234));

            assert!("<@1234>".parse::<ChannelId>().is_err());
            assert!("<@&1234>".parse::<UserId>().is_err());
            assert!("<#1234>".parse::<RoleId>().is_err());
        }
    }
}
