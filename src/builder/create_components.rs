use std::borrow::Cow;

use serde::Serialize;

use crate::model::prelude::*;

/// A builder for creating a components action row in a message.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#action-row).
#[derive(Clone, Debug)]
#[must_use]
pub enum CreateActionRow<'a> {
    Buttons(Cow<'a, [CreateButton<'a>]>),
    SelectMenu(CreateSelectMenu<'a>),
}

impl<'a> CreateActionRow<'a> {
    pub fn buttons(buttons: impl Into<Cow<'a, [CreateButton<'a>]>>) -> Self {
        Self::Buttons(buttons.into())
    }

    pub fn select_menu(select_menu: impl Into<CreateSelectMenu<'a>>) -> Self {
        Self::SelectMenu(select_menu.into())
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
        }

        map.end()
    }
}

/// A builder for creating components in a structured way.
///
/// This enum supports both V1 and V2 components, with the exception of `ActionRow`, which is a V1
/// component.
///
/// ## V2 Components and Message Flags
/// To send V2 components, you must set [`MessageFlags::IS_COMPONENTS_V2`].
///
/// ### Limitations
/// - The total number of components is limited to **40**..
/// - The maximum character count for text within components is **4000**.
/// - The ability to set the `content` and `embeds` field will be disabled
/// - No support for audio files
/// - No simple text preview for files
/// - No embeds for urls
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateComponent<'a> {
    /// Represents an action row component (V1).
    ///
    /// An action row is a container for other interactive components, such as buttons and select
    /// menus.
    ActionRow(CreateActionRow<'a>),
    /// Represents a section component (V2).
    ///
    /// A section is used to structure and group other components with an accessory.
    Section(CreateSection<'a>),
    /// Represents a text display component (V2).
    ///
    /// This component is used for displaying text within a message, separate from interactive
    /// elements.
    TextDisplay(CreateTextDisplay<'a>),
    /// Represents a media gallery component (V2).
    ///
    /// A media gallery allows embedding images, videos, or other media assets within a message.
    MediaGallery(CreateMediaGallery<'a>),
    /// Represents a file component (V2).
    ///
    /// This component is used for attaching and displaying files within a message.
    File(CreateFile<'a>),
    /// Represents a separator component (V2).
    ///
    /// A separator is used to visually divide sections within a message for better readability.
    Separator(CreateSeparator),
    /// Represents a container component (V2).
    ///
    /// A container is a flexible component that can hold multiple nested components.
    Container(CreateContainer<'a>),
    /// Represents a label component (V2).
    ///
    /// A label is used to hold other components in a modal.
    Label(CreateLabel<'a>),
}

/// A builder to create a section component, supports up to a max of **3** components with an
/// accessory.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSection<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    components: Cow<'a, [CreateSectionComponent<'a>]>,
    accessory: CreateSectionAccessory<'a>,
}

impl<'a> CreateSection<'a> {
    /// Creates a new builder with the specified components and accessory.
    ///
    /// Note: You may specify no more than **3** components or this will error on send.
    pub fn new(
        components: impl Into<Cow<'a, [CreateSectionComponent<'a>]>>,
        accessory: CreateSectionAccessory<'a>,
    ) -> Self {
        CreateSection {
            kind: ComponentType::Section,
            components: components.into(),
            accessory,
        }
    }

    /// Sets the components for the section. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: This will replace all existing components. Use [`Self::add_component()`] to add
    /// additional components.
    pub fn components(
        mut self,
        components: impl Into<Cow<'a, [CreateSectionComponent<'a>]>>,
    ) -> Self {
        self.components = components.into();
        self
    }

    /// Adds an additional component to this section.
    ///
    /// **Note**: This will add additional components. Use [`Self::components()`] to replace them.
    pub fn add_component(mut self, component: CreateSectionComponent<'a>) -> Self {
        self.components.to_mut().push(component);
        self
    }

    /// Sets the accessory for this section. Replaces the current value as set in [`Self::new`].
    pub fn accessory(mut self, accessory: CreateSectionAccessory<'a>) -> Self {
        self.accessory = accessory;
        self
    }
}

/// An enum of all valid section components.
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateSectionComponent<'a> {
    TextDisplay(CreateTextDisplay<'a>),
}

/// A builder to create a text display component.
#[derive(Clone, Debug, Serialize)]
pub struct CreateTextDisplay<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    content: Cow<'a, str>,
}

impl<'a> CreateTextDisplay<'a> {
    /// Creates a new text display component.
    ///
    /// Note: All components on a message shares the same **4000** character limit.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        CreateTextDisplay {
            kind: ComponentType::TextDisplay,
            content: content.into(),
        }
    }

    /// Sets the content of this text display component. Replaces the current value as set in
    /// [`Self::new`].
    ///
    /// Note: All components on a message shares the same **4000** character limit.
    #[must_use]
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = content.into();
        self
    }
}

/// An enum of all valid section accessories.
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateSectionAccessory<'a> {
    Thumbnail(CreateThumbnail<'a>),
    Button(CreateButton<'a>),
}

/// A builder to create a thumbnail for a section.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateThumbnail<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    media: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

impl<'a> CreateThumbnail<'a> {
    /// Creates a new thumbnail with a media item.
    pub fn new(media: CreateUnfurledMediaItem<'a>) -> Self {
        CreateThumbnail {
            kind: ComponentType::Thumbnail,
            media,
            description: None,
            spoiler: None,
        }
    }

    /// Sets the media item. Replaces the current value as set in [`Self::new`].
    pub fn media(mut self, media: CreateUnfurledMediaItem<'a>) -> Self {
        self.media = media;
        self
    }

    /// Sets the description for this thumbnail.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets if this thumbnail is spoilered.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

/// A builder to create a media item.
#[derive(Clone, Debug, Serialize, Default)]
#[must_use]
pub struct CreateUnfurledMediaItem<'a> {
    url: Cow<'a, str>,
}

impl<'a> CreateUnfurledMediaItem<'a> {
    /// Creates a new media item.
    pub fn new(url: impl Into<Cow<'a, str>>) -> Self {
        CreateUnfurledMediaItem {
            url: url.into(),
        }
    }

    /// Sets the url to this media item. Replaces the current value as set in [`Self::new`].
    pub fn url(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.url = url.into();
        self
    }
}

/// A builder to create a media gallery, a component that can contain multiple pieces of media.
///
/// Note: May contain up to **10** items.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateMediaGallery<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    items: Cow<'a, [CreateMediaGalleryItem<'a>]>,
}

impl<'a> CreateMediaGallery<'a> {
    /// Creates a new media gallery with up to **10** items.
    pub fn new(items: impl Into<Cow<'a, [CreateMediaGalleryItem<'a>]>>) -> Self {
        CreateMediaGallery {
            kind: ComponentType::MediaGallery,
            items: items.into(),
        }
    }

    /// Sets the items of the gallery. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: This will replace all existing items. Use [`Self::add_item()`] to add additional
    /// items
    pub fn items(mut self, items: impl Into<Cow<'a, [CreateMediaGalleryItem<'a>]>>) -> Self {
        self.items = items.into();
        self
    }

    /// Adds a singular item to the gallery.
    ///
    /// **Note**: This will add a singular item. Use [`Self::items()`] to replace all items.
    pub fn add_item(mut self, item: CreateMediaGalleryItem<'a>) -> Self {
        self.items.to_mut().push(item);
        self
    }
}

/// Builder to create individual media gallery items.
#[derive(Clone, Debug, Serialize, Default)]
#[must_use]
pub struct CreateMediaGalleryItem<'a> {
    media: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

impl<'a> CreateMediaGalleryItem<'a> {
    /// Create a new media gallery item.
    pub fn new(media: CreateUnfurledMediaItem<'a>) -> Self {
        CreateMediaGalleryItem {
            media,
            description: None,
            spoiler: None,
        }
    }

    /// Sets the internal media item. Replaces the current value as set in [`Self::new`].
    pub fn media(mut self, media: CreateUnfurledMediaItem<'a>) -> Self {
        self.media = media;
        self
    }

    /// Sets the description of this item in the gallery.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specifies if this piece of media should be spoilered.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

/// A builder for specifying a file to be uploaded.
///
/// This builder **only** supports the `attachment://filename.extension` format.
/// This means that you must first upload the file as an attachment in the message
/// and then reference it using this format.
///
/// # Usage
///
/// 1. Upload the file as an attachment.
/// 2. Use the `attachment://` scheme to reference the uploaded file.
///
/// ## Example
///
/// If you upload an attachment to the message that is called "example.txt", you set the url of the
/// item to "attachment://example.txt".
///
/// For more details on naming and rules for attachments,
/// refer to the [Discord Documentation](https://discord.com/developers/docs/reference#uploading-files).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateFile<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    file: CreateUnfurledMediaItem<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
}

impl<'a> CreateFile<'a> {
    /// Create a new builder for the file component. Refer to this builders documentation for
    /// limits.
    pub fn new(file: CreateUnfurledMediaItem<'a>) -> Self {
        CreateFile {
            kind: ComponentType::File,
            file,
            spoiler: None,
        }
    }

    // Only supports `attachment://filename.extension` format, refer to this builders documentation
    // for more details. Replaces the current value as set in [`Self::new`].
    pub fn file(mut self, file: CreateUnfurledMediaItem<'a>) -> Self {
        self.file = file;
        self
    }

    ///  Sets if this file should be spoilered or not.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }
}

/// A builder for creating a separator.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSeparator {
    #[serde(rename = "type")]
    kind: ComponentType,
    divider: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing: Option<Spacing>,
}

impl CreateSeparator {
    /// Creates a new separator, with or without a divider.
    pub fn new(divider: bool) -> Self {
        CreateSeparator {
            kind: ComponentType::Separator,
            divider,
            spacing: None,
        }
    }

    /// Sets if this separator should have a divider or not. Replaces the current value as set in
    /// [`Self::new`].
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// Sets the spacing of this separator.
    pub fn spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = Some(spacing);
        self
    }
}

/// A builder to create a container, which acts similarly to embeds.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateContainer<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    accent_color: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spoiler: Option<bool>,
    components: Cow<'a, [CreateContainerComponent<'a>]>,
}

impl<'a> CreateContainer<'a> {
    /// Create a new container, with an array of components inside.
    pub fn new(components: impl Into<Cow<'a, [CreateContainerComponent<'a>]>>) -> Self {
        CreateContainer {
            kind: ComponentType::Container,
            accent_color: None,
            spoiler: None,
            components: components.into(),
        }
    }

    // Set the colour of the left-hand side of the container.
    pub fn accent_colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.accent_color = Some(colour.into());
        self
    }

    /// Set the colour of the left-hand side of the container.
    ///
    /// This is an alias of [`Self::accent_colour`].
    pub fn accent_color<C: Into<Colour>>(self, colour: C) -> Self {
        self.accent_colour(colour)
    }

    /// Sets if this container is spoilered or not.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = Some(spoiler);
        self
    }

    /// Sets the components of this container. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: This will replace all existing components. Use [`Self::add_component()`] to add
    /// additional components.
    pub fn components(
        mut self,
        components: impl Into<Cow<'a, [CreateContainerComponent<'a>]>>,
    ) -> Self {
        self.components = components.into();
        self
    }

    /// Adds an additional component to this container.
    ///
    /// **Note**: This will add additional components. Use [`Self::components()`] to replace them.
    pub fn add_component(mut self, component: CreateContainerComponent<'a>) -> Self {
        self.components.to_mut().push(component);
        self
    }
}

/// An enum of all valid container components.
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
pub enum CreateContainerComponent<'a> {
    ActionRow(CreateActionRow<'a>),
    Section(CreateSection<'a>),
    TextDisplay(CreateTextDisplay<'a>),
    MediaGallery(CreateMediaGallery<'a>),
    File(CreateFile<'a>),
    Separator(CreateSeparator),
}

/// A builder for creating a label that can hold an [`InputText`] or [`SelectMenu`].
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#label).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateLabel<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    component: CreateLabelComponent<'a>,
}

impl<'a> CreateLabel<'a> {
    /// Create a select menu with a specific label.
    pub fn select_menu(label: impl Into<Cow<'a, str>>, select_menu: CreateSelectMenu<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::SelectMenu(select_menu),
        }
    }

    /// Create a text input with a specific label.
    pub fn input_text(label: impl Into<Cow<'a, str>>, input_text: CreateInputText<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::InputText(input_text),
        }
    }

    /// Create a file upload with a specific label.
    pub fn file_upload(label: impl Into<Cow<'a, str>>, file_upload: CreateFileUpload<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::FileUpload(file_upload),
        }
    }

    /// Sets the description of this component, which will display underneath the label text.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// An enum of all valid label components.
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
enum CreateLabelComponent<'a> {
    SelectMenu(CreateSelectMenu<'a>),
    InputText(CreateInputText<'a>),
    FileUpload(CreateFileUpload<'a>),
}

/// A builder for creating a file upload in a modal.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file-upload).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateFileUpload<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    custom_id: Cow<'a, str>,
    min_values: u8,
    max_values: u8,
    required: bool,
}

impl<'a> CreateFileUpload<'a> {
    /// Creates a builder with the given custom id.
    pub fn new(custom_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: ComponentType::FileUpload,
            custom_id: custom_id.into(),
            min_values: 1,
            max_values: 1,
            required: true,
        }
    }

    /// The minimum number of files that must be uploaded. Must be a number from 0 through 10, and
    /// defaults to 1.
    pub fn min_values(mut self, min_values: u8) -> Self {
        self.min_values = min_values;
        self
    }

    /// The maximum number of files that can be uploaded. Defaults to 1, but can be at most 10.
    pub fn max_values(mut self, max_values: u8) -> Self {
        self.max_values = max_values;
        self
    }

    // Whether the file upload is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
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

/// [Discord docs](https://discord.com/developers/docs/components/reference#component-object-component-types).
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#component-object-component-types).
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#string-select-select-option-structure)
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
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-input).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInputText<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    custom_id: Cow<'a, str>,
    style: InputTextStyle,
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
    pub fn new(style: InputTextStyle, custom_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            style,
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
