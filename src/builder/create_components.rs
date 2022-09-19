use crate::model::application::component::{ButtonStyle, InputTextStyle};
use crate::model::channel::ReactionType;
use crate::model::prelude::component::{
    ActionRow,
    ActionRowComponent,
    Button,
    ComponentType,
    InputText,
    SelectMenu,
    SelectMenuOption,
};

/// A builder for creating an [`ActionRow`].
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateActionRow(ActionRow);

impl Default for CreateActionRow {
    fn default() -> Self {
        Self(ActionRow {
            components: Vec::new(),
            kind: ComponentType::ActionRow,
        })
    }
}

impl CreateActionRow {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a button.
    pub fn add_button(mut self, button: CreateButton) -> Self {
        self.0.components.push(ActionRowComponent::Button(button.0));
        self
    }

    /// Adds a select menu.
    pub fn add_select_menu(mut self, menu: CreateSelectMenu) -> Self {
        self.0.components.push(ActionRowComponent::SelectMenu(menu.0));
        self
    }

    /// Adds an input text.
    pub fn add_input_text(mut self, input_text: CreateInputText) -> Self {
        self.0.components.push(ActionRowComponent::InputText(input_text.0));
        self
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::application::component::Button
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateButton(Button);

impl Default for CreateButton {
    /// Creates a primary button.
    fn default() -> Self {
        Self(Button {
            style: ButtonStyle::Primary,
            label: None,
            custom_id: None,
            url: None,
            emoji: None,
            disabled: false,
            kind: ComponentType::Button,
        })
    }
}

impl CreateButton {
    /// Creates a primary button. Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the style of the button.
    pub fn style(mut self, kind: ButtonStyle) -> Self {
        self.0.style = kind;
        self
    }

    /// The label of the button.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    /// Sets the custom id of the button, a developer-defined identifier.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = Some(id.into());
        self
    }

    /// The url for url style button.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    /// Sets emoji of the button.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.0.emoji = Some(emoji.into());
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::component::SelectMenu
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenu(SelectMenu);
impl CreateSelectMenu {
    /// Creates a builder with given custom id (a developer-defined identifier), and a list of
    /// options, leaving all other fields empty.
    pub fn new(custom_id: impl Into<String>, options: Vec<CreateSelectMenuOption>) -> Self {
        Self(SelectMenu {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
            values: vec![],
            options: options.into_iter().map(|x| x.0).collect(),
            kind: ComponentType::SelectMenu,
        })
    }

    /// The placeholder of the select menu.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.0.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = id.into();
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(mut self, min: u64) -> Self {
        self.0.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(mut self, max: u64) -> Self {
        self.0.max_values = Some(max);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }

    pub fn options(mut self, options: Vec<CreateSelectMenuOption>) -> Self {
        self.0.options = options.into_iter().map(|x| x.0).collect();
        self
    }
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenuOption(SelectMenuOption);

impl CreateSelectMenuOption {
    /// Creates a select menu option with the given label and value, leaving all other fields
    /// empty.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self(SelectMenuOption {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: false,
        })
    }

    /// Sets the label of this option, replacing the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = label.into();
        self
    }

    /// Sets the value of this option, replacing the current value as set in [`Self::new`].
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = value.into();
        self
    }

    /// Sets the description shown on this option.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.0.description = Some(description.into());
        self
    }

    /// Sets emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.0.emoji = Some(emoji.into());
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(mut self, disabled: bool) -> Self {
        self.0.default = disabled;
        self
    }
}

/// A builder for creating an [`InputText`].
///
/// [`InputText`]: crate::model::application::component::InputText
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInputText(InputText);

impl CreateInputText {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(
        style: InputTextStyle,
        label: impl Into<String>,
        custom_id: impl Into<String>,
    ) -> Self {
        Self(InputText {
            style,
            label: label.into(),
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: String::new(),
            required: None,

            kind: ComponentType::InputText,
        })
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.0.style = kind;
        self
    }

    /// Sets the label of this input text. Replaces the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = label.into();
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = id.into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.0.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u64) -> Self {
        self.0.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u64) -> Self {
        self.0.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = value.into();
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.0.required = Some(required);
        self
    }
}
