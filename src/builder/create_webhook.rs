use std::borrow::Cow;

use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
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

    /// Creates the webhook.
    ///
    /// # Errors
    ///
    /// If the provided name is less than 2 characters, returns [`ModelError::TooSmall`]. If it
    /// is more than 100 characters, returns [`ModelError::TooLarge`].
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission, or if invalid data is
    /// given.
    ///
    /// [`Text`]: ChannelType::Text
    /// [`News`]: ChannelType::News
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, channel_id: ChannelId) -> Result<Webhook> {
        crate::model::error::Minimum::WebhookName.check_underflow(self.name.chars().count())?;
        crate::model::error::Maximum::WebhookName.check_overflow(self.name.chars().count())?;

        http.create_webhook(channel_id, &self, self.audit_log_reason).await
    }
}
