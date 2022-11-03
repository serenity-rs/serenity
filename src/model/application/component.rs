use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

use crate::internal::prelude::*;
use crate::json::from_value;
use crate::model::channel::ReactionType;
use crate::model::utils::deserialize_val;

enum_number! {
    /// The type of a component
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ComponentType {
        ActionRow = 1,
        Button = 2,
        StringSelect = 3,
        InputText = 4,
        UserSelect = 5,
        RoleSelect = 6,
        MentionableSelect = 7,
        ChannelSelect = 8,
        _ => Unknown(u8),
    }
}

/// An action row.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionRow {
    /// The type of component this ActionRow is.
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components of this ActionRow.
    #[serde(default)]
    pub components: Vec<ActionRowComponent>,
}

// A component which can be inside of an [`ActionRow`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ActionRowComponent {
    Button(Button),
    SelectMenu(SelectMenu),
    InputText(InputText),
}

impl<'de> Deserialize<'de> for ActionRowComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::Button => from_value(value).map(ActionRowComponent::Button),
            ComponentType::InputText => from_value(value).map(ActionRowComponent::InputText),
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => from_value(value).map(ActionRowComponent::SelectMenu),
            ComponentType::ActionRow => {
                return Err(DeError::custom("Invalid component type ActionRow"))
            },
            ComponentType::Unknown(i) => {
                return Err(DeError::custom(format_args!("Unknown component type {}", i)))
            },
        }
        .map_err(DeError::custom)
    }
}

impl Serialize for ActionRowComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Button(c) => c.serialize(serializer),
            Self::InputText(c) => c.serialize(serializer),
            Self::SelectMenu(c) => c.serialize(serializer),
        }
    }
}

impl From<Button> for ActionRowComponent {
    fn from(component: Button) -> Self {
        ActionRowComponent::Button(component)
    }
}

impl From<SelectMenu> for ActionRowComponent {
    fn from(component: SelectMenu) -> Self {
        ActionRowComponent::SelectMenu(component)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ButtonKind {
    Link { url: String },
    NonLink { custom_id: String, style: ButtonStyle },
}

impl Serialize for ButtonKind {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a> {
            style: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            url: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_id: Option<&'a str>,
        }

        let helper = match self {
            ButtonKind::Link {
                url,
            } => Helper {
                style: 5,
                url: Some(url),
                custom_id: None,
            },
            ButtonKind::NonLink {
                custom_id,
                style,
            } => Helper {
                style: (*style).into(),
                url: None,
                custom_id: Some(custom_id),
            },
        };
        helper.serialize(serializer)
    }
}

/// A button component.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#button-object-button-structure).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Button {
    /// The component type, it will always be [`ComponentType::Button`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The button kind and style.
    #[serde(flatten)]
    pub data: ButtonKind,
    /// The text which appears on the button.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The emoji of this button, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
    /// Whether the button is disabled.
    #[serde(default)]
    pub disabled: bool,
}

enum_number! {
    /// The style of a button.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ButtonStyle {
        Primary = 1,
        Secondary = 2,
        Success = 3,
        Danger = 4,
        // No Link, because we represent Link using enum variants
        _ => Unknown(u8),
    }
}

/// A select menu component.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectMenu {
    /// The component type, which may either be [`ComponentType::StringSelect`],
    /// [`ComponentType::UserSelect`], [`ComponentType::RoleSelect`],
    /// [`ComponentType::MentionableSelect`], or [`ComponentType::ChannelSelect`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<String>,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: Option<String>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u64>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u64>,
    /// The options of this select menu.
    #[serde(default)]
    pub options: Vec<SelectMenuOption>,
    /// The result location for modals
    #[serde(default)]
    pub values: Vec<String>,
}

/// A select menu component options.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectMenuOption {
    /// The text displayed on this option.
    pub label: String,
    /// The value to be sent for this option.
    pub value: String,
    /// The description shown for this option.
    pub description: Option<String>,
    /// The emoji displayed on this option.
    pub emoji: Option<ReactionType>,
    /// Render this option as the default selection.
    #[serde(default)]
    pub default: bool,
}

/// An input text component for modal interactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: String,
    /// The input from the user
    pub value: String,
}

enum_number! {
    /// The style of the input text
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-styles).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum InputTextStyle {
        Short = 1,
        Paragraph = 2,
        _ => Unknown(u8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{assert_json, json};

    #[test]
    fn test_button_serde() {
        let mut button = Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                custom_id: "hello".into(),
                style: ButtonStyle::Danger,
            },
            label: Some("a".into()),
            emoji: None,
            disabled: false,
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 4, "custom_id": "hello", "label": "a", "disabled": false}),
        );

        button.data = ButtonKind::Link {
            url: "https://google.com".into(),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 5, "url": "https://google.com", "label": "a", "disabled": false}),
        );
    }
}
