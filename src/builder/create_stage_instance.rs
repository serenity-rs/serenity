#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Builder for creating a stage instance
///
/// [Discord docs](https://discord.com/developers/docs/resources/stage-instance#create-stage-instance)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateStageInstance<'a> {
    channel_id: Option<ChannelId>, // required field, filled in Builder impl
    topic: String,
    privacy_level: StageInstancePrivacyLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    send_start_notification: Option<bool>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateStageInstance<'a> {
    /// Creates a builder with the provided topic.
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            channel_id: None,
            topic: topic.into(),
            privacy_level: StageInstancePrivacyLevel::default(),
            send_start_notification: None,
            audit_log_reason: None,
        }
    }

    /// Sets the topic of the stage channel instance, replacing the current value as set in
    /// [`Self::new`].
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }

    /// Whether or not to notify @everyone that a stage instance has started.
    pub fn send_start_notification(mut self, send_start_notification: bool) -> Self {
        self.send_start_notification = Some(send_start_notification);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for CreateStageInstance<'a> {
    type Context<'ctx> = ChannelId;
    type Built = StageInstance;

    /// Creates the stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is already a stage instance currently.
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<StageInstance> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.channel(ctx) {
                    if channel.kind != ChannelType::Stage {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self.channel_id = Some(ctx);
        cache_http.http().create_stage_instance(&self, self.audit_log_reason).await
    }
}
