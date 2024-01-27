use std::borrow::Cow;

use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::user::CurrentUser;

/// A builder to edit the current user's settings, to be used in conjunction with
/// [`CurrentUser::edit`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#modify-current-user)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditProfile<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<Option<String>>,
}

impl<'a> EditProfile<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the avatar of the current user.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::builder::{EditProfile, CreateAttachment};
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// # use serenity::http::Http;
    /// #
    /// # #[cfg(feature = "http")]
    /// # async fn foo_(http: &Http, current_user: &mut CurrentUser) -> Result<(), SerenityError> {
    /// let avatar = CreateAttachment::path("./my_image.jpg").await.expect("Failed to read image.");
    /// current_user.edit(http, EditProfile::new().avatar(&avatar)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn avatar(mut self, avatar: &CreateAttachment<'_>) -> Self {
        self.avatar = Some(Some(avatar.to_base64()));
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
    pub fn username(mut self, username: impl Into<Cow<'a, str>>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the banner of the current user.
    pub fn banner(mut self, banner: &CreateAttachment<'_>) -> Self {
        self.banner = Some(Some(banner.to_base64()));
        self
    }

    /// Deletes the current user's banner, resetting it to the default.
    pub fn delete_banner(mut self) -> Self {
        self.banner = Some(None);
        self
    }

    /// Edit the current user's profile with the fields set.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set. May also return an [`Error::Json`]
    /// if there is an error in deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http) -> Result<CurrentUser> {
        http.edit_profile(&self).await
    }
}
