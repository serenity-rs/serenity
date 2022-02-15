use std::collections::HashMap;

use serde_json::{json, Value};

use crate::internal::prelude::*;
use crate::model::channel::{PermissionOverwrite, PermissionOverwriteType, VideoQualityMode};
use crate::model::id::ChannelId;

/// A builder to edit a [`GuildChannel`] for use via [`GuildChannel::edit`]
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
/// #     let mut channel = ChannelId(0);
/// // assuming a channel has already been bound
/// if let Err(why) = channel.edit(&http, |c| c.name("new name").topic("a test topic")).await {
///     // properly handle the error
/// }
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`GuildChannel::edit`]: crate::model::channel::GuildChannel::edit
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
