use ::internal::prelude::*;

/// A builder to edit the current user's settings, to be used in conjunction
/// with [`Context::edit_profile`].
///
/// [`Context::edit_profile`]: ../client/struct.Context.html#method.edit_profile
#[derive(Clone, Debug, Default)]
pub struct EditProfile(pub JsonMap);

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
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// # client.on_message(|context, _| {
    /// #
    /// use serenity::utils;
    ///
    /// // assuming a `context` has been bound
    ///
    /// let base64 = utils::read_image("./my_image.jpg")
    ///     .expect("Failed to read image");
    ///
    /// let _ = context.edit_profile(|profile| {
    ///     profile.avatar(Some(&base64))
    /// });
    /// # });
    /// ```
    ///
    /// [`utils::read_image`]: ../fn.read_image.html
    pub fn avatar(mut self, avatar: Option<&str>) -> Self {
        let avatar = avatar.map_or(Value::Null, |x| Value::String(x.to_owned()));

        self.0.insert("avatar".to_owned(), avatar);

        self
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
    pub fn email(mut self, email: &str) -> Self {
        self.0.insert("email".to_owned(), Value::String(email.to_owned()));

        self
    }

    /// Modifies the current user's password.
    ///
    /// Note that when modifying the password, the current password must also be
    /// [provided].
    ///
    /// [provided]: #method.password
    pub fn new_password(mut self, new_password: &str) -> Self {
        self.0.insert("new_password".to_owned(), Value::String(new_password.to_owned()));

        self
    }

    /// Used for providing the current password as verification when
    /// [modifying the password] or [modifying the associated email address].
    ///
    /// [modifying the password]: #method.new_password
    /// [modifying the associated email address]: #method.email
    pub fn password(mut self, password: &str) -> Self {
        self.0.insert("password".to_owned(), Value::String(password.to_owned()));

        self
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(mut self, username: &str) -> Self {
        self.0.insert("username".to_owned(), Value::String(username.to_owned()));

        self
    }
}
