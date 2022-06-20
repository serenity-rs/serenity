#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::id::RoleId;
#[cfg(feature = "http")]
use crate::model::prelude::*;

/// A builder to add parameters when using [`GuildId::add_member`].
///
/// [`GuildId::add_member`]: crate::model::id::GuildId::add_member
#[derive(Clone, Debug, Serialize)]
pub struct AddMember {
    #[cfg(feature = "http")]
    #[serde(skip)]
    guild_id: GuildId,
    #[cfg(feature = "http")]
    #[serde(skip)]
    user_id: UserId,

    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
}

impl AddMember {
    pub fn new(
        #[cfg(feature = "http")] guild_id: GuildId,
        #[cfg(feature = "http")] user_id: UserId,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            guild_id,
            #[cfg(feature = "http")]
            user_id,
            access_token: None,
            nick: None,
            roles: None,
            mute: None,
            deaf: None,
        }
    }

    /// Sets the OAuth2 access token for this request.
    ///
    /// Requires the access token to have the `guilds.join` scope granted.
    pub fn access_token(mut self, access_token: impl Into<String>) -> Self {
        self.access_token = Some(access_token.into());
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
        self.roles = Some(roles.into_iter().map(Into::into).collect());
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

    /// Adds a [`User`] to the corresponding guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid values are set.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<Option<Member>> {
        http.as_ref().add_guild_member(self.guild_id.into(), self.user_id.into(), &self).await
    }
}
