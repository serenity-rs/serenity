#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Builder for creating a [`StageInstance`].
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateStageInstance<'a> {
    channel_id: ChannelId,
    topic: String,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateStageInstance<'a> {
    /// Creates a builder with the provided Channel Id and topic.
    pub fn new(channel_id: impl Into<ChannelId>, topic: impl Into<String>) -> Self {
        Self {
            channel_id: channel_id.into(),
            topic: topic.into(),
            audit_log_reason: None,
        }
    }

    /// Creates the stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is already a stage instance currently.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn execute(self, cache_http: impl CacheHttp) -> Result<StageInstance> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(self.channel_id) {
                    if channel.kind != ChannelType::Stage {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http) -> Result<StageInstance> {
        http.create_stage_instance(&self, self.audit_log_reason).await
    }

    /// Sets the stage channel id of the stage channel instance, replacing the current value as set
    /// in [`Self::new`].
    pub fn channel_id(mut self, id: impl Into<ChannelId>) -> Self {
        self.channel_id = id.into();
        self
    }

    /// Sets the topic of the stage channel instance, replacing the current value as set in
    /// [`Self::new`].
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
