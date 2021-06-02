//! Miscellaneous helper traits, enums, and structs for models.

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
#[cfg(all(feature = "model", feature = "utils"))]
use std::str::FromStr;

use super::prelude::*;
#[cfg(all(feature = "model", any(feature = "cache", feature = "utils")))]
use crate::utils;

/// Allows something - such as a channel or role - to be mentioned in a message.
pub trait Mentionable {
    /// Creates a [`Mention`] that will be able to notify or create a link to the
    /// item.
    ///
    /// [`Mention`] implements [`Display`], so [`ToString::to_string()`] can
    /// be called on it, or inserted directly into a [`format_args!`] type of
    /// macro.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "client")] {
    /// # use serenity::model::guild::Member;
    /// # use serenity::model::channel::GuildChannel;
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::prelude::Context;
    /// # use serenity::Error;
    /// use serenity::model::misc::Mentionable;
    /// async fn greet(
    ///     ctx: Context,
    ///     member: Member,
    ///     to_channel: GuildChannel,
    ///     rules_channel: ChannelId,
    /// ) -> Result<(), Error> {
    ///     to_channel
    ///         .id
    ///         .send_message(ctx, |m| {
    ///             m.content(format_args!(
    ///                 "Hi {member}, welcome to the server! \
    ///                 Please refer to {rules} for our code of conduct, \
    ///                 and enjoy your stay.",
    ///                 member = member.mention(),
    ///                 rules = rules_channel.mention(),
    ///             ))
    ///         })
    ///         .await?;
    ///     Ok(())
    /// }
    /// # }
    /// ```
    /// ```
    /// # use serenity::model::id::{RoleId, ChannelId, UserId};
    /// use serenity::model::misc::Mentionable;
    /// let user: UserId = 1.into();
    /// let channel: ChannelId = 2.into();
    /// let role: RoleId = 3.into();
    /// assert_eq!(
    ///     "<@1> <#2> <@&3>",
    ///     format!("{} {} {}", user.mention(), channel.mention(), role.mention(),),
    /// )
    /// ```
    fn mention(&self) -> Mention;
}

/// A struct that represents some way to insert a notification, link, or emoji
/// into a message.
///
/// [`Display`] is the primary way of utilizing a [`Mention`], either in a
/// [`format_args!`] type of macro or with [`ToString::to_string()`]. A
/// [`Mention`] is created using [`Mentionable::mention()`], or with
/// [`From`]/[`Into`].
///
/// # Examples
///
/// ```
/// # use serenity::model::id::{RoleId, ChannelId, UserId};
/// use serenity::model::misc::Mention;
/// let user: UserId = 1.into();
/// let channel: ChannelId = 2.into();
/// let role: RoleId = 3.into();
/// assert_eq!(
///     "<@1> <#2> <@&3>",
///     format!("{} {} {}", Mention::from(user), Mention::from(channel), Mention::from(role),),
/// )
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Mention(MentionableImpl);

#[derive(Debug, Clone, Copy)]
enum MentionableImpl {
    Channel(ChannelId),
    User(UserId),
    Role(RoleId),
    Emoji(EmojiId, bool),
}

macro_rules! mention {
    ($i:ident: $($t:ty, $e:expr;)*) => {$(
        impl From<$t> for Mention {
            #[inline(always)]
            fn from($i: $t) -> Self {
                $e.into()
            }
        }
    )*};
}

mention!(value:
    MentionableImpl, Mention(value);
    &'_ Channel, value.id();
    ChannelId, MentionableImpl::Channel(value);
    &'_ ChannelCategory, value.id;
    &'_ GuildChannel, value.id;
    &'_ PrivateChannel, value.id;
    &'_ CurrentUser, value.id;
    &'_ Member, value.user.id;
    UserId, MentionableImpl::User(value);
    &'_ User, value.id;
    RoleId, MentionableImpl::Role(value);
    &'_ Role, value.id;
    EmojiId, MentionableImpl::Emoji(value, false);
    (EmojiId, bool), MentionableImpl::Emoji(value.0, value.1);
    &'_ Emoji, MentionableImpl::Emoji(value.id, value.animated);
);

impl Display for Mention {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            MentionableImpl::Channel(id) => f.write_fmt(format_args!("<#{}>", id.0)),
            MentionableImpl::User(id) => f.write_fmt(format_args!("<@{}>", id.0)),
            MentionableImpl::Role(id) => f.write_fmt(format_args!("<@&{}>", id.0)),
            MentionableImpl::Emoji(id, animated) => {
                f.write_fmt(format_args!("<{}:_:{}>", if animated { "a" } else { "" }, id.0,))
            },
        }
    }
}

macro_rules! mentionable {
    ($i:ident = $e:expr, $($t:ty;)* ) => {$(
        impl Mentionable for $t {
            #[inline(always)]
            fn mention(&self) -> Mention {
                let $i = self;
                $e.into()
            }
        }
    )*};
}

mentionable!(v = *v,
    ChannelId;
    RoleId;
    UserId;
    Mention;
);
mentionable!(v = v,
    Channel;
    ChannelCategory;
    CurrentUser;
    Emoji;
    Member;
    PrivateChannel;
    Role;
    User;
    GuildChannel;
);

#[cfg(all(feature = "model", feature = "utils"))]
#[derive(Debug)]
#[non_exhaustive]
pub enum UserParseError {
    InvalidUsername,
    Rest(Box<Error>),
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserParseError::InvalidUsername => f.write_str("invalid username"),
            UserParseError::Rest(_) => f.write_str("could not fetch"),
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for UserParseError {}

macro_rules! impl_from_str {
    (id: $($id:ident, $err:ident, $parse_function:ident;)*) => {
        $(
            #[cfg(all(feature = "model", feature = "utils"))]
            #[derive(Debug)]
            pub enum $err {
                InvalidFormat,
            }

            #[cfg(all(feature = "model", feature = "utils"))]
            impl fmt::Display for $err {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        $err::InvalidFormat => f.write_str("invalid id format"),
                    }
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
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        $err::NotPresentInCache => f.write_str("not present in cache"),
                        $err::$invalid_variant => f.write_str($desc),
                    }
                }
            }

            #[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
            impl StdError for $err {}
        )*
    };
}

impl_from_str! { id:
    UserId, UserIdParseError, parse_username;
    RoleId, RoleIdParseError, parse_role;
    ChannelId, ChannelIdParseError, parse_channel;
}

impl_from_str! { struct:
    Channel, ChannelId, ChannelParseError, InvalidChannel, parse_channel, "invalid channel";
    Role, RoleId, RoleParseError, InvalidRole, parse_role, "invalid role";
}

/// A version of an emoji used only when solely the animated state, Id, and name are known.
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
    pub fn url(&self) -> String {
        match self.animated {
            true => format!(cdn!("/emojis/{}.gif"), self.id),
            false => format!(cdn!("/emojis/{}.png"), self.id),
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for EmojiIdentifier {
    type Err = ();

    fn from_str(s: &str) -> StdResult<Self, ()> {
        utils::parse_emoji(s).ok_or(())
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
        use crate::utils::Colour;

        #[tokio::test]
        async fn test_mention() {
            let channel = Channel::Guild(GuildChannel {
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
                slow_mode_rate: Some(0),
                rtc_region: None,
                video_quality_mode: None,
            });
            let emoji = Emoji {
                animated: false,
                available: true,
                id: EmojiId(5),
                name: "a".to_string(),
                managed: true,
                require_colons: true,
                roles: vec![],
                user: None,
            };
            let role = Role {
                id: RoleId(2),
                guild_id: GuildId(1),
                colour: Colour::ROSEWATER,
                hoist: false,
                managed: false,
                mentionable: false,
                name: "fake role".to_string(),
                permissions: Permissions::empty(),
                position: 1,
                tags: RoleTags::default(),
            };
            let user = User {
                id: UserId(6),
                avatar: None,
                bot: false,
                discriminator: 4132,
                name: "fake".to_string(),
                public_flags: None,
            };
            let member = Member {
                deaf: false,
                guild_id: GuildId(2),
                joined_at: None,
                mute: false,
                nick: None,
                roles: vec![],
                user: user.clone(),
                pending: false,
                premium_since: None,
                #[cfg(feature = "unstable_discord_api")]
                permissions: None,
            };

            assert_eq!(ChannelId(1).mention().to_string(), "<#1>");
            assert_eq!(channel.mention().to_string(), "<#4>");
            assert_eq!(emoji.mention().to_string(), "<:_:5>");
            assert_eq!(member.mention().to_string(), "<@6>");
            assert_eq!(role.mention().to_string(), "<@&2>");
            assert_eq!(role.id.mention().to_string(), "<@&2>");
            assert_eq!(user.mention().to_string(), "<@6>");
            assert_eq!(user.id.mention().to_string(), "<@6>");
        }

        #[test]
        #[allow(clippy::unwrap_used)]
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
