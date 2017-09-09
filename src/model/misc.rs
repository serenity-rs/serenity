use super::*;
use internal::RwLockExt;

#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
#[cfg(all(feature = "cache", feature = "utils"))]
use std::error::Error as StdError;
#[cfg(all(feature = "model", feature = "utils"))]
use std::str::FromStr;
#[cfg(all(feature = "model", feature = "utils"))]
use std::fmt;
#[cfg(all(feature = "cache", feature = "utils"))]
use std;
#[cfg(any(all(feature = "cache", feature = "utils"), all(feature = "model", feature = "utils")))]
use utils;

/// Allows something - such as a channel or role - to be mentioned in a message.
pub trait Mentionable {
    /// Creates a mentionable string, that will be able to notify and/or create
    /// a link to the item.
    fn mention(&self) -> String;
}

impl Mentionable for ChannelId {
    fn mention(&self) -> String { format!("<#{}>", self.0) }
}

impl Mentionable for Channel {
    fn mention(&self) -> String {
        match *self {
            Channel::Guild(ref x) => format!("<#{}>", x.with(|x| x.id.0)),
            Channel::Private(ref x) => format!("<#{}>", x.with(|x| x.id.0)),
            Channel::Group(ref x) => format!("<#{}>", x.with(|x| x.channel_id.0)),
            Channel::Category(_) => panic!("Categories can't be mentioned"),
        }
    }
}

impl Mentionable for Emoji {
    fn mention(&self) -> String { format!("<:{}:{}>", self.name, self.id.0) }
}

impl Mentionable for Member {
    fn mention(&self) -> String { format!("<@{}>", self.user.with(|u| u.id.0)) }
}

impl Mentionable for RoleId {
    fn mention(&self) -> String { format!("<@&{}>", self.0) }
}

impl Mentionable for Role {
    fn mention(&self) -> String { format!("<@&{}>", self.id.0) }
}

impl Mentionable for UserId {
    fn mention(&self) -> String { format!("<@{}>", self.0) }
}

impl Mentionable for User {
    fn mention(&self) -> String { format!("<@{}>", self.id.0) }
}

#[cfg(all(feature = "cache", feature = "utils"))]
#[derive(Debug)]
pub enum UserParseError {
    InvalidUsername,
    Rest(Box<std::error::Error>),
}


#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl StdError for UserParseError {
    fn description(&self) -> &str {
        use self::UserParseError::*;

        match *self {
            InvalidUsername => "invalid username",
            Rest(_) => "could not fetch",
        }
    }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl FromStr for User {
    type Err = UserParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match utils::parse_username(s) {
            Some(x) => {
                UserId(x as u64).get().map_err(
                    |e| UserParseError::Rest(Box::new(e)),
                )
            },
            _ => Err(UserParseError::InvalidUsername),
        }
    }
}

#[cfg(all(feature = "cache", feature = "utils"))]
#[derive(Debug)]
pub enum UserIdParseError {
    NotPresentInCache,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for UserIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl StdError for UserIdParseError {
    fn description(&self) -> &str {
        use self::UserIdParseError::*;

        match *self {
            NotPresentInCache => "not present in cache",
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for UserId {
    type Err = UserIdParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        utils::parse_username(s)
            .ok_or_else(|| UserIdParseError::NotPresentInCache)
            .map(UserId)
    }
}

#[cfg(all(feature = "cache", feature = "utils"))]
#[derive(Debug)]
pub enum RoleParseError {
    NotPresentInCache,
    InvalidRole,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for RoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl StdError for RoleParseError {
    fn description(&self) -> &str {
        use self::RoleParseError::*;

        match *self {
            NotPresentInCache => "not present in cache",
            InvalidRole => "invalid role",
        }
    }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl FromStr for Role {
    type Err = RoleParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match utils::parse_role(s) {
            Some(x) => {
                match RoleId(x).find() {
                    Some(user) => Ok(user),
                    _ => Err(RoleParseError::NotPresentInCache),
                }
            },
            _ => Err(RoleParseError::InvalidRole),
        }
    }
}

#[cfg(all(feature = "cache", feature = "utils"))]
#[derive(Debug)]
pub enum RoleIdParseError {
    NotPresentInCache,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for RoleIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

#[cfg(all(feature = "cache", feature = "utils"))]
impl StdError for RoleIdParseError {
    fn description(&self) -> &str {
        use self::RoleIdParseError::*;

        match *self {
            NotPresentInCache => "not present in cache",
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for RoleId {
    type Err = RoleIdParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        utils::parse_role(s)
            .ok_or_else(|| RoleIdParseError::NotPresentInCache)
            .map(RoleId)
    }
}

/// A version of an emoji used only when solely the Id and name are known.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EmojiIdentifier {
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can
    /// only contain alphanumeric characters and underscores.
    pub name: String,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl EmojiIdentifier {
    /// Generates a URL to the emoji's image.
    #[inline]
    pub fn url(&self) -> String { format!(cdn!("/emojis/{}.png"), self.id) }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for EmojiIdentifier {
    type Err = ();

    fn from_str(s: &str) -> StdResult<Self, ()> { utils::parse_emoji(s).ok_or_else(|| ()) }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for ChannelId {
    type Err = ();

    fn from_str(s: &str) -> StdResult<Self, ()> {
        utils::parse_channel(s).ok_or_else(|| ()).map(ChannelId)
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStr for Channel {
    type Err = ();

    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_channel(s) {
            Some(x) => {
                match ChannelId(x).find() {
                    Some(channel) => Ok(channel),
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}

/// A component that was affected during a service incident.
///
/// This is pulled from the Discord status page.
#[derive(Clone, Debug, Deserialize)]
pub struct AffectedComponent {
    pub name: String,
}

/// An incident retrieved from the Discord status page.
///
/// This is not necessarily a representation of an ongoing incident.
#[derive(Clone, Debug, Deserialize)]
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
#[derive(Clone, Debug, Deserialize)]
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
#[derive(Clone, Debug, Deserialize)]
pub struct Maintenance {
    pub description: String,
    pub id: String,
    pub name: String,
    pub start: String,
    pub stop: String,
}
