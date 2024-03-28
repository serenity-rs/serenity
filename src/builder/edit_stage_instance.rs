use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Edits a [`StageInstance`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/stage-instance#modify-stage-instance)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditStageInstance<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<StageInstancePrivacyLevel>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditStageInstance<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic(mut self, topic: impl Into<Cow<'a, str>>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Sets the privacy level of the stage instance
    pub fn privacy_level(mut self, privacy_level: StageInstancePrivacyLevel) -> Self {
        self.privacy_level = Some(privacy_level);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Edits the stage instance
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or there is no stage
    /// instance currently.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, channel_id: ChannelId) -> Result<StageInstance> {
        http.edit_stage_instance(channel_id, &self, self.audit_log_reason).await
    }
}
