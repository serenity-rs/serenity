use std::borrow::Cow;

use super::DataUri;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::model::prelude::*;

/// A builder which edits the properties of the bot's [`Member`], to be used in conjunction with
/// [`GuildId::edit_current_member`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#modify-current-member)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditCurrentMember<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<Option<Cow<'a, str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<Option<DataUri<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<DataUri<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bio: Option<Option<Cow<'a, str>>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditCurrentMember<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the bots's nickname. Pass `None` or an empty string to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    pub fn nickname(mut self, nick: Option<Cow<'a, str>>) -> Self {
        self.nick = Some(nick);
        self
    }

    /// Changes the bot's guild-specific base64 encoded banner image.
    ///
    /// The `banner` must be base64-encoded 16:9 png/jpeg image data.
    pub fn banner(mut self, banner: Option<DataUri<'a>>) -> Self {
        self.banner = Some(banner);
        self
    }

    /// Changes the bot's guild-specific base64 encoded avatar image.
    ///
    /// The `avatar` must be base64-encoded png/jpeg image data.
    pub fn avatar(mut self, avatar: Option<DataUri<'a>>) -> Self {
        self.avatar = Some(avatar);
        self
    }

    /// Changes the bot's bio (about me) in the guild.
    pub fn bio(mut self, bio: Option<Cow<'a, str>>) -> Self {
        self.bio = Some(bio);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Edits the properties of the application's guild member.
    ///
    /// For details on permissions requirements, refer to each specific method.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<Member> {
        http.edit_current_member(guild_id, &self, self.audit_log_reason).await
    }
}
