#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
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

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for CreateWebhook<'a> {
    type Context<'ctx> = ChannelId;
    type Built = Webhook;

    /// Creates the webhook.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidChannelType`] if the corresponding
    /// channel is not of type [`Text`] or [`News`].
    ///
    /// If the provided name is less than 2 characters, returns [`ModelError::NameTooShort`]. If it
    /// is more than 100 characters, returns [`ModelError::NameTooLong`].
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission, or if invalid data is
    /// given.
    ///
    /// [`Text`]: ChannelType::Text
    /// [`News`]: ChannelType::News
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(ctx) {
                    if !channel.is_text_based() {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        if self.name.len() < 2 {
            return Err(Error::Model(ModelError::NameTooShort));
        } else if self.name.len() > 100 {
            return Err(Error::Model(ModelError::NameTooLong));
        }

        cache_http.http().create_webhook(ctx, &self, self.audit_log_reason).await
    }
}
