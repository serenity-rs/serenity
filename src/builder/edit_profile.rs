use internal::prelude::*;
use utils::VecMap;

/// A builder to edit the current user's settings, to be used in conjunction
/// with [`CurrentUser::edit`].
///
/// [`CurrentUser::edit`]: ../model/user/struct.CurrentUser.html#method.edit
#[derive(Clone, Debug, Default)]
pub struct EditProfile(pub VecMap<&'static str, Value>);

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
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// # struct Handler;
    ///
    /// # impl EventHandler for Handler {
    ///    # fn message(&self, context: Context, _: Message) {
    ///         use serenity::utils;
    ///
    ///         // assuming a `context` has been bound
    ///
    ///         let base64 = utils::read_image("./my_image.jpg")
    ///         .expect("Failed to read image");
    ///
    ///         let _ = context.edit_profile(|mut profile| {
    ///             profile.avatar(Some(&base64));
    ///
    ///             profile
    ///         });
    ///    # }
    /// }
    /// #
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// # client.start().unwrap();
    /// ```
    ///
    /// [`utils::read_image`]: ../fn.read_image.html
    pub fn avatar(&mut self, avatar: Option<&str>) {
        let avatar = avatar.map_or(Value::Null, |x| Value::String(x.to_string()));

        self.0.insert("avatar", avatar);
    }

    /// Modifies the current user's password.
    ///
    /// Note that when modifying the password, the current password must also be
    /// [provided].
    ///
    /// [provided]: #method.password
    pub fn new_password(&mut self, new_password: &str) {
        self.0.insert("new_password", Value::String(new_password.to_string()));
    }

    /// Used for providing the current password as verification when
    /// [modifying the password] or [modifying the associated email address].
    ///
    /// [modifying the password]: #method.new_password
    /// [modifying the associated email address]: #method.email
    pub fn password(&mut self, password: &str) {
        self.0.insert("password", Value::String(password.to_string()));
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(&mut self, username: &str) {
        self.0.insert("username", Value::String(username.to_string()));
    }
}
