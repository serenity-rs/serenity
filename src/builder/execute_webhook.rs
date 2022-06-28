#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create the content for a [`Webhook`]'s execution.
///
/// Refer to [`Http::execute_webhook`] for restrictions on the execution payload and its fields.
///
/// # Examples
///
/// Creating two embeds, and then sending them as part of the payload using [`Webhook::execute`]:
///
/// ```rust,no_run
/// use serenity::builder::CreateEmbed;
/// use serenity::http::Http;
/// use serenity::model::webhook::Webhook;
/// use serenity::utils::Colour;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http = Http::new("token");
/// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let webhook = Webhook::from_url(&http, url).await?;
///
/// let website = CreateEmbed::default()
///     .title("The Rust Language Website")
///     .description("Rust is a systems programming language.")
///     .colour(Colour::from_rgb(222, 165, 132));
///
/// let resources = CreateEmbed::default()
///     .title("Rust Resources")
///     .description("A few resources to help with learning Rust")
///     .colour(0xDEA584)
///     .field("The Rust Book", "A comprehensive resource for Rust.", false)
///     .field("Rust by Example", "A collection of Rust examples", false);
///
/// let msg = webhook
///     .execute()
///     .content("Here's some information on Rust:")
///     .embeds(vec![website, resources])
///     .execute(&http)
///     .await?;
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct ExecuteWebhook<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    webhook: &'a Webhook,
    #[cfg(not(feature = "http"))]
    webhook: PhantomData<&'a ()>,

    tts: bool,
    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,

    #[serde(skip)]
    wait: bool,
    #[serde(skip)]
    thread_id: Option<ChannelId>,
    #[serde(skip)]
    files: Vec<AttachmentType<'a>>,
}

impl<'a> ExecuteWebhook<'a> {
    pub fn new(#[cfg(feature = "http")] webhook: &'a Webhook) -> Self {
        Self {
            #[cfg(feature = "http")]
            webhook,
            #[cfg(not(feature = "http"))]
            webhook: PhantomData::default(),

            tts: false,
            embeds: Vec::new(),
            avatar_url: None,
            content: None,
            allowed_mentions: None,
            components: None,
            username: None,
            flags: None,

            wait: false,
            thread_id: None,
            files: Vec::new(),
        }
    }

    /// Override the default avatar of the webhook with an image URL.
    ///
    /// # Examples
    ///
    /// Overriding the default avatar:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let avatar_url = "https://i.imgur.com/KTs6whd.jpg";
    ///
    /// webhook.execute().avatar_url(avatar_url).content("Here's a webhook").execute(&http).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn avatar_url(mut self, avatar_url: impl Into<String>) -> Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    /// Set the content of the message.
    ///
    /// Note that when setting at least one embed via [`Self::embeds`], this may be omitted.
    ///
    /// # Examples
    ///
    /// Sending a webhook with a content of `"foo"`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook.execute().content("foo").execute(&http).await;
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

    /// Execute within the context of a thread belonging to the Webhook's channel. If the thread is
    /// archived, it will automatically be unarchived. If the provided thread Id doesn't belong to
    /// the current webhook's thread, then the API will return an error at request execution time.
    ///
    /// # Examples
    ///
    /// Execute a webhook with message content of `test`, in a thread with Id `12345678`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// webhook.execute().in_thread(12345678).content("test").execute(&http).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn in_thread(mut self, thread_id: impl Into<ChannelId>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// Appends a file to the webhook message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(mut self, file: T) -> Self {
        self.files.push(file.into());
        self
    }

    /// Appends a list of files to the webhook message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the webhook message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Creates components for this message. Requires an application-owned webhook, meaning either
    /// the webhook's `kind` field is set to [`WebhookType::Application`], or it was created by an
    /// application (and has kind [`WebhookType::Incoming`]).
    pub fn components<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message. Requires an application-owned webhook. See
    /// [`Self::components`] for details.
    pub fn set_components(mut self, components: CreateComponents) -> Self {
        self.components = Some(components);
        self
    }

    /// Set the embeds associated with the message.
    ///
    /// # Examples
    ///
    /// Refer to the [struct-level documentation] for an example on how to use embeds.
    ///
    /// [struct-level documentation]: #examples
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
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook.execute().content("hello").tts(true).execute(&http).await;
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

    /// Set this to `true` to wait for server confirmation of the message having been sent before
    /// receiving a response. See the [Discord docs] for more details.
    ///
    /// [Discord docs]: https://discord.com/developers/docs/resources/webhook#execute-webhook-query-string-params
    pub fn wait(mut self, wait: bool) -> Self {
        self.wait = wait;
        self
    }

    /// Override the default username of the webhook.
    ///
    /// # Examples
    ///
    /// Overriding the username to `"hakase"`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook.execute().content("hello").username("hakase").execute(&http).await;
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
    /// # use serenity::http::Http;
    /// # use serenity::model::channel::MessageFlags;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook
    ///     .execute()
    ///     .content("https://docs.rs/serenity/latest/serenity/")
    ///     .flags(MessageFlags::SUPPRESS_EMBEDS)
    ///     .execute(&http)
    ///     .await;
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

    /// Executes the webhook with the given content.
    ///
    /// If [`Self::wait`] is set to false, then this function will return `Ok(None)`. Otherwise
    /// Discord will wait for confirmation that the message was sent, and this function will return
    /// `Ok(Some(Message))`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the token field of the current webhook is `None`.
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    /// Additionally, this error variant is returned if the webhook attempts to execute in the
    /// context of a thread whose Id is invalid, or does not belonging to the webhook's
    /// associated [`Channel`].
    ///
    /// Finally, may return an [`Error::Json`] if there is an error in deserialising Discord's
    /// response.
    #[cfg(feature = "http")]
    pub async fn execute(mut self, http: impl AsRef<Http>) -> Result<Option<Message>> {
        let token = self.webhook.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let id = self.webhook.id.into();
        let thread_id = self.thread_id.map(Into::into);
        let files = std::mem::take(&mut self.files);

        if files.is_empty() {
            http.as_ref().execute_webhook(id, thread_id, token, self.wait, &self).await
        } else {
            http.as_ref()
                .execute_webhook_with_files(id, thread_id, token, self.wait, files, &self)
                .await
        }
    }
}
