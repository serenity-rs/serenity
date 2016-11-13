use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::default::Default;

/// A builder to create the inner content of a [`Webhook`]'s execution.
///
/// This is a structured way of cleanly creating the inner execution payload,
/// to reduce potential argument counts.
///
/// Refer to the documentation for [`execute_webhook`] on restrictions with
/// execution payloads and its fields.
///
/// [`Webhook`]: ../model/struct.Webhook.html
/// [`execute_webhook`]: ../client/http/fn.execute_webhook.html
pub struct ExecuteWebhook(pub ObjectBuilder);

impl ExecuteWebhook {
    /// Override the default avatar of the webhook with an image URL.
    pub fn avatar_url(self, avatar_url: &str) -> Self {
        ExecuteWebhook(self.0.insert("avatar_url", avatar_url))
    }

    /// Set the content of the message.
    pub fn content(self, content: &str) -> Self {
        ExecuteWebhook(self.0.insert("content", content))
    }

    // Set the embeds associated with the message.
    pub fn embeds(self, embeds: Vec<Value>) -> Self {
        ExecuteWebhook(self.0.insert("embeds", embeds))
    }

    /// Whether the message is a text-to-speech message.
    ///
    /// Think carefully before setting this to `true`.
    pub fn tts(self, tts: bool) -> Self {
        ExecuteWebhook(self.0.insert("tts", tts))
    }

    /// Override the default username of the webhook.
    pub fn username(self, username: &str) -> Self {
        ExecuteWebhook(self.0.insert("username", username))
    }
}

impl Default for ExecuteWebhook {
    /// Returns a default set of values for a [`Webhook`] execution.
    ///
    /// The only default value is `tts` being set to `true`. In the event that
    /// there is a bug that Discord defaults `tts` to `true`, at least
    /// serenity.rs won't be a part of it.
    fn default() -> ExecuteWebhook {
        ExecuteWebhook(ObjectBuilder::new().insert("tts", false))
    }
}
