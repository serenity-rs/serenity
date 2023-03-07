use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

use crate::json::{from_value, JsonMap, Value};
use crate::model::channel::ReactionType;

/// The type of a component
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ComponentType {
    ActionRow = 1,
    Button = 2,
    SelectMenu = 3,
    InputText = 4,
    Unknown = !0,
}

enum_number!(ComponentType {
    ActionRow,
    Button,
    SelectMenu,
    InputText
});

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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = map
            .get("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(ComponentType::deserialize)
            .map_err(DeError::custom)?;

        match kind {
            ComponentType::Button => from_value::<Button>(Value::from(map))
                .map(ActionRowComponent::Button)
                .map_err(DeError::custom),
            ComponentType::SelectMenu => from_value::<SelectMenu>(Value::from(map))
                .map(ActionRowComponent::SelectMenu)
                .map_err(DeError::custom),
            ComponentType::InputText => from_value::<InputText>(Value::from(map))
                .map(ActionRowComponent::InputText)
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown component type")),
        }
    }
}

impl Serialize for ActionRowComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Button(c) => Button::serialize(c, serializer),
            Self::SelectMenu(c) => SelectMenu::serialize(c, serializer),
            Self::InputText(c) => InputText::serialize(c, serializer),
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

/// A button component.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Button {
    /// The component type, it will always be [`ComponentType::Button`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The button style.
    pub style: ButtonStyle,
    /// The text which appears on the button.
    pub label: Option<String>,
    /// The emoji of this button, if there is one.
    pub emoji: Option<ReactionType>,
    /// An identifier defined by the developer for the button.
    pub custom_id: Option<String>,
    /// The url of the button, if there is one.
    pub url: Option<String>,
    /// Whether the button is disabled.
    #[serde(default)]
    pub disabled: bool,
}

/// The style of a button.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
    Unknown = !0,
}

enum_number!(ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Link
});

/// A select menu component.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectMenu {
    /// The component type, it will always be [`ComponentType::SelectMenu`].
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

/// The style of the input text
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InputTextStyle {
    Short = 1,
    Paragraph = 2,
    Unknown = !0,
}

enum_number!(InputTextStyle {
    Short,
    Paragraph,
    Unknown
});
