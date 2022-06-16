use crate::internal::prelude::*;
use crate::json::Value;
use crate::model::application::component::{ButtonStyle, InputTextStyle};
use crate::model::channel::ReactionType;

/// A builder for creating several [`ActionRow`]s.
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateComponents(pub Vec<CreateActionRow>);

impl CreateComponents {
    /// Creates an action row.
    pub fn create_action_row<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateActionRow) -> &mut CreateActionRow,
    {
        let mut data = CreateActionRow::default();
        f(&mut data);

        self.add_action_row(data);

        self
    }

    /// Adds an action row.
    pub fn add_action_row(&mut self, row: CreateActionRow) -> &mut Self {
        self.0.push(row);

        self
    }

    /// Set a single action row.
    /// Calling this will overwrite all action rows.
    pub fn set_action_row(&mut self, row: CreateActionRow) -> &mut Self {
        self.0 = vec![row];

        self
    }

    /// Sets all the action rows.
    pub fn set_action_rows(&mut self, rows: Vec<CreateActionRow>) -> &mut Self {
        self.0.extend(rows);
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
pub struct CreateActionRow {
    components: Vec<ComponentBuilder>,
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
    /// Creates a button.
    pub fn create_button<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateButton) -> &mut CreateButton,
    {
        let mut data = CreateButton::default();
        f(&mut data);

        self.add_button(data);

        self
    }

    /// Adds a button.
    pub fn add_button(&mut self, button: CreateButton) -> &mut Self {
        self.components.push(ComponentBuilder::Button(button));
        self
    }

    /// Creates a select menu.
    pub fn create_select_menu<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenu) -> &mut CreateSelectMenu,
    {
        let mut data = CreateSelectMenu::default();
        f(&mut data);

        self.add_select_menu(data);

        self
    }

    /// Adds a select menu.
    pub fn add_select_menu(&mut self, menu: CreateSelectMenu) -> &mut Self {
        self.components.push(ComponentBuilder::SelectMenu(menu));
        self
    }

    /// Creates an input text.
    pub fn create_input_text<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateInputText) -> &mut CreateInputText,
    {
        let mut data = CreateInputText::default();
        f(&mut data);

        self.add_input_text(data);

        self
    }

    /// Adds an input text.
    pub fn add_input_text(&mut self, input_text: CreateInputText) -> &mut Self {
        self.components.push(ComponentBuilder::InputText(input_text));
        self
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::application::component::Button
#[derive(Clone, Debug, Serialize)]
pub struct CreateButton {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<JsonMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    style: ButtonStyle,
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
    pub fn style(&mut self, kind: ButtonStyle) -> &mut Self {
        self.style = kind;
        self
    }

    /// The label of the button.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the custom id of the button, a developer-defined identifier.
    pub fn custom_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.custom_id = Some(id.into());
        self
    }

    /// The url for url style button.
    pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    /// Sets emoji of the button.
    pub fn emoji<R: Into<ReactionType>>(&mut self, emoji: R) -> &mut Self {
        self._emoji(emoji.into())
    }

    fn _emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".into(), Value::from(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".into(), Value::from(animated));
                map.insert("id".into(), Value::String(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".into(), Value::String(name));
                }
            },
        };

        self.emoji = Some(map);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.disabled = Some(disabled);
        self
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::component::SelectMenu
#[derive(Clone, Debug, Serialize)]
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
    pub fn placeholder(&mut self, label: impl Into<String>) -> &mut Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier.
    pub fn custom_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.custom_id = Some(id.into());
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(&mut self, min: u64) -> &mut Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(&mut self, max: u64) -> &mut Self {
        self.max_values = Some(max);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn options<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenuOptions) -> &mut CreateSelectMenuOptions,
    {
        let mut options = CreateSelectMenuOptions::default();
        f(&mut options);

        self.options = Some(options);
        self
    }
}

/// A builder for creating several [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateSelectMenuOptions(pub Vec<CreateSelectMenuOption>);

impl CreateSelectMenuOptions {
    /// Creates an option.
    pub fn create_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenuOption) -> &mut CreateSelectMenuOption,
    {
        let mut data = CreateSelectMenuOption::default();
        f(&mut data);

        self.add_option(data);

        self
    }

    /// Adds an option.
    pub fn add_option(&mut self, option: CreateSelectMenuOption) -> &mut Self {
        self.0.push(option);

        self
    }

    /// Sets all the options.
    pub fn set_options(&mut self, options: Vec<CreateSelectMenuOption>) -> &mut Self {
        self.0.extend(options);

        self
    }
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateSelectMenuOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<JsonMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl CreateSelectMenuOption {
    /// Creates an option.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        let mut opt = Self::default();
        opt.label(label).value(value);
        opt
    }

    /// Sets the label of this option.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the value of this option.
    pub fn value(&mut self, value: impl Into<String>) -> &mut Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the description shown on this option.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Sets emoji of the option.
    pub fn emoji<R: Into<ReactionType>>(&mut self, emoji: R) -> &mut Self {
        self._emoji(emoji.into())
    }

    fn _emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".into(), Value::String(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".into(), Value::from(animated));
                map.insert("id".into(), Value::String(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".into(), Value::from(name));
                }
            },
        };

        self.emoji = Some(map);
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(&mut self, disabled: bool) -> &mut Self {
        self.default = Some(disabled);
        self
    }
}

/// A builder for creating an [`InputText`].
///
/// [`InputText`]: crate::model::application::component::InputText
#[derive(Clone, Debug, Serialize)]
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
    pub fn custom_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.custom_id = Some(id.into());
        self
    }

    /// Sets the style of this input text
    pub fn style(&mut self, kind: InputTextStyle) -> &mut Self {
        self.style = Some(kind);
        self
    }

    /// Sets the label of this input text.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(&mut self, label: impl Into<String>) -> &mut Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(&mut self, min: u64) -> &mut Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(&mut self, max: u64) -> &mut Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(&mut self, value: impl Into<String>) -> &mut Self {
        self.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.required = Some(required);
        self
    }
}
