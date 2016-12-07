use super::{
    ChannelId,
    Channel,
    Emoji,
    Member,
    RoleId,
    Role,
    UserId,
    User,
    IncidentStatus,
    EmojiIdentifier
};
use ::internal::prelude::*;
use std::str::FromStr;
use std::result::Result as StdResult;
use ::utils;

/// Allows something - such as a channel or role - to be mentioned in a message.
pub trait Mentionable {
    fn mention(&self) -> String;
}

impl Mentionable for ChannelId {
    fn mention(&self) -> String {
        format!("<#{}>", self.0)
    }
}

impl Mentionable for Channel {
    fn mention(&self) -> String {
        match *self {
            Channel::Guild(ref x) => {
                format!("<#{}>", x.id.0)
            },
            Channel::Private(ref x) => {
                format!("<#{}>", x.id.0)
            },
            Channel::Group(ref x) => {
                format!("<#{}>", x.channel_id.0)
            }
        }
    }
}

impl Mentionable for Emoji {
    fn mention(&self) -> String {
        format!("<:{}:{}>", self.name, self.id.0)
    }
}

impl Mentionable for Member {
    fn mention(&self) -> String {
        format!("<@{}>", self.user.id.0)
    }
}

impl Mentionable for RoleId {
    fn mention(&self) -> String {
        format!("<@&{}>", self.0)
    }
}

impl Mentionable for Role {
    fn mention(&self) -> String {
        format!("<@&{}>", self.id.0)
    }
}

impl Mentionable for UserId {
    fn mention(&self) -> String {
        format!("<@{}>", self.0)
    }
}

impl Mentionable for User {
    fn mention(&self) -> String {
        format!("<@{}>", self.id.0)
    }
}

#[cfg(feature = "cache")]
impl FromStr for User {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_username(s) {
            Some(x) => {
                match UserId(x as u64).find() {
                    Some(user) => Ok(user),
                    _ => Err(())
                }
            },
            _ => Err(())
        }
    }
}

impl FromStr for UserId {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_username(s) {
            Some(x) => Ok(UserId(x)),
            _ => Err(())
        }
    }
}

#[cfg(feature = "cache")]
impl FromStr for Role {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_role(s) {
            Some(x) => {
                match RoleId(x).find() {
                    Some(user) => Ok(user),
                    _ => Err(())
                }
            },
            _ => Err(())
        }
    }
}

impl FromStr for RoleId {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_role(s) {
            Some(x) => Ok(RoleId(x)),
            _ => Err(())
        }
    }
}

impl FromStr for EmojiIdentifier {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_emoji(s) {
            Some(x) => Ok(x),
            _ => Err(())
        }
    }
}

impl FromStr for ChannelId {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_channel(s) {
            Some(x) => Ok(ChannelId(x)),
            _ => Err(())
        }
    }
}

#[cfg(feature = "cache")]
impl FromStr for Channel {
    type Err = ();
    fn from_str(s: &str) -> StdResult<Self, ()> {
        match utils::parse_channel(s) {
            Some(x) => {
                match ChannelId(x).find() {
                    Some(channel) => Ok(channel),
                    _ => Err(())
                }
            },
            _ => Err(())
        }
    }
}

impl IncidentStatus {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        Self::decode_str(value)
    }
}
