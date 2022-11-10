use super::{CreateActionRow, CreateInputText, CreateInteractionResponse, CreateModal};
use crate::client::Context;
use crate::collector::ModalInteractionCollector;
use crate::model::id::InteractionId;
use crate::model::prelude::component::{ActionRowComponent, InputTextStyle};
use crate::model::prelude::ModalInteraction;

#[cfg(feature = "collector")]
pub struct QuickModalResponse {
    pub interaction: ModalInteraction,
    pub inputs: Vec<String>,
}

/// Convenience builder to create a modal, wait for the user to submit and parse the response.
///
/// ```rust
/// # use serenity::{builder::*, model::prelude::*, prelude::*, Result};
/// # async fn _foo(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
/// let modal = CreateQuickModal::new("About you")
///     .timeout(std::time::Duration::from_secs(600))
///     .short_field("First name")
///     .short_field("Last name")
///     .paragraph_field("Hobbies and interests");
/// let response = interaction.quick_modal(ctx, modal).await?;
/// let inputs = response.unwrap().inputs;
/// let (first_name, last_name, hobbies) = (&inputs[0], &inputs[1], &inputs[2]);
/// # Ok(()) }
/// ```
#[cfg(feature = "collector")]
#[must_use]
pub struct CreateQuickModal {
    title: String,
    timeout: Option<std::time::Duration>,
    input_texts: Vec<CreateInputText>,
}

#[cfg(feature = "collector")]
impl CreateQuickModal {
    pub fn new(title: impl Into<String>) -> Self {
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
    /// As the `custom_id` field of [`CreateInputText`], just supply an empty string. All custom IDs
    /// are overwritten by [`CreateQuickModal`] when sending the modal.
    pub fn field(mut self, input_text: CreateInputText) -> Self {
        self.input_texts.push(input_text);
        self
    }

    /// Convenience method to add a single-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn short_field(self, label: impl Into<String>) -> Self {
        self.field(CreateInputText::new(InputTextStyle::Short, label, ""))
    }

    /// Convenience method to add a multi-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn paragraph_field(self, label: impl Into<String>) -> Self {
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
        let modal_custom_id = interaction_id.get().to_string();
        let builder = CreateInteractionResponse::Modal(
            CreateModal::new(&modal_custom_id, self.title).components(
                self.input_texts
                    .into_iter()
                    .enumerate()
                    .map(|(i, input_text)| {
                        CreateActionRow::InputText(input_text.custom_id(i.to_string()))
                    })
                    .collect(),
            ),
        );
        builder.execute(ctx, interaction_id, token).await?;

        let modal_interaction = ModalInteractionCollector::new(&ctx.shard)
            .custom_ids(vec![modal_custom_id])
            .collect_single()
            .await;
        let modal_interaction = match modal_interaction {
            Some(x) => x,
            None => return Ok(None),
        };

        let inputs = modal_interaction
            .data
            .components
            .iter()
            .filter_map(|row| match row.components.first() {
                Some(ActionRowComponent::InputText(text)) => Some(text.value.clone()),
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
            inputs,
            interaction: modal_interaction,
        }))
    }
}
