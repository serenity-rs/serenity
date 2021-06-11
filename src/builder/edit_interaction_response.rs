use std::collections::HashMap;

use serde_json::Value;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::builder::CreateComponents;
use crate::utils;

#[derive(Clone, Debug, Default)]
pub struct EditInteractionResponse(pub HashMap<&'static str, Value>);

impl EditInteractionResponse {
    /// Sets the `InteractionApplicationCommandCallbackData` for the message.

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self._content(content.to_string())
    }

    fn _content(&mut self, content: String) -> &mut Self {
        self.0.insert("content", Value::String(content));
        self
    }

    /// Creates an embed for the message.
    pub fn create_embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.add_embed(embed)
    }

    /// Adds an embed for the message.
    pub fn add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::Object(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::Array(vec![]));

        if let Some(embeds) = embeds.as_array_mut() {
            embeds.push(embed);
        }

        self
    }

    /// Sets the embeds for the message.
    ///
    /// **Note**: You can only have up to 10 embeds per message.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        if self.0.contains_key("embeds") {
            self.0.remove_entry("embeds");
        }

        for embed in embeds {
            self.add_embed(embed);
        }

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
        let allowed_mentions = Value::Object(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }

    /// Sets the components of this message.
    #[cfg(feature = "unstable_discord_api")]
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.0.insert("components", Value::Array(components.0));
        self
    }
}
