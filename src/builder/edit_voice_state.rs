use crate::model::{Timestamp, id::ChannelId};

/// A builder which edits a user's voice state, to be used in conjunction with
/// [`GuildChannel::edit_voice_state`].
///
/// [`GuildChannel::edit_voice_state`]: crate::model::channel::GuildChannel::edit_voice_state
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditVoiceState {
    pub(crate) channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_to_speak_timestamp: Option<Option<Timestamp>>
}

impl EditVoiceState {
    /// Whether to suppress the user. Setting this to false will invite a user
    /// to speak.
    ///
    /// Requires the [Mute Members] permission to suppress another user or
    /// unsuppress the current user.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn suppress(&mut self, deafen: bool) -> &mut Self {
        self.suppress = Some(deafen);
        self
    }

    /// Requests or clears a request to speak. This is equivalent to passing the
    /// current time to [`Self::request_to_speak_timestamp`].
    ///
    /// Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: crate::model::permissions::Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak(&mut self, request: bool) -> &mut Self {
        if request {
            self.request_to_speak_timestamp(Some(Timestamp::now()));
        } else {
            self.request_to_speak_timestamp(None::<Timestamp>);
        }

        self
    }

    /// Sets the current bot user's request to speak timestamp. This can be any
    /// present or future time. Set this to [`None`] to clear a request to speak.
    ///
    /// Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: crate::model::permissions::Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak_timestamp<T: Into<Timestamp>>(
        &mut self,
        timestamp: Option<T>,
    ) -> &mut Self {
        self.request_to_speak_timestamp = Some(timestamp.map(Into::into));
        self
    }
}
