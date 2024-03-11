use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/webhook#create-webhook)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateWebhook<'a> {
    name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateWebhook<'a> {
    /// Creates a new builder with the given webhook name, leaving all other fields empty.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            avatar: None,
            audit_log_reason: None,
        }
    }

    /// Set the webhook's name, replacing the current value as set in [`Self::new`].
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the webhook's default avatar.
    pub fn avatar(mut self, avatar: &CreateAttachment<'_>) -> Self {
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
    /// If the provided name is less than 2 characters, returns [`ModelError::TooSmall`]. If it
    /// is more than 100 characters, returns [`ModelError::TooLarge`].
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
                if let Some(channel) = cache.channel(ctx) {
                    // forum channels are not text-based, but webhooks can be created in them
                    // and used to send messages in their posts
                    if !channel.is_text_based() && channel.kind != ChannelType::Forum {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        crate::model::error::Minimum::WebhookName.check_underflow(self.name.chars().count())?;
        crate::model::error::Maximum::WebhookName.check_overflow(self.name.chars().count())?;

        cache_http.http().create_webhook(ctx, &self, self.audit_log_reason).await
    }
}
