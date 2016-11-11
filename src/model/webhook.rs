use serde_json::builder::ObjectBuilder;
use std::mem;
use super::{Message, Webhook};
use ::builder::ExecuteWebhook;
use ::client::http;
use ::prelude_internal::*;

impl Webhook {
    /// Deletes the webhook.
    ///
    /// As this calls the [`http::delete_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`http::delete_webhook_with_token`]: ../client/http/fn.delete_webhook_with_token.html
    pub fn delete(&self) -> Result<()> {
        http::delete_webhook_with_token(self.id.0, &self.token)
    }

    ///
    /// Edits the webhook in-place. All fields are optional.
    ///
    /// To nullify the avatar, pass `Some("")`. Otherwise, passing `None` will
    /// not modify the avatar.
    ///
    /// Refer to [`http::edit_webhook`] for restrictions on editing webhooks.
    ///
    /// As this calls the [`http::edit_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// # Examples
    ///
    /// Editing a webhook's name:
    ///
    /// ```rust,no_run
    /// use serenity::client::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let _ = webhook.edit(Some("new name"), None).expect("err editing");
    /// ```
    ///
    /// Setting a webhook's avatar:
    ///
    /// ```rust,no_run
    /// use serenity::client::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let image = serenity::utils::read_image("./webhook_img.png")
    ///     .expect("err reading image");
    ///
    /// let _ = webhook.edit(None, Some(&image)).expect("err editing");
    /// ```
    ///
    /// [`http::edit_webhook`]: ../client/http/fn.edit_webhook.html
    /// [`http::edit_webhook_with_token`]: ../client/http/fn.edit_webhook_with_token.html
    pub fn edit(&mut self, name: Option<&str>, avatar: Option<&str>)
        -> Result<()> {
        if name.is_none() && avatar.is_none() {
            return Ok(());
        }

        let mut map = ObjectBuilder::new();

        if let Some(avatar) = avatar {
            map = map.insert("avatar", if avatar.is_empty() {
                Value::Null
            } else {
                Value::String(avatar.to_owned())
            });
        }

        if let Some(name) = name {
            map = map.insert("name", name);
        }

        let map = map.build();

        match http::edit_webhook_with_token(self.id.0, &self.token, map) {
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
    /// use serenity::client::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let mut webhook = http::get_webhook_with_token(id, token)
    ///     .expect("valid webhook");
    ///
    /// let _ = webhook.execute(|w| w.content("test")).expect("err executing");
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the
    /// username to `serenity`, and sending an embed:
    ///
    /// ```rust,no_run
    /// use serenity::client::http;
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
    ///     .expect("err executing");
    /// ```
    pub fn execute<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(ExecuteWebhook) -> ExecuteWebhook {
        let map = f(ExecuteWebhook::default()).0.build();

        http::execute_webhook(self.id.0, &self.token, map)
    }

    /// Retrieves the latest information about the webhook, editing the
    /// webhook in-place.
    ///
    /// As this calls the [`http::get_webhook_with_token`] function,
    /// authentication is not required.
    ///
    /// [`http::get_webhook_with_token`]: ../client/http/fn.get_webhook_with_token.html
    pub fn refresh(&mut self) -> Result<()> {
        match http::get_webhook_with_token(self.id.0, &self.token) {
            Ok(replacement) => {
                mem::replace(self, replacement);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }
}
