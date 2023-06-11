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

/// An image hash returned from the Discord API.
///
/// Note: This is parsed into a compact form when constructed, then turned back
/// into the cannonical hex representation used by the API when needed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageHash {
    is_animated: bool,
    hash: [u8; 16],
}

impl ImageHash {
    #[must_use]
    pub fn is_animated(&self) -> bool {
        self.is_animated
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
        if self.is_animated {
            f.write_str("a_")?;
        }

        for byte in self.hash {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ImageHashParseError {
    InvalidLength(usize),
    MissingAnimatedMark,
    UnparsableBytes(std::num::ParseIntError),
}

impl std::fmt::Display for ImageHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(f, "Invalid length {length}, expected 32 or 34 characters")
            },
            Self::MissingAnimatedMark => f.write_str("Input is 34 characters long, but missing a_"),
            Self::UnparsableBytes(bytes) => write!(f, "Could not parse to hex: {bytes}"),
        }
    }
}

impl std::str::FromStr for ImageHash {
    type Err = ImageHashParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let (hex, is_animated) = if s.len() == 34 {
            if &s.as_bytes()[0..2] != b"a_" {
                return Err(ImageHashParseError::MissingAnimatedMark);
            }

            (&s[2..], true)
        } else if s.len() == 32 {
            (s, false)
        } else {
            return Err(Self::Err::InvalidLength(s.len()));
        };

        let mut hash = [0u8; 16];
        for i in (0..hex.len()).step_by(2) {
            let hex_byte = &hex[i..i + 2];
            hash[i / 2] =
                u8::from_str_radix(hex_byte, 16).map_err(ImageHashParseError::UnparsableBytes)?;
        }

        Ok(Self {
            is_animated,
            hash,
        })
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
