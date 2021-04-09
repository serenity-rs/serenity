use std::collections::HashMap;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::json::{from_number, prelude::*, Value};
use crate::{
    model::interactions::{
        InteractionApplicationCommandCallbackDataFlags,
        InteractionResponseType,
    },
    utils,
};

#[derive(Clone, Debug)]
pub struct CreateInteractionResponse(pub HashMap<&'static str, Value>);

impl CreateInteractionResponse {
    /// Sets the InteractionResponseType of the message.
    ///
    /// Defaults to `Acknowledge`.
    pub fn kind(&mut self, kind: InteractionResponseType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));
        self
    }

    /// Sets the `InteractionApplicationCommandCallbackData` for the message.
    pub fn interaction_response_data<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateInteractionResponseData) -> &mut CreateInteractionResponseData,
    {
        let mut data = CreateInteractionResponseData::default();
        f(&mut data);
        let map = utils::hashmap_to_json_map(data.0);
        let data = Value::from(map);

        self.0.insert("data", data);
        self
    }
}

impl<'a> Default for CreateInteractionResponse {
    fn default() -> CreateInteractionResponse {
        let mut map = HashMap::new();
        map.insert("type", from_number(4));

        CreateInteractionResponse(map)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreateInteractionResponseData(pub HashMap<&'static str, Value>);

impl CreateInteractionResponseData {
    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.0.insert("tts", Value::from(tts));
        self
    }

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

    /// Create an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.set_embed(embed)
    }

    /// Set an embed for the message.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::Array(vec![]));

        if let Some(embeds) = embeds.as_array_mut() {
            embeds.push(embed);
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
        let allowed_mentions = Value::from(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }

    pub fn flags(&mut self, flags: InteractionApplicationCommandCallbackDataFlags) -> &mut Self {
        self.0.insert("flags", from_number(flags.bits()));
        self
    }
}
