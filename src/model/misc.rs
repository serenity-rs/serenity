//! Miscellaneous helper traits, enums, and structs for models.

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
#[cfg(all(feature = "model", feature = "utils"))]
use std::fmt;
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
#[cfg(all(feature = "model", feature = "utils"))]
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
                    let id = match utils::$parse_function(s) {
                        Some(id) => id,
                        None => s.parse::<u64>().map_err(|_| $err::InvalidFormat)?,
                    };

                    Ok($id(id))
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

/// A version of an emoji used only when solely the animated state, Id, and name are known.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-emoji).
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EmojiIdentifier {
    /// Whether the emoji is animated
    pub animated: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can
    /// only contain alphanumeric characters and underscores.
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

/// A component that was affected during a service incident.
///
/// This is pulled from the Discord status page.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AffectedComponent {
    pub name: String,
}

/// An incident retrieved from the Discord status page.
///
/// This is not necessarily a representation of an ongoing incident.
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
    pub short_link: String,
    pub status: String,
    pub updated_at: String,
}

/// An update to an incident from the Discord status page.
///
/// This will typically state what new information has been discovered about an
/// incident.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IncidentUpdate {
    pub affected_components: Vec<AffectedComponent>,
    pub body: String,
    pub created_at: String,
    pub display_at: String,
    pub id: String,
    pub incident_id: String,
    pub status: IncidentStatus,
    pub updated_at: String,
}

/// The type of status update during a service incident.
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Identified,
    Investigating,
    Monitoring,
    Postmortem,
    Resolved,
}

/// A Discord status maintenance message. This can be either for active
/// maintenances or for scheduled maintenances.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Maintenance {
    pub description: String,
    pub id: String,
    pub name: String,
    pub start: String,
    pub stop: String,
}

#[cfg(test)]
mod test {
    use crate::model::prelude::*;

    #[test]
    fn test_formatters() {
        assert_eq!(ChannelId(1).to_string(), "1");
        assert_eq!(EmojiId(2).to_string(), "2");
        assert_eq!(GuildId(3).to_string(), "3");
        assert_eq!(RoleId(4).to_string(), "4");
        assert_eq!(UserId(5).to_string(), "5");
    }

    #[cfg(feature = "utils")]
    mod utils {
        use crate::model::prelude::*;

        #[cfg(feature = "model")]
        #[test]
        fn parse_mentions() {
            assert_eq!("<@1234>".parse::<UserId>().unwrap(), UserId(1234));
            assert_eq!("<@&1234>".parse::<RoleId>().unwrap(), RoleId(1234));
            assert_eq!("<#1234>".parse::<ChannelId>().unwrap(), ChannelId(1234));

            assert!("<@1234>".parse::<ChannelId>().is_err());
            assert!("<@&1234>".parse::<UserId>().is_err());
            assert!("<#1234>".parse::<RoleId>().is_err());
        }
    }
}
