#[cfg(feature = "unstable")]
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
        #[cfg(feature = "unstable")]
        Section = 9,
        #[cfg(feature = "unstable")]
        TextDisplay = 10,
        #[cfg(feature = "unstable")]
        Thumbnail = 11,
        #[cfg(feature = "unstable")]
        MediaGallery = 12,
        #[cfg(feature = "unstable")]
        File = 13,
        #[cfg(feature = "unstable")]
        Separator = 14,
        #[cfg(feature = "unstable")]
        Container = 17,
        _ => Unknown(u8),
    }
}

/// Represents Discord components, a part of messages that are usually interactable.
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
#[cfg(feature = "unstable")]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
    Section(Section),
    TextDisplay(TextDisplay),
    Thumbnail(Thumbnail),
    MediaGallery(MediaGallery),
    Separator(Separator),
    File(FileComponent),
    Container(Container),
    Unknown,
    // always update the macro below.
}

// TODO: add something like this to every variant.
// The component type, it will always be [`ComponentType::Thing`].

#[cfg(feature = "unstable")]
impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ComponentRaw {
            #[serde(rename = "type")]
            kind: ComponentType,
        }

        let value = <&RawValue>::deserialize(deserializer)?;
        let raw = ComponentRaw::deserialize(value).map_err(DeError::custom)?;

        match raw.kind {
            ComponentType::ActionRow => Deserialize::deserialize(value).map(Component::ActionRow),
            ComponentType::Button => Deserialize::deserialize(value).map(Component::Button),
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => {
                Deserialize::deserialize(value).map(Component::SelectMenu)
            },
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
            // TODO: maybe just not include it so the deserialization doesn't explode.
            // With all new component types right now, the ENTIRE message won't deserialize.
            // I need to do other stuff in other places too so that its as resilent as possible, it
            // should not die when discord adds new stuff.
            _ => Err(DeError::custom("Unknown component type")),
        }
        .map_err(DeError::custom)
    }
}

/// A component that is a container for up to 3 text display components and an accessory.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct Section {
    /// Always [`ComponentType::Section`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components inside of the section.
    ///
    /// As of 2025-02-28, this is limited to just [`ComponentType::TextDisplay`] with up to 3 max.
    pub components: FixedArray<Component>,
    /// The accessory to the side of the section.
    ///
    /// As of 2025-02-28, this is limited to [`ComponentType::Button`] or
    /// [`ComponentType::Thumbnail`]
    pub accessory: Box<Component>,
}

/// A section component's thumbnail.
///
/// See [`Section`] for how this fits within a section.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
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

/// An abstraction over a resolved and unresolved unfurled media item.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
#[cfg(feature = "unstable")]
pub enum MediaItem {
    Resolved(ResolvedUnfurledMediaItem),
    Unresolved(UnfurledMediaItem),
}

/// An unfurled media item, stores the url to the item.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct UnfurledMediaItem {
    /// The url of this item.
    pub url: FixedString<u16>,
}

/// A resolved unfurled media item, with extra metadata added by Discord.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct ResolvedUnfurledMediaItem {
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
    /// The loading state of the item, declaring if it has fully loaded yet.
    pub loading_state: Option<UnfurledMediaItemLoadingState>,
}

#[cfg(feature = "unstable")]
enum_number! {
    /// The loading state of the media item.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum UnfurledMediaItemLoadingState {
        DiscordUnknown = 0,
        Loading = 1,
        LoadingSuccess = 2,
        LoadingNotFound = 3,
        _ => Unknown(u8),
    }
}

/// A text display component.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct TextDisplay {
    /// The content of this text display component.
    pub content: FixedString<u16>,
}

/// A media gallery component.
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
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
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct MediaGalleryItem {
    /// The internal media piece that this item contains.
    pub media: UnfurledMediaItem,
    /// The description of the media item.
    pub description: Option<FixedString<u16>>,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
}

/// A separator component
///
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct Separator {
    /// Always [`ComponentType::Separator`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Whether or not this contains a separating divider.
    pub divider: Option<bool>,
    /// The spacing of the separator.
    pub spacing: Option<SeparatorSpacingSize>,
}

#[cfg(feature = "unstable")]
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
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
#[cfg(feature = "unstable")]
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
/// [Incomplete Discord docs](https://github.com/Lulalaby/discord-api-docs/pull/30)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[non_exhaustive]
#[cfg(feature = "unstable")]
pub struct Container {
    /// Always [`ComponentType::Container`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The accent colour, similar to an embeds accent.
    pub accent_color: Option<Colour>,
    /// Whether or not this component is spoilered.
    pub spoiler: Option<bool>,
    /// The components within this container.
    ///
    /// As of 2025-02-28, this can be [`ComponentType::ActionRow`], [`ComponentType::Section`],
    /// [`ComponentType::TextDisplay`], [`ComponentType::MediaGallery`], [`ComponentType::File`] or
    /// [`ComponentType::Separator`]
    pub components: FixedArray<Component>,
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
            ComponentType::InputText => {
                Deserialize::deserialize(raw_data).map(ActionRowComponent::InputText)
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
                return Err(DeError::custom(format_args!("Unknown component type {i}")));
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
