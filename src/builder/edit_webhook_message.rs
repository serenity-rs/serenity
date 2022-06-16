use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};

/// A builder to specify the fields to edit in an existing [`Webhook`]'s message.
///
/// [`Webhook`]: crate::model::webhook::Webhook
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditWebhookMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<CreateEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
}

impl EditWebhookMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
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
    pub fn embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds = Some(embeds);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
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
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.components = Some(components);
        self
    }
}
