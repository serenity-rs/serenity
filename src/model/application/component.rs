use serde::de::Error as DeError;
use serde::ser::{Serialize, Serializer};

use crate::internal::prelude::*;
use crate::json::from_value;
use crate::model::prelude::*;
use crate::model::utils::{default_true, deserialize_val};

enum_number! {
    /// The type of a component
    ///
    /// [Discord docs](https://discord.com/developers/docs/components/reference#component-object).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
        Section = 9,
        TextDisplay = 10,
        Thumbnail = 11,
        MediaGallery = 12,
        File = 13,
        Separator = 14,
        Container = 17,
        Label = 18,
        FileUpload = 19,
        RadioGroup = 21,
        CheckboxGroup = 22,
        Checkbox = 23,
        _ => Unknown(u8),
    }
}

/// An action row.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActionRow {
    /// Always [`ComponentType::ActionRow`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components of this ActionRow.
    #[serde(default)]
    pub components: Vec<ActionRowComponent>,
}

/// A component which can be inside of an [`ActionRow`].
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row-action-row-child-components).
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
            ComponentType::Section
            | ComponentType::TextDisplay
            | ComponentType::Thumbnail
            | ComponentType::MediaGallery
            | ComponentType::File
            | ComponentType::Separator
            | ComponentType::Container
            | ComponentType::Label
            | ComponentType::FileUpload
            | ComponentType::RadioGroup
            | ComponentType::CheckboxGroup
            | ComponentType::Checkbox => {
                return Err(DeError::custom(format_args!(
                    "Component type not allowed in ActionRow"
                )))
            },
            ComponentType::Unknown(i) => {
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
    Link { url: String },
    Premium { sku_id: SkuId },
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
                style: (*style).into(),
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#button-button-structure).
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
///
/// Discord docs: [String Select](https://docs.discord.com/developers/components/reference#string-select), [User Select](https://docs.discord.com/developers/components/reference#user-select), [Role Select](https://docs.discord.com/developers/components/reference#role-select), [Mentionable Select](https://docs.discord.com/developers/components/reference#mentionable-select), [Channel Select](https://docs.discord.com/developers/components/reference#channel-select).
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
    pub custom_id: Option<String>,
    /// The options of this select menu.
    ///
    /// Required for [`ComponentType::StringSelect`] and unavailable for all others.
    #[serde(default)]
    pub options: Vec<SelectMenuOption>,
    /// List of channel types to include in the [`ComponentType::ChannelSelect`].
    #[serde(default)]
    pub channel_types: Vec<ChannelType>,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<String>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u8>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u8>,
    /// Whether select menu is disabled.
    #[serde(default)]
    pub disabled: bool,
    /// Whether this component is required to be filled (for modals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// A select menu component options.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#string-select-select-option-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the input; max 100 characters
    pub custom_id: String,
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
    pub label: Option<String>,
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
    pub value: Option<String>,
    /// Custom placeholder text if the input is empty; max 100 characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

enum_number! {
    /// The style of the input text
    ///
    /// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-styles).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum InputTextStyle {
        Short = 1,
        Paragraph = 2,
        _ => Unknown(u8),
    }
}

/// An unfurled media item used within components.
///
/// Supports both uploaded media (via `attachment://<filename>`) and externally hosted media
/// (via direct URL).
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#unfurled-media-item).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnfurledMediaItem {
    /// The URL of the media item. Supports arbitrary URLs and `attachment://<filename>` references.
    pub url: String,
}

impl UnfurledMediaItem {
    /// Creates a new unfurled media item from a URL.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
        }
    }
}

/// A section component — top-level layout component that associates content with an accessory.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#section).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Section {
    /// The component type. Always [`ComponentType::Section`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Child components representing the content of the section (1–3 components).
    pub components: Vec<SectionChildComponent>,
    /// A button or thumbnail that is contextually associated to the content.
    pub accessory: SectionAccessoryComponent,
}

/// A child component within a [`Section`].
///
/// Currently only [`TextDisplay`] is supported as a section child.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SectionChildComponent {
    TextDisplay(TextDisplay),
}

impl<'de> Deserialize<'de> for SectionChildComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::TextDisplay => {
                from_value(value).map(SectionChildComponent::TextDisplay).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "invalid section child component type: {other:?}"
            ))),
        }
    }
}

impl Serialize for SectionChildComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::TextDisplay(c) => c.serialize(serializer),
        }
    }
}

impl From<TextDisplay> for SectionChildComponent {
    fn from(component: TextDisplay) -> Self {
        SectionChildComponent::TextDisplay(component)
    }
}

/// An accessory component within a [`Section`].
///
/// Currently only [`Button`] and [`Thumbnail`] are supported.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SectionAccessoryComponent {
    Button(Button),
    Thumbnail(Thumbnail),
}

impl<'de> Deserialize<'de> for SectionAccessoryComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::Button => {
                from_value(value).map(SectionAccessoryComponent::Button).map_err(DeError::custom)
            },
            ComponentType::Thumbnail => {
                from_value(value).map(SectionAccessoryComponent::Thumbnail).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "invalid section accessory component type: {other:?}"
            ))),
        }
    }
}

impl Serialize for SectionAccessoryComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Button(c) => c.serialize(serializer),
            Self::Thumbnail(c) => c.serialize(serializer),
        }
    }
}

impl From<Button> for SectionAccessoryComponent {
    fn from(component: Button) -> Self {
        SectionAccessoryComponent::Button(component)
    }
}

impl From<Thumbnail> for SectionAccessoryComponent {
    fn from(component: Thumbnail) -> Self {
        SectionAccessoryComponent::Thumbnail(component)
    }
}

/// A text display component — displays markdown formatted text.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-display).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextDisplay {
    /// The component type. Always [`ComponentType::TextDisplay`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// The text content, formatted with Discord's markdown.
    pub content: String,
}

/// A thumbnail component — displays a small image as an accessory.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#thumbnail).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Thumbnail {
    /// The component type. Always [`ComponentType::Thumbnail`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// The media item (URL or attachment reference).
    pub media: UnfurledMediaItem,
    /// Alt text for the media, max 1024 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the thumbnail should be a spoiler (blurred out). Defaults to false.
    #[serde(default)]
    pub spoiler: bool,
}

/// A media gallery item used within a [`MediaGallery`].
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#media-gallery-item-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct MediaGalleryItem {
    /// The media item (URL or attachment reference).
    pub media: UnfurledMediaItem,
    /// Alt text for the media, max 1024 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the media should be a spoiler (blurred out). Defaults to false.
    #[serde(default)]
    pub spoiler: bool,
}

/// A media gallery component — displays 1–10 media attachments.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#media-gallery).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct MediaGallery {
    /// The component type. Always [`ComponentType::MediaGallery`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// 1 to 10 media gallery items.
    pub items: Vec<MediaGalleryItem>,
}

/// A file component — displays an uploaded file as an attachment.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileComponent {
    /// The component type. Always [`ComponentType::File`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// The file to display, only supports `attachment://<filename>` references.
    pub file: UnfurledMediaItem,
    /// Whether the file should be a spoiler (blurred out). Defaults to false.
    #[serde(default)]
    pub spoiler: bool,
}

/// A separator component — adds vertical padding between components.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#separator).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Separator {
    /// The component type. Always [`ComponentType::Separator`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Whether a visual divider should be displayed. Defaults to true.
    #[serde(default = "default_true")]
    pub divider: bool,
    /// Size of separator padding: 1 for small, 2 for large. Defaults to 1.
    #[serde(default)]
    pub spacing: SeparatorSpacing,
}

enum_number! {
    /// The spacing size of a separator.
    ///
    /// [Discord docs](https://discord.com/developers/docs/components/reference#separator-separator-structure).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum SeparatorSpacing {
        Small = 1,
        Large = 2,
        _ => Unknown(u8),
    }
}

impl Default for SeparatorSpacing {
    fn default() -> Self {
        Self::Small
    }
}

/// A container component — visually groups a set of components.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#container).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Container {
    /// The component type. Always [`ComponentType::Container`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Child components encapsulated within the Container.
    pub components: Vec<ContainerChildComponent>,
    /// Color for the accent on the container as RGB from 0x000000 to 0xFFFFFF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<u32>,
    /// Whether the container should be a spoiler (blurred out). Defaults to false.
    #[serde(default)]
    pub spoiler: bool,
}

/// A child component within a [`Container`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ContainerChildComponent {
    ActionRow(ActionRow),
    TextDisplay(TextDisplay),
    Section(Section),
    MediaGallery(MediaGallery),
    Separator(Separator),
    File(FileComponent),
}

impl<'de> Deserialize<'de> for ContainerChildComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::ActionRow => {
                from_value(value).map(ContainerChildComponent::ActionRow).map_err(DeError::custom)
            },
            ComponentType::TextDisplay => {
                from_value(value).map(ContainerChildComponent::TextDisplay).map_err(DeError::custom)
            },
            ComponentType::Section => {
                from_value(value).map(ContainerChildComponent::Section).map_err(DeError::custom)
            },
            ComponentType::MediaGallery => {
                from_value(value).map(ContainerChildComponent::MediaGallery).map_err(DeError::custom)
            },
            ComponentType::Separator => {
                from_value(value).map(ContainerChildComponent::Separator).map_err(DeError::custom)
            },
            ComponentType::File => {
                from_value(value).map(ContainerChildComponent::File).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "invalid container child component type: {other:?}"
            ))),
        }
    }
}

impl Serialize for ContainerChildComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::ActionRow(c) => c.serialize(serializer),
            Self::TextDisplay(c) => c.serialize(serializer),
            Self::Section(c) => c.serialize(serializer),
            Self::MediaGallery(c) => c.serialize(serializer),
            Self::Separator(c) => c.serialize(serializer),
            Self::File(c) => c.serialize(serializer),
        }
    }
}

impl From<ActionRow> for ContainerChildComponent {
    fn from(component: ActionRow) -> Self {
        ContainerChildComponent::ActionRow(component)
    }
}

impl From<TextDisplay> for ContainerChildComponent {
    fn from(component: TextDisplay) -> Self {
        ContainerChildComponent::TextDisplay(component)
    }
}

impl From<Section> for ContainerChildComponent {
    fn from(component: Section) -> Self {
        ContainerChildComponent::Section(component)
    }
}

impl From<MediaGallery> for ContainerChildComponent {
    fn from(component: MediaGallery) -> Self {
        ContainerChildComponent::MediaGallery(component)
    }
}

impl From<Separator> for ContainerChildComponent {
    fn from(component: Separator) -> Self {
        ContainerChildComponent::Separator(component)
    }
}

impl From<FileComponent> for ContainerChildComponent {
    fn from(component: FileComponent) -> Self {
        ContainerChildComponent::File(component)
    }
}

/// A label component — wraps modal components with a label and optional description.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#label).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Label {
    /// The component type. Always [`ComponentType::Label`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// The label text; max 45 characters.
    pub label: String,
    /// An optional description text for the label; max 100 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The component within the label.
    pub component: LabelChildComponent,
}

/// A child component within a [`Label`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum LabelChildComponent {
    InputText(InputText),
    SelectMenu(SelectMenu),
    FileUpload(FileUpload),
    RadioGroup(RadioGroup),
    CheckboxGroup(CheckboxGroup),
    Checkbox(Checkbox),
}

impl<'de> Deserialize<'de> for LabelChildComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::InputText => {
                from_value(value).map(LabelChildComponent::InputText).map_err(DeError::custom)
            },
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => {
                from_value(value).map(LabelChildComponent::SelectMenu).map_err(DeError::custom)
            },
            ComponentType::FileUpload => {
                from_value(value).map(LabelChildComponent::FileUpload).map_err(DeError::custom)
            },
            ComponentType::RadioGroup => {
                from_value(value).map(LabelChildComponent::RadioGroup).map_err(DeError::custom)
            },
            ComponentType::CheckboxGroup => {
                from_value(value).map(LabelChildComponent::CheckboxGroup).map_err(DeError::custom)
            },
            ComponentType::Checkbox => {
                from_value(value).map(LabelChildComponent::Checkbox).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "invalid label child component type: {other:?}"
            ))),
        }
    }
}

impl Serialize for LabelChildComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::InputText(c) => c.serialize(serializer),
            Self::SelectMenu(c) => c.serialize(serializer),
            Self::FileUpload(c) => c.serialize(serializer),
            Self::RadioGroup(c) => c.serialize(serializer),
            Self::CheckboxGroup(c) => c.serialize(serializer),
            Self::Checkbox(c) => c.serialize(serializer),
        }
    }
}

impl From<InputText> for LabelChildComponent {
    fn from(component: InputText) -> Self {
        LabelChildComponent::InputText(component)
    }
}

impl From<SelectMenu> for LabelChildComponent {
    fn from(component: SelectMenu) -> Self {
        LabelChildComponent::SelectMenu(component)
    }
}

impl From<FileUpload> for LabelChildComponent {
    fn from(component: FileUpload) -> Self {
        LabelChildComponent::FileUpload(component)
    }
}

impl From<RadioGroup> for LabelChildComponent {
    fn from(component: RadioGroup) -> Self {
        LabelChildComponent::RadioGroup(component)
    }
}

impl From<CheckboxGroup> for LabelChildComponent {
    fn from(component: CheckboxGroup) -> Self {
        LabelChildComponent::CheckboxGroup(component)
    }
}

impl From<Checkbox> for LabelChildComponent {
    fn from(component: Checkbox) -> Self {
        LabelChildComponent::Checkbox(component)
    }
}

/// A file upload component — allows users to upload files in modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file-upload).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileUpload {
    /// The component type. Always [`ComponentType::FileUpload`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Developer-defined identifier for the input; 1-100 characters.
    pub custom_id: String,
    /// Minimum number of items that must be uploaded (defaults to 1); min 0, max 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_values: Option<u8>,
    /// Maximum number of items that can be uploaded (defaults to 1); max 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_values: Option<u8>,
    /// Whether file upload is required (defaults to true).
    #[serde(default = "default_true")]
    pub required: bool,
}

/// A radio group option.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#radio-group-radio-group-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct RadioGroupOption {
    /// Dev-defined value of the option; max 100 characters.
    pub value: String,
    /// User-facing label of the option; max 100 characters.
    pub label: String,
    /// Optional description for the option; max 100 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Shows the option as selected by default.
    #[serde(default)]
    pub default: bool,
}

/// A radio group component — single-choice set of options.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#radio-group).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct RadioGroup {
    /// The component type. Always [`ComponentType::RadioGroup`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Developer-defined identifier for the input; 1-100 characters.
    pub custom_id: String,
    /// List of options to show; min 2, max 10.
    pub options: Vec<RadioGroupOption>,
    /// Whether a selection is required (defaults to true).
    #[serde(default = "default_true")]
    pub required: bool,
}

/// A checkbox group option.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#checkbox-group-checkbox-group-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckboxGroupOption {
    /// Dev-defined value of the option; max 100 characters.
    pub value: String,
    /// User-facing label of the option; max 100 characters.
    pub label: String,
    /// Optional description for the option; max 100 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Shows the option as selected by default.
    #[serde(default)]
    pub default: bool,
}

/// A checkbox group component — multi-selectable group of checkboxes.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#checkbox-group).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckboxGroup {
    /// The component type. Always [`ComponentType::CheckboxGroup`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Developer-defined identifier for the input; 1-100 characters.
    pub custom_id: String,
    /// List of options to show; min 1, max 10.
    pub options: Vec<CheckboxGroupOption>,
    /// Minimum number of items that must be chosen (defaults to 1); min 0, max 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_values: Option<u8>,
    /// Maximum number of items that can be chosen; min 1, max 10 (defaults to the number of options).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_values: Option<u8>,
    /// Whether selecting within the group is required (defaults to true).
    #[serde(default = "default_true")]
    pub required: bool,
}

/// A checkbox component — single checkbox for yes/no choice.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#checkbox).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Checkbox {
    /// The component type. Always [`ComponentType::Checkbox`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Optional identifier for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Developer-defined identifier for the input; 1-100 characters.
    pub custom_id: String,
    /// Whether the checkbox is selected by default.
    #[serde(default)]
    pub default: bool,
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

        button.data = ButtonKind::Premium {
            sku_id: 1234965026943668316.into(),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 6, "sku_id": "1234965026943668316", "label": "a", "disabled": false}),
        );
    }
}
