use serde::de::Error as DeError;
use serde::ser::{Serialize, Serializer};
use serde_json::from_value;

use crate::model::prelude::*;
use crate::model::utils::{default_true, deserialize_val};

enum_number! {
    /// The type of a component
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#action-rows).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActionRow {
    /// Always [`ComponentType::ActionRow`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components of this ActionRow.
    #[serde(default)]
    pub components: FixedArray<ActionRowComponent>,
}

/// A component which can be inside of an [`ActionRow`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#component-object-component-types).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown component type {i}")))
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

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ButtonKind {
    Link { url: FixedString },
    Premium { sku_id: SkuId },
    NonLink { custom_id: FixedString, style: ButtonStyle },
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
            #[serde(skip_serializing_if = "Option::is_none")]
            sku_id: Option<SkuId>,
        }

        let helper = match self {
            ButtonKind::Link {
                url,
            } => Helper {
                style: 5,
                url: Some(url),
                custom_id: None,
                sku_id: None,
            },
            ButtonKind::Premium {
                sku_id,
            } => Helper {
                style: 6,
                url: None,
                custom_id: None,
                sku_id: Some(*sku_id),
            },
            ButtonKind::NonLink {
                custom_id,
                style,
            } => Helper {
                style: style.0,
                url: None,
                custom_id: Some(custom_id),
                sku_id: None,
            },
        };
        helper.serialize(serializer)
    }
}

/// A button component.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#button-object-button-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Button {
    /// The component type, it will always be [`ComponentType::Button`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The button kind and style.
    #[serde(flatten)]
    pub data: ButtonKind,
    /// The text which appears on the button.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<FixedString>,
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SelectMenu {
    /// The component type, which may either be [`ComponentType::StringSelect`],
    /// [`ComponentType::UserSelect`], [`ComponentType::RoleSelect`],
    /// [`ComponentType::MentionableSelect`], or [`ComponentType::ChannelSelect`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: Option<FixedString>,
    /// The options of this select menu.
    ///
    /// Required for [`ComponentType::StringSelect`] and unavailable for all others.
    #[serde(default)]
    pub options: FixedArray<SelectMenuOption>,
    /// List of channel types to include in the [`ComponentType::ChannelSelect`].
    #[serde(default)]
    pub channel_types: FixedArray<ChannelType>,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<FixedString>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u8>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u8>,
    /// Whether select menu is disabled.
    #[serde(default)]
    pub disabled: bool,
}

/// A select menu component options.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SelectMenuOption {
    /// The text displayed on this option.
    pub label: FixedString,
    /// The value to be sent for this option.
    pub value: FixedString,
    /// The description shown for this option.
    pub description: Option<FixedString>,
    /// The emoji displayed on this option.
    pub emoji: Option<ReactionType>,
    /// Render this option as the default selection.
    #[serde(default)]
    pub default: bool,
}

/// An input text component for modal interactions
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the input; max 100 characters
    pub custom_id: FixedString<u16>,
    /// The [`InputTextStyle`]. Required when sending modal data.
    ///
    /// Discord docs are wrong here; it says the field is always sent in modal submit interactions
    /// but it's not. It's only required when _sending_ modal data to Discord.
    /// <https://github.com/discord/discord-api-docs/issues/6141>
    pub style: Option<InputTextStyle>,
    /// Label for this component; max 45 characters. Required when sending modal data.
    ///
    /// Discord docs are wrong here; it says the field is always sent in modal submit interactions
    /// but it's not. It's only required when _sending_ modal data to Discord.
    /// <https://github.com/discord/discord-api-docs/issues/6141>
    pub label: Option<FixedString<u8>>,
    /// Minimum input length for a text input; min 0, max 4000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u16>,
    /// Maximum input length for a text input; min 1, max 4000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u16>,
    /// Whether this component is required to be filled (defaults to true)
    #[serde(default = "default_true")]
    pub required: bool,
    /// When sending: Pre-filled value for this component; max 4000 characters (may be None).
    ///
    /// When receiving: The input from the user (always Some)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<FixedString<u16>>,
    /// Custom placeholder text if the input is empty; max 100 characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<FixedString<u16>>,
}

enum_number! {
    /// The style of the input text
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-styles).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InputTextStyle {
        Short = 1,
        Paragraph = 2,
        _ => Unknown(u8),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::utils::assert_json;

    #[test]
    fn test_button_serde() {
        let mut button = Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                custom_id: FixedString::from_static_trunc("hello"),
                style: ButtonStyle::Danger,
            },
            label: Some(FixedString::from_static_trunc("a")),
            emoji: None,
            disabled: false,
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 4, "custom_id": "hello", "label": "a", "disabled": false}),
        );

        button.data = ButtonKind::Link {
            url: FixedString::from_static_trunc("https://google.com"),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 5, "url": "https://google.com", "label": "a", "disabled": false}),
        );

        button.data = ButtonKind::Premium {
            sku_id: 1234965026943668316.into(),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 6, "sku_id": "1234965026943668316", "label": "a", "disabled": false}),
        );
    }
}
