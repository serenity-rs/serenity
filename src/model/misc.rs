//! Miscellaneous helper traits, enums, and structs for models.

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
use std::fmt::Write;
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
use std::str::FromStr;

use super::prelude::*;
use crate::internal::prelude::*;
#[cfg(all(feature = "model", any(feature = "cache", feature = "utils")))]
use crate::utils;

/// Hides the implementation detail of ImageHash as an enum.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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

    #[must_use]
    fn into_arraystring(self) -> ArrayString<34> {
        let ImageHashInner::Normal {
            hash,
            is_animated,
        } = &self.0
        else {
            return ArrayString::from_str("clyde").expect("the string clyde is less than 34 chars");
        };

        let mut out = ArrayString::new();
        if *is_animated {
            out.push_str("a_");
        }

        for byte in hash {
            write!(out, "{byte:02x}").expect("ImageHash should fit into 34 char ArrayString");
        }

        out
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
        self.into_arraystring().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ImageHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let helper = ArrayString::<34>::deserialize(deserializer)?;
        Self::from_str(&helper).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ImageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.into_arraystring().fmt(f)
    }
}

/// An error returned when [`ImageHash`] is passed an erronous value.
#[derive(Debug, Clone)]
pub enum ImageHashParseError {
    /// The given hash was not a valid [`ImageHash`] length, containing the invalid length.
    InvalidLength(usize),
}

impl std::error::Error for ImageHashParseError {}

impl std::fmt::Display for ImageHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(f, "Invalid length {length}, expected 32 or 34 characters")
            },
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
            hash[i / 2] = u8::from_str_radix(hex_byte, 16).unwrap_or_else(|err| {
                tracing::warn!("Invalid byte in ImageHash ({s}): {err}");
                0
            });
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
    pub name: FixedString,
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

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for EmojiIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.animated {
            f.write_str("<a:")?;
        } else {
            f.write_str("<:")?;
        }

        f.write_str(&self.name)?;

        f.write_char(':')?;
        fmt::Display::fmt(&self.id, f)?;
        f.write_char('>')
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
    pub created_at: FixedString,
    pub id: FixedString,
    pub impact: FixedString,
    pub incident_updates: FixedArray<IncidentUpdate>,
    pub monitoring_at: Option<FixedString>,
    pub name: FixedString,
    pub page_id: FixedString,
    pub resolved_at: Option<FixedString>,
    pub shortlink: FixedString,
    pub status: FixedString,
    pub updated_at: FixedString,
}

/// An update to an incident from the Discord status page.
///
/// This will typically state what new information has been discovered about an incident.
///
/// [Discord docs](https://discordstatus.com/api) (see "Unresolved incident" example)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IncidentUpdate {
    pub body: FixedString,
    pub created_at: FixedString,
    pub display_at: FixedString,
    pub id: FixedString,
    pub incident_id: FixedString,
    pub status: FixedString,
    pub updated_at: FixedString,
}

/// A Discord status maintenance message. This can be either for active maintenances or for
/// scheduled maintenances.
///
/// [Discord docs](https://discordstatus.com/api) (see "scheduled maintenances" examples)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Maintenance {
    pub created_at: FixedString,
    pub id: FixedString,
    pub impact: FixedString,
    pub incident_updates: FixedArray<IncidentUpdate>,
    pub monitoring_at: Option<FixedString>,
    pub name: FixedString,
    pub page_id: FixedString,
    pub resolved_at: Option<FixedString>,
    pub scheduled_for: FixedString,
    pub scheduled_until: FixedString,
    pub shortlink: FixedString,
    pub status: FixedString,
    pub updated_at: FixedString,
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
}
