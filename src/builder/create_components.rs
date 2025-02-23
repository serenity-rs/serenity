use std::borrow::Cow;

use serde::Serialize;

use crate::model::prelude::*;

#[derive(Clone, Debug)]
struct StaticU8<const VAL: u8>;

impl<const VAL: u8> Serialize for StaticU8<VAL> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u8(VAL)
    }
}

/// A builder for creating a components action row in a message.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#component-object).
#[derive(Clone, Debug)]
#[must_use]
pub enum CreateActionRow<'a> {
    Buttons(Cow<'a, [CreateButton<'a>]>),
    SelectMenu(CreateSelectMenu<'a>),
    /// Only valid in modals!
    InputText(CreateInputText<'a>),
}

impl<'a> CreateActionRow<'a> {
    pub fn buttons(buttons: impl Into<Cow<'a, [CreateButton<'a>]>>) -> Self {
        Self::Buttons(buttons.into())
    }

    pub fn select_menu(select_menu: impl Into<CreateSelectMenu<'a>>) -> Self {
        Self::SelectMenu(select_menu.into())
    }

    pub fn input_text(input_text: impl Into<CreateInputText<'a>>) -> Self {
        Self::InputText(input_text.into())
    }
}

impl serde::Serialize for CreateActionRow<'_> {
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

/// TODO: doc
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateComponent<'a> {
    /// A regular action row, a V1 component.
    ActionRow(CreateActionRow<'a>),
    /// A section, V2 component.
    Section(CreateSection<'a>),
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSection<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<9>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    components: Cow<'a, [CreateSectionComponent<'a>]>,
    accessory: CreateSectionAccessory<'a>,
}

impl<'a> CreateSection<'a> {
    // TODO: change type
    pub fn new(
        components: impl Into<Cow<'a, [CreateSectionComponent<'a>]>>,
        accessory: CreateSectionAccessory<'a>,
    ) -> Self {
        CreateSection {
            kind: StaticU8::<9>,
            components: components.into(),
            accessory,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateSectionComponent<'a> {
    TextDisplay(CreateTextDisplay<'a>),
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateTextDisplay<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<10>,
    content: Cow<'a, str>,
}

impl<'a> CreateTextDisplay<'a> {
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        CreateTextDisplay {
            kind: StaticU8::<10>,
            content: content.into(),
        }
    }

    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = content.into();
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateSectionAccessory<'a> {
    Thumbnail(CreateThumbnail<'a>),
    Button(CreateButton<'a>),
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateThumbnail<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<11>,
    media: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

impl<'a> CreateThumbnail<'a> {
    pub fn new(media: CreateUnfurledMediaItem<'a>) -> Self {
        CreateThumbnail {
            kind: StaticU8::<11>,
            media,
            description: None,
            spoiler: None,
        }
    }

    pub fn media(mut self, media: CreateUnfurledMediaItem<'a>) -> Self {
        self.media = media;
        self
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

#[derive(Clone, Debug, Serialize, Default)]
#[must_use]
pub struct CreateUnfurledMediaItem<'a> {
    url: Cow<'a, str>,
}

impl<'a> CreateUnfurledMediaItem<'a> {
    pub fn new(url: impl Into<Cow<'a, str>>) -> Self {
        CreateUnfurledMediaItem {
            url: url.into(),
        }
    }

    pub fn url(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.url = url.into();
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateMediaGallery<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<12>,
    items: Cow<'a, [CreateSectionComponent<'a>]>,
}

impl<'a> CreateMediaGallery<'a> {
    pub fn new(items: impl Into<Cow<'a, [CreateSectionComponent<'a>]>>) -> Self {
        CreateMediaGallery {
            kind: StaticU8::<12>,
            items: items.into(),
        }
    }

    pub fn items(mut self, items: impl Into<Cow<'a, [CreateSectionComponent<'a>]>>) -> Self {
        self.items = items.into();
        self
    }
}

#[derive(Clone, Debug, Serialize, Default)]
#[must_use]
pub struct CreateMediaGalleryItem<'a> {
    media: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateFile<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<13>,
    file: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

impl<'a> CreateFile<'a> {
    pub fn new(file: impl Into<CreateUnfurledMediaItem<'a>>) -> Self {
        CreateFile {
            kind: StaticU8::<13>,
            file: file.into(),
            spoiler: None,
        }
    }

    // Only supports attachment:// format.
    pub fn file(mut self, file: impl Into<CreateUnfurledMediaItem<'a>>) -> Self {
        self.file = file.into();
        self
    }

    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSeparator {
    #[serde(rename = "type")]
    kind: StaticU8<14>,
    divider: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing: Option<Spacing>,
}

impl CreateSeparator {
    pub fn new(divider: bool) -> Self {
        CreateSeparator {
            kind: StaticU8::<14>,
            divider,
            spacing: None,
        }
    }

    pub fn spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = Some(spacing);
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateContainer<'a> {
    #[serde(rename = "type")]
    kind: StaticU8<17>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accent_color: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
    components: Cow<'a, [CreateComponent<'a>]>,
}

impl<'a> CreateContainer<'a> {
    pub fn new(components: impl Into<Cow<'a, [CreateComponent<'a>]>>) -> Self {
        CreateContainer {
            kind: StaticU8::<17>,
            accent_color: None,
            spoiler: None,
            components: components.into(),
        }
    }

    pub fn accent_color(mut self, accent_color: Colour) -> Self {
        self.accent_color = Some(accent_color);
        self
    }

    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }

    pub fn components(mut self, components: impl Into<Cow<'a, [CreateComponent<'a>]>>) -> Self {
        self.components = components.into();
        self
    }
}

enum_number! {
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Spacing {
        Small = 1,
        Large = 2,
        _ => Unknown(u8),
    }
}

/// A builder for creating a button component in a message
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateButton<'a> {
    style: ButtonStyle,
    #[serde(rename = "type")]
    kind: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_id: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sku_id: Option<SkuId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(default)]
    disabled: bool,
}

impl<'a> CreateButton<'a> {
    /// Creates a link button to the given URL. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
    pub fn new_link(url: impl Into<Cow<'a, str>>) -> Self {
        Self {
            style: ButtonStyle::Unknown(5),
            kind: ComponentType::Button,
            url: Some(url.into()),
            custom_id: None,
            sku_id: None,
            label: None,
            emoji: None,
            disabled: false,
        }
    }

    /// Creates a new premium button associated with the given SKU.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
    pub fn new_premium(sku_id: impl Into<SkuId>) -> Self {
        Self {
            style: ButtonStyle::Unknown(6),
            kind: ComponentType::Button,
            url: None,
            custom_id: None,
            emoji: None,
            label: None,
            sku_id: Some(sku_id.into()),
            disabled: false,
        }
    }

    /// Creates a normal button with the given custom ID. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    pub fn new(custom_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: ComponentType::Button,
            style: ButtonStyle::Primary,
            url: None,
            custom_id: Some(custom_id.into()),
            sku_id: None,
            label: None,
            emoji: None,
            disabled: false,
        }
    }

    /// Sets the custom id of the button, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    ///
    /// Has no effect on link buttons and premium buttons.
    pub fn custom_id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        if self.url.is_none() {
            self.custom_id = Some(id.into());
        }

        self
    }

    /// Sets the style of this button.
    ///
    /// Has no effect on link buttons and premium buttons.
    pub fn style(mut self, new_style: ButtonStyle) -> Self {
        if self.url.is_none() {
            self.style = new_style;
        }

        self
    }

    /// Sets label of the button.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets emoji of the button.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<Button> for CreateButton<'_> {
    fn from(button: Button) -> Self {
        let (style, url, custom_id, sku_id) = match button.data {
            ButtonKind::Link {
                url,
            } => (ButtonStyle::Unknown(5), Some(url.into()), None, None),
            ButtonKind::Premium {
                sku_id,
            } => (ButtonStyle::Unknown(6), None, None, Some(sku_id)),
            ButtonKind::NonLink {
                custom_id,
                style,
            } => (style, None, Some(custom_id.into()), None),
        };

        Self {
            style,
            kind: ComponentType::Button,
            url,
            custom_id,
            sku_id,
            label: button.label.map(Into::into),
            emoji: button.emoji,
            disabled: button.disabled,
        }
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

/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure).
#[derive(Clone, Debug)]
pub enum CreateSelectMenuKind<'a> {
    String {
        options: Cow<'a, [CreateSelectMenuOption<'a>]>,
    },
    User {
        default_users: Option<Cow<'a, [UserId]>>,
    },
    Role {
        default_roles: Option<Cow<'a, [RoleId]>>,
    },
    Mentionable {
        default_users: Option<Cow<'a, [UserId]>>,
        default_roles: Option<Cow<'a, [RoleId]>>,
    },
    Channel {
        channel_types: Option<Cow<'a, [ChannelType]>>,
        default_channels: Option<Cow<'a, [GenericChannelId]>>,
    },
}

impl Serialize for CreateSelectMenuKind<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Json<'a> {
            #[serde(rename = "type")]
            kind: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<&'a [CreateSelectMenuOption<'a>]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_types: Option<&'a [ChannelType]>,
            #[serde(skip_serializing_if = "<[_]>::is_empty")]
            default_values: &'a [CreateSelectMenuDefault],
        }

        #[expect(clippy::ref_option)]
        fn map<'a>(
            values: &'a Option<Cow<'a, [impl Into<Mention> + Copy]>>,
        ) -> impl Iterator<Item = CreateSelectMenuDefault> + 'a {
            // Calling `.iter().flatten()` on the `Option` treats `None` like an empty vec
            values
                .as_ref()
                .map(|s| s.iter())
                .into_iter()
                .flatten()
                .map(|&i| CreateSelectMenuDefault(i.into()))
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
            default_values: &default_values,
        };

        json.serialize(serializer)
    }
}

/// A builder for creating a select menu component in a message
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenu<'a> {
    custom_id: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    #[serde(flatten)]
    kind: CreateSelectMenuKind<'a>,
}

impl<'a> CreateSelectMenu<'a> {
    /// Creates a builder with given custom id (a developer-defined identifier), and a list of
    /// options, leaving all other fields empty.
    pub fn new(custom_id: impl Into<Cow<'a, str>>, kind: CreateSelectMenuKind<'a>) -> Self {
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
    pub fn placeholder(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
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
}

/// A builder for creating an option of a select menu component in a message
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenuOption<'a> {
    label: Cow<'a, str>,
    value: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl<'a> CreateSelectMenuOption<'a> {
    /// Creates a select menu option with the given label and value, leaving all other fields
    /// empty.
    pub fn new(label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: None,
        }
    }

    /// Sets the label of this option, replacing the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the value of this option, replacing the current value as set in [`Self::new`].
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the description shown on this option.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
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
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-structure).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInputText<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    custom_id: Cow<'a, str>,
    style: InputTextStyle,
    label: Option<Cow<'a, str>>,
    min_length: Option<u16>,
    max_length: Option<u16>,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<Cow<'a, str>>,
}

impl<'a> CreateInputText<'a> {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(
        style: InputTextStyle,
        label: impl Into<Cow<'a, str>>,
        custom_id: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            style,
            label: Some(label.into()),
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: None,
            required: true,

            kind: ComponentType::InputText,
        }
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.style = kind;
        self
    }

    /// Sets the label of this input text. Replaces the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        self.custom_id = id.into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u16) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u16) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}
