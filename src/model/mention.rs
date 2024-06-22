#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
#[cfg(all(feature = "model", feature = "utils"))]
use std::str::FromStr;

use super::prelude::*;
use crate::internal::prelude::*;
#[cfg(all(feature = "model", feature = "utils"))]
use crate::utils;

/// Allows something - such as a channel or role - to be mentioned in a message.
pub trait Mentionable {
    /// Creates a [`Mention`] that will be able to notify or create a link to the item.
    ///
    /// [`Mention`] implements [`Display`], so [`ToString::to_string()`] can be called on it, or
    /// inserted directly into a [`format_args!`] type of macro.
    ///
    /// [`Display`]: fmt::Display
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "client")] {
    /// # use serenity::builder::CreateMessage;
    /// # use serenity::model::guild::Member;
    /// # use serenity::model::channel::GuildChannel;
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::prelude::Context;
    /// # use serenity::Error;
    /// use serenity::model::mention::Mentionable;
    /// async fn greet(
    ///     ctx: Context,
    ///     member: Member,
    ///     to_channel: GuildChannel,
    ///     rules_channel: ChannelId,
    /// ) -> Result<(), Error> {
    ///     let builder = CreateMessage::new().content(format!(
    ///         "Hi {member}, welcome to the server! \
    ///         Please refer to {rules} for our code of conduct, \
    ///         and enjoy your stay.",
    ///         member = member.mention(),
    ///         rules = rules_channel.mention(),
    ///     ));
    ///     to_channel.id.send_message(&ctx.http, builder).await?;
    ///     Ok(())
    /// }
    /// # }
    /// ```
    /// ```
    /// # use serenity::model::id::{RoleId, ChannelId, UserId};
    /// use serenity::model::mention::Mentionable;
    /// let user = UserId::new(1);
    /// let channel = ChannelId::new(2);
    /// let role = RoleId::new(3);
    /// assert_eq!(
    ///     "<@1> <#2> <@&3>",
    ///     format!("{} {} {}", user.mention(), channel.mention(), role.mention(),),
    /// )
    /// ```
    fn mention(&self) -> Mention;
}

/// A struct that represents some way to insert a notification, link, or emoji into a message.
///
/// [`Display`] is the primary way of utilizing a [`Mention`], either in a [`format_args!`] type of
/// macro or with [`ToString::to_string()`]. A [`Mention`] is created using
/// [`Mentionable::mention()`], or with [`From`]/[`Into`].
///
/// [`Display`]: fmt::Display
///
/// # Examples
///
/// ```
/// # use serenity::model::id::{RoleId, ChannelId, UserId};
/// use serenity::model::mention::Mention;
/// let user = UserId::new(1);
/// let channel = ChannelId::new(2);
/// let role = RoleId::new(3);
/// assert_eq!(
///     "<@1> <#2> <@&3>",
///     format!("{} {} {}", Mention::from(user), Mention::from(channel), Mention::from(role),),
/// )
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Mention {
    Channel(ChannelId),
    Role(RoleId),
    User(UserId),
}

macro_rules! mention {
    ($i:ident: $($t:ty, $e:expr;)*) => {$(
        impl From<$t> for Mention {
            fn from($i: $t) -> Self {
                $e
            }
        }
    )*};
}

mention!(value:
    ChannelId, Mention::Channel(value);
    RoleId, Mention::Role(value);
    UserId, Mention::User(value);
);

impl fmt::Display for Mention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_arraystring())
    }
}

impl ToArrayString for Mention {
    const MAX_LENGTH: usize = 20 + 4;
    type ArrayString = ArrayString<{ 20 + 4 }>;

    fn to_arraystring(self) -> Self::ArrayString {
        let mut out = Self::ArrayString::new();
        match self {
            Self::Channel(id) => aformat_into!(out, "<#{id}>"),
            Self::Role(id) => aformat_into!(out, "<@&{id}>"),
            Self::User(id) => aformat_into!(out, "<@{id}>"),
        };

        out
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
#[derive(Debug)]
pub enum MentionParseError {
    InvalidMention,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for MentionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid mention")
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for MentionParseError {}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for Mention {
    type Err = MentionParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let m = if let Some(id) = utils::parse_channel_mention(s) {
            id.mention()
        } else if let Some(id) = utils::parse_role_mention(s) {
            id.mention()
        } else if let Some(id) = utils::parse_user_mention(s) {
            id.mention()
        } else {
            return Err(MentionParseError::InvalidMention);
        };

        Ok(m)
    }
}

impl<T> Mentionable for T
where
    T: Into<Mention> + Copy,
{
    fn mention(&self) -> Mention {
        (*self).into()
    }
}

macro_rules! mentionable {
    ($i:ident: $t:ty, $e:expr) => {
        impl Mentionable for $t {
            fn mention(&self) -> Mention {
                let $i = self;
                $e.into()
            }
        }
    };
}

#[cfg(feature = "model")]
mentionable!(value: Channel, value.id());

mentionable!(value: GuildChannel, value.id);
mentionable!(value: PrivateChannel, value.id);
mentionable!(value: Member, value.user.id);
mentionable!(value: CurrentUser, value.id);
mentionable!(value: User, value.id);
mentionable!(value: Role, value.id);

#[cfg(feature = "utils")]
#[cfg(test)]
mod test {
    use crate::model::prelude::*;

    #[test]
    fn test_mention() {
        let role = Role::default();
        let channel = Channel::Guild(GuildChannel {
            id: ChannelId::new(4),
            ..Default::default()
        });
        let user = User {
            id: UserId::new(6),
            ..Default::default()
        };
        let member = Member {
            user: user.clone(),
            ..Default::default()
        };

        assert_eq!(ChannelId::new(1).mention().to_string(), "<#1>");
        #[cfg(feature = "model")]
        assert_eq!(channel.mention().to_string(), "<#4>");
        assert_eq!(member.mention().to_string(), "<@6>");
        assert_eq!(role.mention().to_string(), "<@&0>");
        assert_eq!(role.id.mention().to_string(), "<@&0>");
        assert_eq!(user.mention().to_string(), "<@6>");
        assert_eq!(user.id.mention().to_string(), "<@6>");
    }
}
