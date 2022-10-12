#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;
#[cfg(all(feature = "model", feature = "utils"))]
use std::str::FromStr;

use super::prelude::*;
#[cfg(all(feature = "model", feature = "utils"))]
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
    /// [`Display`]: fmt::Display
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
    /// use serenity::model::mention::Mentionable;
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
    /// use serenity::model::mention::Mentionable;
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
/// [`Display`]: fmt::Display
///
/// # Examples
///
/// ```
/// # use serenity::model::id::{RoleId, ChannelId, UserId};
/// use serenity::model::mention::Mention;
/// let user: UserId = 1.into();
/// let channel: ChannelId = 2.into();
/// let role: RoleId = 3.into();
/// assert_eq!(
///     "<@1> <#2> <@&3>",
///     format!("{} {} {}", Mention::from(user), Mention::from(channel), Mention::from(role),),
/// )
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Mention {
    Channel(ChannelId),
    Role(RoleId),
    User(UserId),
    Emoji(EmojiId, bool),
}

macro_rules! mention {
    ($i:ident: $($t:ty, $e:expr;)*) => {$(
        impl From<$t> for Mention {
            #[inline(always)]
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
    EmojiId, Mention::Emoji(value, false);
    (EmojiId, bool), Mention::Emoji(value.0, value.1);
);

impl fmt::Display for Mention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Mention::Channel(id) => f.write_fmt(format_args!("<#{}>", id.0)),
            Mention::Role(id) => f.write_fmt(format_args!("<@&{}>", id.0)),
            Mention::User(id) => f.write_fmt(format_args!("<@{}>", id.0)),
            Mention::Emoji(id, animated) => {
                f.write_fmt(format_args!("<{}:omitted:{}>", if animated { "a" } else { "" }, id.0,))
            },
        }
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
        let m = if let Some(id) = utils::parse_channel(s) {
            ChannelId(id).mention()
        } else if let Some(id) = utils::parse_role(s) {
            RoleId(id).mention()
        } else if let Some(id) = utils::parse_username(s) {
            UserId(id).mention()
        } else if let Some(emoji_ident) = utils::parse_emoji(s) {
            emoji_ident.mention()
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
    ($i:ident: $($t:ty, $e:expr;)*) => {$(
        impl Mentionable for $t {
            #[inline(always)]
            fn mention(&self) -> Mention {
                let $i = self;
                $e.into()
            }
        }
    )*};
}

#[cfg(feature = "model")]
mentionable!(value: Channel, value.id(););

mentionable!(value:
    ChannelCategory, value.id;
    GuildChannel, value.id;
    PrivateChannel, value.id;
    CurrentUser, value.id;
    Member, value.user.id;
    User, value.id;
    Role, value.id;
    Emoji, (value.id, value.animated);
    EmojiIdentifier, (value.id, value.animated);
);

#[cfg(feature = "utils")]
#[cfg(test)]
mod test {
    use crate::model::prelude::*;
    use crate::utils::Colour;

    #[test]
    fn test_mention() {
        let channel = Channel::Guild(GuildChannel {
            bitrate: None,
            parent_id: None,
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
            rate_limit_per_user: Some(0),
            rtc_region: None,
            video_quality_mode: None,
            message_count: None,
            member_count: None,
            thread_metadata: None,
            member: None,
            default_auto_archive_duration: None,
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
            icon: None,
            unicode_emoji: None,
        };
        let user = User {
            id: UserId(6),
            avatar: None,
            bot: false,
            discriminator: 4132,
            name: "fake".to_string(),
            public_flags: None,
            banner: None,
            accent_colour: None,
            member: None,
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
            permissions: None,
            avatar: None,
            communication_disabled_until: None,
        };

        assert_eq!(ChannelId(1).mention().to_string(), "<#1>");
        #[cfg(feature = "model")]
        assert_eq!(channel.mention().to_string(), "<#4>");
        assert_eq!(emoji.mention().to_string(), "<:omitted:5>");
        assert_eq!(member.mention().to_string(), "<@6>");
        assert_eq!(role.mention().to_string(), "<@&2>");
        assert_eq!(role.id.mention().to_string(), "<@&2>");
        assert_eq!(user.mention().to_string(), "<@6>");
        assert_eq!(user.id.mention().to_string(), "<@6>");
    }
}
