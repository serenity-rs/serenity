use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::json::NULL;

/// A builder to edit the current user's settings, to be used in conjunction
/// with [`CurrentUser::edit`].
///
/// [`CurrentUser::edit`]: crate::model::user::CurrentUser::edit
#[derive(Clone, Debug, Default)]
pub struct EditProfile(pub HashMap<&'static str, Value>);

impl EditProfile {
    /// Sets the avatar of the current user. [`None`] can be passed to remove an
    /// avatar.
    ///
    /// A base64-encoded string is accepted as the avatar content.
    ///
    /// # Examples
    ///
    /// A utility method - [`utils::read_image`] - is provided to read an
    /// image from a file and return its contents in base64-encoded form:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "client", feature = "cache", feature = "gateway"))]
    /// # {
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// # struct Handler;
    /// #
    /// # #[serenity::async_trait]
    /// # impl EventHandler for Handler {
    /// #     async fn message(&self, context: Context, _: Message) {
    /// use serenity::utils;
    ///
    /// // assuming a `context` has been bound
    ///
    /// let base64 = utils::read_image("./my_image.jpg").expect("Failed to read image");
    ///
    /// let mut user = context.cache.current_user();
    /// let _ = user.edit(&context, |p| p.avatar(Some(base64))).await;
    /// #     }
    /// # }
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: crate::utils::read_image
    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        let avatar = avatar.map_or(NULL, Value::String);
        self.0.insert("avatar", avatar);
        self
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(&mut self, username: impl Into<String>) -> &mut Self {
        self.0.insert("username", Value::from(username.into()));
        self
    }
}
