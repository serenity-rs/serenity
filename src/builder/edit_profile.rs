#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::user::CurrentUser;

/// A builder to edit the current user's settings, to be used in conjunction
/// with [`CurrentUser::edit`].
///
/// [`CurrentUser::edit`]: crate::model::user::CurrentUser::edit
#[derive(Debug, Serialize)]
#[must_use]
pub struct EditProfile<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    user: &'a mut CurrentUser,
    #[cfg(not(feature = "http"))]
    user: PhantomData<&'a ()>,

    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

impl<'a> EditProfile<'a> {
    pub fn new(#[cfg(feature = "http")] user: &'a mut CurrentUser) -> Self {
        Self {
            #[cfg(feature = "http")]
            user,
            #[cfg(not(feature = "http"))]
            user: PhantomData::default(),

            avatar: None,
            username: None,
        }
    }

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
    /// let mut user = context.cache.current_user().clone();
    /// let _ = user.edit(&context, |p| p.avatar(Some(base64))).await;
    /// #     }
    /// # }
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: crate::utils::read_image
    pub fn avatar(mut self, avatar: Option<String>) -> Self {
        self.avatar = Some(avatar);
        self
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Edits the current user's profile.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set. May also return an [`Error::Json`]
    /// if there is an error in deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<()> {
        *self.user = http.as_ref().edit_profile(&self).await?;
        Ok(())
    }
}
