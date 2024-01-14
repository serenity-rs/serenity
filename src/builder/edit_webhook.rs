use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/webhook#modify-webhook)
#[derive(Debug, Default, Clone, Serialize)]
#[must_use]
pub struct EditWebhook<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditWebhook<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the webhook's name.
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the channel to move the webhook to.
    pub fn channel_id(mut self, channel_id: ChannelId) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Set the webhook's default avatar.
    pub fn avatar(mut self, avatar: &CreateAttachment<'_>) -> Self {
        self.avatar = Some(Some(avatar.to_base64()));
        self
    }

    /// Delete the webhook's avatar, resetting it to the default logo.
    pub fn delete_avatar(mut self) -> Self {
        self.avatar = Some(None);
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
impl Builder for EditWebhook<'_> {
    type Context<'ctx> = (WebhookId, Option<&'ctx str>);
    type Built = Webhook;

    /// Edits the webhook corresponding to the provided [`WebhookId`] and token, and returns the
    /// resulting new [`Webhook`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Returns [`Error::Json`] if there is an error in deserialising Discord's response.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        match ctx.1 {
            Some(token) => {
                cache_http
                    .http()
                    .edit_webhook_with_token(ctx.0, token, &self, self.audit_log_reason)
                    .await
            },
            None => cache_http.http().edit_webhook(ctx.0, &self, self.audit_log_reason).await,
        }
    }
}
