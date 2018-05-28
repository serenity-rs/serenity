use chrono::{DateTime, FixedOffset};
use model::prelude::*;
use std::borrow::Cow;
use std::cell::RefCell;
use super::deserialize_user;

/// A trait for allowing both u8 or &str or (u8, &str) to be passed into the `ban` methods in `Guild` and `Member`.
pub trait BanOptions {
    fn dmd(&self) -> u8 { 0 }
    fn reason(&self) -> &str { "" }
}

impl BanOptions for u8 {
    fn dmd(&self) -> u8 { *self }
}

impl BanOptions for str {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for &'a str {
    fn reason(&self) -> &str { self }
}

impl BanOptions for String {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for (u8, &'a str) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { self.1 }
}

impl BanOptions for (u8, String) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { &self.1 }
}

/// Information about a member of a guild.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Member {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: GuildId,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Attached User struct.
    #[serde(deserialize_with = "deserialize_user",
            serialize_with = "serialize_user")]
    pub user: Rc<RefCell<User>>,
}

impl Member {
    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[inline]
    pub fn display_name(&self) -> Cow<String> {
        self.nick
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| {
                Cow::Owned(unsafe { (*self.user.as_ptr()).name.clone() })
            })
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[inline]
    pub fn distinct(&self) -> String {
        unsafe {
            let user = &*self.user.as_ptr();

            format!(
                "{}#{}",
                self.display_name(),
                user.discriminator,
            )
        }
    }
}

/// A partial amount of data for a member.
///
/// This is used in [`Message`]s from [`Guild`]s.
///
/// [`Guild`]: struct.Guild.html
/// [`Message`]: ../channel/struct.Message.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialMember {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
}
