use std::collections::HashMap;

use super::CreateAllowedMentions;
use crate::internal::prelude::*;
use crate::utils;

/// A builder to specify the fields to edit in an existing [`Webhook`]'s message.
///
/// [`Webhook`]: crate::model::webhook::Webhook
#[derive(Clone, Debug, Default)]
pub struct EditWebhookMessage(pub HashMap<&'static str, Value>);

impl EditWebhookMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.0.insert("content", Value::String(content.to_string()));
        self
    }

    /// Set the embeds associated with the message.
    ///
    /// This should be used in combination with [`Embed::fake`], creating one
    /// or more fake embeds to send to the API.
    ///
    /// # Examples
    ///
    /// Refer to [struct-level documentation of `ExecuteWebhook`] for an example
    /// on how to use embeds.
    ///
    /// [`Embed::fake`]: crate::model::channel::Embed::fake
    /// [struct-level documentation of `ExecuteWebhook`]: crate::builder::ExecuteWebhook#examples
    #[inline]
    pub fn embeds(&mut self, embeds: Vec<Value>) -> &mut Self {
        self.0.insert("embeds", Value::Array(embeds));
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = utils::hashmap_to_json_map(allowed_mentions.0);
        let allowed_mentions = Value::from(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }
}
