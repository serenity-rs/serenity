//! Developer note:
//!
//! This is a set of embed builders for rich embeds.
//!
//! These are used in the [`ChannelId::send_message`] and
//! [`ExecuteWebhook::embeds`] methods, both as part of builders.
//!
//! Documentation for embeds can be found [here].
//!
//! [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
//! [here]: https://discord.com/developers/docs/resources/channel#embed-object

#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::channel::{Embed, EmbedAuthor, EmbedField, EmbedFooter};
#[cfg(feature = "http")]
use crate::model::prelude::*;
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
/// # Examples
///
/// Refer to the documentation for [`ChannelId::send_message`] for a very in-depth
/// example on how to use this.
///
/// [`ExecuteWebhook::embeds`]: crate::builder::ExecuteWebhook::embeds
#[derive(Clone, Debug, Serialize)]
#[must_use]
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
    timestamp: Option<String>,
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
    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more information.
    pub fn author(mut self, author: CreateEmbedAuthor) -> Self {
        self.author = Some(author);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`Self::colour`].
    #[cfg(feature = "utils")]
    #[inline]
    pub fn color<C: Into<Colour>>(self, colour: C) -> Self {
        self.colour(colour)
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(feature = "utils")]
    #[inline]
    pub fn colour<C: Into<Colour>>(self, colour: C) -> Self {
        self._colour(colour.into())
    }

    #[cfg(feature = "utils")]
    fn _colour(mut self, colour: Colour) -> Self {
        self.colour = Some(colour.0);
        self
    }

    /// Set the colour of the left-hand side of the embed.
    ///
    /// This is an alias of [`colour`].
    #[cfg(not(feature = "utils"))]
    #[inline]
    pub fn color(self, colour: u32) -> Self {
        self.colour(colour)
    }

    /// Set the colour of the left-hand side of the embed.
    #[cfg(not(feature = "utils"))]
    pub fn colour(mut self, colour: u32) -> Self {
        self.colour = Some(colour);
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 4096 characters.
    #[inline]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a field. Note that this will not overwrite other fields, and will add to them.
    ///
    /// **Note**: Maximum amount of characters you can put is 256 in a field name an 1024 in a
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
        self.footer = Some(footer);
        self
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    #[inline]
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image = Some(HoldsUrl::new(url.into()));
        self
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail = Some(HoldsUrl::new(url.into()));
        self
    }

    /// Set the timestamp.
    ///
    /// You may pass a direct string:
    ///
    /// - `2017-01-03T23:00:00Z`
    /// - `2004-06-08T16:04:23Z`
    /// - `2004-06-08T16:04:23Z`
    ///
    /// This timestamp must be in RFC 3339 format. It must also be in UTC format.
    ///
    /// You can also pass an instance of `chrono::DateTime<Utc>`, which will construct the
    /// timestamp string out of it.
    ///
    /// # Examples
    ///
    /// Passing a string timestamp:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::builder::CreateEmbed;
    /// use serenity::model::channel::Message;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, mut msg: Message) {
    ///         if msg.content == "~embed" {
    ///             let embed = CreateEmbed::default().title("hello").timestamp("2004-06-08T16:04:23Z");
    ///             let _ = msg.channel_id.send_message().embed(embed).execute(&context.http).await;
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
    ///
    /// Creating a join-log:
    ///
    /// Note: this example isn't efficient and is for demonstrative purposes.
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
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
    ///                 let author = CreateEmbedAuthor::default()
    ///                     .icon_url(member.user.face())
    ///                     .name(member.user.name);
    ///                 let mut embed = CreateEmbed::default().title("Member Join").author(author);
    ///                 if let Some(ref joined_at) = member.joined_at {
    ///                     embed = embed.timestamp(joined_at)
    ///                 }
    ///                 let _ = channel.send_message().embed(embed).execute(&context).await;
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
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.timestamp = Some(timestamp.into().to_string());
        self
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the URL to direct to when clicking on the title.
    #[inline]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Same as calling [`Self::image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
    #[inline]
    pub fn attachment(mut self, filename: impl Into<String>) -> Self {
        let mut filename = filename.into();
        filename.insert_str(0, "attachment://");

        self.image = Some(HoldsUrl::new(filename));
        self
    }

    #[cfg(feature = "http")]
    pub(crate) fn check_length(&self) -> Result<()> {
        let mut length = 0;
        if let Some(ref author) = self.author {
            if let Some(ref name) = author.name {
                length += name.chars().count();
            }
        }

        if let Some(ref description) = self.description {
            length += description.chars().count();
        }

        for field in &self.fields {
            length += field.name.chars().count();
            length += field.value.chars().count();
        }

        if let Some(ref footer) = self.footer {
            if let Some(ref text) = footer.text {
                length += text.chars().count();
            }
        }

        if let Some(ref title) = self.title {
            length += title.chars().count();
        }

        let max_length = crate::constants::EMBED_MAX_LENGTH;
        if length > max_length {
            let overflow = length - max_length;
            return Err(Error::Model(ModelError::EmbedTooLarge(overflow)));
        }

        Ok(())
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
        let mut b = CreateEmbed {
            description: embed.description,
            timestamp: embed.timestamp,
            title: embed.title,
            url: embed.url,
            ..Default::default()
        };

        if let Some(colour) = embed.colour {
            b = b.colour(colour);
        }

        if let Some(thumbnail) = embed.thumbnail {
            b = b.thumbnail(thumbnail.url);
        }

        if let Some(image) = embed.image {
            b = b.image(image.url);
        }

        if let Some(author) = embed.author {
            b = b.author(author.into());
        }

        if let Some(footer) = embed.footer {
            b = b.footer(footer.into());
        }

        for field in embed.fields {
            b = b.field(field.name, field.value, field.inline);
        }

        b
    }
}

/// A builder to create a fake [`Embed`] object's author, for use with the [`CreateEmbed::author`]
/// method.
///
/// Requires that you specify a [`Self::name`].
///
/// [`Embed`]: crate::model::channel::Embed
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
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
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the author's name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl From<EmbedAuthor> for CreateEmbedAuthor {
    fn from(author: EmbedAuthor) -> Self {
        Self {
            icon_url: author.icon_url,
            name: Some(author.name),
            url: author.url,
        }
    }
}

/// A builder to create a fake [`Embed`] object's footer, for use with the [`CreateEmbed::footer`]
/// method.
///
/// This does not have any required fields.
///
/// [`Embed`]: crate::model::channel::Embed
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateEmbedFooter {
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

impl CreateEmbedFooter {
    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the footer's text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl From<EmbedFooter> for CreateEmbedFooter {
    fn from(footer: EmbedFooter) -> Self {
        Self {
            icon_url: footer.icon_url,
            text: Some(footer.text),
        }
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

        let builder = CreateEmbed::from(embed)
            .colour(0xFF0011)
            .description("This is a hakase description")
            .image("https://i.imgur.com/XfWpfCV.gif")
            .title("still a hakase")
            .url("https://i.imgur.com/XfWpfCV.gif");

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
