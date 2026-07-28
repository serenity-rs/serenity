use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// Edits a [`StageInstance`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/stage-instance#modify-stage-instance)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditStageInstance<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<StageInstancePrivacyLevel>,

    #[serde(skip)]
    audit_log_reason: Option<Cow<'a, str>>,
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
    pub fn audit_log_reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.audit_log_reason = Some(reason.into());
        self
    }

    pub fn into_owned(self) -> EditStageInstance<'static> {
        let Self {
            topic,
            privacy_level,
            audit_log_reason,
        } = self;
        EditStageInstance {
            topic: topic.map(|t| t.into_owned().into()),
            privacy_level,
            audit_log_reason: audit_log_reason.map(|r| r.into_owned().into()),
        }
    }

    /// Edits the stage instance
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or there is no stage
    /// instance currently.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, channel_id: ChannelId) -> Result<StageInstance> {
        http.edit_stage_instance(channel_id, &self, self.audit_log_reason.as_deref()).await
    }
}
