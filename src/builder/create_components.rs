use serde::Serialize;

use crate::model::prelude::*;

/// A builder for creating a components action row in a message.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#component-object).
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
                url: url.into().into(),
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
                custom_id: custom_id.into().into(),
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
            *custom_id = id.into().into();
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
        self.0.label = Some(label.into().into());
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

/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure).
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
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure).
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
}

/// A builder for creating an option of a select menu component in a message
///
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure)
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
/// [Discord docs](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-structure).
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
            label: Some(label.into().into()),
            custom_id: custom_id.into().into(),

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
        self.0.label = Some(label.into().into());
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = id.into().into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.0.placeholder = Some(label.into().into());
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
        self.0.value = Some(value.into().into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.0.required = required;
        self
    }
}
