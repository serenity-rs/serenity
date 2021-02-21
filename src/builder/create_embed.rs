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
//! [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
//! [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
//! [here]: https://discord.com/developers/docs/resources/channel#embed-object

use std::collections::HashMap;
use std::fmt::Display;

use chrono::{DateTime, TimeZone};

use crate::json::Value;
use crate::json::{from_number, json};
use crate::model::channel::Embed;
use crate::utils;
#[cfg(feature = "utils")]
use crate::utils::Colour;

/// A builder to create a fake [`Embed`] object, for use with the
/// [`ChannelId::send_message`] and [`ExecuteWebhook::embeds`] methods.
///
/// # Examples
///
/// Refer to the documentation for [`ChannelId::send_message`] for a very in-depth
/// example on how to use this.
///
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`Embed`]: crate::model::channel::Embed
/// [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
#[derive(Clone, Debug)]
pub struct CreateEmbed(pub HashMap<&'static str, Value>);

impl CreateEmbed {
    /// Build the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more
    /// information.
    pub fn author<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbedAuthor) -> &mut CreateEmbedAuthor,
    {
        let mut author = CreateEmbedAuthor::default();
        f(&mut author);
        self.set_author(author)
    }

    /// Set the author of the embed.
    pub fn set_author(&mut self, author: CreateEmbedAuthor) -> &mut Self {
        let map = utils::hashmap_to_json_map(author.0);

        self.0.insert("author", Value::from(map));
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: Self::colour
    #[cfg(feature = "utils")]
    #[inline]
    pub fn color<C: Into<Colour>>(&mut self, colour: C) -> &mut Self {
        self.colour(colour);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(feature = "utils")]
    #[inline]
    pub fn colour<C: Into<Colour>>(&mut self, colour: C) -> &mut Self {
        self._colour(colour.into());
        self
    }

    #[cfg(feature = "utils")]
    fn _colour(&mut self, colour: Colour) {
        self.0.insert("color", from_number(u64::from(colour.0)));
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    ///
    /// [`colour`]: Self::colour
    #[cfg(not(feature = "utils"))]
    #[inline]
    pub fn color(&mut self, colour: u32) -> &mut Self {
        self.colour(colour);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(not(feature = "utils"))]
    pub fn colour(&mut self, colour: u32) -> &mut Self {
        self.0.insert("color", Value::Number(Number::from(colour)));
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 2048 characters.
    #[inline]
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::String(description.to_string()));
        self
    }

    /// Set a field. Note that this will not overwrite other fields, and will
    /// add to them.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field
    /// name and 1024 in a field value.
    #[inline]
    pub fn field<T, U>(&mut self, name: T, value: U, inline: bool) -> &mut Self
    where
        T: ToString,
        U: ToString,
    {
        self._field(name.to_string(), value.to_string(), inline);
        self
    }

    fn _field(&mut self, name: String, value: String, inline: bool) {
        {
            let entry = self.0.entry("fields").or_insert_with(|| Value::Array(vec![]));

            if let Value::Array(ref mut inner) = *entry {
                inner.push(json!({
                    "inline": inline,
                    "name": name,
                    "value": value,
                }));
            }
        }
    }

    /// Adds multiple fields at once.
    ///
    /// This is sugar to reduce the need of calling [`field`] manually multiple times.
    ///
    /// [`field`]: Self::field
    pub fn fields<T, U, It>(&mut self, fields: It) -> &mut Self
    where
        It: IntoIterator<Item = (T, U, bool)>,
        T: ToString,
        U: ToString,
    {
        for (name, value, inline) in fields {
            self.field(name, value, inline);
        }

        self
    }

    /// Build the footer of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedFooter`] for more
    /// information.
    pub fn footer<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbedFooter) -> &mut CreateEmbedFooter,
    {
        let mut create_embed_footer = CreateEmbedFooter::default();
        f(&mut create_embed_footer);
        self.set_footer(create_embed_footer)
    }

    /// Set the footer of the embed.
    pub fn set_footer(&mut self, create_embed_footer: CreateEmbedFooter) -> &mut Self {
        let footer = create_embed_footer.0;
        let map = utils::hashmap_to_json_map(footer);

        self.0.insert("footer", Value::from(map));
        self
    }

    fn url_object(&mut self, name: &'static str, url: String) -> &mut Self {
        let obj = json!({
            "url": url,
        });

        self.0.insert(name, obj);
        self
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    #[inline]
    pub fn image<S: ToString>(&mut self, url: S) -> &mut Self {
        self.url_object("image", url.to_string());

        self
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail<S: ToString>(&mut self, url: S) -> &mut Self {
        self.url_object("thumbnail", url.to_string());
        self
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
    /// You can also pass an instance of `chrono::DateTime<Utc>`,
    /// which will construct the timestamp string out of it.
    ///
    /// # Examples
    ///
    /// Passing a string timestamp:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::prelude::*;
    /// use serenity::model::channel::Message;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, mut msg: Message) {
    ///         if msg.content == "~embed" {
    ///             let _ = msg.channel_id.send_message(&context.http, |m| {
    ///                 m.embed(|e| {
    ///                     e.title("hello").timestamp("2004-06-08T16:04:23")
    ///                 });
    ///
    ///                 m
    ///             })
    ///             .await;
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Creating a join-log:
    ///
    /// Note: this example isn't efficient and is for demonstrative purposes.
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::prelude::*;
    /// use serenity::model::guild::Member;
    /// use serenity::model::id::GuildId;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn guild_member_addition(&self, context: Context, guild_id: GuildId, member: Member) {
    ///         if let Ok(guild) = guild_id.to_partial_guild(&context).await {
    ///             let channels = guild.channels(&context)
    ///                 .await
    ///                 .unwrap();
    ///
    ///             let channel_search = channels.values()
    ///                 .find(|c| c.name == "join-log");
    ///
    ///             if let Some(channel) = channel_search {
    ///                 let user = &member.user;
    ///
    ///                 let _ = channel.send_message(&context, |m| {
    ///                     m.embed(|e| {
    ///                         e.author(|a| {
    ///                             a.icon_url(&user.face()).name(&user.name)
    ///                         });
    ///                         e.title("Member Join");
    ///
    ///                         if let Some(ref joined_at) = member.joined_at {
    ///                             e.timestamp(joined_at);
    ///                         }
    ///
    ///                         e
    ///                     })
    ///                 })
    ///                 .await;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn timestamp<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self._timestamp(timestamp.into());
        self
    }

    fn _timestamp(&mut self, timestamp: Timestamp) {
        self.0.insert("timestamp", Value::String(timestamp.ts));
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.0.insert("title", Value::String(title.to_string()));
        self
    }

    /// Set the URL to direct to when clicking on the title.
    #[inline]
    pub fn url<S: ToString>(&mut self, url: S) -> &mut Self {
        self.0.insert("url", Value::String(url.to_string()));
        self
    }

    /// Same as calling [`image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
    ///
    /// [`ChannelId::send_files`]: crate::model::id::ChannelId::send_files
    ///
    /// [`image`]: Self::image
    #[inline]
    pub fn attachment<S: ToString>(&mut self, filename: S) -> &mut Self {
        let mut filename = filename.to_string();
        filename.insert_str(0, "attachment://");
        self.url_object("image", filename);

        self
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> CreateEmbed {
        let mut map = HashMap::new();
        map.insert("type", Value::String("rich".to_string()));

        CreateEmbed(map)
    }
}

impl From<Embed> for CreateEmbed {
    /// Converts the fields of an embed into the values for a new embed builder.
    ///
    /// Some values - such as Proxy URLs - are not preserved.
    fn from(embed: Embed) -> Self {
        let mut b = CreateEmbed::default();
        b.colour(embed.colour);

        if let Some(author) = embed.author {
            b.author(move |a| {
                a.name(&author.name);

                if let Some(icon_url) = author.icon_url {
                    a.icon_url(&icon_url);
                }

                if let Some(url) = author.url {
                    a.url(&url);
                }

                a
            });
        }

        if let Some(description) = embed.description {
            b.description(&description);
        }

        for field in embed.fields {
            b.field(field.name, field.value, field.inline);
        }

        if let Some(image) = embed.image {
            b.image(&image.url);
        }

        if let Some(timestamp) = embed.timestamp {
            b.timestamp(timestamp);
        }

        if let Some(thumbnail) = embed.thumbnail {
            b.thumbnail(&thumbnail.url);
        }

        if let Some(url) = embed.url {
            b.url(&url);
        }

        if let Some(title) = embed.title {
            b.title(&title);
        }

        if let Some(footer) = embed.footer {
            b.footer(move |f| {
                f.text(&footer.text);

                if let Some(icon_url) = footer.icon_url {
                    f.icon_url(&icon_url);
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
/// [`Embed`]: crate::model::channel::Embed
/// [`name`]: Self::name
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedAuthor(pub HashMap<&'static str, Value>);

impl CreateEmbedAuthor {
    /// Set the URL of the author's icon.
    pub fn icon_url<S: ToString>(&mut self, icon_url: S) -> &mut Self {
        self.0.insert("icon_url", Value::String(icon_url.to_string()));
        self
    }

    /// Set the author's name.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Set the author's URL.
    pub fn url<S: ToString>(&mut self, url: S) -> &mut Self {
        self.0.insert("url", Value::String(url.to_string()));
        self
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the
/// [`CreateEmbed::footer`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: crate::model::channel::Embed
#[derive(Clone, Debug, Default)]
pub struct CreateEmbedFooter(pub HashMap<&'static str, Value>);

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url<S: ToString>(&mut self, icon_url: S) -> &mut Self {
        self.0.insert("icon_url", Value::String(icon_url.to_string()));
        self
    }

    /// Set the footer's text.
    pub fn text<S: ToString>(&mut self, text: S) -> &mut Self {
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
        Self {
            ts,
        }
    }
}

impl<'a> From<&'a str> for Timestamp {
    fn from(ts: &'a str) -> Self {
        Self {
            ts: ts.to_string(),
        }
    }
}

impl<'a, Tz: TimeZone> From<&'a DateTime<Tz>> for Timestamp
where
    Tz::Offset: Display,
{
    fn from(dt: &'a DateTime<Tz>) -> Self {
        Self {
            ts: dt.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::CreateEmbed;
    use crate::json::json;
    use crate::json::Value;
    use crate::{
        model::channel::{Embed, EmbedField, EmbedFooter, EmbedImage, EmbedVideo},
        utils::{self, Colour},
    };

    #[test]
    fn test_from_embed() {
        let embed = Embed {
            author: None,
            colour: Colour::new(0xFF0011),
            description: Some("This is a test description".to_string()),
            fields: vec![
                EmbedField {
                    inline: false,
                    name: "a".to_string(),
                    value: "b".to_string(),
                },
                EmbedField {
                    inline: true,
                    name: "c".to_string(),
                    value: "z".to_string(),
                },
            ],
            footer: Some(EmbedFooter {
                icon_url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
                proxy_icon_url: None,
                text: "This is a hakase footer".to_string(),
            }),
            image: Some(EmbedImage {
                height: 213,
                proxy_url: "a".to_string(),
                url: "https://i.imgur.com/XfWpfCV.gif".to_string(),
                width: 224,
            }),
            kind: "rich".to_string(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some("hakase".to_string()),
            url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
            video: Some(EmbedVideo {
                height: 213,
                url: "https://i.imgur.com/XfWpfCV.mp4".to_string(),
                width: 224,
            }),
        };

        let mut builder = CreateEmbed::from(embed);
        builder.colour(0xFF0011);
        builder.description("This is a hakase description");
        builder.image("https://i.imgur.com/XfWpfCV.gif");
        builder.title("still a hakase");
        builder.url("https://i.imgur.com/XfWpfCV.gif");

        let built = Value::from(utils::hashmap_to_json_map(builder.0));

        let obj = json!({
            "color": 0xFF0011,
            "description": "This is a hakase description",
            "title": "still a hakase",
            "type": "rich",
            "url": "https://i.imgur.com/XfWpfCV.gif",
            "fields": [
                {
                    "inline": false,
                    "name": "a",
                    "value": "b",
                },
                {
                    "inline": true,
                    "name": "c",
                    "value": "z",
                },
            ],
            "image": {
                "url": "https://i.imgur.com/XfWpfCV.gif",
            },
            "footer": {
                "text": "This is a hakase footer",
                "icon_url": "https://i.imgur.com/XfWpfCV.gif",
            }
        });

        assert_eq!(built, obj);
    }
}
