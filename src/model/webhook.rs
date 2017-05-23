use std::mem;
use super::*;
use ::internal::prelude::*;

#[cfg(feature="model")]
use ::builder::ExecuteWebhook;
#[cfg(feature="model")]
use ::http;

/// A representation of a webhook, which is a low-effort way to post messages to
/// channels. They do not necessarily require a bot user or authentication to
/// use.
#[derive(Clone, Debug, Deserialize)]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The default avatar.
    ///
    /// This can be modified via [`ExecuteWebhook::avatar`].
    ///
    /// [`ExecuteWebhook::avatar`]: ../builder/struct.ExecuteWebhook.html#method.avatar
    pub avatar: Option<String>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: ChannelId,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The default name of the webhook.
    ///
    /// This can be modified via [`ExecuteWebhook::username`].
    ///
    /// [`ExecuteWebhook::username`]: ../builder/struct.ExecuteWebhook.html#method.username
    pub name: Option<String>,
    /// The webhook's secure token.
    pub token: String,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
}

#[cfg(feature="model")]
impl Webhook {
    /// Deletes the webhook.
    ///
    /// As this calls the [`http::delete_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`http::delete_webhook_with_token`]: ../http/fn.delete_webhook_with_token.html
    #[inline]
    pub fn delete(&self) -> Result<()> {
        http::delete_webhook_with_token(self.id.0, &self.token)
    }

    ///
    /// Edits the webhook in-place. All fields are optional.
    ///
    /// To nullify the avatar, pass `Some("")`. Otherwise, passing `None` will
    /// not modify the avatar.
    ///
    /// Refer to [`http::edit_webhook`] for httprictions on editing webhooks.
    ///
    /// As this calls the [`http::edit_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// # Examples
    ///
    /// Editing a webhook's name:
    ///
    /// ```rust,no_run
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let _ = webhook.edit(Some("new name"), None).expect("Error editing");
    /// ```
    ///
    /// Setting a webhook's avatar:
    ///
    /// ```rust,no_run
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let image = serenity::utils::read_image("./webhook_img.png")
    ///     .expect("Error reading image");
    ///
    /// let _ = webhook.edit(None, Some(&image)).expect("Error editing");
    /// ```
    ///
    /// [`http::edit_webhook`]: ../http/fn.edit_webhook.html
    /// [`http::edit_webhook_with_token`]: ../http/fn.edit_webhook_with_token.html
    pub fn edit(&mut self, name: Option<&str>, avatar: Option<&str>) -> Result<()> {
        if name.is_none() && avatar.is_none() {
            return Ok(());
        }

        let mut map = Map::new();

        if let Some(avatar) = avatar {
            map.insert("avatar".to_owned(), if avatar.is_empty() {
                Value::Null
            } else {
                Value::String(avatar.to_owned())
            });
        }

        if let Some(name) = name {
            map.insert("name".to_owned(), Value::String(name.to_owned()));
        }

        match http::edit_webhook_with_token(self.id.0, &self.token, &map) {
            Ok(replacement) => {
                mem::replace(self, replacement);

                Ok(())
            },
            Err(why) => Err(why),
        }
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
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let _ = webhook.execute(|w| w.content("test")).expect("Error executing");
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the
    /// username to `serenity`, and sending an embed:
    ///
    /// ```rust,no_run
    /// use serenity::http;
    /// use serenity::model::Embed;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let embed = Embed::fake(|e| e
    ///     .title("Rust's website")
    ///     .description("Rust is a systems programming language that runs
    ///                   blazingly fast, prevents segfaults, and guarantees
    ///                   thread safety.")
    ///     .url("https://rust-lang.org"));
    ///
    /// let _ = webhook.execute(|w| w
    ///     .content("test")
    ///     .username("serenity")
    ///     .embeds(vec![embed]))
    ///     .expect("Error executing");
    /// ```
    #[inline]
    pub fn execute<F: FnOnce(ExecuteWebhook) -> ExecuteWebhook>(&self, f: F) -> Result<Message> {
        http::execute_webhook(self.id.0, &self.token, &f(ExecuteWebhook::default()).0)
    }

    /// Retrieves the latest information about the webhook, editing the
    /// webhook in-place.
    ///
    /// As this calls the [`http::get_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`http::get_webhook_with_token`]: ../http/fn.get_webhook_with_token.html
    pub fn refresh(&mut self) -> Result<()> {
        match http::get_webhook_with_token(self.id.0, &self.token) {
            Ok(replacement) => {
                let _ = mem::replace(self, replacement);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }
}

#[cfg(feature="model")]
impl WebhookId {
    /// Retrieves the webhook by the Id.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get(&self) -> Result<Webhook> {
        http::get_webhook(self.0)
    }
}
