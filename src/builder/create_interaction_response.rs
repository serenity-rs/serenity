#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::application::interaction::{InteractionResponseType, MessageFlags};
use crate::model::channel::AttachmentType;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInteractionResponse<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    id: InteractionId,
    #[serde(skip)]
    #[cfg(feature = "http")]
    token: &'a str,
    #[cfg(not(feature = "http"))]
    token: PhantomData<&'a ()>,

    #[serde(rename = "type")]
    kind: InteractionResponseType,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<CreateInteractionResponseData<'a>>,
}

impl<'a> CreateInteractionResponse<'a> {
    pub fn new(
        #[cfg(feature = "http")] id: InteractionId,
        #[cfg(feature = "http")] token: &'a str,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            id,
            #[cfg(feature = "http")]
            token,
            #[cfg(not(feature = "http"))]
            token: PhantomData::default(),

            kind: InteractionResponseType::ChannelMessageWithSource,
            data: None,
        }
    }

    /// Sets the InteractionResponseType of the message.
    ///
    /// Defaults to `ChannelMessageWithSource`.
    pub fn kind(mut self, kind: InteractionResponseType) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the `InteractionApplicationCommandCallbackData` for the message.
    pub fn interaction_response_data(mut self, data: CreateInteractionResponseData<'a>) -> Self {
        self.data = Some(data);
        self
    }

    /// Creates a response to the corresponding interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if invalid data is provided, or an [`Error::Json`] if there is an error
    /// when deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(mut self, http: impl AsRef<Http>) -> Result<()> {
        self.check_lengths()?;
        let files = self.data.as_mut().map_or_else(Vec::new, |d| std::mem::take(&mut d.files));

        if files.is_empty() {
            http.as_ref().create_interaction_response(self.id.into(), self.token, &self).await
        } else {
            http.as_ref()
                .create_interaction_response_with_files(self.id.into(), self.token, &self, files)
                .await
        }
    }

    #[cfg(feature = "http")]
    fn check_lengths(&self) -> Result<()> {
        if let Some(ref data) = self.data {
            if let Some(ref content) = data.content {
                let length = content.chars().count();
                let max_length = constants::MESSAGE_CODE_LIMIT;
                if length > max_length {
                    let overflow = length - max_length;
                    return Err(Error::Model(ModelError::MessageTooLong(overflow)));
                }
            }

            if data.embeds.len() > constants::EMBED_MAX_COUNT {
                return Err(Error::Model(ModelError::EmbedAmount));
            }
            for embed in &data.embeds {
                embed.check_length()?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInteractionResponseData<'a> {
    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip)]
    files: Vec<AttachmentType<'a>>,
}

impl<'a> CreateInteractionResponseData<'a> {
    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = Some(tts);
        self
    }

    /// Appends a file to the message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(mut self, file: T) -> Self {
        self.files.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Create an embed for the message.
    pub fn embed<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.add_embed(embed)
    }

    /// Adds an embed to the message.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Adds multiple embeds for the message.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embed(self, embed: CreateEmbed) -> Self {
        self.set_embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds.into_iter().collect();
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

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Adds or removes the ephemeral flag
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        let flags = self.flags.unwrap_or_else(MessageFlags::empty);

        let flags = if ephemeral {
            flags | MessageFlags::EPHEMERAL
        } else {
            flags & !MessageFlags::EPHEMERAL
        };

        self.flags = Some(flags);
        self
    }

    /// Creates components for this message.
    pub fn components<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(mut self, components: CreateComponents) -> Self {
        self.components = Some(components);
        self
    }

    /// Sets the custom id for modal interactions
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = Some(id.into());
        self
    }

    /// Sets the title for modal interactions
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[non_exhaustive]
pub struct AutocompleteChoice {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateAutocompleteResponse {
    choices: Vec<AutocompleteChoice>,
}

impl CreateAutocompleteResponse {
    /// For autocomplete responses this sets their autocomplete suggestions.
    ///
    /// See the official docs on [`Application Command Option Choices`] for more information.
    ///
    /// [`Application Command Option Choices`]: https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure
    pub fn set_choices(&mut self, choices: Vec<AutocompleteChoice>) -> &mut Self {
        self.choices = choices;
        self
    }

    /// Add an int autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice(&mut self, name: impl Into<String>, value: i64) -> &mut Self {
        self.add_choice(AutocompleteChoice {
            name: name.into(),
            value: Value::from(value),
        })
    }

    /// Adds a string autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be up to 100 characters.
    pub fn add_string_choice(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.add_choice(AutocompleteChoice {
            name: name.into(),
            value: Value::String(value.into()),
        })
    }

    /// Adds a number autocomplete choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice(&mut self, name: impl Into<String>, value: f64) -> &mut Self {
        self.add_choice(AutocompleteChoice {
            name: name.into(),
            value: Value::from(value),
        })
    }

    fn add_choice(&mut self, value: AutocompleteChoice) -> &mut Self {
        self.choices.push(value);
        self
    }
}
