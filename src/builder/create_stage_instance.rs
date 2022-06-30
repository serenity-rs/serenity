#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Builder for creating a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateStageInstance {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
}

impl CreateStageInstance {
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
            if let Some(channel_id) = self.channel_id {
                if let Some(cache) = cache_http.cache() {
                    if let Some(channel) = cache.guild_channel(channel_id) {
                        if channel.kind != ChannelType::Stage {
                            return Err(Error::Model(ModelError::InvalidChannelType));
                        }
                    }
                }
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http) -> Result<StageInstance> {
        http.create_stage_instance(&self).await
    }

    /// Sets the stage channel id of the stage channel instance.
    pub fn channel_id(mut self, id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }
}
