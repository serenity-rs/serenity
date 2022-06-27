#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

/// A builder to specify the fields to edit in an existing [`Webhook`]'s message.
///
/// [`Webhook`]: crate::model::webhook::Webhook
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct EditWebhookMessage<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    webhook: &'a Webhook,
    #[cfg(not(feature = "http"))]
    webhook: PhantomData<&'a ()>,
    #[cfg(feature = "http")]
    #[serde(skip)]
    message_id: MessageId,

    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<CreateEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
}

impl<'a> EditWebhookMessage<'a> {
    pub fn new(
        #[cfg(feature = "http")] webhook: &'a Webhook,
        #[cfg(feature = "http")] message_id: MessageId,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            webhook,
            #[cfg(not(feature = "http"))]
            webhook: PhantomData::default(),
            #[cfg(feature = "http")]
            message_id,

            content: None,
            embeds: None,
            allowed_mentions: None,
            components: None,
        }
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set the embeds associated with the message.
    ///
    /// # Examples
    ///
    /// Refer to [struct-level documentation of `ExecuteWebhook`] for an example
    /// on how to use embeds.
    ///
    /// [struct-level documentation of `ExecuteWebhook`]: crate::builder::ExecuteWebhook#examples
    #[inline]
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = Some(embeds);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Creates components for this message. Requires an application-owned webhook, meaning either
    /// the webhook's `kind` field is set to [`WebhookType::Application`], or it was created by an
    /// application (and has kind [`WebhookType::Incoming`]).
    ///
    /// [`WebhookType::Application`]: crate::model::webhook::WebhookType
    /// [`WebhookType::Incoming`]: crate::model::webhook::WebhookType
    pub fn components<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.components = Some(components);
        self
    }

    /// Edits the webhook message.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the token field of the current webhook is `None`.
    ///
    /// May also return an [`Error::Http`] if the content is malformed, the webhook's token is
    /// invalid, or the given message Id does not belong to the current webhook.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<Message> {
        let token = self.webhook.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        http.as_ref()
            .edit_webhook_message(self.webhook.id.into(), token, self.message_id.into(), &self)
            .await
    }
}
