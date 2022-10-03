use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateWebhook<'a> {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateWebhook<'a> {
    /// Creates a new builder with the given webhook name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            avatar: None,
            audit_log_reason: None,
        }
    }

    /// Creates the webhook.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidChannelType`] if the
    /// corresponding channel is not of type [`Text`] or [`News`].
    ///
    /// If the provided name is less than 2 characters, returns [`ModelError::NameTooShort`]. If it
    /// is more than 100 characters, returns [`ModelError::NameTooLong`].
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission, or if invalid data is
    /// given.
    ///
    /// [`Text`]: ChannelType::Text
    /// [`News`]: ChannelType::News
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
    ) -> Result<Webhook> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(channel_id) {
                    if !channel.is_text_based() {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self._execute(cache_http.http(), channel_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, channel_id: ChannelId) -> Result<Webhook> {
        if self.name.len() < 2 {
            return Err(Error::Model(ModelError::NameTooShort));
        } else if self.name.len() > 100 {
            return Err(Error::Model(ModelError::NameTooLong));
        }

        http.create_webhook(channel_id, &self, self.audit_log_reason).await
    }

    /// Set the webhook's name, replacing the current value as set in [`Self::new`].
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the webhook's default avatar.
    pub fn avatar(mut self, avatar: &CreateAttachment) -> Self {
        self.avatar = Some(avatar.to_base64());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
