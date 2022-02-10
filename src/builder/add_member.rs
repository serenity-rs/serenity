use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::model::id::RoleId;

/// A builder to add parameters when using [`GuildId::add_member`].
///
/// [`GuildId::add_member`]: crate::model::id::GuildId::add_member
#[derive(Clone, Debug, Default)]
pub struct AddMember(pub HashMap<&'static str, Value>);

impl AddMember {
    /// Sets the OAuth2 access token for this request.
    ///
    /// Requires the access token to have the `guilds.join` scope granted.
    pub fn access_token(&mut self, access_token: impl ToString) -> &mut Self {
        self.0.insert("access_token", Value::String(access_token.to_string()));
        self
    }

    /// Sets the member's nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname(&mut self, nickname: impl ToString) -> &mut Self {
        self.0.insert("nick", Value::String(nickname.to_string()));
        self
    }

    /// Sets the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles(&mut self, roles: impl IntoIterator<Item = impl AsRef<RoleId>>) -> &mut Self {
        let roles = roles.into_iter().map(|x| Value::Number(Number::from(x.as_ref().0))).collect();

        self.0.insert("roles", Value::Array(roles));
        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn mute(&mut self, mute: bool) -> &mut Self {
        self.0.insert("mute", Value::Bool(mute));
        self
    }

    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(&mut self, deafen: bool) -> &mut Self {
        self.0.insert("deaf", Value::Bool(deafen));
        self
    }
}
