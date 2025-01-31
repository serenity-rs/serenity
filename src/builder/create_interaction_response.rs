use std::borrow::Cow;
use std::collections::HashMap;

use super::create_poll::Ready;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    CreatePoll,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object).
#[derive(Clone, Debug)]
pub enum CreateInteractionResponse<'a> {
    /// Acknowledges a Ping (only required when your bot uses an HTTP endpoint URL).
    ///
    /// Corresponds to Discord's `PONG`.
    Pong,
    /// Responds to an interaction with a message.
    ///
    /// Corresponds to Discord's `CHANNEL_MESSAGE_WITH_SOURCE`.
    Message(CreateInteractionResponseMessage<'a>),
    /// Acknowledges the interaction in order to edit a response later. The user sees a loading
    /// state.
    ///
    /// Corresponds to Discord's `DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE`.
    Defer(CreateInteractionResponseMessage<'a>),
    /// Only valid for component-based interactions (seems to work for modal submit interactions
    /// too even though it's not documented).
    ///
    /// Acknowledges the interaction. You can optionally edit the original message later. The user
    /// does not see a loading state.
    ///
    /// Corresponds to Discord's `DEFERRED_UPDATE_MESSAGE`.
    Acknowledge,
    /// Only valid for component-based interactions.
    ///
    /// Edits the message the component was attached to.
    ///
    /// Corresponds to Discord's `UPDATE_MESSAGE`.
    UpdateMessage(CreateInteractionResponseMessage<'a>),
    /// Only valid for autocomplete interactions.
    ///
    /// Responds to the autocomplete interaction with suggested choices.
    ///
    /// Corresponds to Discord's `APPLICATION_COMMAND_AUTOCOMPLETE_RESULT`.
    Autocomplete(CreateAutocompleteResponse<'a>),
    /// Not valid for Modal and Ping interactions
    ///
    /// Responds to the interaction with a popup modal.
    ///
    /// Corresponds to Discord's `MODAL`.
    Modal(CreateModal<'a>),
    /// Not valid for autocomplete and Ping interactions. Only available for applications with
    /// Activities enabled.
    ///
    /// Responds to the interaction by launching the Activity associated with the app.
    ///
    /// Corresponds to Discord's `LAUNCH_ACTIVITY`.
    LaunchActivity,
}

impl serde::Serialize for CreateInteractionResponse<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &match self {
            Self::Pong => 1,
            Self::Message(_) => 4,
            Self::Defer(_) => 5,
            Self::Acknowledge => 6,
            Self::UpdateMessage(_) => 7,
            Self::Autocomplete(_) => 8,
            Self::Modal(_) => 9,
            Self::LaunchActivity => 12,
        })?;

        match self {
            Self::Pong => map.serialize_entry("data", &None::<()>)?,
            Self::Message(x) => map.serialize_entry("data", &x)?,
            Self::Defer(x) => map.serialize_entry("data", &x)?,
            Self::Acknowledge => map.serialize_entry("data", &None::<()>)?,
            Self::UpdateMessage(x) => map.serialize_entry("data", &x)?,
            Self::Autocomplete(x) => map.serialize_entry("data", &x)?,
            Self::Modal(x) => map.serialize_entry("data", &x)?,
            Self::LaunchActivity => map.serialize_entry("data", &None::<()>)?,
        }

        map.end()
    }
}

impl CreateInteractionResponse<'_> {
    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<(), ModelError> {
        if let CreateInteractionResponse::Message(data)
        | CreateInteractionResponse::Defer(data)
        | CreateInteractionResponse::UpdateMessage(data) = self
        {
            super::check_lengths(data.content.as_deref(), data.embeds.as_deref(), 0)
        } else {
            Ok(())
        }
    }

    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        http: &Http,
        interaction_id: InteractionId,
        interaction_token: &str,
    ) -> Result<()> {
        self.check_length()?;
        let files = match &mut self {
            CreateInteractionResponse::Message(msg)
            | CreateInteractionResponse::Defer(msg)
            | CreateInteractionResponse::UpdateMessage(msg) => msg.attachments.take_files(),
            _ => Vec::new(),
        };

        if let Self::Message(msg) | Self::Defer(msg) | Self::UpdateMessage(msg) = &mut self {
            if msg.allowed_mentions.is_none() {
                msg.allowed_mentions.clone_from(&http.default_allowed_mentions);
            }
        }

        http.create_interaction_response(interaction_id, interaction_token, &self, files).await
    }
}

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-messages).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInteractionResponseMessage<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Cow<'a, [CreateEmbed<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<InteractionResponseFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Cow<'a, [CreateActionRow<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    poll: Option<CreatePoll<'a, Ready>>,
    attachments: EditAttachments<'a>,
}

impl<'a> CreateInteractionResponseMessage<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

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
    pub fn add_file(mut self, file: CreateAttachment<'a>) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed to the message.
    ///
    /// Calling this while editing a message will overwrite existing embeds.
    pub fn add_embed(mut self, embed: CreateEmbed<'a>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().push(embed);
        self
    }

    /// Adds multiple embeds for the message.
    ///
    /// Calling this while editing a message will overwrite existing embeds.
    pub fn add_embeds(mut self, embeds: impl IntoIterator<Item = CreateEmbed<'a>>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().extend(embeds);
        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed<'a>) -> Self {
        self.embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embeds`]
    /// instead.
    pub fn embeds(mut self, embeds: impl Into<Cow<'a, [CreateEmbed<'a>]>>) -> Self {
        self.embeds = Some(embeds.into());
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: InteractionResponseFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Adds or removes the ephemeral flag.
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        let mut flags = self.flags.unwrap_or_else(InteractionResponseFlags::empty);

        if ephemeral {
            flags |= InteractionResponseFlags::EPHEMERAL;
        } else {
            flags &= !InteractionResponseFlags::EPHEMERAL;
        }

        self.flags = Some(flags);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        self.components = Some(components.into());
        self
    }

    /// Adds a poll to the message. Only one poll can be added per message.
    ///
    /// See [`CreatePoll`] for more information on creating and configuring a poll.
    pub fn poll(mut self, poll: CreatePoll<'a, Ready>) -> Self {
        self.poll = Some(poll);
        self
    }

    super::button_and_select_menu_convenience_methods!(self.components);
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
#[must_use]
pub enum AutocompleteValue<'a> {
    String(Cow<'a, str>),
    Integer(u64),
    Float(f64),
}

impl<'a> From<Cow<'a, str>> for AutocompleteValue<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::String(value)
    }
}

impl From<String> for AutocompleteValue<'static> {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for AutocompleteValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl From<u64> for AutocompleteValue<'static> {
    fn from(value: u64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for AutocompleteValue<'static> {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

// Same as CommandOptionChoice according to Discord, see
// [Autocomplete docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-autocomplete).
#[must_use]
#[derive(Clone, Debug, Serialize)]
pub struct AutocompleteChoice<'a> {
    pub name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    pub value: AutocompleteValue<'a>,
}

impl<'a> AutocompleteChoice<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, value: impl Into<AutocompleteValue<'a>>) -> Self {
        Self {
            name: name.into(),
            name_localizations: None,
            value: value.into(),
        }
    }

    pub fn add_localized_name(
        mut self,
        locale: impl Into<Cow<'a, str>>,
        localized_name: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.name_localizations
            .get_or_insert_with(Default::default)
            .insert(locale.into(), localized_name.into());
        self
    }
}

impl<'a, S: Into<Cow<'a, str>>> From<S> for AutocompleteChoice<'a> {
    fn from(value: S) -> Self {
        let value = value.into();
        let name = value.clone();
        Self::new(name, value)
    }
}

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-autocomplete)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateAutocompleteResponse<'a> {
    choices: Cow<'a, [AutocompleteChoice<'a>]>,
}

impl<'a> CreateAutocompleteResponse<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// For autocomplete responses this sets their autocomplete suggestions.
    ///
    /// See the official docs on [`Application Command Option Choices`] for more information.
    ///
    /// [`Application Command Option Choices`]: https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure
    pub fn set_choices(mut self, choices: impl Into<Cow<'a, [AutocompleteChoice<'a>]>>) -> Self {
        self.choices = choices.into();
        self
    }

    /// Add an autocomplete choice.
    ///
    /// # Limitations
    /// - There can be no more than 25 choices set.
    /// - Name must be between 1 and 100 characters.
    /// - If value is an integer/float must be between -2^53 and 2^53.
    pub fn add_choice(mut self, value: impl Into<AutocompleteChoice<'a>>) -> Self {
        self.choices.to_mut().push(value.into());
        self
    }

    /// Creates a response to an autocomplete interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: &Http,
        interaction_id: InteractionId,
        interaction_token: &str,
    ) -> Result<()> {
        http.create_interaction_response(interaction_id, interaction_token, &self, Vec::new()).await
    }
}

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-modal).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateModal<'a> {
    components: Cow<'a, [CreateActionRow<'a>]>,
    custom_id: Cow<'a, str>,
    title: Cow<'a, str>,
}

impl<'a> CreateModal<'a> {
    /// Creates a new modal.
    pub fn new(custom_id: impl Into<Cow<'a, str>>, title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            components: Cow::default(),
            custom_id: custom_id.into(),
            title: title.into(),
        }
    }

    /// Sets the components of this message.
    ///
    /// Overwrites existing components.
    pub fn components(mut self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        self.components = components.into();
        self
    }
}
