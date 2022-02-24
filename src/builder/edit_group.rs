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
}
