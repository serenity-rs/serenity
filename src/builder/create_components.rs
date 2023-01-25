use serde::{Deserialize, Serialize};

use crate::model::prelude::*;

/// A builder for creating an [`ActionRow`].
///
/// [`ActionRow`]: crate::model::application::ActionRow
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateActionRow {
    Buttons(Vec<CreateButton>),
    SelectMenu(CreateSelectMenu),
    /// Only valid in modals!
    InputText(CreateInputText),
}

#[derive(Serialize, Deserialize)]
struct ActionRowJson {
    #[serde(rename = "type")]
    kind: u8,
    components: Vec<serde_json::Value>,
}

impl serde::Serialize for CreateActionRow {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error as _;

        let components: Vec<serde_json::Value> = match self {
            Self::Buttons(x) => x
                .iter()
                .map(|button| serde_json::to_value(button).map_err(S::Error::custom))
                .collect::<Result<Vec<_>, _>>()?,
            Self::SelectMenu(x) => vec![serde_json::to_value(x).map_err(S::Error::custom)?],
            Self::InputText(x) => vec![serde_json::to_value(x).map_err(S::Error::custom)?],
        };

        ActionRowJson {
            kind: 1,
            components,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CreateActionRow {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let ActionRowJson {
            kind,
            components,
        } = ActionRowJson::deserialize(deserializer)?;

        if kind != 1 {
            return Err(D::Error::custom("expected action row to be of type 1"));
        }

        // A `Buttons` variant could contain 0 buttons internally, which is why
        // this check is need
        if components.is_empty() {
            return Err(D::Error::custom("expected at least one component"));
        }

        // Determine the type of component by looking at the first one
        let first_component = &components[0];

        let component_kind = first_component
            .get("type")
            .ok_or_else(|| D::Error::custom("expected component to have a type field"))?;

        match component_kind.as_u64() {
            Some(2) => {
                let buttons: Vec<CreateButton> = components
                    .into_iter()
                    .map(|x| serde_json::from_value::<CreateButton>(x).map_err(D::Error::custom))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Self::Buttons(buttons))
            },
            Some(3 | 5 | 6 | 7 | 8) => {
                // Make sure there is only 1 component
                if components.len() != 1 {
                    return Err(D::Error::custom("expected only one select menu"));
                }

                let select_menu =
                    serde_json::from_value::<CreateSelectMenu>(first_component.clone())
                        .map_err(D::Error::custom)?;

                Ok(Self::SelectMenu(select_menu))
            },
            Some(4) => {
                // Make sure there is only 1 component
                if components.len() != 1 {
                    return Err(D::Error::custom("expected only one input text"));
                }

                let input_text = serde_json::from_value::<CreateInputText>(first_component.clone())
                    .map_err(D::Error::custom)?;

                Ok(Self::InputText(input_text))
            },
            _ => Err(D::Error::custom("expected buttons, select_menu, or input_text")),
        }
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::application::Button
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub struct CreateButton(Button);

impl CreateButton {
    /// Creates a link button to the given URL. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
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

    /// Creates a normal button with the given custom ID. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    ///
    /// Clicking this button will not trigger an interaction event in your bot.
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                style: ButtonStyle::Primary,
                custom_id: custom_id.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Sets the custom id of the button, a developer-defined identifier. Replaces the current value
    /// as set in [`Self::new`].
    ///
    /// Has no effect on link buttons.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        if let ButtonKind::NonLink {
            custom_id, ..
        } = &mut self.0.data
        {
            *custom_id = id.into();
        }
        self
    }

    /// Sets the style of this button.
    ///
    /// Has no effect on link buttons.
    pub fn style(mut self, new_style: ButtonStyle) -> Self {
        if let ButtonKind::NonLink {
            style, ..
        } = &mut self.0.data
        {
            *style = new_style;
        }
        self
    }

    /// Sets label of the button.
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

#[derive(Clone, Debug, PartialEq)]
pub enum CreateSelectMenuKind {
    String { options: Vec<CreateSelectMenuOption> },
    User,
    Role,
    Mentionable,
    Channel { channel_types: Option<Vec<ChannelType>> },
}

#[derive(Serialize, Deserialize)]
struct CreateSelectMenuKindJson {
    #[serde(rename = "type")]
    kind: u8,
    options: Option<Vec<CreateSelectMenuOption>>,
    channel_types: Option<Vec<ChannelType>>,
}

impl Serialize for CreateSelectMenuKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[rustfmt::skip]
        let json = CreateSelectMenuKindJson {
            kind: match self {
                Self::String { .. } => 3,
                Self::User { .. } => 5,
                Self::Role { .. } => 6,
                Self::Mentionable { .. } => 7,
                Self::Channel { .. } => 8,
            },
            options: match self {
                Self::String { options } => Some(options.clone()),
                _ => None,
            },
                channel_types: match self {
                Self::Channel { channel_types } => channel_types.clone(),
                _ => None,
            },
        };

        json.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CreateSelectMenuKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = CreateSelectMenuKindJson::deserialize(deserializer)?;

        Ok(match json.kind {
            3 => Self::String {
                options: json.options.unwrap_or_default(),
            },
            5 => Self::User,
            6 => Self::Role,
            7 => Self::Mentionable,
            8 => Self::Channel {
                channel_types: json.channel_types,
            },
            _ => return Err(serde::de::Error::custom("invalid select menu type")),
        })
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::SelectMenu
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
/// [`SelectMenuOption`]: crate::model::application::SelectMenuOption
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
/// [`InputText`]: crate::model::application::InputText
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod test {
    use crate::all::*;
    use crate::json::{assert_json, json};

    #[test]
    fn serialize_create_button() {
        let button = CreateButton::new("create_button_test_id");

        assert_json(
            &button,
            json!({"type": 2, "style": 1, "custom_id": "create_button_test_id", "disabled": false}),
        );
    }

    #[test]
    fn serialize_create_action_row() {
        let action_row_buttons = CreateActionRow::Buttons(vec![
            CreateButton::new("button_id_1"),
            CreateButton::new("button_id_2"),
            CreateButton::new("button_id_3"),
        ]);

        assert_json(
            &action_row_buttons,
            json!({
                "type": 1,
                "components": [
                    {"type": 2, "style": 1, "custom_id": "button_id_1", "disabled": false},
                    {"type": 2, "style": 1, "custom_id": "button_id_2", "disabled": false},
                    {"type": 2, "style": 1, "custom_id": "button_id_3", "disabled": false},
                ],
            }),
        );

        let action_row_select_menu = CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "select_menu_id",
            CreateSelectMenuKind::Channel {
                channel_types: None,
            },
        ));

        assert_json(
            &action_row_select_menu,
            json!({
                "type": 1,
                "components": [
                    {
                        "channel_types": null,
                        "custom_id": "select_menu_id",
                        "options": null,
                        "type": 8,
                    },
                ],
            }),
        );

        let action_row_input_text = CreateActionRow::InputText(CreateInputText::new(
            InputTextStyle::Short,
            "input_text_label",
            "input_text_id",
        ));

        assert_json(
            &action_row_input_text,
            json!({
                "type": 1,
                "components": [
                    {
                        "custom_id": "input_text_id",
                        "label": "input_text_label",
                        "style": 1,
                        "type": 4,
                    },
                ],
            }),
        );
    }

    #[test]
    /// Test deserializing an empty action row. This should error.
    fn test_deserialize_empty_create_action_row() {
        let action_row = CreateActionRow::Buttons(vec![]);

        let serialized_action_row: String = serde_json::to_string(&action_row).unwrap();

        let deserized_action_row: Result<CreateActionRow, _> =
            serde_json::from_str(&serialized_action_row);

        assert!(deserized_action_row.is_err());

        if let Err(e) = deserized_action_row {
            assert_eq!(e.to_string(), "expected at least one component");
        }
    }

    #[test]
    /// Test deserializing when the kind is no 1. This should error.
    fn test_deserialize_invalid_kind_create_action_row() {
        let action_row = CreateActionRow::Buttons(vec![CreateButton::new("button_id_1")]);

        let serialized_action_row: String = serde_json::to_string(&action_row).unwrap();

        let deserized_action_row: Result<CreateActionRow, _> =
            serde_json::from_str(&serialized_action_row.replace('1', "2"));

        assert!(deserized_action_row.is_err());

        if let Err(e) = deserized_action_row {
            assert_eq!(e.to_string(), "expected action row to be of type 1");
        }
    }

    #[test]
    /// Make sure that the `CreateActionRow` enum can be deserialized properly
    /// into the button variant.
    fn test_deserialize_button_create_action_row() {
        let action_row = CreateActionRow::Buttons(vec![
            CreateButton::new("button_id_1").label("test").style(ButtonStyle::Primary),
            CreateButton::new("button_id_2").label("test").style(ButtonStyle::Secondary),
        ]);

        serde_json::to_string(&CreateActionRow::Buttons(vec![])).unwrap();

        assert_json(
            &action_row,
            json!({
                "type": 1,
                "components": [
                    {"type": 2, "style": 1, "custom_id": "button_id_1", "disabled": false, "label": "test"},
                    {"type": 2, "style": 2, "custom_id": "button_id_2", "disabled": false, "label": "test"},
                ],
            }),
        );

        let serialized_action_row: String = serde_json::to_string(&action_row).unwrap();

        let deserized_action_row: CreateActionRow =
            serde_json::from_str(&serialized_action_row).unwrap();

        // Make sure it's a button variant
        if let CreateActionRow::Buttons(buttons) = deserized_action_row {
            assert_eq!(buttons.len(), 2);
        } else {
            panic!("Deserialized action row is not a button variant");
        }
    }

    #[test]
    /// Make sure that the `CreateActionRow` enum can be deserialized properly
    /// into the select menu variant.
    fn test_deserialize_select_menu_create_action_row() {
        let action_row = CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "select_menu_id",
            CreateSelectMenuKind::Channel {
                channel_types: None,
            },
        ));

        assert_json(
            &action_row,
            json!({
                "type": 1,
                "components": [
                    {
                        "channel_types": null,
                        "custom_id": "select_menu_id",
                        "options": null,
                        "type": 8,
                    },
                ],
            }),
        );

        let serialized_action_row: String = serde_json::to_string(&action_row).unwrap();

        let deserized_action_row: CreateActionRow =
            serde_json::from_str(&serialized_action_row).unwrap();

        // Make sure it's a select menu variant
        if let CreateActionRow::SelectMenu(select_menu) = deserized_action_row {
            assert_eq!(select_menu.custom_id, "select_menu_id");
        } else {
            panic!("Deserialized action row is not a select menu variant");
        }
    }

    #[test]
    /// Make sure that the `CreateActionRow` enum can be deserialized properly
    /// into the input text variant.
    fn test_deserialize_input_text_create_action_row() {
        let action_row = CreateActionRow::InputText(CreateInputText::new(
            InputTextStyle::Short,
            "input_text_label",
            "input_text_id",
        ));

        assert_json(
            &action_row,
            json!({
                "type": 1,
                "components": [
                    {
                        "custom_id": "input_text_id",
                        "label": "input_text_label",
                        "style": 1,
                        "type": 4,
                    },
                ],
            }),
        );

        let serialized_action_row: String = serde_json::to_string(&action_row).unwrap();

        let deserized_action_row: CreateActionRow =
            serde_json::from_str(&serialized_action_row).unwrap();

        // Make sure it's a input text variant
        if let CreateActionRow::InputText(input_text) = deserized_action_row {
            assert_eq!(input_text.custom_id, "input_text_id");
        } else {
            panic!("Deserialized action row is not a input text variant");
        }
    }

    #[test]
    /// Test serializing a CreateSelectMenuKind
    fn test_serialize_create_select_menu_kind() {
        let kind = CreateSelectMenuKind::Channel {
            channel_types: Some(vec![
                ChannelType::Text,
                ChannelType::Voice,
                ChannelType::Category,
                ChannelType::News,
            ]),
        };

        assert_json(
            &kind,
            json!({
                "channel_types": [0, 2, 4, 5],
                "options": null,
                "type": 8,
            }),
        );
    }
}
