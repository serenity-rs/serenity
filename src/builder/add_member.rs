#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to add parameters when using [`GuildId::add_member`].
///
/// [`GuildId::add_member`]: crate::model::id::GuildId::add_member
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct AddMember {
    access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<RoleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
}

impl AddMember {
    /// Constructs a new builder with the given access token, leaving all other fields empty.
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            nick: None,
            roles: Vec::new(),
            mute: None,
            deaf: None,
        }
    }

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Option<Member>> {
        http.as_ref().add_guild_member(guild_id.into(), user_id.into(), &self).await
    }

    /// Sets the OAuth2 access token for this request, replacing the current one.
    ///
    /// Requires the access token to have the `guilds.join` scope granted.
    pub fn access_token(mut self, access_token: impl Into<String>) -> Self {
        self.access_token = access_token.into();
        self
    }

    /// Sets the member's nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nick = Some(nickname.into());
        self
    }

    /// Sets the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles(mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn mute(mut self, mute: bool) -> Self {
        self.mute = Some(mute);
        self
    }

    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(mut self, deafen: bool) -> Self {
        self.deaf = Some(deafen);
        self
    }
}
