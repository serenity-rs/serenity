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

use crate::model::channel::{Embed, EmbedField};
use crate::model::Timestamp;
#[cfg(feature = "utils")]
use crate::utils::Colour;

#[derive(Clone, Debug, Serialize)]
struct HoldsUrl {
    url: String,
}

impl HoldsUrl {
    fn new(url: String) -> Self {
        Self {
            url,
        }
    }
}

/// A builder to create a fake [`Embed`] object, for use with the
/// [`ChannelId::send_message`] and [`ExecuteWebhook::embeds`] methods.
///
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`Embed`]: crate::model::channel::Embed
/// [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
#[derive(Clone, Debug, Serialize)]
pub struct CreateEmbed {
    fields: Vec<EmbedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<CreateEmbedAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<CreateEmbedFooter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<HoldsUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<HoldsUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "color")]
    colour: Option<u32>,

    #[serde(rename = "type")]
    kind: &'static str,
}

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
        self.author = Some(author);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`Self::colour`].
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
        self.colour = Some(colour.0);
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    #[cfg(not(feature = "utils"))]
    #[inline]
    pub fn color(&mut self, colour: u32) -> &mut Self {
        self.colour(colour);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(not(feature = "utils"))]
    pub fn colour(&mut self, colour: u32) -> &mut Self {
        self.colour = Some(colour);
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 4096 characters.
    #[inline]
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Set a field. Note that this will not overwrite other fields, and will
    /// add to them.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field
    /// name and 1024 in a field value.
    #[inline]
    pub fn field(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> &mut Self {
        self.fields.push(EmbedField::new(name, value, inline));

        self
    }

    /// Adds multiple fields at once.
    ///
    /// This is sugar to reduce the need of calling [`Self::field`] manually multiple times.
    pub fn fields<N, V>(&mut self, fields: impl IntoIterator<Item = (N, V, bool)>) -> &mut Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let fields =
            fields.into_iter().map(|(name, value, inline)| EmbedField::new(name, value, inline));
        self.fields.extend(fields);

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
        self.footer = Some(create_embed_footer);
        self
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    #[inline]
    pub fn image(&mut self, url: impl Into<String>) -> &mut Self {
        self.image = Some(HoldsUrl::new(url.into()));
        self
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail(&mut self, url: impl Into<String>) -> &mut Self {
        self.thumbnail = Some(HoldsUrl::new(url.into()));
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
    /// let mut embed = CreateEmbed::default();
    /// embed.title("hello").timestamp(timestamp);
    /// ```
    ///
    /// Creating a join-log:
    ///
    /// Note: this example isn't efficient and is for demonstrative purposes.
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::guild::Member;
    /// use serenity::model::id::GuildId;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn guild_member_addition(&self, context: Context, member: Member) {
    ///         let guild_id = member.guild_id;
    ///         if let Ok(guild) = guild_id.to_partial_guild(&context).await {
    ///             let channels = guild.channels(&context).await.unwrap();
    ///
    ///             let channel_search = channels.values().find(|c| c.name == "join-log");
    ///
    ///             if let Some(channel) = channel_search {
    ///                 let user = &member.user;
    ///
    ///                 let _ = channel
    ///                     .send_message(&context, |m| {
    ///                         m.embed(|e| {
    ///                             if let Some(joined_at) = member.joined_at {
    ///                                 e.timestamp(joined_at);
    ///                             }
    ///                             e.author(|a| a.icon_url(user.face()).name(&user.name))
    ///                                 .title("Member Join")
    ///                         })
    ///                     })
    ///                     .await;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn timestamp<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Set the URL to direct to when clicking on the title.
    #[inline]
    pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    /// Same as calling [`Self::image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
    ///
    /// [`ChannelId::send_files`]: crate::model::id::ChannelId::send_files
    #[inline]
    pub fn attachment(&mut self, filename: impl Into<String>) -> &mut Self {
        let mut filename = filename.into();
        filename.insert_str(0, "attachment://");

        self.image = Some(HoldsUrl::new(filename));
        self
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            description: None,
            thumbnail: None,
            timestamp: None,
            kind: "rich",
            author: None,
            colour: None,
            footer: None,
            image: None,
            title: None,
            url: None,
        }
    }
}

impl From<Embed> for CreateEmbed {
    /// Converts the fields of an embed into the values for a new embed builder.
    ///
    /// Some values - such as Proxy URLs - are not preserved.
    fn from(embed: Embed) -> Self {
        let mut b = CreateEmbed::default();

        if let Some(colour) = embed.colour {
            b.colour(colour);
        }

        if let Some(author) = embed.author {
            b.author(move |a| {
                a.name(author.name);

                if let Some(icon_url) = author.icon_url {
                    a.icon_url(icon_url);
                }

                if let Some(url) = author.url {
                    a.url(url);
                }

                a
            });
        }

        if let Some(description) = embed.description {
            b.description(description);
        }

        for field in embed.fields {
            b.field(field.name, field.value, field.inline);
        }

        if let Some(image) = embed.image {
            b.image(image.url);
        }

        if let Some(timestamp) = embed.timestamp {
            b.timestamp(timestamp);
        }

        if let Some(thumbnail) = embed.thumbnail {
            b.thumbnail(thumbnail.url);
        }

        if let Some(url) = embed.url {
            b.url(url);
        }

        if let Some(title) = embed.title {
            b.title(title);
        }

        if let Some(footer) = embed.footer {
            b.footer(move |f| {
                if let Some(icon_url) = footer.icon_url {
                    f.icon_url(icon_url);
                }
                f.text(footer.text)
            });
        }

        b
    }
}

/// A builder to create a fake [`Embed`] object's author, for use with the
/// [`CreateEmbed::author`] method.
///
/// Requires that you specify a [`Self::name`].
///
/// [`Embed`]: crate::model::channel::Embed
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateEmbedAuthor {
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl CreateEmbedAuthor {
    /// Set the URL of the author's icon.
    pub fn icon_url(&mut self, icon_url: impl Into<String>) -> &mut Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the author's name.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the author's URL.
    pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the
/// [`CreateEmbed::footer`] method.
///
/// This does not require any field be set.
///
/// [`Embed`]: crate::model::channel::Embed
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateEmbedFooter {
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(&mut self, icon_url: impl Into<String>) -> &mut Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the footer's text.
    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        self.text = Some(text.into());
        self
    }
}

#[cfg(test)]
mod test {
    use super::CreateEmbed;
    use crate::json::{json, to_value};
    use crate::model::channel::{Embed, EmbedField, EmbedFooter, EmbedImage, EmbedVideo};
    use crate::utils::Colour;

    #[test]
    fn test_from_embed() {
        let embed = Embed {
            author: None,
            colour: Some(Colour::new(0xFF0011)),
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
                text: "This is a hakase footer".to_string(),
                icon_url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
                proxy_icon_url: None,
            }),
            image: Some(EmbedImage {
                url: "https://i.imgur.com/XfWpfCV.gif".to_string(),
                proxy_url: Some("a".to_string()),
                height: Some(213),
                width: Some(224),
            }),
            kind: Some("rich".to_string()),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some("hakase".to_string()),
            url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
            video: Some(EmbedVideo {
                url: "https://i.imgur.com/XfWpfCV.mp4".to_string(),
                proxy_url: Some("a".to_string()),
                height: Some(213),
                width: Some(224),
            }),
        };

        let mut builder = CreateEmbed::from(embed);
        builder.colour(0xFF0011);
        builder.description("This is a hakase description");
        builder.image("https://i.imgur.com/XfWpfCV.gif");
        builder.title("still a hakase");
        builder.url("https://i.imgur.com/XfWpfCV.gif");

        let built = to_value(builder).unwrap();

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
