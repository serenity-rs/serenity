//! Miscellaneous helper traits, enums, and structs for models.

use super::prelude::*;
use internal::RwLockExt;

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
#[cfg(all(feature = "model", feature = "utils"))]
use std::str::FromStr;
#[cfg(all(feature = "model", feature = "utils"))]
use std::fmt;
#[cfg(all(feature = "model", any(feature = "cache", feature = "utils")))]
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
            Channel::Guild(ref x) => x.with(Mentionable::mention),
            Channel::Private(ref x) => x.with(Mentionable::mention),
            Channel::Group(ref x) => x.with(Mentionable::mention),
            Channel::Category(ref x) => x.with(Mentionable::mention),
        }
    }
}

impl Mentionable for ChannelCategory {
    fn mention(&self) -> String {
        format!("<#{}>", self.name)
    }
}

impl Mentionable for CurrentUser {
    fn mention(&self) -> String {
        format!("<@{}>", self.id.0)
    }
}

impl Mentionable for Emoji {
    fn mention(&self) -> String { format!("<:{}:{}>", self.name, self.id.0) }
}

impl Mentionable for Group {
    fn mention(&self) -> String {
        format!("<#{}>", self.channel_id.0)
    }
}

impl Mentionable for Member {
    fn mention(&self) -> String { format!("<@{}>", self.user.with(|u| u.id.0)) }
}

impl Mentionable for PrivateChannel {
    fn mention(&self) -> String {
        format!("<#{}>", self.id.0)
    }
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

impl Mentionable for GuildChannel {
    fn mention(&self) -> String { format!("<#{}>", self.id.0) }
}

#[cfg(all(feature = "model", feature = "utils"))]
#[derive(Debug)]
pub enum UserParseError {
    InvalidUsername,
    Rest(Box<Error>),
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for UserParseError {
    fn description(&self) -> &str {
        use self::UserParseError::*;

        match *self {
            InvalidUsername => "invalid username",
            Rest(_) => "could not fetch",
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for User {
    type Err = UserParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match utils::parse_username(s) {
            Some(x) => UserId(x as u64)
                .get()
                .map_err(|e| UserParseError::Rest(Box::new(e))),
            _ => Err(UserParseError::InvalidUsername),
        }
    }
}

macro_rules! impl_from_str {
    (id: $($id:tt, $err:ident;)*) => {
        $(
            #[cfg(all(feature = "model", feature = "utils"))]
            #[derive(Debug)]
            pub enum $err {
                InvalidFormat,
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl fmt::Display for $err {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl StdError for $err {
                fn description(&self) -> &str {
                    use self::$err::*;

                    match *self {
                        InvalidFormat => "invalid id format",
                    }
                }
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl FromStr for $id {
                type Err = $err;

                fn from_str(s: &str) -> StdResult<Self, Self::Err> {
                    Ok(match utils::parse_username(s) {
                        Some(id) => $id(id),
                        None => s.parse::<u64>().map($id).map_err(|_| $err::InvalidFormat)?,
                    })
                }
            }
        )*
    };

    (struct: $($struct:ty, $id:tt, $err:ident, $invalid_variant:tt, $parse_fn:ident, $desc:expr;)*) => {
        $(
            #[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
            #[derive(Debug)]
            pub enum $err {
                NotPresentInCache,
                $invalid_variant,
            }

            #[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
            impl fmt::Display for $err {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
            }

            #[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
            impl StdError for $err {
                fn description(&self) -> &str {
                    use self::$err::*;

                    match *self {
                        NotPresentInCache => "not present in cache",
                        $invalid_variant => $desc,
                    }
                }
            }

            #[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
            impl FromStr for $struct {
                type Err = $err;

                fn from_str(s: &str) -> StdResult<Self, Self::Err> {
                    match utils::$parse_fn(s) {
                        Some(x) => match $id(x).find() {
                            Some(user) => Ok(user),
                            _ => Err($err::NotPresentInCache),
                        },
                        _ => Err($err::$invalid_variant),
                    }
                }
            }
        )*
    };
}

impl_from_str! { id:
    UserId, UserIdParseError;
    RoleId, RoleIdParseError;
    ChannelId, ChannelIdParseError;
}

impl_from_str! { struct:
    Channel, ChannelId, ChannelParseError, InvalidChannel, parse_channel, "invalid channel";
    Role, RoleId, RoleParseError, InvalidRole, parse_role, "invalid role";
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


/// A component that was affected during a service incident.
///
/// This is pulled from the Discord status page.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AffectedComponent {
    pub name: String,
}

/// An incident retrieved from the Discord status page.
///
/// This is not necessarily a representation of an ongoing incident.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maintenance {
    pub description: String,
    pub id: String,
    pub name: String,
    pub start: String,
    pub stop: String,
}

#[cfg(test)]
mod test {
    use model::prelude::*;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use utils::Colour;

    #[test]
    fn test_formatters() {
        assert_eq!(ChannelId(1).to_string(), "1");
        assert_eq!(EmojiId(2).to_string(), "2");
        assert_eq!(GuildId(3).to_string(), "3");
        assert_eq!(RoleId(4).to_string(), "4");
        assert_eq!(UserId(5).to_string(), "5");
    }

    #[cfg(feature = "utils")]
    #[test]
    fn test_mention() {
        let channel = Channel::Guild(Arc::new(RwLock::new(GuildChannel {
            bitrate: None,
            category_id: None,
            guild_id: GuildId(1),
            kind: ChannelType::Text,
            id: ChannelId(4),
            last_message_id: None,
            last_pin_timestamp: None,
            name: "a".to_string(),
            permission_overwrites: vec![],
            position: 1,
            topic: None,
            user_limit: None,
            nsfw: false,
        })));
        let emoji = Emoji {
            animated: false,
            id: EmojiId(5),
            name: "a".to_string(),
            managed: true,
            require_colons: true,
            roles: vec![],
        };
        let role = Role {
            id: RoleId(2),
            colour: Colour::ROSEWATER,
            hoist: false,
            managed: false,
            mentionable: false,
            name: "fake role".to_string(),
            permissions: Permissions::empty(),
            position: 1,
        };
        let user = User {
            id: UserId(6),
            avatar: None,
            bot: false,
            discriminator: 4132,
            name: "fake".to_string(),
        };
        let member = Member {
            deaf: false,
            guild_id: GuildId(2),
            joined_at: None,
            mute: false,
            nick: None,
            roles: vec![],
            user: Arc::new(RwLock::new(user.clone())),
        };

        assert_eq!(ChannelId(1).mention(), "<#1>");
        assert_eq!(channel.mention(), "<#4>");
        assert_eq!(emoji.mention(), "<:a:5>");
        assert_eq!(member.mention(), "<@6>");
        assert_eq!(role.mention(), "<@&2>");
        assert_eq!(role.id.mention(), "<@&2>");
        assert_eq!(user.mention(), "<@6>");
        assert_eq!(user.id.mention(), "<@6>");
    }
}
