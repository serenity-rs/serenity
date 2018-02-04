//! Webhook model and implementations.

use futures::Future;
use super::id::{ChannelId, GuildId, WebhookId};
use super::user::User;
use super::WrappedClient;
use ::FutureResult;

#[cfg(feature = "model")]
use builder::ExecuteWebhook;
#[cfg(feature = "model")]
use internal::prelude::*;
#[cfg(feature = "model")]
use super::channel::Message;

/// A representation of a webhook, which is a low-effort way to post messages to
/// channels. They do not necessarily require a bot user or authentication to
/// use.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(skip)]
    pub(crate) client: WrappedClient,
}

#[cfg(feature = "model")]
impl Webhook {
    /// Deletes the webhook.
    ///
    /// As this calls the [`http::delete_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`http::delete_webhook_with_token`]: ../http/fn.delete_webhook_with_token.html
    #[inline]
    pub fn delete(&self) -> FutureResult<()> {
        let done = ftryopt!(self.client)
            .http
            .delete_webhook_with_token(self.id.0, &self.token);

        Box::new(done)
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
    pub fn edit(&mut self, name: Option<&str>, avatar: Option<&str>)
        -> Box<Future<Item = Webhook, Error = Error>> {
        let done = ftryopt!(self.client)
            .http
            .edit_webhook_with_token(self.id.0, &self.token, name, avatar);

        Box::new(done)
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
    /// let _ = webhook.execute(false, |w| w.content("test")).expect("Error executing");
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the
    /// username to `serenity`, and sending an embed:
    ///
    /// ```rust,no_run
    /// use serenity::http;
    /// use serenity::model::channel::Embed;
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
    /// let _ = webhook.execute(false, |w| w
    ///     .content("test")
    ///     .username("serenity")
    ///     .embeds(vec![embed]))
    ///     .expect("Error executing");
    /// ```
    #[inline]
    pub fn execute<'a, F: FnOnce(ExecuteWebhook) -> ExecuteWebhook>(
        &'a self,
        wait: bool,
        f: F,
    ) -> Box<Future<Item = Option<Message>, Error = Error> + 'a> {
        let done = ftryopt!(self.client)
            .http
            .execute_webhook(self.id.0, &self.token, wait, f);

        Box::new(done)
    }
}
