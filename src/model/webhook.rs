//! Webhook model and implementations.

use super::{
    id::{
        ChannelId,
        GuildId,
        WebhookId
    },
    user::User
};

use std::fmt;

#[cfg(feature = "model")]
use crate::builder::ExecuteWebhook;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use std::mem;
#[cfg(feature = "model")]
use super::channel::Message;
#[cfg(feature = "model")]
use crate::utils;
#[cfg(feature = "model")]
use crate::http::Http;

/// A representation of a webhook, which is a low-effort way to post messages to
/// channels. They do not necessarily require a bot user or authentication to
/// use.
#[derive(Clone, Deserialize, Serialize)]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The default avatar.
    ///
    /// This can be modified via [`ExecuteWebhook::avatar_url`].
    ///
    /// [`ExecuteWebhook::avatar_url`]: ../../builder/struct.ExecuteWebhook.html#method.avatar_url
    pub avatar: Option<String>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: ChannelId,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The default name of the webhook.
    ///
    /// This can be modified via [`ExecuteWebhook::username`].
    ///
    /// [`ExecuteWebhook::username`]: ../../builder/struct.ExecuteWebhook.html#method.username
    pub name: Option<String>,
    /// The webhook's secure token.
    pub token: String,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

impl fmt::Debug for Webhook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Webhook")
            .field("id", &self.id)
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
    /// Deletes the webhook.
    ///
    /// As this calls the [`Http::delete_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`Http::delete_webhook_with_token`]: ../../http/client/struct.Http.html#method.delete_webhook_with_token
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_webhook_with_token(self.id.0, &self.token).await
    }

    ///
    /// Edits the webhook in-place. All fields are optional.
    ///
    /// To nullify the avatar, pass `Some("")`. Otherwise, passing `None` will
    /// not modify the avatar.
    ///
    /// Refer to [`Http::edit_webhook`] for httprictions on editing webhooks.
    ///
    /// As this calls the [`Http::edit_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// # Examples
    ///
    /// Editing a webhook's name:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http.get_webhook_with_token(id, token).await?;
    ///
    /// webhook.edit(&http, Some("new name"), None).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Setting a webhook's avatar:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http.get_webhook_with_token(id, token).await?;
    ///
    /// let image = serenity::utils::read_image("./webhook_img.png")?;
    ///
    /// webhook.edit(&http, None, Some(&image)).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Http::edit_webhook`]: ../../http/client/struct.Http.html#method.edit_webhook
    /// [`Http::edit_webhook_with_token`]: ../../http/client/struct.Http.html#method.edit_webhook_with_token
    pub async fn edit(&mut self, http: impl AsRef<Http>, name: Option<&str>, avatar: Option<&str>) -> Result<()> {
        if name.is_none() && avatar.is_none() {
            return Ok(());
        }

        let mut map = Map::new();

        if let Some(avatar) = avatar {
            map.insert(
                "avatar".to_string(),
                if avatar.is_empty() {
                    Value::Null
                } else {
                    Value::String(avatar.to_string())
                },
            );
        }

        if let Some(name) = name {
            map.insert("name".to_string(), Value::String(name.to_string()));
        }

        *self = http
            .as_ref()
            .edit_webhook_with_token(self.id.0, &self.token, &map)
            .await?;

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
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http.get_webhook_with_token(id, token).await?;
    ///
    /// webhook.execute(&http, false, |mut w| {
    ///     w.content("test");
    ///     w
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the
    /// username to `serenity`, and sending an embed:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    /// use serenity::model::channel::Embed;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http.get_webhook_with_token(id, token).await?;
    ///
    /// let embed = Embed::fake(|mut e| {
    ///     e.title("Rust's website");
    ///     e.description("Rust is a systems programming language that runs
    ///                    blazingly fast, prevents segfaults, and guarantees
    ///                    thread safety.");
    ///     e.url("https://rust-lang.org");
    ///     e
    /// });
    ///
    /// webhook.execute(&http, false, |mut w| {
    ///     w.content("test");
    ///     w.username("serenity");
    ///     w.embeds(vec![embed]);
    ///     w
    /// })
    /// .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn execute<F>(&self, http: impl AsRef<Http>, wait: bool, f: F) -> Result<Option<Message>>
    where F: FnOnce(&mut ExecuteWebhook) -> &mut ExecuteWebhook
    {
        let mut execute_webhook = ExecuteWebhook::default();
        f(&mut execute_webhook);
        let map = utils::hashmap_to_json_map(execute_webhook.0);

        http.as_ref().execute_webhook(self.id.0, &self.token, wait, &map).await
    }

    /// Retrieves the latest information about the webhook, editing the
    /// webhook in-place.
    ///
    /// As this calls the [`Http::get_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`Http::get_webhook_with_token`]: ../../http/client/struct.Http.html#method.get_webhook_with_token
    pub async fn refresh(&mut self, http: impl AsRef<Http>) -> Result<()> {
        match http.as_ref().get_webhook_with_token(self.id.0, &self.token).await {
            Ok(replacement) => {
                let _ = mem::replace(self, replacement);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Returns the url of the webhook.
    ///
    /// ```rust,ignore
    /// assert_eq!(hook.url(), "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV")
    /// ```
    pub fn url(&self) -> String {
        format!("https://discord.com/api/webhooks/{}/{}", self.id, self.token)
    }
}

#[cfg(feature = "model")]
impl WebhookId {
    /// Requests [`Webhook`] over REST API.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [`Webhook`]: struct.Webhook.html
    /// [Manage Webhooks]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[inline]
    pub async fn to_webhook(self, http: impl AsRef<Http>) -> Result<Webhook> {
        http.as_ref().get_webhook(self.0).await
    }
}
