use serde::Serialize;

use crate::model::application::component::{ButtonStyle, InputTextStyle};
use crate::model::channel::{ChannelType, ReactionType};
use crate::model::prelude::component::{Button, ButtonKind, ComponentType};

/// A builder for creating several [`ActionRow`]s.
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateComponents(pub Vec<CreateActionRow>);

impl CreateComponents {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

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
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

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
pub struct CreateButton(Button);

impl CreateButton {
    /// Creates a link button to the given URL.
    pub fn new_link(url: impl Into<String>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::Link {
                url: url.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Creates a normal button with the given custom ID
    pub fn new(style: ButtonStyle, custom_id: impl Into<String>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                style,
                custom_id: custom_id.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// The label of the button.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
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

#[derive(Clone, Debug)]
pub enum CreateSelectMenuKind {
    String { options: Vec<CreateSelectMenuOption> },
    User,
    Role,
    Mentionable,
    Channel { channel_types: Option<Vec<ChannelType>> },
}

impl Serialize for CreateSelectMenuKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Json<'a> {
            #[serde(rename = "type")]
            kind: u8,
            options: Option<&'a [CreateSelectMenuOption]>,
            channel_types: Option<&'a [ChannelType]>,
        }

        #[rustfmt::skip]
        let json = Json {
            kind: match self {
                Self::String { .. } => 3,
                Self::User { .. } => 5,
                Self::Role { .. } => 6,
                Self::Mentionable { .. } => 7,
                Self::Channel { .. } => 8,
            },
            options: match self {
                Self::String { options } => Some(options),
                _ => None,
            },
                channel_types: match self {
                Self::Channel { channel_types } => channel_types.as_deref(),
                _ => None,
            },
        };

        json.serialize(serializer)
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::component::SelectMenu
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenu {
    custom_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    #[serde(flatten)]
    kind: CreateSelectMenuKind,
}

impl CreateSelectMenu {
    /// Creates a builder with given custom id (a developer-defined identifier), and a list of
    /// options, leaving all other fields empty.
    pub fn new(custom_id: impl Into<String>, kind: CreateSelectMenuKind) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: None,
            kind,
        }
    }

    /// The placeholder of the select menu.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = id.into();
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
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenuOption {
    label: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl CreateSelectMenuOption {
    /// Creates a select menu option with the given label and value, leaving all other fields
    /// empty.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: None,
        }
    }

    /// Sets the label of this option, replacing the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the value of this option, replacing the current value as set in [`Self::new`].
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
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
    style: InputTextStyle,
    label: String,
    custom_id: String,

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

impl CreateInputText {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(
        style: InputTextStyle,
        label: impl Into<String>,
        custom_id: impl Into<String>,
    ) -> Self {
        Self {
            style,
            label: label.into(),
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: None,
            required: None,

            kind: 4,
        }
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.style = kind;
        self
    }

    /// Sets the label of this input text. Replaces the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = id.into();
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
