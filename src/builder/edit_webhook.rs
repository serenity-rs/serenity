use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[derive(Debug, Default, Clone, Serialize)]
#[must_use]
pub struct EditWebhook<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditWebhook<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the webhook corresponding to the provided Id and token, and returns the resulting new
    /// [`Webhook`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Returns [`Error::Json`] if there is an error in deserialising Discord's response.
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        webhook_id: WebhookId,
        token: Option<&str>,
    ) -> Result<Webhook> {
        let id = webhook_id;
        match token {
            Some(token) => {
                http.as_ref().edit_webhook_with_token(id, token, &self, self.audit_log_reason).await
            },
            None => http.as_ref().edit_webhook(id, &self, self.audit_log_reason).await,
        }
    }

    /// Set the webhook's name.
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the channel to move the webhook to.
    pub fn channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Set the webhook's default avatar.
    pub fn avatar(mut self, avatar: &CreateAttachment) -> Self {
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
