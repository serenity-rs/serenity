use ::model::{ChannelId, RoleId};
use ::internal::prelude::*;

/// A builder which edits the properties of a [`Member`], to be used in
/// conjunction with [`Context::edit_member`] and [`Member::edit`].
///
/// [`Context::edit_member`]: ../client/struct.Context.html#method.edit_member
/// [`Member`]: ../model/struct.Member.html
/// [`Member::edit`]: ../model/struct.Member.html#method.edit
#[derive(Clone, Debug, Default)]
pub struct EditMember(pub JsonMap);

impl EditMember {
    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: ../model/permissions/constant.DEAFEN_MEMBERS.html
    pub fn deafen(mut self, deafen: bool) -> Self {
        self.0.insert("deaf".to_owned(), Value::Bool(deafen));

        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: ../model/permissions/constant.MUTE_MEMBERS.html
    pub fn mute(mut self, mute: bool) -> Self {
        self.0.insert("mute".to_owned(), Value::Bool(mute));

        self
    }

    /// Changes the member's nickname. Pass an empty string to reset the
    /// nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: ../model/permissions/constant.MANAGE_NICKNAMES.html
    pub fn nickname(mut self, nickname: &str) -> Self {
        self.0.insert("nick".to_owned(), Value::String(nickname.to_owned()));

        self
    }

    /// Set the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission to modify.
    ///
    /// [Manage Roles]: ../model/permissions/constant.MANAGE_ROLES.html
    pub fn roles(mut self, roles: &[RoleId]) -> Self {
        let role_ids = roles.iter().map(|x| Value::Number(Number::from(x.0))).collect();

        self.0.insert("roles".to_owned(), Value::Array(role_ids));

        self
    }

    /// The Id of the voice channel to move the member to.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../model/permissions/constant.MOVE_MEMBERS.html
    pub fn voice_channel<C: Into<ChannelId>>(mut self, channel_id: C) -> Self {
        self.0.insert("channel_id".to_owned(), Value::Number(Number::from(channel_id.into().0)));

        self
    }
}
