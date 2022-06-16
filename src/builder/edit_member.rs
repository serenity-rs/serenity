use crate::model::id::{ChannelId, RoleId};
use crate::model::Timestamp;

/// A builder which edits the properties of a [`Member`], to be used in
/// conjunction with [`Member::edit`].
///
/// [`Member`]: crate::model::guild::Member
/// [`Member::edit`]: crate::model::guild::Member::edit
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditMember {
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    communication_disabled_until: Option<Option<String>>,
}

impl EditMember {
    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(&mut self, deafen: bool) -> &mut Self {
        self.deaf = Some(deafen);
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

    /// Changes the member's nickname. Pass an empty string to reset the
    /// nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname(&mut self, nickname: impl Into<String>) -> &mut Self {
        self.nick = Some(nickname.into());
        self
    }

    /// Set the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission to modify.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles(&mut self, roles: impl IntoIterator<Item = impl Into<RoleId>>) -> &mut Self {
        self.roles = Some(roles.into_iter().map(Into::into).collect());
        self
    }

    /// The Id of the voice channel to move the member to.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: crate::model::permissions::Permissions::MOVE_MEMBERS
    #[inline]
    pub fn voice_channel<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self.channel_id = Some(Some(channel_id.into()));
        self
    }

    /// Disconnects the user from their voice channel if any
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: crate::model::permissions::Permissions::MOVE_MEMBERS
    pub fn disconnect_member(&mut self) -> &mut Self {
        self.channel_id = Some(None);
        self
    }

    /// Times the user out until `time`, an ISO8601-formatted datetime string.
    ///
    /// `time` is considered invalid if it is not a valid ISO8601 timestamp or if it is greater
    /// than 28 days from the current time.
    ///
    /// Requires the [Moderate Members] permission.
    ///
    /// [Moderate Members]: crate::model::permissions::Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    pub fn disable_communication_until(&mut self, time: String) -> &mut Self {
        self.communication_disabled_until = Some(Some(time));
        self
    }

    /// Times the user out until `time`.
    ///
    /// `time` is considered invalid if it is greater than 28 days from the current time.
    /// Requires the [Moderate Members] permission.
    ///
    /// [Moderate Members]: crate::model::permissions::Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    pub fn disable_communication_until_datetime(&mut self, time: Timestamp) -> &mut Self {
        self.disable_communication_until(time.to_string());
        self
    }

    /// Allow a user to communicate, removing their timeout, if there is one.
    ///
    /// Requires the [Moderate Members] permission.
    ///
    /// [Moderate Members]: crate::model::permissions::Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    pub fn enable_communication(&mut self) -> &mut Self {
        self.communication_disabled_until = Some(None);
        self
    }
}
