use std::collections::HashMap;

use crate::{internal::prelude::*, json::from_number};
use crate::{
    json::NULL,
    model::id::{ChannelId, RoleId},
};

/// A builder which edits the properties of a [`Member`], to be used in
/// conjunction with [`Member::edit`].
///
/// [`Member`]: crate::model::guild::Member
/// [`Member::edit`]: crate::model::guild::Member::edit
#[derive(Clone, Debug, Default)]
pub struct EditMember(pub HashMap<&'static str, Value>);

impl EditMember {
    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(&mut self, deafen: bool) -> &mut Self {
        self.0.insert("deaf", Value::from(deafen));
        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn mute(&mut self, mute: bool) -> &mut Self {
        self.0.insert("mute", Value::from(mute));
        self
    }

    /// Changes the member's nickname. Pass an empty string to reset the
    /// nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname<S: ToString>(&mut self, nickname: S) -> &mut Self {
        self.0.insert("nick", Value::String(nickname.to_string()));
        self
    }

    /// Set the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission to modify.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles<T: AsRef<RoleId>, It: IntoIterator<Item = T>>(&mut self, roles: It) -> &mut Self {
        let role_ids = roles.into_iter().map(|x| from_number(x.as_ref().0)).collect();

        self._roles(role_ids);
        self
    }

    fn _roles(&mut self, roles: Vec<Value>) {
        self.0.insert("roles", Value::Array(roles));
    }

    /// The Id of the voice channel to move the member to.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: crate::model::permissions::Permissions::MOVE_MEMBERS
    #[inline]
    pub fn voice_channel<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self._voice_channel(channel_id.into());

        self
    }

    fn _voice_channel(&mut self, channel_id: ChannelId) {
        let num = from_number(channel_id.0);
        self.0.insert("channel_id", num);
    }

    /// Disconnects the user from their voice channel if any
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: crate::model::permissions::Permissions::MOVE_MEMBERS
    pub fn disconnect_member(&mut self) -> &mut Self {
        self.0.insert("channel_id", NULL);

        self
    }
}
