use std::collections::HashMap;

use serde_json::Value;

/// A builder to edit a [`Group`] for use via [`Group::edit`]
///
/// Defaults are not directly provided by the builder itself.
///
/// # Examples
///
/// Edit a channel, providing a new name and topic:
///
/// ```rust,no_run
/// # use serenity::{http::Http, model::id::ChannelId};
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// #     let http = Http::default();
/// #     let mut group = ChannelId(0);
/// // assuming a channel has already been bound
/// if let Err(why) = group.edit(&http, |g| g.name("new name")).await {
///     // properly handle the error
/// }
/// #     Ok(())
/// # }
/// ```
///
/// [`Group`]: crate::model::channel::Group
/// [`Group::edit`]: crate::model::channel::Group::edit
#[derive(Clone, Debug, Default)]
pub struct EditGroup(pub HashMap<&'static str, Value>);

impl EditGroup {
    /// The name of the group.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Remove name
    pub fn remove_name(&mut self) -> &mut Self {
        self.0.insert("name", Value::Null);
        self
    }

    /// Set the icon of the group. Pass [`None`] to remove the icon.
    ///
    /// # Examples
    ///
    /// Using the utility function - [`utils::read_image`] - to read an image
    /// from the cwd and encode it in base64 to send to Discord.
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::channel::Group};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut group: Group = todo!();
    /// use serenity::utils;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let base64_icon = utils::read_image("./group_icon.png")?;
    ///
    /// group.edit(&http, |mut g| g.icon(Some(&base64_icon))).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`utils::read_image`]: crate::utils::read_image
    pub fn icon(&mut self, icon: Option<&str>) -> &mut Self {
        self.0.insert("icon", icon.map_or_else(|| Value::Null, |x| Value::String(x.to_string())));
        self
    }
}
