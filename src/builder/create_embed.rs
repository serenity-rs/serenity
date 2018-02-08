//! Developer note:
//!
//! This is a set of embed builders for rich embeds.
//!
//! These are used in the [`ChannelId::send_message`] and
//! [`ExecuteWebhook::embeds`] methods, both as part of builders.
//!
//! The only builder that should be exposed is [`CreateEmbed`]. The rest of
//! these have no real reason for being exposed, but are for completeness' sake.
//!
//! Documentation for embeds can be found [here].
//!
//! [`ChannelId::send_message`]: ../model/id/struct.ChannelId.html#method.send_message
//! [`CreateEmbed`]: struct.CreateEmbed.html
//! [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
//! [here]: https://discordapp.com/developers/docs/resources/channel#embed-object

use chrono::{DateTime, TimeZone};
use internal::prelude::*;
use model::channel::Embed;
use serde_json::Value;
use std::default::Default;
use std::fmt::Display;
use utils;
use utils::VecMap;

#[cfg(feature = "utils")]
use utils::Colour;

/// A builder to create a fake [`Embed`] object, for use with the
/// [`ChannelId::send_message`] and [`ExecuteWebhook::embeds`] methods.
///
/// # Examples
///
/// Refer to the documentation for [`ChannelId::send_message`] for a very in-depth
/// example on how to use this.
///
/// [`ChannelId::send_message`]: ../model/id/struct.ChannelId.html#method.send_message
/// [`Embed`]: ../model/channel/struct.Embed.html
/// [`ExecuteWebhook::embeds`]: struct.ExecuteWebhook.html#method.embeds
#[derive(Clone, Debug)]
pub struct CreateEmbed(pub VecMap<&'static str, Value>);

impl CreateEmbed {
    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more
    /// information.
    ///
    /// [`CreateEmbedAuthor`]: struct.CreateEmbedAuthor.html
    pub fn author<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbedAuthor) -> CreateEmbedAuthor {
        let map = utils::vecmap_to_json_map(f(CreateEmbedAuthor::default()).0);

        self.0.insert("author", Value::Object(map));

        CreateEmbed(self.0)
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: #method.colour
    #[cfg(feature = "utils")]
    #[inline]
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self { self.colour(colour.into()) }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(feature = "utils")]
    pub fn colour<C: Into<Colour>>(mut self, colour: C) -> Self {
        self.0.insert(
            "color",
            Value::Number(Number::from(u64::from(colour.into().0))),
        );

        CreateEmbed(self.0)
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: #method.colour
    #[cfg(not(feature = "utils"))]
    #[inline]
    pub fn color(self, colour: u32) -> Self { self.colour(colour) }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(not(feature = "utils"))]
    pub fn colour(mut self, colour: u32) -> Self {
        self.0
            .insert("color", Value::Number(Number::from(colour)));

        CreateEmbed(self.0)
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 2048 characters.
    pub fn description<D: Display>(mut self, description: D) -> Self {
        self.0.insert(
            "description",
            Value::String(description.to_string()),
        );

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
    pub fn field<T, U>(mut self, name: T, value: U, inline: bool) -> Self
        where T: Display, U: Display {
        {
            let entry = self.0
                .entry("fields")
                .or_insert_with(|| Value::Array(vec![]));

            if let Value::Array(ref mut inner) = *entry {
                inner.push(json!({
                    "inline": inline,
                    "name": name.to_string(),
                    "value": value.to_string(),
                }));
            }
        }

        self
    }

    /// Adds multiple fields at once.
    pub fn fields<T, U, It>(mut self, fields: It) -> Self
        where It: IntoIterator<Item=(T, U, bool)>,
              T: Display,
              U: Display {
        for field in fields {
            self = self.field(field.0.to_string(), field.1.to_string(), field.2);
        }

        self
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
        let map = utils::vecmap_to_json_map(footer);

        self.0.insert("footer", Value::Object(map));

        CreateEmbed(self.0)
    }

    fn url_object(mut self, name: &'static str, url: &str) -> Self {
        let obj = json!({
            "url": url.to_string()
        });

        self.0.insert(name, obj);

        CreateEmbed(self.0)
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    #[inline]
    pub fn image<S: AsRef<str>>(self, url: S) -> Self {
        self.url_object("image", url.as_ref())
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail<S: AsRef<str>>(self, url: S) -> Self {
        self.url_object("thumbnail", url.as_ref())
    }

    /// Set the timestamp.
    ///
    /// You may pass a direct string:
    ///
    /// - `2017-01-03T23:00:00`
    /// - `2004-06-08T16:04:23`
    /// - `2004-06-08T16:04:23`
    ///
    /// This timestamp must be in ISO-8601 format. It must also be in UTC format.
    ///
    /// You can also pass anything that implements `chrono::TimeZone`.
    ///
    /// # Examples
    ///
    /// Passing a string timestamp:
    ///
    /// ```rust,no_run
    /// use serenity::prelude::*;
    /// use serenity::model::channel::Message;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         if msg.content == "~embed" {
    ///             let _ = msg.channel_id.send_message(|m| m
    ///              .embed(|e| e
    ///                     .title("hello")
    ///                     .timestamp("2004-06-08T16:04:23")));
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// Creating a join-log:
    ///
    /// Note: this example isn't efficient and is for demonstrative purposes.
    ///
    /// ```rust,no_run
    /// use serenity::prelude::*;
    /// use serenity::model::guild::Member;
    /// use serenity::model::id::GuildId;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn guild_member_addition(&self, _: Context, guild_id: GuildId, member: Member) {
    ///         use serenity::CACHE;
    ///         let cache = CACHE.read();
    ///
    ///         if let Some(guild) = cache.guild(guild_id) {
    ///             let guild = guild.read();
    ///
    ///             let channel_search = guild
    ///                 .channels
    ///                 .values()
    ///                 .find(|c| c.read().name == "join-log");
    ///
    ///             if let Some(channel) = channel_search {
    ///                 let user = member.user.read();
    ///
    ///                 let _ = channel.read().send_message(|m| m
    ///                     .embed(|e| {
    ///                         let mut e = e
    ///                             .author(|a| a.icon_url(&user.face()).name(&user.name))
    ///                             .title("Member Join");
    ///
    ///                         if let Some(ref joined_at) = member.joined_at {
    ///                             e = e.timestamp(joined_at);
    ///                         }
    ///
    ///                         e
    ///                     }));
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.0
            .insert("timestamp", Value::String(timestamp.into().ts));

        CreateEmbed(self.0)
    }

    /// Set the title of the embed.
    pub fn title<D: Display>(mut self, title: D) -> Self {
        self.0
            .insert("title", Value::String(title.to_string()));

        CreateEmbed(self.0)
    }

    /// Set the URL to direct to when clicking on the title.
    pub fn url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.0
            .insert("url", Value::String(url.as_ref().to_string()));

        CreateEmbed(self.0)
    }

    /// Same as calling [`image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
    ///
    /// [`ChannelId::send_files`]: ../model/id/struct.ChannelId.html#send_files
    pub fn attachment<S: AsRef<str>>(self, filename: S) -> Self {
        self.image(&format!("attachment://{}", filename.as_ref()))
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> CreateEmbed {
        let mut map = VecMap::new();
        map.insert("type", Value::String("rich".to_string()));

        CreateEmbed(map)
    }
}

impl From<Embed> for CreateEmbed {
    /// Converts the fields of an embed into the values for a new embed builder.
    ///
    /// Some values - such as Proxy URLs - are not preserved.
    fn from(embed: Embed) -> CreateEmbed {
        let mut b = CreateEmbed::default().colour(embed.colour);

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
            b = b.field(field.name, field.value, field.inline);
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

        if let Some(footer) = embed.footer {
            b = b.footer(move |mut f| {
                f = f.text(&footer.text);

                if let Some(icon_url) = footer.icon_url {
                    f = f.icon_url(&icon_url);
                }

                f
            });
        }

        b
    }
}

/// A builder to create a fake [`Embed`] object's author, for use with the
/// [`CreateEmbed::author`] method.
///
/// Requires that you specify a [`name`].
///
/// [`Embed`]: ../model/channel/struct.Embed.html
/// [`CreateEmbed::author`]: struct.CreateEmbed.html#method.author
/// [`name`]: #method.name
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedAuthor(pub VecMap<&'static str, Value>);

impl CreateEmbedAuthor {
    /// Set the URL of the author's icon.
    pub fn icon_url(mut self, icon_url: &str) -> Self {
        self.0.insert("icon_url", Value::String(icon_url.to_string()));

        self
    }

    /// Set the author's name.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name", Value::String(name.to_string()));

        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: &str) -> Self {
        self.0.insert("url", Value::String(url.to_string()));

        self
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the
/// [`CreateEmbed::footer`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: ../model/channel/struct.Embed.html
/// [`CreateEmbed::footer`]: struct.CreateEmbed.html#method.footer
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedFooter(pub VecMap<&'static str, Value>);

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(mut self, icon_url: &str) -> Self {
        self.0.insert("icon_url", Value::String(icon_url.to_string()));

        self
    }

    /// Set the footer's text.
    pub fn text<D: Display>(mut self, text: D) -> Self {
        self.0.insert("text", Value::String(text.to_string()));

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

impl<'a> From<&'a str> for Timestamp {
    fn from(ts: &'a str) -> Self {
        Timestamp {
            ts: ts.to_string(),
        }
    }
}

impl<'a, Tz: TimeZone> From<&'a DateTime<Tz>> for Timestamp
    where Tz::Offset: Display {
    fn from(dt: &'a DateTime<Tz>) -> Self {
        Timestamp {
            ts: dt.to_rfc3339(),
        }
    }
}
