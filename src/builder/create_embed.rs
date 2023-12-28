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

#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create an embed in a message
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#embed-object)
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateEmbed {
    inner: Embed,
    fields: Vec<EmbedField>,
}

impl serde::Serialize for CreateEmbed {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let owned_self = self.clone();

        let mut embed = owned_self.inner;
        embed.fields = owned_self.fields.into();
        embed.serialize(serializer)
    }
}

impl CreateEmbed {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more information.
    pub fn author(mut self, author: CreateEmbedAuthor) -> Self {
        self.inner.author = Some(author.0);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`Self::colour`].
    #[inline]
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self {
        self.colour(colour)
    }

    /// Set the colour of the left-hand side of the embed.
    #[inline]
    pub fn colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.inner.colour = Some(colour.into());
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 4096 characters.
    #[inline]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.inner.description = Some(description.into().into());
        self
    }

    /// Set a field. Note that this will not overwrite other fields, and will add to them.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field name and 1024 in a
    /// field value.
    #[inline]
    pub fn field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        self.fields.push(EmbedField::new(name, value, inline));
        self
    }

    /// Adds multiple fields at once.
    ///
    /// This is sugar to reduce the need of calling [`Self::field`] manually multiple times.
    pub fn fields<N, V>(mut self, fields: impl IntoIterator<Item = (N, V, bool)>) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let fields =
            fields.into_iter().map(|(name, value, inline)| EmbedField::new(name, value, inline));
        self.fields.extend(fields);
        self
    }

    /// Set the footer of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedFooter`] for more information.
    pub fn footer(mut self, footer: CreateEmbedFooter) -> Self {
        self.inner.footer = Some(footer.0);
        self
    }

    /// Set the image associated with the embed.
    ///
    /// Refer [Discord Documentation](https://discord.com/developers/docs/reference#uploading-files)
    /// for rules on naming local attachments.
    #[inline]
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.inner.image = Some(EmbedImage {
            url: url.into().into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    /// Set the thumbnail of the embed.
    #[inline]
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.inner.thumbnail = Some(EmbedThumbnail {
            url: url.into().into(),
            proxy_url: None,
            height: None,
            width: None,
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
    #[inline]
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.inner.timestamp = Some(timestamp.into());
        self
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.inner.title = Some(title.into().into());
        self
    }

    /// Set the URL to direct to when clicking on the title.
    #[inline]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.inner.url = Some(url.into().into());
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
    #[inline]
    pub fn attachment(self, filename: impl Into<String>) -> Self {
        let mut filename = filename.into();
        filename.insert_str(0, "attachment://");
        self.image(filename)
    }

    #[cfg(feature = "http")]
    pub(super) fn check_length(&self) -> Result<()> {
        let mut length = 0;
        if let Some(ref author) = self.inner.author {
            length += author.name.chars().count();
        }

        if let Some(ref description) = self.inner.description {
            length += description.chars().count();
        }

        for field in &self.fields {
            length += field.name.chars().count();
            length += field.value.chars().count();
        }

        if let Some(ref footer) = self.inner.footer {
            length += footer.text.chars().count();
        }

        if let Some(ref title) = self.inner.title {
            length += title.chars().count();
        }

        super::check_overflow(length, crate::constants::EMBED_MAX_LENGTH)
            .map_err(|overflow| Error::Model(ModelError::EmbedTooLarge(overflow)))
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            inner: Embed {
                fields: FixedArray::new(),
                description: None,
                thumbnail: None,
                timestamp: None,
                kind: Some("rich".to_string().into()),
                author: None,
                colour: None,
                footer: None,
                image: None,
                title: None,
                url: None,
                video: None,
                provider: None,
            },
        }
    }
}

impl From<Embed> for CreateEmbed {
    fn from(mut embed: Embed) -> Self {
        Self {
            fields: std::mem::take(&mut embed.fields).into_vec(),
            inner: embed,
        }
    }
}

/// A builder to create the author data of an embed. See [`CreateEmbed::author`]
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbedAuthor(EmbedAuthor);

impl CreateEmbedAuthor {
    /// Creates an author object with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self(EmbedAuthor {
            name: name.into().into(),
            icon_url: None,
            url: None,
            // Has no builder method because I think this field is only relevant when receiving (?)
            proxy_icon_url: None,
        })
    }

    /// Set the author's name, replacing the current value as set in [`Self::new`].
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = name.into().into();
        self
    }

    /// Set the URL of the author's icon.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.0.icon_url = Some(icon_url.into().into());
        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into().into());
        self
    }
}

impl From<EmbedAuthor> for CreateEmbedAuthor {
    fn from(author: EmbedAuthor) -> Self {
        Self(author)
    }
}

#[cfg(feature = "model")]
impl From<User> for CreateEmbedAuthor {
    fn from(user: User) -> Self {
        let avatar_icon = user.face();
        Self::new(user.name).icon_url(avatar_icon)
    }
}

#[cfg(feature = "model")]
impl From<&User> for CreateEmbedAuthor {
    fn from(user: &User) -> Self {
        let avatar_icon = user.face();
        Self::new(user.name.clone()).icon_url(avatar_icon)
    }
}

/// A builder to create the footer data for an embed. See [`CreateEmbed::footer`]
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbedFooter(EmbedFooter);

impl CreateEmbedFooter {
    /// Creates a new footer object with the given text, leaving all other fields empty.
    pub fn new(text: impl Into<String>) -> Self {
        Self(EmbedFooter {
            text: text.into().into(),
            icon_url: None,
            // Has no builder method because I think this field is only relevant when receiving (?)
            proxy_icon_url: None,
        })
    }

    /// Set the footer's text, replacing the current value as set in [`Self::new`].
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.0.text = text.into().into();
        self
    }

    /// Set the icon URL's value.
    ///
    /// Refer [`CreateEmbed::image`] for rules on naming local attachments.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.0.icon_url = Some(icon_url.into().into());
        self
    }
}

impl From<EmbedFooter> for CreateEmbedFooter {
    fn from(footer: EmbedFooter) -> Self {
        Self(footer)
    }
}
