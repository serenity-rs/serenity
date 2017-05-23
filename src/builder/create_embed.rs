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
//! [`Context::send_message`]: ../client/struct.Context.html#method.send_message
//! [`CreateEmbed`]: struct.CreateEmbed.html
//! [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
//! [here]: https://discordapp.com/developers/docs/resources/channel#embed-object

use serde_json::Value;
use std::default::Default;
use time::Tm;
use ::internal::prelude::*;
use ::model::Embed;

#[cfg(feature="utils")]
use ::utils::Colour;

/// A builder to create a fake [`Embed`] object, for use with the
/// [`Context::send_message`] and [`ExecuteWebhook::embeds`] methods.
///
/// # Examples
///
/// Refer to the documentation for [`Context::send_message`] for a very in-depth
/// example on how to use this.
///
/// [`Context::send_message`]: ../client/struct.Context.html#method.send_message
/// [`Embed`]: ../model/struct.Embed.html
/// [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
#[derive(Clone, Debug)]
pub struct CreateEmbed(pub Map<String, Value>);

impl CreateEmbed {
    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more
    /// information.
    ///
    /// [`CreateEmbedAuthor`]: struct.CreateEmbedAuthor.html
    pub fn author<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedAuthor) -> CreateEmbedAuthor {
        let author = f(CreateEmbedAuthor::default()).0;

        self.0.insert("author".to_owned(), Value::Object(author));

        CreateEmbed(self.0)
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: #method.colour
    #[cfg(feature="utils")]
    #[inline]
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self {
        self.colour(colour.into())
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(feature="utils")]
    pub fn colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.0.insert("color".to_owned(), Value::Number(Number::from(colour.into().0 as u64)));

        CreateEmbed(self.0)
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: #method.colour
    #[cfg(not(feature="utils"))]
    #[inline]
    pub fn color(self, colour: u32) -> Self {
        self.colour(colour)
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(not(feature="utils"))]
    pub fn colour(mut self, colour: u32) -> Self {
        self.0.insert("color".to_owned(), Value::Number(Number::from(colour)));

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
        let field = f(CreateEmbedField::default()).0;

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
            arr.push(Value::Object(field));

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
        let footer = f(CreateEmbedFooter::default()).0;

        self.0.insert("footer".to_owned(), Value::Object(footer));

        CreateEmbed(self.0)
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    pub fn image(mut self, url: &str) -> Self {
        let image = json!({
            "url": url.to_owned()
        });

        self.0.insert("image".to_owned(), image);

        CreateEmbed(self.0)
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    pub fn thumbnail(mut self, url: &str) -> Self {
        let thumbnail = json!({
            "url": url.to_owned(),
        });

        self.0.insert("thumbnail".to_owned(), thumbnail);

        CreateEmbed(self.0)
    }

    /// Set the timestamp.
    ///
    /// **Note**: This timestamp must be in ISO-8601 format. It must also be
    /// in UTC format.
    ///
    /// # Examples
    ///
    /// You may pass a direct string:
    ///
    /// - `2017-01-03T23:00:00`
    /// - `2004-06-08T16:04:23`
    /// - `2004-06-08T16:04:23`
    ///
    /// Or a `time::Tm`:
    ///
    /// ```rust,ignore
    /// extern crate time;
    ///
    /// let now = time::now();
    ///
    /// embed = embed.timestamp(now);
    /// // ...
    /// ```
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.0.insert("timestamp".to_owned(), Value::String(timestamp.into().ts));

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
        let mut map = Map::new();
        map.insert("type".to_owned(), Value::String("rich".to_owned()));

        CreateEmbed(map)
    }
}

impl From<Embed> for CreateEmbed {
    /// Converts the fields of an embed into the values for a new embed builder.
    ///
    /// Some values - such as Proxy URLs - are not preserved.
    fn from(embed: Embed) -> CreateEmbed {
        let mut b = CreateEmbed::default()
            .colour(embed.colour);

        if let Some(author) = embed.author {
            b = b.author(move |mut a| {
                a = a.name(&author.name);

                if let Some(icon_url) = author.icon_url {
                    a = a.icon_url(&icon_url);
                }

                if let Some(url) = author.url {
                    a = a.url(&url);
                }

                a
            });
        }

        if let Some(description) = embed.description {
            b = b.description(&description);
        }

        for field in embed.fields {
            b = b.field(move |f| f
                .inline(field.inline)
                .name(&field.name)
                .value(&field.value));
        }

        if let Some(image) = embed.image {
            b = b.image(&image.url);
        }

        if let Some(timestamp) = embed.timestamp {
            b = b.timestamp(timestamp);
        }

        if let Some(thumbnail) = embed.thumbnail {
            b = b.thumbnail(&thumbnail.url);
        }

        if let Some(url) = embed.url {
            b = b.url(&url);
        }

        if let Some(title) = embed.title {
            b = b.title(&title);
        }

        b
    }
}

/// A builder to create a fake [`Embed`] object's author, for use with the
/// [`CreateEmbed::author`] method.
///
/// Requires that you specify a [`name`].
///
/// [`Embed`]: ../model/struct.Embed.html
/// [`CreateEmbed::author`]: struct.CreateEmbed.html#method.author
/// [`name`]: #method.name
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedAuthor(pub Map<String, Value>);

impl CreateEmbedAuthor {
    /// Set the URL of the author's icon.
    pub fn icon_url(mut self, icon_url: &str) -> Self {
        self.0.insert("icon_url".to_owned(), Value::String(icon_url.to_owned()));

        self
    }

    /// Set the author's name.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name".to_owned(), Value::String(name.to_owned()));

        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: &str) -> Self {
        self.0.insert("url".to_owned(), Value::String(url.to_owned()));

        self
    }
}

/// A builder to create a fake [`Embed`] object's field, for use with the
/// [`CreateEmbed::field`] method.
///
/// This does not require any field be set. `inline` is set to `true` by
/// default.
///
/// [`Embed`]: ../model/struct.Embed.html
/// [`CreateEmbed::field`]: struct.CreateEmbed.html#method.field
#[derive(Clone, Debug)]
pub struct CreateEmbedField(pub Map<String, Value>);

impl CreateEmbedField {
    /// Set whether the field is inlined. Set to true by default.
    pub fn inline(mut self, inline: bool) -> Self {
        self.0.insert("inline".to_owned(), Value::Bool(inline));

        self
    }

    /// Set the field's name. It can't be longer than 256 characters.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name".to_owned(), Value::String(name.to_owned()));

        self
    }

    /// Set the field's value. It can't be longer than 1024 characters.
    pub fn value(mut self, value: &str) -> Self {
        self.0.insert("value".to_owned(), Value::String(value.to_owned()));

        self
    }
}

impl Default for CreateEmbedField {
    /// Creates a builder with default values, setting the value of `inline` to
    /// `true`.
    fn default() -> CreateEmbedField {
        let mut map = Map::new();
        map.insert("inline".to_owned(), Value::Bool(true));

        CreateEmbedField(map)
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the
/// [`CreateEmbed::footer`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: ../model/struct.Embed.html
/// [`CreateEmbed::footer`]: struct.CreateEmbed.html#method.footer
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedFooter(pub Map<String, Value>);

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(mut self, icon_url: &str) -> Self {
        self.0.insert("icon_url".to_owned(), Value::String(icon_url.to_owned()));

        self
    }

    /// Set the footer's text.
    pub fn text(mut self, text: &str) -> Self {
        self.0.insert("text".to_owned(), Value::String(text.to_owned()));

        self
    }
}

#[derive(Clone, Debug)]
pub struct Timestamp {
    pub ts: String,
}

impl From<String> for Timestamp {
    fn from(ts: String) -> Self {
        Timestamp {
            ts: ts,
        }
    }
}

impl From<Tm> for Timestamp {
    fn from(tm: Tm) -> Self {
        Timestamp {
            ts: tm.to_utc().rfc3339().to_string(),
        }
    }
}
