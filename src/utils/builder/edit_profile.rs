use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::default::Default;

/// A builder to edit the current user's settings, to be used in conjunction
/// with [`Context::edit_profile`].
///
/// [`Context::edit_profile`]: ../../client/struct.Context.html#method.edit_profile
pub struct EditProfile(pub ObjectBuilder);

impl EditProfile {
    /// Sets the avatar of the current user. `None` can be passed to remove an
    /// avatar.
    ///
    /// A base64-encoded string is accepted as the avatar content.
    ///
    /// # Examples
    ///
    /// A utility method - [`utils::read_image`] - is provided to read an
    /// image from a file and return its contents in base64-encoded form:
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assuming you are in a context
    ///
    /// let base64 = utils::read_image("./my_image.jpg")
    ///     .expect("Failed to read image");
    ///
    /// let _ = context.edit_profile(|profile| {
    ///     profile.avatar(Some(&base64))
    /// });
    /// ```
    ///
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    pub fn avatar(self, icon: Option<&str>) -> Self {
        EditProfile(self.0
            .insert("avatar",
                    icon.map_or_else(|| Value::Null,
                                     |x| Value::String(x.to_owned()))))
    }

    /// Modifies the current user's email address.
    ///
    /// Note that when modifying the email address, the current password must
    /// also be [provided].
    ///
    /// No validation is performed on this by the library.
    ///
    /// **Note**: This can only be used by user accounts.
    ///
    /// [provided]: #method.password
    pub fn email(self, email: &str) -> Self {
        EditProfile(self.0.insert("email", email))
    }

    /// Modifies the current user's password.
    ///
    /// Note that when modifying the password, the current password must also be
    /// [provided].
    ///
    /// [provided]: #method.password
    pub fn new_password(self, new_password: &str) -> Self {
        EditProfile(self.0.insert("new_password", new_password))
    }

    /// Used for providing the current password as verification when
    /// [modifying the password] or [modifying the associated email address].
    ///
    /// [modifying the password]: #method.new_password
    /// [modifying the associated email address]: #method.email
    pub fn password(self, password: &str) -> Self {
        EditProfile(self.0.insert("password", password))
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(self, username: &str) -> Self {
        EditProfile(self.0.insert("username", username))
    }
}

impl Default for EditProfile {
    fn default() -> EditProfile {
        EditProfile(ObjectBuilder::new())
    }
}
