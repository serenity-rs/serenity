use serde_json::builder::ObjectBuilder;
use std::default::Default;
use ::model::{ChannelId, RoleId};

/// A builder which edits the properties of a [`Member`], to be used in
/// conjunction with [`Context::edit_member`] and [`Member::edit`].
///
/// [`Context::edit_member`]: ../../client/struct.Context.html#method.edit_member
/// [`Member`]: ../../model/struct.Member.html
/// [`Member::edit`]: ../../model/struct.Member.html#method.edit
pub struct EditMember(pub ObjectBuilder);

impl EditMember {
    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: ../../model/permissions/constant.DEAFEN_MEMBERS.html
    pub fn deafen(self, deafen: bool) -> Self {
        EditMember(self.0.insert("deaf", deafen))
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: ../../model/permissions/constant.MUTE_MEMBERS.html
    pub fn mute(self, mute: bool) -> Self {
        EditMember(self.0.insert("mute", mute))
    }

    /// Changes the member's nickname. Pass an empty string to reset the
    /// nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: ../../model/permissions/constant.MANAGE_NICKNAMES.html
    pub fn nickname(self, nickname: &str) -> Self {
        EditMember(self.0.insert("nick", nickname))
    }

    /// Set the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission to modify.
    ///
    /// [Manage Roles]: ../../model/permissions/constant.MANAGE_ROLES.html
    pub fn roles(self, roles: &[RoleId]) -> Self {
        EditMember(self.0
            .insert_array("roles",
                          |a| roles.iter().fold(a, |a, id| a.push(id.0))))
    }

    /// The Id of the voice channel to move the member to.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../../model/permissions/constant.MOVE_MEMBERS.html
    pub fn voice_channel<C: Into<ChannelId>>(self, channel_id: C) -> Self {
        EditMember(self.0.insert("channel_id", channel_id.into().0))
    }
}

impl Default for EditMember {
    fn default() -> EditMember {
        EditMember(ObjectBuilder::new())
    }
}
