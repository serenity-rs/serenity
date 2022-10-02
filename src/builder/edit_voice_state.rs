#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder which edits a user's voice state, to be used in conjunction with
/// [`GuildChannel::edit_voice_state`].
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditVoiceState {
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_to_speak_timestamp: Option<Option<Timestamp>>,
}

impl EditVoiceState {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the given user's voice state in a stage channel. Pass [`None`] for `user_id` to edit
    /// the current user's voice state.
    ///
    /// **Note**: Requires the [Request to Speak] permission. Also requires the [Mute Members]
    /// permission to suppress another user or unsuppress the current user. This is not required if
    /// suppressing the current user.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidChannelType`] if the channel is
    /// not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        channel_id: ChannelId,
        user_id: Option<UserId>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(channel_id) {
                    if channel.kind != ChannelType::Stage {
                        return Err(Error::from(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self.channel_id = Some(channel_id);
        self._execute(cache_http.http(), guild_id, user_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, guild_id: GuildId, user_id: Option<UserId>) -> Result<()> {
        if let Some(user_id) = user_id {
            http.edit_voice_state(guild_id, user_id, &self).await
        } else {
            http.edit_voice_state_me(guild_id, &self).await
        }
    }

    /// Whether to suppress the user. Setting this to false will invite a user to speak.
    ///
    /// **Note**: Requires the [Mute Members] permission to suppress another user or unsuppress the
    /// current user. This is not required if suppressing the current user.
    ///
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    pub fn suppress(mut self, deafen: bool) -> Self {
        self.suppress = Some(deafen);
        self
    }

    /// Requests or clears a request to speak. Passing `true` is equivalent to passing the current
    /// time to [`Self::request_to_speak_timestamp`].
    ///
    /// **Note**: Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak(mut self, request: bool) -> Self {
        self.request_to_speak_timestamp = Some(request.then(Timestamp::now));
        self
    }

    /// Sets the current bot user's request to speak timestamp. This can be any present or future
    /// time.
    ///
    /// **Note**: Requires the [Request to Speak] permission.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    pub fn request_to_speak_timestamp(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.request_to_speak_timestamp = Some(Some(timestamp.into()));
        self
    }
}
