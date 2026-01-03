use nonmax::NonMaxU32;
use serde::de::Error as DeError;
use serde::ser::{Serialize, Serializer};
use serde_json::value::RawValue;

use crate::model::prelude::*;
use crate::model::utils::default_true;

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
        Section = 9,
        TextDisplay = 10,
        Thumbnail = 11,
        MediaGallery = 12,
        File = 13,
        Separator = 14,
        Container = 17,
        Label = 18,
        FileUpload = 19,
        _ => Unknown(u8),
    }
}

/// Represents top-level Discord components, a part of messages that are usually interactable.
///
/// # Component Versioning
///
/// - When `IS_COMPONENTS_V2` is **not** set, the **only** valid top-level component is
///   [`ActionRow`].
/// - When `IS_COMPONENTS_V2` **is** set, other component types may be used at the top level, but
///   other message limitations are applied.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub enum Component {
    ActionRow(ActionRow),
    Section(Section),
    TextDisplay(TextDisplay),
    MediaGallery(MediaGallery),
    File(FileComponent),
    Separator(Separator),
    Container(Container),
    Label(Label),
    Unknown(u8),
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_json::value::RawValue;

        #[derive(Deserialize)]
        struct ComponentRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let value = <&RawValue>::deserialize(deserializer)?;
        let raw = ComponentRaw::deserialize(value).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::ActionRow => Deserialize::deserialize(value).map(Component::ActionRow),
            ComponentType::Section => Deserialize::deserialize(value).map(Component::Section),
            ComponentType::TextDisplay => {
                Deserialize::deserialize(value).map(Component::TextDisplay)
            },
            ComponentType::MediaGallery => {
                Deserialize::deserialize(value).map(Component::MediaGallery)
            },
            ComponentType::Separator => Deserialize::deserialize(value).map(Component::Separator),
            ComponentType::File => Deserialize::deserialize(value).map(Component::File),
            ComponentType::Container => Deserialize::deserialize(value).map(Component::Container),
            ComponentType::Label => Deserialize::deserialize(value).map(Component::Label),
            ComponentType(i) => Ok(Component::Unknown(i)),
        }
        .map_err(DeError::custom)
    }
}

/// A component that is a container for up to 3 text display components and an accessory.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#section)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Section {
    /// Always [`ComponentType::Section`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components inside of the section. At least one is required, with a maximum limit of 3.
    pub components: FixedArray<SectionComponent>,
    /// The accessory to the side of the section.
    pub accessory: Box<SectionAccessory>,
}

/// A child component representing the content of a section.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#section-section-child-components)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub enum SectionComponent {
    TextDisplay(TextDisplay),
}

impl<'de> Deserialize<'de> for SectionComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct SectionComponentRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;
        let raw = SectionComponentRaw::deserialize(raw_data).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::TextDisplay => {
                Deserialize::deserialize(raw_data).map(SectionComponent::TextDisplay)
            },
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown section component type {i}")));
            },
        }
        .map_err(DeError::custom)
    }
}

/// A component that is contextually associated to the content of a section.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#section-section-accessory-components)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub enum SectionAccessory {
    Button(Button),
    Thumbnail(Thumbnail),
}

impl<'de> Deserialize<'de> for SectionAccessory {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct SectionAccessoryRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;
        let raw = SectionAccessoryRaw::deserialize(raw_data).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::Button => {
                Deserialize::deserialize(raw_data).map(SectionAccessory::Button)
            },
            ComponentType::Thumbnail => {
                Deserialize::deserialize(raw_data).map(SectionAccessory::Thumbnail)
            },
            ComponentType(i) => {
                return Err(DeError::custom(format_args!(
                    "Unknown section accessory component type {i}"
                )));
            },
        }
        .map_err(DeError::custom)
    }
}

/// A section component's thumbnail.
///
/// See [`Section`] for how this fits within a section.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#thumbnail)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Thumbnail {
    /// Always [`ComponentType::Thumbnail`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The internal media item this contains.
    pub media: UnfurledMediaItem,
    /// The description of the thumbnail.
    pub description: Option<FixedString<u16>>,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
}

/// A url or attachment.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#unfurled-media-item)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnfurledMediaItem {
    /// The url of this item.
    pub url: FixedString<u16>,
    /// The proxied discord url.
    pub proxy_url: Option<FixedString<u16>>,
    /// The width of the media item.
    pub width: Option<NonMaxU32>,
    /// The height of the media item.
    pub height: Option<NonMaxU32>,
    /// The content type of the media item.
    pub content_type: Option<FixedString>,
}

/// A component that allows you to add text to your message, similiar to the `content` field of a
/// message.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-display)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TextDisplay {
    /// Always [`ComponentType::TextDisplay`]
    #[serde(rename = "type")]
    pub kind: ComponentType,

    /// Discord’s official documentation does not mention this field; however, it is currently
    /// returned by the API and represents meaningful data, so it is included here. This
    /// behavior is undocumented and may change or be removed by Discord at any time.
    #[cfg(feature = "unstable")]
    pub content: Option<String>,
}

/// A Media Gallery is a component that allows you to display media attachments in an organized
/// gallery format.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#media-gallery)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MediaGallery {
    /// Always [`ComponentType::MediaGallery`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Array of images this media gallery can contain, max of 10.
    pub items: FixedArray<MediaGalleryItem>,
}

/// An individual media gallery item.
///
/// Belongs to [`MediaGallery`].
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#media-gallery-media-gallery-item-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MediaGalleryItem {
    /// The internal media piece that this item contains.
    pub media: UnfurledMediaItem,
    /// The description of the media item.
    pub description: Option<FixedString<u16>>,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
}

/// A component that adds vertical padding and visual division between other components.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#separator)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Separator {
    /// Always [`ComponentType::Separator`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Whether or not this contains a separating divider.
    pub divider: Option<bool>,
    /// The spacing of the separator.
    pub spacing: Option<SeparatorSpacingSize>,
}

enum_number! {
    /// The size of a separator component.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum SeparatorSpacingSize {
        Small = 1,
        Large = 2,
        _ => Unknown(u8),
    }
}

/// A file component, will not render a text preview to the user.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct FileComponent {
    /// Always [`ComponentType::File`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The file this component internally contains.
    pub file: UnfurledMediaItem,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
}

/// A container component, similar to an embed but without all the functionality.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#container)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct Container {
    /// Always [`ComponentType::Container`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The accent colour, similar to an embeds accent.
    pub accent_color: Option<Colour>,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
    /// The components within this container.
    pub components: FixedArray<ContainerComponent>,
}

/// A child component encapsulated within a container.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#container-container-child-components)
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub enum ContainerComponent {
    ActionRow(ActionRow),
    Section(Section),
    TextDisplay(TextDisplay),
    MediaGallery(MediaGallery),
    File(FileComponent),
    Separator(Separator),
}

impl<'de> Deserialize<'de> for ContainerComponent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_json::value::RawValue;

        #[derive(Deserialize)]
        struct ContainerComponentRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let value = <&RawValue>::deserialize(deserializer)?;
        let raw = ContainerComponentRaw::deserialize(value).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::ActionRow => {
                Deserialize::deserialize(value).map(ContainerComponent::ActionRow)
            },
            ComponentType::Section => {
                Deserialize::deserialize(value).map(ContainerComponent::Section)
            },
            ComponentType::TextDisplay => {
                Deserialize::deserialize(value).map(ContainerComponent::TextDisplay)
            },
            ComponentType::MediaGallery => {
                Deserialize::deserialize(value).map(ContainerComponent::MediaGallery)
            },
            ComponentType::Separator => {
                Deserialize::deserialize(value).map(ContainerComponent::Separator)
            },
            ComponentType::File => Deserialize::deserialize(value).map(ContainerComponent::File),
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown container component type {i}")));
            },
        }
        .map_err(DeError::custom)
    }
}

/// A layout component that wraps modal components with a label and optional description.
///
/// **Note**: Labels can only appear within modals, and will not include the `label` or
/// `description` field when part of a modal response.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#label-label-interaction-response-structure)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct Label {
    /// Always [`ComponentType::Label`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The component within the label.
    pub component: LabelComponent,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[serde(untagged)]
#[non_exhaustive]
pub enum LabelComponent {
    SelectMenu(SelectMenu),
    InputText(InputText),
    FileUpload(FileUpload),
}

impl<'de> Deserialize<'de> for LabelComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct LabelComponentRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;
        let raw = LabelComponentRaw::deserialize(raw_data).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => {
                Deserialize::deserialize(raw_data).map(LabelComponent::SelectMenu)
            },
            ComponentType::InputText => {
                Deserialize::deserialize(raw_data).map(LabelComponent::InputText)
            },
            ComponentType::FileUpload => {
                Deserialize::deserialize(raw_data).map(LabelComponent::FileUpload)
            },
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown label component type {i}")));
            },
        }
        .map_err(DeError::custom)
    }
}

/// An interactive component that allows users to upload files in modals.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct FileUpload {
    /// Always [`ComponentType::FileUpload`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the file upload; max 100 characters
    pub custom_id: FixedString,
    /// IDs of the uploaded files found in [`ModalInteractionData::resolved`].
    pub values: FixedArray<AttachmentId>,
}

/// An action row.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#action-row).
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#action-row-action-row-child-components).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ActionRowComponent {
    Button(Button),
    SelectMenu(SelectMenu),
}

impl<'de> Deserialize<'de> for ActionRowComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct ActionRowRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let raw_data = <&RawValue>::deserialize(deserializer)?;
        let raw = ActionRowRaw::deserialize(raw_data).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::Button => {
                Deserialize::deserialize(raw_data).map(ActionRowComponent::Button)
            },
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => {
                Deserialize::deserialize(raw_data).map(ActionRowComponent::SelectMenu)
            },
            ComponentType::ActionRow => {
                return Err(DeError::custom("Invalid component type ActionRow"));
            },
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown action row component type {i}")));
            },
        }
        .map_err(DeError::custom)
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#button).
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#component-object-component-types).
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
    pub custom_id: FixedString,
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
    #[serde(default)]
    pub values: FixedArray<String>,
}

/// A select menu component options.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#string-select-select-option-structure).
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-input).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the input; max 100 characters
    pub custom_id: FixedString,
    /// The [`InputTextStyle`]. Required when sending modal data.
    ///
    /// Discord docs are wrong here; it says the field is always sent in modal submit interactions
    /// but it's not. It's only required when _sending_ modal data to Discord.
    /// <https://github.com/discord/discord-api-docs/issues/6141>
    pub style: Option<InputTextStyle>,
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
    /// [Discord docs](https://discord.com/developers/docs/components/reference#text-input-text-input-styles).
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
