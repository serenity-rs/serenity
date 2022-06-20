#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(all(feature = "http", feature = "cache"))]
use crate::model::channel::ChannelType;
#[cfg(feature = "http")]
use crate::model::channel::StageInstance;
use crate::model::id::ChannelId;
#[cfg(all(feature = "http", feature = "cache"))]
use crate::model::prelude::*;

/// Creates a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateStageInstance {
    channel_id: ChannelId,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
}

impl CreateStageInstance {
    pub fn new(id: impl Into<ChannelId>) -> Self {
        Self {
            channel_id: id.into(),
            topic: None,
        }
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Creates the stage instance.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidChannelType`] if the channel is not
    /// a stage channel. Otherwise, returns [`Error::Http`], as well as if there is a already a
    /// stage instance currently.
    #[cfg(feature = "http")]
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

        cache_http.http().create_stage_instance(&self).await
    }
}
