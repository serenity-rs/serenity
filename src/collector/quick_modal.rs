use std::borrow::Cow;

use crate::builder::{
    CreateComponent,
    CreateInputText,
    CreateInteractionResponse,
    CreateLabel,
    CreateModal,
    CreateTextDisplay,
};
use crate::collector::ModalInteractionCollector;
use crate::gateway::client::Context;
use crate::internal::prelude::*;
use crate::model::prelude::*;

pub struct QuickModalResponse {
    pub interaction: ModalInteraction,
    pub inputs: FixedArray<FixedString<u16>>,
}

/// Convenience builder to create a modal, wait for the user to submit and parse the response.
///
/// ```rust
/// # use serenity::{builder::*, model::prelude::*, prelude::*, collector::*, Result};
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
#[must_use]
pub struct CreateQuickModal<'a> {
    title: Cow<'a, str>,
    timeout: Option<std::time::Duration>,
    components: Vec<CreateComponent<'a>>,
}

impl<'a> CreateQuickModal<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            timeout: None,
            components: Vec::new(),
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

    /// Adds a text display field.
    pub fn text(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.components.push(CreateComponent::TextDisplay(CreateTextDisplay::new(content)));
        self
    }

    /// Adds an input text field.
    pub fn field(
        mut self,
        label: impl Into<Cow<'a, str>>,
        input_text: CreateInputText<'a>,
    ) -> Self {
        self.components.push(CreateComponent::Label(CreateLabel::input_text(label, input_text)));
        self
    }

    /// Adds an input text field with a description.
    pub fn field_with_description(
        mut self,
        label: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
        input_text: CreateInputText<'a>,
    ) -> Self {
        self.components.push(CreateComponent::Label(
            CreateLabel::input_text(label, input_text).description(description),
        ));
        self
    }

    /// Convenience method to add a single-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn short_field(self, label: impl Into<Cow<'a, str>>) -> Self {
        let input_text =
            CreateInputText::new(InputTextStyle::Short, self.components.len().to_string());
        self.field(label, input_text)
    }

    /// Convenience method to add a multi-line input text field.
    ///
    /// Wraps [`Self::field`].
    pub fn paragraph_field(self, label: impl Into<Cow<'a, str>>) -> Self {
        let input_text =
            CreateInputText::new(InputTextStyle::Paragraph, self.components.len().to_string());
        self.field(label, input_text)
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
        let builder = CreateInteractionResponse::Modal(
            CreateModal::new(interaction_id.to_string(), self.title).components(self.components),
        );
        builder.execute(&ctx.http, interaction_id, token).await?;

        let collector = ModalInteractionCollector::new(ctx)
            .custom_ids(vec![FixedString::from_str_trunc(&interaction_id.to_string())]);

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
            .filter_map(|component| {
                if let Component::Label(label) = component
                    && let LabelComponent::InputText(text) = &label.component
                {
                    if let Some(value) = &text.value {
                        Some(value.clone())
                    } else {
                        tracing::warn!("input text value was empty in modal response");
                        None
                    }
                } else {
                    if !matches!(component, Component::TextDisplay(_)) {
                        tracing::warn!("expected input text in modal response, got {component:?}");
                    }
                    None
                }
            })
            .collect();

        Ok(Some(QuickModalResponse {
            inputs: FixedArray::from_vec_trunc(inputs),
            interaction: modal_interaction,
        }))
    }
}

pub trait QuickModal {
    fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal<'_>,
    ) -> impl Future<Output = Result<Option<QuickModalResponse>>>;
}

impl QuickModal for CommandInteraction {
    async fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal<'_>,
    ) -> Result<Option<QuickModalResponse>> {
        builder.execute(ctx, self.id, &self.token).await
    }
}

impl QuickModal for ComponentInteraction {
    async fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal<'_>,
    ) -> Result<Option<QuickModalResponse>> {
        builder.execute(ctx, self.id, &self.token).await
    }
}
