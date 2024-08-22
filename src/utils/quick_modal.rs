use std::borrow::Cow;

use crate::builder::{CreateActionRow, CreateInputText, CreateInteractionResponse, CreateModal};
use crate::collector::ModalInteractionCollector;
use crate::gateway::client::Context;
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[cfg(feature = "collector")]
pub struct QuickModalResponse {
    pub interaction: ModalInteraction,
    pub inputs: FixedArray<FixedString<u16>>,
}

/// Convenience builder to create a modal, wait for the user to submit and parse the response.
///
/// ```rust
/// # use serenity::{builder::*, model::prelude::*, prelude::*, utils::CreateQuickModal, Result};
/// # async fn foo_(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
/// let modal = CreateQuickModal::new("About you")
///     .timeout(std::time::Duration::from_secs(600))
///     .short_field("First name")
///     .short_field("Last name")
///     .paragraph_field("Hobbies and interests");
/// let response = interaction.quick_modal(ctx, modal).await?;
/// let inputs = response.unwrap().inputs;
/// let (first_name, last_name, hobbies) = (&inputs[0], &inputs[1], &inputs[2]);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "collector")]
#[must_use]
pub struct CreateQuickModal<'a> {
    title: Cow<'a, str>,
    timeout: Option<std::time::Duration>,
    input_texts: Vec<CreateInputText<'a>>,
}

#[cfg(feature = "collector")]
impl<'a> CreateQuickModal<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            timeout: None,
            input_texts: Vec::new(),
        }
    }

    /// Sets a timeout when waiting for the modal response.
    ///
    /// You should almost always set a timeout here. Otherwise, if the user exits the modal, you
    /// will wait forever.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Adds an input text field.
    ///
    /// As the `custom_id` field of [`CreateInputText`], just supply an empty string. All custom
    /// IDs are overwritten by [`CreateQuickModal`] when sending the modal.
    pub fn field(mut self, input_text: CreateInputText<'a>) -> Self {
        self.input_texts.push(input_text);
        self
    }

    /// Convenience method to add a single-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn short_field(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.field(CreateInputText::new(InputTextStyle::Short, label, ""))
    }

    /// Convenience method to add a multi-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn paragraph_field(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.field(CreateInputText::new(InputTextStyle::Paragraph, label, ""))
    }

    /// # Errors
    ///
    /// See [`CreateInteractionResponse::execute()`].
    pub async fn execute(
        self,
        ctx: &Context,
        interaction_id: InteractionId,
        token: &str,
    ) -> Result<Option<QuickModalResponse>, crate::Error> {
        let modal_custom_id = interaction_id.to_arraystring();
        let builder = CreateInteractionResponse::Modal(
            CreateModal::new(modal_custom_id.as_str(), self.title).components(
                self.input_texts
                    .into_iter()
                    .enumerate()
                    .map(|(i, input_text)| {
                        CreateActionRow::InputText(input_text.custom_id(i.to_string()))
                    })
                    .collect::<Vec<_>>(),
            ),
        );
        builder.execute(&ctx.http, interaction_id, token).await?;

        let collector = ModalInteractionCollector::new(ctx.shard.clone())
            .custom_ids(vec![FixedString::from_str_trunc(&modal_custom_id)]);

        let collector = match self.timeout {
            Some(timeout) => collector.timeout(timeout),
            None => collector,
        };

        let modal_interaction = collector.next().await;

        let Some(modal_interaction) = modal_interaction else { return Ok(None) };

        let inputs = modal_interaction
            .data
            .components
            .iter()
            .filter_map(|row| match row.components.first() {
                Some(ActionRowComponent::InputText(text)) => {
                    if let Some(value) = &text.value {
                        Some(value.clone())
                    } else {
                        tracing::warn!("input text value was empty in modal response");
                        None
                    }
                },
                Some(other) => {
                    tracing::warn!("expected input text in modal response, got {:?}", other);
                    None
                },
                None => {
                    tracing::warn!("empty action row");
                    None
                },
            })
            .collect();

        Ok(Some(QuickModalResponse {
            inputs: FixedArray::from_vec_trunc(inputs),
            interaction: modal_interaction,
        }))
    }
}
