use std::fmt;
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
use ::prelude_internal::*;

pub trait Mentionable {
    fn mention(&self) -> String;
}

impl Mentionable for ChannelId {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for Channel {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for Emoji {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for Member {
    fn mention(&self) -> String {
        format!("{}", self.user)
    }
}

impl Mentionable for RoleId {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for Role {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for UserId {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

impl Mentionable for User {
    fn mention(&self) -> String {
        format!("{}", self)
    }
}

/// A mention targeted at a certain model.
///
/// A mention can be created by calling `.mention()` on anything that is
/// mentionable - or an item's Id - and can be formatted into a string using
/// [`format!`]:
///
/// ```rust,ignore
/// let message = format!("Mentioning {}", user.mention());
/// ```
///
/// If a `String` is required, call `mention.to_string()`.
pub struct Mention {
    pub prefix: &'static str,
    pub id: u64,
}

impl fmt::Display for Mention {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str(self.prefix));
        try!(fmt::Display::fmt(&self.id, f));
        fmt::Write::write_char(f, '>')
    }
}

impl IncidentStatus {
    pub fn decode(value: Value) -> Result<Self> {
        Self::decode_str(value)
    }
}
