use super::{
    ChannelId,
    Channel,
    Emoji,
    Member,
    RoleId,
    Role,
    UserId,
    User,
    IncidentStatus
};
use ::internal::prelude::*;

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

impl IncidentStatus {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        Self::decode_str(value)
    }
}
