#[cfg(not(feature = "model"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::channel::MessageFlags;

/// A builder to create the inner content of a [`Webhook`]'s execution.
///
/// This is a structured way of cleanly creating the inner execution payload,
/// to reduce potential argument counts.
///
/// Refer to the documentation for [`execute_webhook`] on restrictions with
/// execution payloads and its fields.
///
/// # Examples
///
/// Creating two embeds, and then sending them as part of the delivery
/// payload of [`Webhook::execute`]:
///
/// ```rust,no_run
/// use serenity::http::Http;
/// use serenity::model::channel::Embed;
/// use serenity::model::webhook::Webhook;
/// use serenity::utils::Colour;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http = Http::new("token");
/// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let webhook = Webhook::from_url(&http, url).await?;
///
/// let website = Embed::fake(|e| {
///     e.title("The Rust Language Website")
///         .description("Rust is a systems programming language.")
///         .colour(Colour::from_rgb(222, 165, 132))
/// });
///
/// let resources = Embed::fake(|e| {
///     e.title("Rust Resources")
///         .description("A few resources to help with learning Rust")
///         .colour(0xDEA584)
///         .field("The Rust Book", "A comprehensive resource for Rust.", false)
///         .field("Rust by Example", "A collection of Rust examples", false)
/// });
///
/// webhook
///     .execute(&http, false, |w| {
///         w.content("Here's some information on Rust:").embeds(vec![website, resources])
///     })
///     .await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`Webhook`]: crate::model::webhook::Webhook
/// [`Webhook::execute`]: crate::model::webhook::Webhook::execute
/// [`execute_webhook`]: crate::http::client::Http::execute_webhook
#[derive(Clone, Debug, Default, Serialize)]
pub struct ExecuteWebhook<'a> {
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
    #[cfg(feature = "model")]
    pub(crate) files: Vec<AttachmentType<'a>>,
    #[cfg(not(feature = "model"))]
    files: PhantomData<&'a ()>,
}

impl<'a> ExecuteWebhook<'a> {
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
    /// webhook.execute(&http, false, |w| w.avatar_url(avatar_url).content("Here's a webhook")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn avatar_url(&mut self, avatar_url: impl Into<String>) -> &mut Self {
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
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook.execute(&http, false, |w| w.content("foo")).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    /// Appends a file to the webhook message.
    #[cfg(feature = "model")]
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.files.push(file.into());
        self
    }

    /// Appends a list of files to the webhook message.
    #[cfg(feature = "model")]
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the webhook message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    #[cfg(feature = "model")]
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
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
    ///
    /// [`WebhookType::Application`]: crate::model::webhook::WebhookType
    /// [`WebhookType::Incoming`]: crate::model::webhook::WebhookType
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message. Requires an application-owned webhook. See
    /// [`components`] for details.
    ///
    /// [`components`]: crate::builder::ExecuteWebhook::components
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.components = Some(components);
        self
    }

    /// Set the embeds associated with the message.
    ///
    /// # Examples
    ///
    /// Refer to the [struct-level documentation] for an example on how to use
    /// embeds.
    ///
    /// [struct-level documentation]: #examples
    pub fn embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
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
    /// let execution = webhook.execute(&http, false, |w| w.content("hello").tts(true)).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn tts(&mut self, tts: bool) -> &mut Self {
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
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let webhook = Webhook::from_id_with_token(&http, 0, "").await?;
    /// #
    /// let execution = webhook.execute(&http, false, |w| w.content("hello").username("hakase")).await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn username(&mut self, username: impl Into<String>) -> &mut Self {
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
    ///     .execute(&http, false, |w| {
    ///         w.content("https://docs.rs/serenity/latest/serenity/")
    ///             .flags(MessageFlags::SUPPRESS_EMBEDS)
    ///     })
    ///     .await;
    ///
    /// if let Err(why) = execution {
    ///     println!("Err sending webhook: {:?}", why);
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.flags = Some(flags);
        self
    }
}
