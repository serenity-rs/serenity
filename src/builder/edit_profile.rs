#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::{channel::AttachmentType, user::CurrentUser};

/// A builder to edit the current user's settings, to be used in conjunction with
/// [`CurrentUser::edit`].
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

impl EditProfile {
    /// Edit the current user's profile with the fields set.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set. May also return an [`Error::Json`]
    /// if there is an error in deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<CurrentUser> {
        http.as_ref().edit_profile(&self).await
    }

    /// Set the avatar of the current user.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "client", feature = "cache", feature = "gateway"))]
    /// # {
    /// # use serenity::builder::EditProfile;
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// # struct Handler;
    /// #
    /// # #[serenity::async_trait]
    /// # impl EventHandler for Handler {
    /// #     async fn message(&self, context: Context, _: Message) {
    /// // assuming a `context` has been bound
    /// let mut user = context.cache.current_user().clone();
    ///
    /// let builder = EditProfile::default()
    ///     .avatar(&context, "./my_image.jpg")
    ///     .await
    ///     .expect("Failed to read image.");
    /// let _ = user.edit(&context, builder).await;
    /// #     }
    /// # }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May error if the input is a URL and the HTTP request fails, or if it is a path to a file
    /// that does not exist.
    #[cfg(feature = "http")]
    pub async fn avatar<'a>(
        mut self,
        http: impl AsRef<Http>,
        avatar: impl Into<AttachmentType<'a>>,
    ) -> Result<Self> {
        let avatar_data = avatar.into().data(&http.as_ref().client).await?;
        self.avatar = Some(Some(crate::utils::encode_image(&avatar_data)));
        Ok(self)
    }

    #[cfg(not(feature = "http"))]
    /// Set the current user's avatar. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    pub fn avatar(mut self, avatar: String) -> Self {
        self.avatar = Some(Some(avatar));
        self
    }

    /// Delete the current user's avatar, resetting it to the default logo.
    pub fn delete_avatar(mut self) -> Self {
        self.avatar = Some(None);
        self
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username and current
    /// discriminator, a new unique discriminator will be assigned. If there are no available
    /// discriminators with the requested username, an error will occur.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }
}
