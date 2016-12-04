//! Developer note:
//!
//! This is a set of embed builders for rich embeds.
//!
//! These are used in the [`Context::send_message`] and
//! [`ExecuteWebhook::embeds`] methods, both as part of builders.
//!
//! The only builder that should be exposed is [`CreateEmbed`]. The rest of
//! these have no real reason for being exposed, but are for completeness' sake.
//!
//! Documentation for embeds can be found [here].
//!
//! [`Context::send_message`]: ../../client/struct.Context.html#method.send_message
//! [`CreateEmbed`]: struct.CreateEmbed.html
//! [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
//! [here]: https://discordapp.com/developers/docs/resources/channel#embed-object

use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::collections::BTreeMap;
use std::default::Default;
use ::utils::Colour;

/// A builder to create a fake [`Embed`] object, for use with the
/// [`Context::send_message`] and [`ExecuteWebhook::embeds`] methods.
///
/// # Examples
///
/// Refer to the documentation for [`Context::send_message`] for a very in-depth
/// example on how to use this.
///
/// [`Context::send_message`]: ../../client/struct.Context.html#method.send_message
/// [`Embed`]: ../../model/struct.Embed.html
/// [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
pub struct CreateEmbed(pub BTreeMap<String, Value>);

impl CreateEmbed {
    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more
    /// information.
    ///
    /// [`CreateEmbedAuthor`]: struct.CreateEmbedAuthor.html
    pub fn author<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedAuthor) -> CreateEmbedAuthor {
        let author = f(CreateEmbedAuthor::default()).0.build();

        self.0.insert("author".to_owned(), author);

        CreateEmbed(self.0)
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: #method.colour
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self {
        self.colour(colour.into())
    }

    /// Set the colour of the left-hand side of the embed.
    pub fn colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.0.insert("color".to_owned(), Value::U64(colour.into().value as u64));

        CreateEmbed(self.0)
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 2048 characters.
    pub fn description(mut self, description: &str) -> Self {
        self.0.insert("description".to_owned(), Value::String(description.to_owned()));

        CreateEmbed(self.0)
    }

    /// Set a field. Note that this will not overwrite other fields, and will
    /// add to them.
    ///
    /// Refer to the documentation for [`CreateEmbedField`] for more
    /// information.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field
    /// name and 1024 in a field value and a field is inline by default.
    ///
    /// [`CreateEmbedField`]: struct.CreateEmbedField.html
    pub fn field<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedField) -> CreateEmbedField {
        let field = f(CreateEmbedField::default()).0.build();

        {
            let key = "fields".to_owned();

            let entry = self.0.remove(&key).unwrap_or_else(|| Value::Array(vec![]));
            let mut arr = match entry {
                Value::Array(inner) => inner,
                _ => {
                    // The type of `entry` should always be a `Value::Array`.
                    //
                    // Theoretically this never happens, but you never know.
                    //
                    // In the event that it does, just return the current value.
                    return CreateEmbed(self.0);
                },
            };
            arr.push(field);

            self.0.insert("fields".to_owned(), Value::Array(arr));
        }

        CreateEmbed(self.0)
    }

    /// Set the footer of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedFooter`] for more
    /// information.
    ///
    /// [`CreateEmbedFooter`]: struct.CreateEmbedFooter.html
    pub fn footer<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedFooter) -> CreateEmbedFooter {
        let footer = f(CreateEmbedFooter::default()).0.build();

        self.0.insert("footer".to_owned(), footer);

        CreateEmbed(self.0)
    }

    /// Set the image associated with the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedImage`] for more
    /// information.
    ///
    /// [`CreateEmbedImage`]: struct.CreateEmbedImage.html
    pub fn image<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedImage) -> CreateEmbedImage {
        let image = f(CreateEmbedImage::default()).0.build();

        self.0.insert("image".to_owned(), image);

        CreateEmbed(self.0)
    }

    /// Set the thumbnail of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedThumbnail`] for more
    /// information.
    ///
    /// [`CreateEmbedThumbnail`]: struct.CreateEmbedThumbnail.html
    pub fn thumbnail<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedThumbnail) -> CreateEmbedThumbnail {
        let thumbnail = f(CreateEmbedThumbnail::default()).0.build();

        self.0.insert("thumbnail".to_owned(), thumbnail);

        CreateEmbed(self.0)
    }

    /// Set the timestamp.
    pub fn timestamp(mut self, timestamp: &str) -> Self {
        self.0.insert("timestamp".to_owned(), Value::String(timestamp.to_owned()));

        CreateEmbed(self.0)
    }

    /// Set the title of the embed.
    pub fn title(mut self, title: &str) -> Self {
        self.0.insert("title".to_owned(), Value::String(title.to_owned()));

        CreateEmbed(self.0)
    }

    /// Set the URL to direct to when clicking on the title.
    pub fn url(mut self, url: &str) -> Self {
        self.0.insert("url".to_owned(), Value::String(url.to_owned()));

        CreateEmbed(self.0)
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> CreateEmbed {
        let mut map = BTreeMap::new();
        map.insert("type".to_owned(), Value::String("rich".to_owned()));

        CreateEmbed(map)
    }
}

/// A builder to create a fake [`Embed`] object's author, for use with the
/// [`CreateEmbed::author`] method.
///
/// Requires that you specify a [`name`].
///
/// [`Embed`]: ../../model/struct.Embed.html
/// [`CreateEmbed::author`]: struct.CreateEmbed.html#method.author
/// [`name`]: #method.name
pub struct CreateEmbedAuthor(pub ObjectBuilder);

impl CreateEmbedAuthor {
    /// Set the URL of the author's icon.
    pub fn icon_url(self, icon_url: &str) -> Self {
        CreateEmbedAuthor(self.0.insert("icon_url", icon_url))
    }

    /// Set the author's name.
    pub fn name(self, name: &str) -> Self {
        CreateEmbedAuthor(self.0.insert("name", name))
    }

    /// Set the author's URL.
    pub fn url(self, url: &str) -> Self {
        CreateEmbedAuthor(self.0.insert("url", url))
    }
}

impl Default for CreateEmbedAuthor {
    /// Creates a builder with no default values.
    fn default() -> CreateEmbedAuthor {
        CreateEmbedAuthor(ObjectBuilder::new())
    }
}

/// A builder to create a fake [`Embed`] object's field, for use with the
/// [`CreateEmbed::field`] method.
///
/// This does not require any field be set. `inline` is set to `true` by
/// default.
///
/// [`Embed`]: ../../model/struct.Embed.html
/// [`CreateEmbed::field`]: struct.CreateEmbed.html#method.field
pub struct CreateEmbedField(pub ObjectBuilder);

impl CreateEmbedField {
    /// Set whether the field is inlined. Set to true by default.
    pub fn inline(self, inline: bool) -> Self {
        CreateEmbedField(self.0.insert("inline", inline))
    }

    /// Set the field's name. It can't be longer than 256 characters.
    pub fn name(self, name: &str) -> Self {
        CreateEmbedField(self.0.insert("name", name))
    }

    /// Set the field's value. It can't be longer than 1024 characters.
    pub fn value(self, value: &str) -> Self {
        CreateEmbedField(self.0.insert("value", value))
    }
}

impl Default for CreateEmbedField {
    /// Creates a builder with default values, setting the value of `inline` to
    /// `true`.
    fn default() -> CreateEmbedField {
        CreateEmbedField(ObjectBuilder::new().insert("inline", true))
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the
/// [`CreateEmbed::footer`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: ../../model/struct.Embed.html
/// [`CreateEmbed::footer`]: struct.CreateEmbed.html#method.footer
pub struct CreateEmbedFooter(pub ObjectBuilder);

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(self, icon_url: &str) -> Self {
        CreateEmbedFooter(self.0.insert("icon_url", icon_url))
    }

    /// Set the footer's text.
    pub fn text(self, text: &str) -> Self {
        CreateEmbedFooter(self.0.insert("text", text))
    }
}

impl Default for CreateEmbedFooter {
    /// Creates a builder with no default values.
    fn default() -> CreateEmbedFooter {
        CreateEmbedFooter(ObjectBuilder::new())
    }
}

/// A builder to create a fake [`Embed`] object's image, for use with the
/// [`CreateEmbed::image`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: ../../model/struct.Embed.html
/// [`CreateEmbed::image`]: struct.CreateEmbed.html#method.image
pub struct CreateEmbedImage(pub ObjectBuilder);

impl CreateEmbedImage {
    /// Set the display height of the image.
    pub fn height(self, height: u64) -> Self {
        CreateEmbedImage(self.0.insert("height", height))
    }

    /// Set the image's URL. This only supports HTTP(S).
    pub fn url(self, url: &str) -> Self {
        CreateEmbedImage(self.0.insert("url", url))
    }

    /// Set the display width of the image.
    pub fn width(self, width: u64) -> Self {
        CreateEmbedImage(self.0.insert("widht", width))
    }
}

impl Default for CreateEmbedImage {
    /// Creates a builder with no default values.
    fn default() -> CreateEmbedImage {
        CreateEmbedImage(ObjectBuilder::new())
    }
}

/// A builder to create a fake [`Embed`] object's thumbnail, for use with the
/// [`CreateEmbed::thumbnail`] method.
///
/// Requires that you specify a [`url`].
///
/// [`Embed`]: ../../model/struct.Embed.html
/// [`CreateEmbed::thumbnail`]: struct.CreateEmbed.html#method.thumbnail
/// [`url`]: #method.url
pub struct CreateEmbedThumbnail(pub ObjectBuilder);

impl CreateEmbedThumbnail {
    /// Set the height of the thumbnail, in pixels.
    pub fn height(self, height: u64) -> Self {
        CreateEmbedThumbnail(self.0.insert("height", height))
    }

    /// Set the URL of the thumbnail. This only supports HTTP(S).
    ///
    /// _Must_ be specified.
    pub fn url(self, url: &str) -> Self {
        CreateEmbedThumbnail(self.0.insert("url", url))
    }

    /// Set the width of the thumbnail, in pixels.
    pub fn width(self, width: u64) -> Self {
        CreateEmbedThumbnail(self.0.insert("width", width))
    }
}

impl Default for CreateEmbedThumbnail {
    /// Creates a builder with no default values.
    fn default() -> CreateEmbedThumbnail {
        CreateEmbedThumbnail(ObjectBuilder::new())
    }
}
