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
        RadioGroup = 21,
        CheckboxGroup = 22,
        Checkbox = 23,
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
#[serde(untagged)]
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#section)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#section-section-child-components)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#section-section-accessory-components)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#thumbnail)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#unfurled-media-item)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-display)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TextDisplay {
    /// Always [`ComponentType::TextDisplay`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The text content of this component.
    #[serde(default)]
    pub content: FixedString<u16>,
}

/// A Media Gallery is a component that allows you to display media attachments in an organized
/// gallery format.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#media-gallery)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#media-gallery-media-gallery-item-structure)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#separator)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Separator {
    /// Always [`ComponentType::Separator`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Whether or not this contains a separating divider.
    #[serde(default = "default_true")]
    pub divider: bool,
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#file)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#container)
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#container-container-child-components)
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[serde(untagged)]
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#label-label-interaction-response-structure)
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
    RadioGroup(RadioGroup),
    CheckboxGroup(CheckboxGroup),
    Checkbox(Checkbox),
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
            ComponentType::RadioGroup => {
                Deserialize::deserialize(raw_data).map(LabelComponent::RadioGroup)
            },
            ComponentType::CheckboxGroup => {
                Deserialize::deserialize(raw_data).map(LabelComponent::CheckboxGroup)
            },
            ComponentType::Checkbox => {
                Deserialize::deserialize(raw_data).map(LabelComponent::Checkbox)
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

/// An interactive component that allows users to select a radio button in modals.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct RadioGroup {
    /// Always [`ComponentType::RadioGroup`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the radio group; max 100 characters
    pub custom_id: FixedString,
    /// Value of the selected [`CreateRadioGroupOption`][crate::builder::CreateRadioGroupOption].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<FixedString>,
}

/// An interactive component that allows users to group together multiple checkboxes in modals.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct CheckboxGroup {
    /// Always [`ComponentType::CheckboxGroup`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the checkbox group; max 100 characters
    pub custom_id: FixedString,
    /// Values of the selected
    /// [`CreateCheckboxGroupOption`][crate::builder::CreateCheckboxGroupOption].
    pub values: FixedArray<String>,
}

/// An interactive component that allows users to use a checkbox in modals.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
pub struct Checkbox {
    /// Always [`ComponentType::Checkbox`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the checkbox group; max 100 characters
    pub custom_id: FixedString,
    /// The state of the checkbox (`true` if checked, `false` if unchecked).
    pub value: bool,
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
    pub components: FixedArray<ActionRowComponent>,
}

/// A component which can be inside of an [`ActionRow`].
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row-action-row-child-components).
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#button).
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
        // No Link or Premium, because we represent Link and Premium using enum variants.
        _ => Unknown(u8),
    }
}

/// A select menu component.
///
/// Discord docs: [String Select], [User Select], [Role Select], [Mentionable Select], [Channel
/// Select].
///
/// [String Select]: https://docs.discord.com/developers/components/reference#string-select
/// [User Select]: https://docs.discord.com/developers/components/reference#user-select
/// [Role Select]: https://docs.discord.com/developers/components/reference#role-select
/// [Mentionable Select]: https://docs.discord.com/developers/components/reference#mentionable-select
/// [Channel Select]: https://docs.discord.com/developers/components/reference#channel-select
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct SelectMenu {
    #[serde(flatten)]
    pub kind: SelectMenuKind,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: FixedString,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<FixedString>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u8>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u8>,
    /// Whether the select menu is required to be filled in a modal. Default is true. Ignored in
    /// messages.
    pub required: bool,
    /// Whether select menu is disabled.
    pub disabled: bool,
    /// The selected values chosen by the user. Only present in modals.
    pub values: FixedArray<String>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SelectMenuKind {
    String { options: FixedArray<SelectMenuOption> },
    User {},
    Role {},
    Mentionable {},
    Channel { channel_types: FixedArray<ChannelType> },
}

impl<'de> Deserialize<'de> for SelectMenu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SelectMenuRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
            custom_id: FixedString,
            placeholder: Option<FixedString>,
            min_values: Option<u8>,
            max_values: Option<u8>,
            #[serde(default = "default_true")]
            required: bool,
            #[serde(default)]
            disabled: bool,
            #[serde(default)]
            values: FixedArray<String>,
        }

        #[derive(Deserialize)]
        struct StringSelectRaw {
            #[serde(default)]
            options: FixedArray<SelectMenuOption>,
        }

        #[derive(Deserialize)]
        struct ChannelSelectRaw {
            #[serde(default)]
            channel_types: FixedArray<ChannelType>,
        }

        let value = <&RawValue>::deserialize(deserializer)?;
        let raw = SelectMenuRaw::deserialize(value).map_err(DeError::custom)?;

        let kind = match raw.kind {
            ComponentType::StringSelect => {
                StringSelectRaw::deserialize(value).map(|s| SelectMenuKind::String {
                    options: s.options,
                })
            },
            ComponentType::UserSelect => Ok(SelectMenuKind::User {}),
            ComponentType::RoleSelect => Ok(SelectMenuKind::Role {}),
            ComponentType::MentionableSelect => Ok(SelectMenuKind::Mentionable {}),
            ComponentType::ChannelSelect => {
                ChannelSelectRaw::deserialize(value).map(|s| SelectMenuKind::Channel {
                    channel_types: s.channel_types,
                })
            },
            ComponentType(i) => {
                return Err(DeError::custom(format_args!("Unknown select menu type {i}")));
            },
        }
        .map_err(DeError::custom)?;

        Ok(Self {
            kind,
            custom_id: raw.custom_id,
            placeholder: raw.placeholder,
            min_values: raw.min_values,
            max_values: raw.max_values,
            required: raw.required,
            disabled: raw.disabled,
            values: raw.values,
        })
    }
}

impl Serialize for SelectMenuKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct SelectMenuKindRaw<'a> {
            #[serde(rename = "type")]
            kind: ComponentType,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<&'a [SelectMenuOption]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_types: Option<&'a [ChannelType]>,
        }

        #[rustfmt::skip]
        let helper = SelectMenuKindRaw {
            kind: match self {
                Self::String { .. } => ComponentType::StringSelect,
                Self::User { .. } => ComponentType::UserSelect,
                Self::Role { .. } => ComponentType::RoleSelect,
                Self::Mentionable { .. } => ComponentType::MentionableSelect,
                Self::Channel { .. } => ComponentType::ChannelSelect,
            },
            options: match self {
                Self::String { options } => Some(options),
                _ => None,
            },
            channel_types: match self {
                Self::Channel { channel_types } => Some(channel_types),
                _ => None,
            },
        };

        helper.serialize(serializer)
    }
}

/// A select menu component options.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#string-select-select-option-structure).
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
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-input).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the input; max 100 characters
    pub custom_id: FixedString,
    /// The input from the user.
    pub value: FixedString<u16>,
}

enum_number! {
    /// The style of the input text
    ///
    /// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-styles).
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
