use serde_json::builder::ObjectBuilder;
use std::default::Default;
use ::model::{ChannelId, RoleId};

/// A builder that edits properties of a [`Member`] via
/// [`Context::edit_member`]: ../../client/struct.Context.html

pub struct EditMember(pub ObjectBuilder);

impl EditMember {
    /// Deafens a member.
    pub fn deafen(self, deafen: bool) -> Self {
        EditMember(self.0.insert("deaf", deafen))
    }

    /// Mutes a member.
    pub fn mute(self, mute: bool) -> Self {
        EditMember(self.0.insert("mute", mute))
    }

    /// Changes a members nickname. If the string is empty the nickname will be reset.
    pub fn nickname(self, nickname: &str) -> Self {
        EditMember(self.0.insert("nick", nickname))
    }

    pub fn roles(self, roles: &[RoleId]) -> Self {
        EditMember(self.0
             .insert_array("roles",
                           |a| roles.iter().fold(a, |a, id| a.push(id.0))))

    /// The Id of the voice channel the member is currently in.
    /// If that member is in a voice channel.
    pub fn voice_channel<C: Into<ChannelId>>(self, channel_id: C) -> Self {
        EditMember(self.0.insert("channel_id", channel_id.into().0))
    }
}

impl Default for EditMember {
    fn default() -> EditMember {
        EditMember(ObjectBuilder::new())
    }
}
