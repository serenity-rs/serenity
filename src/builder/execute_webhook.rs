use super::{CreateActionRow, CreateAllowedMentions, CreateAttachment, CreateEmbed};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::check_overflow;

/// A builder to create the content for a [`Webhook`]'s execution.
///
/// Refer to [`Http::execute_webhook`] for restrictions and requirements on the execution payload.
///
/// # Examples
///
/// Creating two embeds, and then sending them as part of the payload using [`Webhook::execute`]:
///
/// ```rust,no_run
/// use serenity::builder::{CreateEmbed, ExecuteWebhook};
/// use serenity::http::Http;
/// use serenity::model::webhook::Webhook;
/// use serenity::model::colour::Colour;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http = Http::new("token");
/// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let webhook = Webhook::from_url(&http, url).await?;
///
/// let website = CreateEmbed::new()
///     .title("The Rust Language Website")
///     .description("Rust is a systems programming language.")
///     .colour(Colour::from_rgb(222, 165, 132));
///
/// let resources = CreateEmbed::new()
///     .title("Rust Resources")
///     .description("A few resources to help with learning Rust")
///     .colour(0xDEA584)
///     .field("The Rust Book", "A comprehensive resource for Rust.", false)
///     .field("Rust by Example", "A collection of Rust examples", false);
///
/// let builder = ExecuteWebhook::new()
///     .content("Here's some information on Rust:")
///     .embeds(vec![website, resources]);
/// webhook.execute(&http, false, builder).await?;
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct ExecuteWebhook {
    tts: bool,
    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<CreateActionRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,

    #[serde(skip)]
    thread_id: Option<ChannelId>,
    #[serde(skip)]
    files: Vec<CreateAttachment>,
}

impl ExecuteWebhook {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes the webhook with the given content.
    ///
    /// If `wait` is set to false, this function will return `Ok(None)` on success. Otherwise,
    /// Discord will wait for confirmation that the message was sent, and this function will
    /// instead return `Ok(Some(Message))`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the content is malformed, if the token is invalid, or if
    /// execution is attempted in a thread not belonging to the webhook's [`Channel`].
    ///
    /// Returns [`Error::Json`] if there is an error in deserialising Discord's response.
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        http: impl AsRef<Http>,
        webhook_id: WebhookId,
        token: &str,
        wait: bool,
    ) -> Result<Option<Message>> {
        self.check_length()?;
        let files = std::mem::take(&mut self.files);
        http.as_ref().execute_webhook(webhook_id, self.thread_id, token, wait, files, &self).await
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT)
                .map_err(|overflow| Error::Model(ModelError::MessageTooLong(overflow)))?;
        }

        check_overflow(self.embeds.len(), constants::EMBED_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::EmbedAmount))?;
        for embed in &self.embeds {
            embed.check_length()?;
        }

        Ok(())
    }

    /// Override the default avatar of the webhook with an image URL.
    ///
    /// # Examples
    ///
    /// Overriding the default avatar:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let builder = ExecuteWebhook::new()
    ///     .avatar_url("https://i.imgur.com/KTs6whd.jpg")
    ///     .content("Here's a webhook");
    /// webhook.execute(&http, false, builder).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn avatar_url(mut self, avatar_url: impl Into<String>) -> Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    /// Set the content of the message.
    ///
    /// Note that when setting at least one embed via [`Self::embeds`], this may be
    /// omitted.
    ///
    /// # Examples
    ///
    /// Sending a webhook with a content of `"foo"`:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let builder = ExecuteWebhook::new().content("foo");
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Execute within a given thread. If the provided thread Id doesn't belong to the current
    /// webhook, the API will return an error.
    ///
    /// **Note**: If the given thread is archived, it will automatically be unarchived.
    ///
    /// # Examples
    ///
    /// Execute a webhook with message content of `test`, in a thread with Id `12345678`:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let builder = ExecuteWebhook::new().in_thread(12345678).content("test");
    /// webhook.execute(&http, false, builder).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn in_thread(mut self, thread_id: impl Into<ChannelId>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// Appends a file to the webhook message.
    pub fn add_file(mut self, file: CreateAttachment) -> Self {
        self.files.push(file);
        self
    }

    /// Appends a list of files to the webhook message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.files.extend(files);
        self
    }

    /// Sets a list of files to include in the webhook message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.files = files.into_iter().collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the components for this message. Requires an application-owned webhook, meaning either
    /// the webhook's `kind` field is set to [`WebhookType::Application`], or it was created by an
    /// application (and has kind [`WebhookType::Incoming`]).
    ///
    /// [`WebhookType::Application`]: crate::model::webhook::WebhookType
    /// [`WebhookType::Incoming`]: crate::model::webhook::WebhookType
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
    super::button_and_select_menu_convenience_methods!();

    /// Set an embed for the message.
    ///
    /// Refer to the [struct-level documentation] for an example on how to use embeds.
    ///
    /// [struct-level documentation]: #examples
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Whether the message is a text-to-speech message.
    ///
    /// # Examples
    ///
    /// Sending a webhook with text-to-speech enabled:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let builder = ExecuteWebhook::new().content("hello").tts(true);
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = tts;
        self
    }

    /// Override the default username of the webhook.
    ///
    /// # Examples
    ///
    /// Overriding the username to `"hakase"`:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let builder = ExecuteWebhook::new().content("hello").username("hakase");
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the flags for the message.
    ///
    /// # Examples
    ///
    /// Suppressing an embed on the message.
    ///
    /// ```rust,no_run
    /// # use serenity::builder::ExecuteWebhook;
    /// # use serenity::http::Http;
    /// # use serenity::model::channel::MessageFlags;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let builder = ExecuteWebhook::new()
    ///     .content("https://docs.rs/serenity/latest/serenity/")
    ///     .flags(MessageFlags::SUPPRESS_EMBEDS);
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }
}
