use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create the content for a [`Webhook`]'s execution.
///
/// Refer to [`Http::execute_webhook`](crate::http::Http::execute_webhook) for restrictions and
/// requirements on the execution payload.
///
/// # Examples
///
/// Creating two embeds, and then sending them as part of the payload using [`Webhook::execute`]:
///
/// ```rust,no_run
/// use serenity::builder::{CreateEmbed, ExecuteWebhook};
/// use serenity::http::Http;
/// use serenity::model::webhook::Webhook;
/// use serenity::model::Colour;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http: Http = unimplemented!();
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
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/webhook#execute-webhook)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct ExecuteWebhook<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<Cow<'a, str>>,
    tts: bool,
    embeds: Cow<'a, [CreateEmbed<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Cow<'a, [CreateActionRow<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_name: Option<Cow<'a, str>>,
    attachments: EditAttachments<'a>,

    #[serde(skip)]
    thread_id: Option<ChannelId>,
}

impl<'a> ExecuteWebhook<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<(), ModelError> {
        super::check_lengths(self.content.as_deref(), Some(&self.embeds), 0)
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
    /// # let http: Http = unimplemented!();
    /// # let webhook: Webhook = unimplemented!();
    /// let builder = ExecuteWebhook::new()
    ///     .avatar_url("https://i.imgur.com/KTs6whd.jpg")
    ///     .content("Here's a webhook");
    /// webhook.execute(&http, false, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn avatar_url(mut self, avatar_url: impl Into<Cow<'a, str>>) -> Self {
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
    /// # let http: Http = unimplemented!();
    /// # let webhook: Webhook = unimplemented!();
    /// let builder = ExecuteWebhook::new().content("foo");
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
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
    /// # use serenity::model::{id::ChannelId, webhook::Webhook};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let builder = ExecuteWebhook::new().in_thread(ChannelId::new(12345678)).content("test");
    /// webhook.execute(&http, false, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn in_thread(mut self, thread_id: ChannelId) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    /// Appends a file to the webhook message.
    pub fn add_file(mut self, file: CreateAttachment<'a>) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the webhook message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the webhook message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the components for this message. Requires an application-owned webhook, meaning either
    /// the webhook's `kind` field is set to [`WebhookType::Application`], or it was created by an
    /// application (and has kind [`WebhookType::Incoming`]).
    ///
    /// [`WebhookType::Application`]: crate::model::webhook::WebhookType
    /// [`WebhookType::Incoming`]: crate::model::webhook::WebhookType
    pub fn components(mut self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        self.components = Some(components.into());
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);

    /// Set an embed for the message.
    ///
    /// Refer to the [struct-level documentation] for an example on how to use embeds.
    ///
    /// [struct-level documentation]: #examples
    pub fn embed(self, embed: CreateEmbed<'a>) -> Self {
        self.embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    pub fn embeds(mut self, embeds: impl Into<Cow<'a, [CreateEmbed<'a>]>>) -> Self {
        self.embeds = embeds.into();
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
    /// # let http: Http = unimplemented!();
    /// # let webhook: Webhook = unimplemented!();
    /// let builder = ExecuteWebhook::new().content("hello").tts(true);
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// # Ok(())
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
    /// # let http: Http = unimplemented!();
    /// # let webhook: Webhook = unimplemented!();
    /// let builder = ExecuteWebhook::new().content("hello").username("hakase");
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn username(mut self, username: impl Into<Cow<'a, str>>) -> Self {
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
    /// # let http: Http = unimplemented!();
    /// # let webhook: Webhook = unimplemented!();
    /// let builder = ExecuteWebhook::new()
    ///     .content("https://docs.rs/serenity/latest/serenity/")
    ///     .flags(MessageFlags::SUPPRESS_EMBEDS);
    /// let execution = webhook.execute(&http, false, builder).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Name of thread to create (requires the webhook channel to be a forum channel)
    pub fn thread_name(mut self, thread_name: Cow<'a, str>) -> Self {
        self.thread_name = Some(thread_name);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for ExecuteWebhook<'_> {
    type Context<'ctx> = (WebhookId, &'ctx str, bool);
    type Built = Option<Message>;

    /// Executes the webhook with the given content.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the content is malformed, if the token is invalid, or if
    /// execution is attempted in a thread not belonging to the webhook's [`Channel`].
    ///
    /// Returns [`Error::Json`] if there is an error in deserialising Discord's response.
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        self.check_length()?;

        let files = self.attachments.take_files();

        let http = cache_http.http();
        if self.allowed_mentions.is_none() {
            self.allowed_mentions.clone_from(&http.default_allowed_mentions);
        }

        http.execute_webhook(ctx.0, self.thread_id, ctx.1, ctx.2, files, &self).await
    }
}
