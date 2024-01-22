//! Developer note:
//!
//! This is a set of embed builders for rich embeds.
//!
//! These are used in the [`ChannelId::send_message`] and [`ExecuteWebhook::embeds`] methods, both
//! as part of builders.
//!
//! The only builder that should be exposed is [`CreateEmbed`]. The rest of these have no real
//! reason for being exposed, but are for completeness' sake.
//!
//! Documentation for embeds can be found [here].
//!
//! [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
//! [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
//! [here]: https://discord.com/developers/docs/resources/channel#embed-object

use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create an embed in a message
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#embed-object)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbed<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<Cow<'a, str>>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<Timestamp>,
    #[serde(rename = "color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<CreateEmbedFooter<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<CreateEmbedImage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<CreateEmbedImage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<CreateEmbedAuthor<'a>>,
    /// No point using a Cow slice, as there is no set_fields method
    /// and CreateEmbedField is not public.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    fields: Vec<CreateEmbedField<'a>>,
}

impl<'a> CreateEmbed<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more information.
    pub fn author(mut self, author: CreateEmbedAuthor<'a>) -> Self {
        self.author = Some(author);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`Self::colour`].
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self {
        self.colour(colour)
    }

    /// Set the colour of the left-hand side of the embed.
    pub fn colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.colour = Some(colour.into());
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 4096 characters.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a field. Note that this will not overwrite other fields, and will add to them.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field name and 1024 in a
    /// field value.
    pub fn field(
        mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
        inline: bool,
    ) -> Self {
        self.fields.push(CreateEmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    /// Adds multiple fields at once.
    ///
    /// This is sugar to reduce the need of calling [`Self::field`] manually multiple times.
    pub fn fields<N, V>(mut self, fields: impl IntoIterator<Item = (N, V, bool)>) -> Self
    where
        N: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        let fields = fields.into_iter().map(|(name, value, inline)| CreateEmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });

        self.fields.extend(fields);
        self
    }

    /// Set the footer of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedFooter`] for more information.
    pub fn footer(mut self, footer: CreateEmbedFooter<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set the image associated with the embed.
    ///
    /// Refer [Discord Documentation](https://discord.com/developers/docs/reference#uploading-files)
    /// for rules on naming local attachments.
    pub fn image(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.image = Some(CreateEmbedImage {
            url: url.into(),
        });
        self
    }

    /// Set the thumbnail of the embed.
    pub fn thumbnail(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.thumbnail = Some(CreateEmbedImage {
            url: url.into(),
        });
        self
    }

    /// Set the timestamp.
    ///
    /// See the documentation of [`Timestamp`] for more information.
    ///
    /// # Examples
    ///
    /// Passing a string timestamp:
    ///
    /// ```rust
    /// # use serenity::builder::CreateEmbed;
    /// # use serenity::model::Timestamp;
    /// let timestamp: Timestamp = "2004-06-08T16:04:23Z".parse().expect("Invalid timestamp!");
    /// let embed = CreateEmbed::new().title("hello").timestamp(timestamp);
    /// ```
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set the title of the embed.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the URL to direct to when clicking on the title.
    pub fn url(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Same as calling [`Self::image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
    ///
    /// Refer [`Self::image`] for rules on naming local attachments.
    ///
    /// [`ChannelId::send_files`]: crate::model::id::ChannelId::send_files
    pub fn attachment(self, filename: impl Into<String>) -> Self {
        let mut filename = filename.into();
        filename.insert_str(0, "attachment://");
        self.image(filename)
    }

    #[cfg(feature = "http")]
    pub(super) fn get_length(&self) -> usize {
        let mut length = 0;
        if let Some(author) = &self.author {
            length += author.name.chars().count();
        }

        if let Some(description) = &self.description {
            length += description.chars().count();
        }

        for field in &self.fields {
            length += field.name.chars().count();
            length += field.value.chars().count();
        }

        if let Some(footer) = &self.footer {
            length += footer.text.chars().count();
        }

        if let Some(title) = &self.title {
            length += title.chars().count();
        }

        length
    }
}

impl Default for CreateEmbed<'_> {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            description: None,
            thumbnail: None,
            timestamp: None,
            kind: Some("rich"),
            author: None,
            colour: None,
            footer: None,
            image: None,
            title: None,
            url: None,
        }
    }
}

impl From<Embed> for CreateEmbed<'_> {
    fn from(embed: Embed) -> Self {
        Self {
            fields: embed.fields.into_iter().map(Into::into).collect(),
            description: embed.description.map(FixedString::into_string).map(Into::into),
            thumbnail: embed.thumbnail.map(Into::into),
            timestamp: embed.timestamp,
            kind: Some("rich"),
            author: embed.author.map(Into::into),
            colour: embed.colour,
            footer: embed.footer.map(Into::into),
            image: embed.image.map(Into::into),
            title: embed.title.map(FixedString::into_string).map(Into::into),
            url: embed.url.map(FixedString::into_string).map(Into::into),
        }
    }
}

/// A builder to create the author data of an embed. See [`CreateEmbed::author`]
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbedAuthor<'a> {
    name: Cow<'a, str>,
    url: Option<Cow<'a, str>>,
    icon_url: Option<Cow<'a, str>>,
}

impl<'a> CreateEmbedAuthor<'a> {
    /// Creates an author object with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            icon_url: None,
            url: None,
        }
    }

    /// Set the author's name, replacing the current value as set in [`Self::new`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the URL of the author's icon.
    pub fn icon_url(mut self, icon_url: impl Into<Cow<'a, str>>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: impl Into<Cow<'a, str>>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl From<EmbedAuthor> for CreateEmbedAuthor<'_> {
    fn from(author: EmbedAuthor) -> Self {
        Self {
            name: author.name.into_string().into(),
            url: author.url.map(|f| f.into_string().into()),
            icon_url: author.icon_url.map(|f| f.into_string().into()),
        }
    }
}

#[cfg(feature = "model")]
impl From<User> for CreateEmbedAuthor<'_> {
    fn from(user: User) -> Self {
        let avatar_icon = user.face();
        Self::new(user.name).icon_url(avatar_icon)
    }
}

#[cfg(feature = "model")]
impl From<&User> for CreateEmbedAuthor<'_> {
    fn from(user: &User) -> Self {
        let avatar_icon = user.face();
        Self::new(user.name.clone()).icon_url(avatar_icon)
    }
}

/// A builder to create the footer data for an embed. See [`CreateEmbed::footer`]
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbedFooter<'a> {
    text: Cow<'a, str>,
    icon_url: Option<Cow<'a, str>>,
}

impl<'a> CreateEmbedFooter<'a> {
    /// Creates a new footer object with the given text, leaving all other fields empty.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            icon_url: None,
        }
    }

    /// Set the footer's text, replacing the current value as set in [`Self::new`].
    pub fn text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the icon URL's value.
    ///
    /// Refer [`CreateEmbed::image`] for rules on naming local attachments.
    pub fn icon_url(mut self, icon_url: impl Into<Cow<'a, str>>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }
}

impl From<EmbedFooter> for CreateEmbedFooter<'_> {
    fn from(footer: EmbedFooter) -> Self {
        Self {
            text: footer.text.into_string().into(),
            icon_url: footer.icon_url.map(|f| f.into_string().into()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct CreateEmbedField<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
    inline: bool,
}

impl<'a> From<&'a EmbedField> for CreateEmbedField<'a> {
    fn from(field: &'a EmbedField) -> Self {
        Self {
            name: field.name.as_str().into(),
            value: field.value.as_str().into(),
            inline: field.inline,
        }
    }
}

impl From<EmbedField> for CreateEmbedField<'_> {
    fn from(field: EmbedField) -> Self {
        Self {
            name: field.name.into_string().into(),
            value: field.value.into_string().into(),
            inline: field.inline,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct CreateEmbedImage<'a> {
    url: Cow<'a, str>,
}

impl From<EmbedImage> for CreateEmbedImage<'_> {
    fn from(field: EmbedImage) -> Self {
        Self {
            url: field.url.into_string().into(),
        }
    }
}

impl From<EmbedThumbnail> for CreateEmbedImage<'_> {
    fn from(field: EmbedThumbnail) -> Self {
        Self {
            url: field.url.into_string().into(),
        }
    }
}
