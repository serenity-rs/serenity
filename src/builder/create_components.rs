use serde::Serialize;

use crate::model::prelude::*;

/// A builder for creating a components action row in a message.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateActionRow {
    Buttons(Vec<CreateButton>),
    SelectMenu(CreateSelectMenu),
    /// Only valid in modals!
    InputText(CreateInputText),
}

impl serde::Serialize for CreateActionRow {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &1_u8)?;

        match self {
            CreateActionRow::Buttons(buttons) => map.serialize_entry("components", &buttons)?,
            CreateActionRow::SelectMenu(select) => map.serialize_entry("components", &[select])?,
            CreateActionRow::InputText(input) => map.serialize_entry("components", &[input])?,
        }

        map.end()
    }
}

/// A builder for creating a button component in a message
#[derive(Clone, Debug, Serialize, PartialEq)]
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

    /// Creates a new premium button associated with the given SKU.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
    pub fn new_premium(sku_id: impl Into<SkuId>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::Premium {
                sku_id: sku_id.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Creates a normal button with the given custom ID. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
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

    /// Sets the custom id of the button, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    ///
    /// Has no effect on link buttons and premium buttons.
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
    /// Has no effect on link buttons and premium buttons.
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

impl From<Button> for CreateButton {
    fn from(button: Button) -> Self {
        Self(button)
    }
}

struct CreateSelectMenuDefault(Mention);

impl Serialize for CreateSelectMenuDefault {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let (id, kind) = match self.0 {
            Mention::Channel(c) => (c.get(), "channel"),
            Mention::Role(r) => (r.get(), "role"),
            Mention::User(u) => (u.get(), "user"),
        };

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("id", &id)?;
        map.serialize_entry("type", kind)?;
        map.end()
    }
}

/// Discord docs: [String Select](https://docs.discord.com/developers/components/reference#string-select), [User Select](https://docs.discord.com/developers/components/reference#user-select), [Role Select](https://docs.discord.com/developers/components/reference#role-select), [Mentionable Select](https://docs.discord.com/developers/components/reference#mentionable-select), [Channel Select](https://docs.discord.com/developers/components/reference#channel-select).
#[derive(Clone, Debug, PartialEq)]
pub enum CreateSelectMenuKind {
    String { options: Vec<CreateSelectMenuOption> },
    User { default_users: Option<Vec<UserId>> },
    Role { default_roles: Option<Vec<RoleId>> },
    Mentionable { default_users: Option<Vec<UserId>>, default_roles: Option<Vec<RoleId>> },
    Channel { channel_types: Option<Vec<ChannelType>>, default_channels: Option<Vec<ChannelId>> },
}

impl Serialize for CreateSelectMenuKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Json<'a> {
            #[serde(rename = "type")]
            kind: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<&'a [CreateSelectMenuOption]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_types: Option<&'a [ChannelType]>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            default_values: Vec<CreateSelectMenuDefault>,
        }

        #[allow(clippy::ref_option)]
        fn map<I: Into<Mention> + Copy>(
            values: &Option<Vec<I>>,
        ) -> impl Iterator<Item = CreateSelectMenuDefault> + '_ {
            // Calling `.iter().flatten()` on the `Option` treats `None` like an empty vec
            values.iter().flatten().map(|&i| CreateSelectMenuDefault(i.into()))
        }

        #[rustfmt::skip]
        let default_values = match self {
            Self::String { .. } => vec![],
            Self::User { default_users: default_values } => map(default_values).collect(),
            Self::Role { default_roles: default_values } => map(default_values).collect(),
            Self::Mentionable { default_users, default_roles } => {
                let users = map(default_users);
                let roles = map(default_roles);
                users.chain(roles).collect()
            },
            Self::Channel { channel_types: _, default_channels: default_values } => map(default_values).collect(),
        };

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
                Self::Channel { channel_types, default_channels: _ } => channel_types.as_deref(),
                _ => None,
            },
            default_values,
        };

        json.serialize(serializer)
    }
}

/// A builder for creating a select menu component in a message
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#component-object).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateSelectMenu {
    custom_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,

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
            required: None,
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
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Sets whether the select menu is required (for modals).
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}

/// A builder for creating an option of a select menu component in a message
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#string-select-select-option-structure)
#[derive(Clone, Debug, Serialize, PartialEq)]
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
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }
}

/// A builder for creating an input text component in a modal
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-structure).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateInputText(InputText);

impl CreateInputText {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(
        style: InputTextStyle,
        label: impl Into<String>,
        custom_id: impl Into<String>,
    ) -> Self {
        Self(InputText {
            style: Some(style),
            label: Some(label.into()),
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: None,
            required: true,

            kind: ComponentType::InputText,
        })
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.0.style = Some(kind);
        self
    }

    /// Sets the label of this input text. Replaces the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = id.into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.0.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u16) -> Self {
        self.0.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u16) -> Self {
        self.0.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.0.required = required;
        self
    }
}

/// A top-level component that can be used in messages with the `IS_COMPONENTS_V2` flag.
///
/// This allows you to use the new Component V2 layout system. Create a message with
/// `MessageFlags::IS_COMPONENTS_V2` and then use these components instead of
/// `content` and `embeds`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CreateComponents {
    ActionRow(CreateActionRow),
    TextDisplay(CreateTextDisplay),
    Section(CreateSection),
    MediaGallery(CreateMediaGallery),
    Separator(CreateSeparator),
    Container(CreateContainer),
    File(CreateFileComponent),
    Label(CreateLabel),
    InputText(CreateInputText),
    FileUpload(CreateFileUpload),
    RadioGroup(CreateRadioGroup),
    CheckboxGroup(CreateCheckboxGroup),
    Checkbox(CreateCheckbox),
}

impl Serialize for CreateComponents {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ActionRow(c) => c.serialize(serializer),
            Self::TextDisplay(c) => c.serialize(serializer),
            Self::Section(c) => c.serialize(serializer),
            Self::MediaGallery(c) => c.serialize(serializer),
            Self::Separator(c) => c.serialize(serializer),
            Self::Container(c) => c.serialize(serializer),
            Self::File(c) => c.serialize(serializer),
            Self::Label(c) => c.serialize(serializer),
            Self::InputText(c) => c.serialize(serializer),
            Self::FileUpload(c) => c.serialize(serializer),
            Self::RadioGroup(c) => c.serialize(serializer),
            Self::CheckboxGroup(c) => c.serialize(serializer),
            Self::Checkbox(c) => c.serialize(serializer),
        }
    }
}

impl From<CreateActionRow> for CreateComponents {
    fn from(component: CreateActionRow) -> Self {
        Self::ActionRow(component)
    }
}

impl From<CreateTextDisplay> for CreateComponents {
    fn from(component: CreateTextDisplay) -> Self {
        Self::TextDisplay(component)
    }
}

impl From<CreateSection> for CreateComponents {
    fn from(component: CreateSection) -> Self {
        Self::Section(component)
    }
}

impl From<CreateMediaGallery> for CreateComponents {
    fn from(component: CreateMediaGallery) -> Self {
        Self::MediaGallery(component)
    }
}

impl From<CreateSeparator> for CreateComponents {
    fn from(component: CreateSeparator) -> Self {
        Self::Separator(component)
    }
}

impl From<CreateContainer> for CreateComponents {
    fn from(component: CreateContainer) -> Self {
        Self::Container(component)
    }
}

impl From<CreateFileComponent> for CreateComponents {
    fn from(component: CreateFileComponent) -> Self {
        Self::File(component)
    }
}

/// A builder for creating a text display component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-display).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateTextDisplay {
    #[serde(rename = "type")]
    kind: u8,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
}

impl CreateTextDisplay {
    /// Creates a new text display component with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            kind: 10,
            content: content.into(),
            id: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the content of the text display.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
}

/// A builder for creating a section component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#section).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateSection {
    #[serde(rename = "type")]
    kind: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    components: Vec<CreateTextDisplay>,
    accessory: CreateSectionAccessory,
}

/// Accessory for a section component — either a button or a thumbnail.
///
/// Used with [`CreateSection`] to associate an interactive element or visual
/// accessory with the section's text content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CreateSectionAccessory {
    Button(CreateButton),
    Thumbnail(CreateThumbnail),
}

impl Serialize for CreateSectionAccessory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Button(c) => c.serialize(serializer),
            Self::Thumbnail(c) => c.serialize(serializer),
        }
    }
}

/// A builder for creating a thumbnail component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#thumbnail).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateThumbnail {
    #[serde(rename = "type")]
    kind: u8,
    media: CreateUnfurledMediaItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    spoiler: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
}

impl CreateThumbnail {
    /// Creates a new thumbnail component with the given media URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            kind: 11,
            media: CreateUnfurledMediaItem {
                url: url.into(),
            },
            description: None,
            spoiler: false,
            id: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the description (alt text) for the thumbnail.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets whether the thumbnail should be a spoiler.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct CreateUnfurledMediaItem {
    url: String,
}

impl CreateSection {
    /// Creates a new section with the given text display content and an accessory.
    pub fn new(content: impl Into<String>, accessory: impl Into<CreateSectionAccessory>) -> Self {
        Self {
            kind: 9,
            id: None,
            components: vec![CreateTextDisplay::new(content)],
            accessory: accessory.into(),
        }
    }

    /// Creates a new section with multiple text display components and an accessory.
    pub fn new_with_components(
        components: Vec<CreateTextDisplay>,
        accessory: impl Into<CreateSectionAccessory>,
    ) -> Self {
        Self {
            kind: 9,
            id: None,
            components,
            accessory: accessory.into(),
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Adds a text display component to the section (up to 3 total).
    pub fn add_text(mut self, text: CreateTextDisplay) -> Self {
        self.components.push(text);
        self
    }
}

impl From<CreateButton> for CreateSectionAccessory {
    fn from(button: CreateButton) -> Self {
        Self::Button(button)
    }
}

impl From<CreateThumbnail> for CreateSectionAccessory {
    fn from(thumbnail: CreateThumbnail) -> Self {
        Self::Thumbnail(thumbnail)
    }
}

impl From<CreateLabel> for CreateComponents {
    fn from(component: CreateLabel) -> Self {
        Self::Label(component)
    }
}

impl From<CreateInputText> for CreateComponents {
    fn from(component: CreateInputText) -> Self {
        Self::InputText(component)
    }
}

impl From<CreateFileUpload> for CreateComponents {
    fn from(component: CreateFileUpload) -> Self {
        Self::FileUpload(component)
    }
}

impl From<CreateRadioGroup> for CreateComponents {
    fn from(component: CreateRadioGroup) -> Self {
        Self::RadioGroup(component)
    }
}

impl From<CreateCheckboxGroup> for CreateComponents {
    fn from(component: CreateCheckboxGroup) -> Self {
        Self::CheckboxGroup(component)
    }
}

impl From<CreateCheckbox> for CreateComponents {
    fn from(component: CreateCheckbox) -> Self {
        Self::Checkbox(component)
    }
}

/// A builder for creating a media gallery component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#media-gallery).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateMediaGallery {
    #[serde(rename = "type")]
    kind: u8,
    items: Vec<CreateMediaGalleryItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
}

/// A builder for creating a media gallery item.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateMediaGalleryItem {
    media: CreateUnfurledMediaItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    spoiler: bool,
}

impl CreateMediaGallery {
    /// Creates a new media gallery component.
    pub fn new() -> Self {
        Self {
            kind: 12,
            items: Vec::new(),
            id: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Adds a media gallery item with the given URL.
    pub fn add_item(mut self, url: impl Into<String>) -> Self {
        self.items.push(CreateMediaGalleryItem {
            media: CreateUnfurledMediaItem {
                url: url.into(),
            },
            description: None,
            spoiler: false,
        });
        self
    }

    /// Adds a media gallery item with description.
    pub fn add_item_with_description(
        mut self,
        url: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.items.push(CreateMediaGalleryItem {
            media: CreateUnfurledMediaItem {
                url: url.into(),
            },
            description: Some(description.into()),
            spoiler: false,
        });
        self
    }

    /// Adds a media gallery item with description and spoiler flag.
    pub fn add_item_full(
        mut self,
        url: impl Into<String>,
        description: impl Into<String>,
        spoiler: bool,
    ) -> Self {
        self.items.push(CreateMediaGalleryItem {
            media: CreateUnfurledMediaItem {
                url: url.into(),
            },
            description: Some(description.into()),
            spoiler,
        });
        self
    }
}

impl Default for CreateMediaGallery {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for creating a separator component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#separator).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateSeparator {
    #[serde(rename = "type")]
    kind: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    divider: bool,
    #[serde(rename = "spacing", skip_serializing_if = "Option::is_none")]
    spacing: Option<u8>,
}

impl CreateSeparator {
    /// Creates a new separator component with default settings (small spacing, divider on).
    pub fn new() -> Self {
        Self {
            kind: 14,
            id: None,
            divider: true,
            spacing: Some(1),
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets whether a visual divider should be displayed. Defaults to true.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// Sets the spacing size: 1 for small, 2 for large. Defaults to 1.
    pub fn spacing(mut self, spacing: u8) -> Self {
        self.spacing = Some(spacing);
        self
    }
}

impl Default for CreateSeparator {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for creating a container component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#container).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateContainer {
    #[serde(rename = "type")]
    kind: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    components: Vec<CreateContainerChild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accent_color: Option<u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    spoiler: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    padding: Option<u8>,
}

#[derive(Clone, Debug)]
enum CreateContainerChild {
    ActionRow(CreateActionRow),
    TextDisplay(CreateTextDisplay),
    Section(CreateSection),
    MediaGallery(CreateMediaGallery),
    Separator(CreateSeparator),
    File(CreateFileComponent),
    Label(CreateLabel),
    InputText(CreateInputText),
}

impl PartialEq for CreateContainerChild {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        if discriminant(self) != discriminant(other) {
            return false;
        }
        match (self, other) {
            (Self::ActionRow(a), Self::ActionRow(b)) => a == b,
            (Self::TextDisplay(a), Self::TextDisplay(b)) => a == b,
            (Self::Section(a), Self::Section(b)) => a == b,
            (Self::MediaGallery(a), Self::MediaGallery(b)) => a == b,
            (Self::Separator(a), Self::Separator(b)) => a == b,
            (Self::File(a), Self::File(b)) => a == b,
            (Self::Label(a), Self::Label(b)) => a == b,
            (Self::InputText(a), Self::InputText(b)) => a == b,
            _ => false,
        }
    }
}

impl Serialize for CreateContainerChild {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ActionRow(c) => c.serialize(serializer),
            Self::TextDisplay(c) => c.serialize(serializer),
            Self::Section(c) => c.serialize(serializer),
            Self::MediaGallery(c) => c.serialize(serializer),
            Self::Separator(c) => c.serialize(serializer),
            Self::File(c) => c.serialize(serializer),
            Self::Label(c) => c.serialize(serializer),
            Self::InputText(c) => c.serialize(serializer),
        }
    }
}

impl CreateContainer {
    /// Creates a new container component.
    pub fn new() -> Self {
        Self {
            kind: 17,
            id: None,
            components: Vec::new(),
            accent_color: None,
            spoiler: false,
            padding: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the accent color as an RGB value (0x000000 to 0xFFFFFF).
    pub fn accent_color(mut self, color: u32) -> Self {
        self.accent_color = Some(color);
        self
    }

    /// Sets the padding style of the container.
    ///
    /// - `1` = small padding (default)
    /// - `2` = large padding
    pub fn padding(mut self, padding: u8) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Sets whether the container should be a spoiler.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }

    /// Adds an action row component to the container.
    pub fn add_action_row(mut self, component: CreateActionRow) -> Self {
        self.components.push(CreateContainerChild::ActionRow(component));
        self
    }

    /// Adds a text display component to the container.
    pub fn add_text_display(mut self, component: CreateTextDisplay) -> Self {
        self.components.push(CreateContainerChild::TextDisplay(component));
        self
    }

    /// Adds a section component to the container.
    pub fn add_section(mut self, component: CreateSection) -> Self {
        self.components.push(CreateContainerChild::Section(component));
        self
    }

    /// Adds a media gallery component to the container.
    pub fn add_media_gallery(mut self, component: CreateMediaGallery) -> Self {
        self.components.push(CreateContainerChild::MediaGallery(component));
        self
    }

    /// Adds a separator component to the container.
    pub fn add_separator(mut self, component: CreateSeparator) -> Self {
        self.components.push(CreateContainerChild::Separator(component));
        self
    }

    /// Adds a file component to the container.
    pub fn add_file(mut self, component: CreateFileComponent) -> Self {
        self.components.push(CreateContainerChild::File(component));
        self
    }

    /// Adds a label component to the container (for modals).
    pub fn add_label(mut self, component: CreateLabel) -> Self {
        self.components.push(CreateContainerChild::Label(component));
        self
    }

    /// Adds an input text component to the container (for modals).
    pub fn add_input_text(mut self, component: CreateInputText) -> Self {
        self.components.push(CreateContainerChild::InputText(component));
        self
    }
}

impl Default for CreateContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for creating a file component.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateFileComponent {
    #[serde(rename = "type")]
    kind: u8,
    file: CreateUnfurledMediaItem,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    spoiler: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
}

impl CreateFileComponent {
    /// Creates a new file component referencing an attachment by filename.
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            kind: 13,
            file: CreateUnfurledMediaItem {
                url: format!("attachment://{}", filename.into()),
            },
            spoiler: false,
            id: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets whether the file should be a spoiler.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }
}

/// A builder for creating a label component for modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#label).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateLabel {
    #[serde(rename = "type")]
    kind: u8,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    component: CreateLabelChild,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
enum CreateLabelChild {
    InputText(CreateInputText),
    SelectMenu(CreateSelectMenu),
    FileUpload(CreateFileUpload),
    RadioGroup(CreateRadioGroup),
    CheckboxGroup(CreateCheckboxGroup),
    Checkbox(CreateCheckbox),
}

impl CreateLabel {
    /// Creates a new label wrapping a text input.
    pub fn new_text_input(
        label: impl Into<String>,
        component: CreateInputText,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::InputText(component),
            id: None,
        }
    }

    /// Creates a new label wrapping a select menu.
    pub fn new_select_menu(
        label: impl Into<String>,
        component: CreateSelectMenu,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::SelectMenu(component),
            id: None,
        }
    }

    /// Creates a new label wrapping a file upload.
    pub fn new_file_upload(
        label: impl Into<String>,
        component: CreateFileUpload,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::FileUpload(component),
            id: None,
        }
    }

    /// Creates a new label wrapping a radio group.
    pub fn new_radio_group(
        label: impl Into<String>,
        component: CreateRadioGroup,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::RadioGroup(component),
            id: None,
        }
    }

    /// Creates a new label wrapping a checkbox group.
    pub fn new_checkbox_group(
        label: impl Into<String>,
        component: CreateCheckboxGroup,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::CheckboxGroup(component),
            id: None,
        }
    }

    /// Creates a new label wrapping a checkbox.
    pub fn new_checkbox(
        label: impl Into<String>,
        component: CreateCheckbox,
    ) -> Self {
        Self {
            kind: 18,
            label: label.into(),
            description: None,
            component: CreateLabelChild::Checkbox(component),
            id: None,
        }
    }

    /// Sets the optional component id.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the description for the label; max 100 characters.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A builder for creating a file upload component for modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file-upload).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateFileUpload {
    #[serde(rename = "type")]
    kind: u8,
    custom_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    required: bool,
}

impl CreateFileUpload {
    /// Creates a new file upload component with the given custom id.
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self {
            kind: 19,
            custom_id: custom_id.into(),
            min_values: None,
            max_values: None,
            required: true,
        }
    }

    /// Sets the minimum number of files (defaults to 1); min 0, max 10.
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum number of files (defaults to 1); max 10.
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets whether file upload is required (defaults to true).
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// A builder for creating a radio group option.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateRadioGroupOption {
    value: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    default: bool,
}

impl CreateRadioGroupOption {
    /// Creates a new radio group option with the given value and label.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            default: false,
        }
    }

    /// Sets the description for this option; max 100 characters.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Shows this option as selected by default.
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}

/// A builder for creating a radio group component for modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#radio-group).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateRadioGroup {
    #[serde(rename = "type")]
    kind: u8,
    custom_id: String,
    options: Vec<CreateRadioGroupOption>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    required: bool,
}

impl CreateRadioGroup {
    /// Creates a new radio group with the given custom id and options.
    pub fn new(custom_id: impl Into<String>, options: Vec<CreateRadioGroupOption>) -> Self {
        Self {
            kind: 21,
            custom_id: custom_id.into(),
            options,
            required: true,
        }
    }

    /// Sets whether a selection is required (defaults to true).
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// A builder for creating a checkbox group option.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateCheckboxGroupOption {
    value: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    default: bool,
}

impl CreateCheckboxGroupOption {
    /// Creates a new checkbox group option with the given value and label.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            default: false,
        }
    }

    /// Sets the description for this option; max 100 characters.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Shows this option as selected by default.
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}

/// A builder for creating a checkbox group component for modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#checkbox-group).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateCheckboxGroup {
    #[serde(rename = "type")]
    kind: u8,
    custom_id: String,
    options: Vec<CreateCheckboxGroupOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    required: bool,
}

impl CreateCheckboxGroup {
    /// Creates a new checkbox group with the given custom id and options.
    pub fn new(custom_id: impl Into<String>, options: Vec<CreateCheckboxGroupOption>) -> Self {
        Self {
            kind: 22,
            custom_id: custom_id.into(),
            options,
            min_values: None,
            max_values: None,
            required: true,
        }
    }

    /// Sets the minimum number of items that must be chosen (defaults to 1); min 0, max 10.
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum number of items that can be chosen; min 1, max 10 (defaults to the number of options).
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets whether selecting within the group is required (defaults to true).
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// A builder for creating a checkbox component for modals.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#checkbox).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateCheckbox {
    #[serde(rename = "type")]
    kind: u8,
    custom_id: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    default: bool,
}

impl CreateCheckbox {
    /// Creates a new checkbox with the given custom id.
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self {
            kind: 23,
            custom_id: custom_id.into(),
            default: false,
        }
    }

    /// Sets whether the checkbox is selected by default.
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}
