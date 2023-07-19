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
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbed(Embed);

impl CreateEmbed {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the author of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedAuthor`] for more information.
    pub fn author(mut self, author: CreateEmbedAuthor) -> Self {
        self.0.author = Some(author.0);
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
        self.0.colour = Some(colour.into());
        self
    }

    /// Set the description of the embed.
    ///
    /// **Note**: This can't be longer than 4096 characters.
    #[inline]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.0.description = Some(description.into());
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
        self.0.fields.push(EmbedField::new(name, value, inline));
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
        self.0.fields.extend(fields);
        self
    }

    /// Set the footer of the embed.
    ///
    /// Refer to the documentation for [`CreateEmbedFooter`] for more information.
    pub fn footer(mut self, footer: CreateEmbedFooter) -> Self {
        self.0.footer = Some(footer.0);
        self
    }

    /// Set the image associated with the embed. This only supports HTTP(S).
    #[inline]
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.0.image = Some(EmbedImage {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.0.thumbnail = Some(EmbedThumbnail {
            url: url.into(),
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
    ///
    /// Creating a join-log:
    ///
    /// Note: this example isn't efficient and is for demonstrative purposes.
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
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
    ///                 let icon_url = member.user.face();
    ///                 let author = CreateEmbedAuthor::new(member.user.name).icon_url(icon_url);
    ///                 let mut embed = CreateEmbed::new().title("Member Join").author(author);
    ///                 if let Some(joined_at) = member.joined_at {
    ///                     embed = embed.timestamp(joined_at);
    ///                 }
    ///                 let builder = CreateMessage::new().embed(embed);
    ///                 let _ = channel.send_message(&context, builder).await;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn timestamp<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.0.timestamp = Some(timestamp.into());
        self
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.0.title = Some(title.into());
        self
    }

    /// Set the URL to direct to when clicking on the title.
    #[inline]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    /// Same as calling [`Self::image`] with "attachment://filename.(jpg, png)".
    ///
    /// Note however, you have to be sure you set an attachment (with [`ChannelId::send_files`])
    /// with the provided filename. Or else this won't work.
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
        if let Some(ref author) = self.0.author {
            length += author.name.chars().count();
        }

        if let Some(ref description) = self.0.description {
            length += description.chars().count();
        }

        for field in &self.0.fields {
            length += field.name.chars().count();
            length += field.value.chars().count();
        }

        if let Some(ref footer) = self.0.footer {
            length += footer.text.chars().count();
        }

        if let Some(ref title) = self.0.title {
            length += title.chars().count();
        }

        super::check_overflow(length, crate::constants::EMBED_MAX_LENGTH)
            .map_err(|overflow| Error::Model(ModelError::EmbedTooLarge(overflow)))
    }
}

impl Default for CreateEmbed {
    /// Creates a builder with default values, setting the `type` to `rich`.
    fn default() -> Self {
        Self(Embed {
            fields: Vec::new(),
            description: None,
            thumbnail: None,
            timestamp: None,
            kind: Some("rich".into()),
            author: None,
            colour: None,
            footer: None,
            image: None,
            title: None,
            url: None,
            video: None,
            provider: None,
        })
    }
}

impl From<Embed> for CreateEmbed {
    fn from(embed: Embed) -> Self {
        Self(embed)
    }
}

/// A builder to create the author data of an emebd. See [`CreateEmbed::author`]
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateEmbedAuthor(EmbedAuthor);

impl CreateEmbedAuthor {
    /// Creates an author object with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self(EmbedAuthor {
            name: name.into(),
            icon_url: None,
            url: None,
            // Has no builder method because I think this field is only relevant when receiving (?)
            proxy_icon_url: None,
        })
    }

    /// Set the author's name, replacing the current value as set in [`Self::new`].
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = name.into();
        self
    }

    /// Set the URL of the author's icon.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.0.icon_url = Some(icon_url.into());
        self
    }

    /// Set the author's URL.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }
}

impl From<EmbedAuthor> for CreateEmbedAuthor {
    fn from(author: EmbedAuthor) -> Self {
        Self(author)
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
            text: text.into(),
            icon_url: None,
            // Has no builder method because I think this field is only relevant when receiving (?)
            proxy_icon_url: None,
        })
    }

    /// Set the footer's text, replacing the current value as set in [`Self::new`].
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.0.text = text.into();
        self
    }

    /// Set the icon URL's value. This only supports HTTP(S).
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.0.icon_url = Some(icon_url.into());
        self
    }
}

impl From<EmbedFooter> for CreateEmbedFooter {
    fn from(footer: EmbedFooter) -> Self {
        Self(footer)
    }
}
