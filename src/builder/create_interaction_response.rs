use std::collections::HashMap;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::builder::CreateComponents;
use crate::json;
use crate::json::prelude::*;
use crate::model::application::interaction::{InteractionResponseType, MessageFlags};
use crate::model::channel::AttachmentType;

#[derive(Clone, Debug)]
pub struct CreateInteractionResponse<'a>(
    pub HashMap<&'static str, Value>,
    pub Vec<AttachmentType<'a>>,
);

impl<'a> CreateInteractionResponse<'a> {
    /// Sets the InteractionResponseType of the message.
    ///
    /// Defaults to `ChannelMessageWithSource`.
    pub fn kind(&mut self, kind: InteractionResponseType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));
        self
    }

    /// Sets the `InteractionApplicationCommandCallbackData` for the message.
    pub fn interaction_response_data<F>(&mut self, f: F) -> &mut Self
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseData<'a>,
        ) -> &'b mut CreateInteractionResponseData<'a>,
    {
        let mut data = CreateInteractionResponseData::default();
        f(&mut data);
        let map = json::hashmap_to_json_map(data.0);

        self.0.insert("data", Value::from(map));
        self.1 = data.1;
        self
    }
}

impl<'a> Default for CreateInteractionResponse<'a> {
    fn default() -> CreateInteractionResponse<'a> {
        let mut map = HashMap::new();
        map.insert("type", from_number(4));

        CreateInteractionResponse(map, Vec::new())
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreateInteractionResponseData<'a>(
    pub HashMap<&'static str, Value>,
    pub Vec<AttachmentType<'a>>,
);

impl<'a> CreateInteractionResponseData<'a> {
    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.0.insert("tts", Value::from(tts));
        self
    }

    /// Appends a file to the message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.1.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.1.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.1 = files.into_iter().map(Into::into).collect();
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
        self.0.insert("content", Value::from(content));
        self
    }

    /// Create an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.add_embed(embed)
    }

    /// Adds an embed to the message.
    pub fn add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = json::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::from(Vec::<Value>::new()));

        if let Some(embeds) = embeds.as_array_mut() {
            embeds.push(embed);
        }

        self
    }

    /// Adds multiple embeds for the message.
    pub fn add_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        for embed in embeds {
            self.add_embed(embed);
        }

        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = json::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);
        self.0.insert("embeds", Value::from(vec![embed]));

        self
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        let embeds = embeds
            .into_iter()
            .map(|embed| json::hashmap_to_json_map(embed.0).into())
            .collect::<Vec<Value>>();

        self.0.insert("embeds", Value::from(embeds));
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = json::hashmap_to_json_map(allowed_mentions.0);
        let allowed_mentions = Value::from(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.0.insert("flags", from_number(flags.bits()));
        self
    }

    /// Adds or removes the ephemeral flag
    pub fn ephemeral(&mut self, ephemeral: bool) -> &mut Self {
        let flags = self
            .0
            .get("flags")
            .map_or(0, |f| f.as_u64().expect("Interaction response flag was not a number"));

        let flags = if ephemeral {
            flags | MessageFlags::EPHEMERAL.bits()
        } else {
            flags & !MessageFlags::EPHEMERAL.bits()
        };

        self.0.insert("flags", from_number(flags));

        self
    }

    /// Creates components for this message.
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.0.insert("components", Value::from(components.0));
        self
    }

    /// Sets the components of this message.
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.0.insert("components", Value::Array(components.0));
        self
    }

    /// Sets the custom id for modal interactions
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::String(id.to_string()));
        self
    }

    /// Sets the title for modal interactions
    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.0.insert("title", Value::String(title.to_string()));
        self
    }
}

#[derive(Clone, Debug)]
pub struct CreateAutocompleteResponse(pub HashMap<&'static str, Value>);

impl Default for CreateAutocompleteResponse {
    fn default() -> CreateAutocompleteResponse {
        let mut map = HashMap::new();
        map.insert("choices", Value::Array(vec![]));
        CreateAutocompleteResponse(map)
    }
}

impl CreateAutocompleteResponse {
    /// For autocomplete responses this sets their autocomplete suggestions.
    ///
    /// See the official docs on [`Application Command Option Choices`] for more information.
    ///
    /// [`Application Command Option Choices`]: https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure
    pub fn set_choices(&mut self, choices: Value) -> &mut Self {
        self.0.insert("choices", choices);
        self
    }

    /// Add an int autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice<D: ToString>(&mut self, name: D, value: i64) -> &mut Self {
        let choice = json!({
            "name": name.to_string(),
            "value" : value
        });
        self.add_choice(choice)
    }

    /// Adds a string autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be up to 100 characters.
    pub fn add_string_choice<D: ToString, E: ToString>(&mut self, name: D, value: E) -> &mut Self {
        let choice = json!({
            "name": name.to_string(),
            "value": value.to_string()
        });
        self.add_choice(choice)
    }

    /// Adds a number autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice<D: ToString>(&mut self, name: D, value: f64) -> &mut Self {
        let choice = json!({
            "name": name.to_string(),
            "value" : value
        });
        self.add_choice(choice)
    }

    fn add_choice(&mut self, value: Value) -> &mut Self {
        let choices = self.0.entry("choices").or_insert_with(|| Value::Array(vec![]));
        let choices_arr = choices.as_array_mut().expect("Must be an array");
        choices_arr.push(value);

        self
    }
}
