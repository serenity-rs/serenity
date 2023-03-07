//! Webhook model and implementations.

use std::fmt;

#[cfg(feature = "model")]
use super::channel::Message;
use super::id::{ChannelId, GuildId, WebhookId};
use super::user::User;
#[cfg(feature = "model")]
use crate::builder::{EditWebhookMessage, ExecuteWebhook};
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json::{self, NULL};
#[cfg(feature = "model")]
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::ModelError;
#[cfg(feature = "model")]
use crate::utils::encode_image;

/// A representation of a type of webhook.
///
/// [Discord docs](https://discord.com/developers/docs/resources/webhook#webhook-object-webhook-types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum WebhookType {
    /// An indicator that the webhook can post messages to channels with
    /// a token.
    Incoming = 1,
    /// An indicator that the webhook is managed by Discord for posting new
    /// messages to channels without a token.
    ChannelFollower = 2,
    /// Application webhooks are webhooks used with Interactions.
    Application = 3,
    /// An indicator that the webhook is of unknown type.
    Unknown = !0,
}

enum_number!(WebhookType {
    Incoming,
    ChannelFollower,
    Application
});

impl WebhookType {
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Incoming => "incoming",
            Self::ChannelFollower => "channel follower",
            Self::Application => "application",
            Self::Unknown => "unknown",
        }
    }
}

/// A representation of a webhook, which is a low-effort way to post messages to
/// channels. They do not necessarily require a bot user or authentication to
/// use.
///
/// [Discord docs](https://discord.com/developers/docs/resources/webhook#webhook-object).
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The type of the webhook.
    #[serde(rename = "type")]
    pub kind: WebhookType,
    /// The default avatar.
    ///
    /// This can be modified via [`ExecuteWebhook::avatar_url`].
    pub avatar: Option<String>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: Option<ChannelId>,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The default name of the webhook.
    ///
    /// This can be modified via [`ExecuteWebhook::username`].
    pub name: Option<String>,
    /// The webhook's secure token.
    pub token: Option<String>,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
}

impl fmt::Debug for Webhook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Webhook")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("avatar", &self.avatar)
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("name", &self.name)
            .field("user", &self.user)
            .finish()
    }
}

#[cfg(feature = "model")]
impl Webhook {
    /// Retrieves a webhook given its Id.
    ///
    /// This method requires authentication, whereas [`Webhook::from_id_with_token`] and
    /// [`Webhook::from_url`] do not.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// let id = 245037420704169985;
    /// let webhook = Webhook::from_id(&http, id).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not authenticated, or if the webhook does
    /// not exist.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn from_id(http: impl AsRef<Http>, webhook_id: impl Into<WebhookId>) -> Result<Self> {
        http.as_ref().get_webhook(webhook_id.into().0).await
    }

    /// Retrieves a webhook given its Id and unique token.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id and its unique token:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let webhook = Webhook::from_id_with_token(&http, id, token).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the webhook does not exist, or if the token is invalid.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn from_id_with_token(
        http: impl AsRef<Http>,
        webhook_id: impl Into<WebhookId>,
        token: &str,
    ) -> Result<Self> {
        http.as_ref().get_webhook_with_token(webhook_id.into().0, token).await
    }

    /// Retrieves a webhook given its url.
    ///
    /// This method does _not_ require authentication
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by url:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let webhook = Webhook::from_url(&http, url).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the url is malformed, or otherwise if the webhook does not exist, or if the token is invalid.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn from_url(http: impl AsRef<Http>, url: &str) -> Result<Self> {
        http.as_ref().get_webhook_from_url(url).await
    }

    /// Deletes the webhook.
    ///
    /// As this calls the [`Http::delete_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the webhook does not exist,
    /// the token is invalid, or if the webhook could not otherwise
    /// be deleted.
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        http.as_ref().delete_webhook_with_token(self.id.0, token).await
    }

    /// Edits the name of a webhook.
    ///
    /// Refer to [`Http::edit_webhook`] for restrictions on webhook names.
    ///
    /// Does not require authentication, as this calls [`Http::edit_webhook_with_token`] internally.
    /// # Examples
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
    /// webhook.edit_name(&http, "new name").await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn edit_name(&mut self, http: impl AsRef<Http>, name: &str) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let mut map = JsonMap::new();
        map.insert("name".to_string(), Value::from(name));
        *self = http.as_ref().edit_webhook_with_token(self.id.0, token, &map).await?;
        Ok(())
    }

    /// Edits a webhook's avatar.
    ///
    /// Refer to [`Http::edit_webhook`] for restrictions on webhook avatars.
    ///
    /// Does not require authentication, as it calls [`Http::edit_webhook_with_token`] internally.
    /// # Examples
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
    /// webhook.edit_avatar(&http, "./webhook_img.png").await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn edit_avatar<'a>(
        &mut self,
        http: impl AsRef<Http>,
        avatar: impl Into<AttachmentType<'a>>,
    ) -> Result<()> {
        let http = http.as_ref();
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let data = avatar.into().data(&http.client).await?;
        let mut map = JsonMap::new();
        map.insert("avatar".to_string(), Value::from(encode_image(&data)));
        *self = http.edit_webhook_with_token(self.id.0, token, &map).await?;
        Ok(())
    }

    /// Deletes a webhook's avatar, resetting it to the default logo.
    ///
    /// Does not require authentication, as it calls [`Http::edit_webhook_with_token`] internally.
    ///
    /// # Examples
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
    /// webhook.delete_avatar(&http).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error in deserialising Discord's response.
    pub async fn delete_avatar(&mut self, http: impl AsRef<Http>) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let mut map = JsonMap::new();
        map.insert("avatar".to_string(), NULL);
        *self = http.as_ref().edit_webhook_with_token(self.id.0, token, &map).await?;
        Ok(())
    }

    /// Executes a webhook with the fields set via the given builder.
    ///
    /// The builder provides a method of setting only the fields you need,
    /// without needing to pass a long set of arguments.
    ///
    /// # Examples
    ///
    /// Execute a webhook with message content of `test`:
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
    /// webhook.execute(&http, false, |w| w.content("test")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the
    /// username to `serenity`, and sending an embed:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// use serenity::model::channel::Embed;
    ///
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let embed = Embed::fake(|e| {
    ///     e.title("Rust's website")
    ///         .description(
    ///             "Rust is a systems programming language that runs
    ///                    blazingly fast, prevents segfaults, and guarantees
    ///                    thread safety.",
    ///         )
    ///         .url("https://rust-lang.org")
    /// });
    ///
    /// webhook
    ///     .execute(&http, false, |w| w.content("test").username("serenity").embeds(vec![embed]))
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the webhook's token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    #[inline]
    pub async fn execute<'a, F>(
        &self,
        http: impl AsRef<Http>,
        wait: bool,
        f: F,
    ) -> Result<Option<Message>>
    where
        for<'b> F: FnOnce(&'b mut ExecuteWebhook<'a>) -> &'b mut ExecuteWebhook<'a>,
    {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let mut execute_webhook = ExecuteWebhook::default();
        f(&mut execute_webhook);

        let map = json::hashmap_to_json_map(execute_webhook.0);

        if execute_webhook.1.is_empty() {
            http.as_ref().execute_webhook(self.id.0, token, wait, &map).await
        } else {
            http.as_ref()
                .execute_webhook_with_files(self.id.0, token, wait, execute_webhook.1.clone(), &map)
                .await
        }
    }

    /// Gets a previously sent message from the webhook.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return [`Error::Http`] if the webhook's token is invalid, or
    /// the given message Id does not belong to the current webhook.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    pub async fn get_message(
        &self,
        http: impl AsRef<Http>,
        message_id: MessageId,
    ) -> Result<Message> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;

        http.as_ref().get_webhook_message(self.id.0, token, message_id.0).await
    }

    /// Edits a webhook message with the fields set via the given builder.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, the webhook's token is invalid, or
    /// the given message Id does not belong to the current webhook.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    pub async fn edit_message<F>(
        &self,
        http: impl AsRef<Http>,
        message_id: MessageId,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditWebhookMessage) -> &mut EditWebhookMessage,
    {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        let mut edit_webhook_message = EditWebhookMessage::default();
        f(&mut edit_webhook_message);

        let map = json::hashmap_to_json_map(edit_webhook_message.0);

        http.as_ref().edit_webhook_message(self.id.0, token, message_id.0, &map).await
    }

    /// Deletes a webhook message.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the webhook's token is invalid or
    /// the given message Id does not belong to the current webhook.
    pub async fn delete_message(
        &self,
        http: impl AsRef<Http>,
        message_id: MessageId,
    ) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        http.as_ref().delete_webhook_message(self.id.0, token, message_id.0).await
    }

    /// Retrieves the latest information about the webhook, editing the
    /// webhook in-place.
    ///
    /// As this calls the [`Http::get_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the http client errors or if Discord returns an error.
    /// Such as if the [`Webhook`] was deleted.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    pub async fn refresh(&mut self, http: impl AsRef<Http>) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        http.as_ref().get_webhook_with_token(self.id.0, token).await.map(|replacement| {
            *self = replacement;
        })
    }

    /// Returns the url of the webhook.
    ///
    /// ```rust,ignore
    /// assert_eq!(hook.url(), "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV")
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    pub fn url(&self) -> Result<String> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        Ok(format!("https://discord.com/api/webhooks/{}/{}", self.id, token))
    }
}

#[cfg(feature = "model")]
impl WebhookId {
    /// Requests [`Webhook`] over REST API.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the http client errors or if Discord returns an error.
    /// Such as if the [`WebhookId`] does not exist.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising the response.
    ///
    /// [Manage Webhooks]: super::permissions::Permissions::MANAGE_WEBHOOKS
    #[inline]
    pub async fn to_webhook(self, http: impl AsRef<Http>) -> Result<Webhook> {
        http.as_ref().get_webhook(self.0).await
    }
}
