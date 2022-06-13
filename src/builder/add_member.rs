use crate::model::id::RoleId;

/// A builder to add parameters when using [`GuildId::add_member`].
///
/// [`GuildId::add_member`]: crate::model::id::GuildId::add_member
#[derive(Clone, Debug, Default, Serialize)]
pub struct AddMember {
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
    /// Sets the OAuth2 access token for this request.
    ///
    /// Requires the access token to have the `guilds.join` scope granted.
    pub fn access_token(&mut self, access_token: impl Into<String>) -> &mut Self {
        self.access_token = Some(access_token.into());
        self
    }

    /// Sets the member's nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname(&mut self, nickname: impl Into<String>) -> &mut Self {
        self.nick = Some(nickname.into());
        self
    }

    /// Sets the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles(&mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> &mut Self {
        self.roles = Some(roles.into_iter().map(Into::into).collect());
        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn mute(&mut self, mute: bool) -> &mut Self {
        self.mute = Some(mute);
        self
    }

    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(&mut self, deafen: bool) -> &mut Self {
        self.deaf = Some(deafen);
        self
    }
}
