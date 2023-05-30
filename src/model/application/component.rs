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
    UserSelect = 5,
    RoleSelect = 6,
    MentionableSelect = 7,
    ChannelSelect = 8,
    Unknown = !0,
}

enum_number!(ComponentType {
    ActionRow,
    Button,
    SelectMenu,
    InputText,
    UserSelect,
    RoleSelect,
    MentionableSelect,
    ChannelSelect,
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
    UserSelect(UserSelect),
    RoleSelect(RoleSelect),
    MentionableSelect(MentionableSelect),
    ChannelSelect(ChannelSelect),
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
            ComponentType::UserSelect => from_value::<UserSelect>(Value::from(map))
                .map(ActionRowComponent::UserSelect)
                .map_err(DeError::custom),
            ComponentType::RoleSelect => from_value::<RoleSelect>(Value::from(map))
                .map(ActionRowComponent::RoleSelect)
                .map_err(DeError::custom),
            ComponentType::MentionableSelect => from_value::<MentionableSelect>(Value::from(map))
                .map(ActionRowComponent::MentionableSelect)
                .map_err(DeError::custom),
            ComponentType::ChannelSelect => from_value::<ChannelSelect>(Value::from(map))
                .map(ActionRowComponent::ChannelSelect)
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
            Self::ChannelSelect(c) => ChannelSelect::serialize(c, serializer),
            Self::UserSelect(c) => UserSelect::serialize(c, serializer),
            Self::RoleSelect(c) => RoleSelect::serialize(c, serializer),
            Self::MentionableSelect(c) => MentionableSelect::serialize(c, serializer),
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

enum_number!(ButtonStyle { Primary, Secondary, Success, Danger, Link });

/// A select menu component. This menu is for Strings only. There are other components that are used for User Select, Role Select, Mention Select, and Channel Select.
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

/// A select menu for only selecting users
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserSelect {
    /// The component type, it will always be [`ComponentType::UserSelect`].
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
    /// The result location for modals
    #[serde(default)]
    pub values: Vec<String>,
}

/// A select menu for only selecting roles
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleSelect {
    /// The component type, it will always be [`ComponentType::RoleSelect`].
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
    /// The result location for modals
    #[serde(default)]
    pub values: Vec<String>,
}

/// A select menu for only selecting mentionable items. A mentionable is either an @user or an @role
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MentionableSelect {
    /// The component type, it will always be [`ComponentType::MentionableSelect`].
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
    /// The result location for modals
    #[serde(default)]
    pub values: Vec<String>,
}
/// A select menu for only selecting channels
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelSelect {
    /// The component type, it will always be [`ComponentType::ChannelSelect`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<String>,

    /// https://discord.com/developers/docs/resources/channel#channel-object-channel-types
    pub channel_types: Option<Vec<u32>>,

    /// An identifier defined by the developer for the select menu.
    pub custom_id: Option<String>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u64>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u64>,
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

enum_number!(InputTextStyle { Short, Paragraph, Unknown });
