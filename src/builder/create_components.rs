use crate::model::application::component::{ButtonStyle, InputTextStyle};
use crate::model::channel::ReactionType;

/// A builder for creating several [`ActionRow`]s.
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateComponents(pub Vec<CreateActionRow>);

impl CreateComponents {
    /// Adds an action row.
    pub fn add_action_row(mut self, row: CreateActionRow) -> Self {
        self.0.push(row);
        self
    }

    pub fn add_action_rows(mut self, rows: Vec<CreateActionRow>) -> Self {
        self.0.extend(rows);
        self
    }

    /// Set a single action row. Calling this will overwrite all action rows.
    pub fn set_action_row(mut self, row: CreateActionRow) -> Self {
        self.0 = vec![row];
        self
    }

    /// Sets all the action rows. Calling this will overwrite all action rows.
    pub fn set_action_rows(mut self, rows: Vec<CreateActionRow>) -> Self {
        self.0 = rows;
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum ComponentBuilder {
    Button(CreateButton),
    SelectMenu(CreateSelectMenu),
    InputText(CreateInputText),
}

/// A builder for creating an [`ActionRow`].
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateActionRow {
    components: Vec<ComponentBuilder>,
    #[serde(rename = "type")]
    kind: u8,
}

impl Default for CreateActionRow {
    fn default() -> Self {
        CreateActionRow {
            components: Vec::new(),
            kind: 1,
        }
    }
}

impl CreateActionRow {
    /// Adds a button.
    pub fn add_button(mut self, button: CreateButton) -> Self {
        self.components.push(ComponentBuilder::Button(button));
        self
    }

    /// Adds a select menu.
    pub fn add_select_menu(mut self, menu: CreateSelectMenu) -> Self {
        self.components.push(ComponentBuilder::SelectMenu(menu));
        self
    }

    /// Adds an input text.
    pub fn add_input_text(mut self, input_text: CreateInputText) -> Self {
        self.components.push(ComponentBuilder::InputText(input_text));
        self
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::application::component::Button
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateButton {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    style: ButtonStyle,

    #[serde(rename = "type")]
    kind: u8,
}

impl Default for CreateButton {
    /// Creates a primary button.
    fn default() -> Self {
        Self {
            style: ButtonStyle::Primary,
            custom_id: None,
            disabled: None,
            label: None,
            emoji: None,
            url: None,
            kind: 2,
        }
    }
}

impl CreateButton {
    /// Sets the style of the button.
    pub fn style(mut self, kind: ButtonStyle) -> Self {
        self.style = kind;
        self
    }

    /// The label of the button.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the custom id of the button, a developer-defined identifier.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = Some(id.into());
        self
    }

    /// The url for url style button.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets emoji of the button.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::component::SelectMenu
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenu {
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<CreateSelectMenuOptions>,

    #[serde(rename = "type")]
    kind: u8,
}

impl Default for CreateSelectMenu {
    fn default() -> Self {
        Self {
            placeholder: None,
            min_values: None,
            max_values: None,
            custom_id: None,
            disabled: None,
            options: None,
            kind: 3,
        }
    }
}

impl CreateSelectMenu {
    /// The placeholder of the select menu.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = Some(id.into());
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(mut self, min: u64) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(mut self, max: u64) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn options(mut self, options: CreateSelectMenuOptions) -> Self {
        self.options = Some(options);
        self
    }
}

/// A builder for creating several [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateSelectMenuOptions(pub Vec<CreateSelectMenuOption>);

impl CreateSelectMenuOptions {
    /// Adds an option.
    pub fn add_option(mut self, option: CreateSelectMenuOption) -> Self {
        self.0.push(option);
        self
    }

    /// Sets all the options.
    pub fn set_options(mut self, options: Vec<CreateSelectMenuOption>) -> Self {
        self.0.extend(options);
        self
    }
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateSelectMenuOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl CreateSelectMenuOption {
    /// Creates an option.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::default().label(label).value(value)
    }

    /// Sets the label of this option.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the value of this option.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the description shown on this option.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(mut self, disabled: bool) -> Self {
        self.default = Some(disabled);
        self
    }
}

/// A builder for creating an [`InputText`].
///
/// [`InputText`]: crate::model::application::component::InputText
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInputText {
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<InputTextStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,

    #[serde(rename = "type")]
    kind: u8,
}

impl Default for CreateInputText {
    fn default() -> Self {
        Self {
            value: None,
            style: None,
            label: None,
            required: None,
            custom_id: None,
            placeholder: None,
            min_length: None,
            max_length: None,
            kind: 4_u8,
        }
    }
}

impl CreateInputText {
    /// Sets the custom id of the input text, a developer-defined identifier.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = Some(id.into());
        self
    }

    /// Sets the style of this input text
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.style = Some(kind);
        self
    }

    /// Sets the label of this input text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u64) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u64) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}
