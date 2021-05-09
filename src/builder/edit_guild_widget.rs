use std::collections::HashMap;

use crate::internal::prelude::*;

/// A builder to specify the fields to edit in a [`GuildWidget`].
///
/// [`GuildWidget`]: crate::model::guild::GuildWidget
#[derive(Clone, Debug, Default)]
pub struct EditGuildWidget(pub HashMap<&'static str, Value>);

impl EditGuildWidget {
    /// Whether the widget is enabled or not.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("enabled", Value::Bool(enabled));

        self
    }

    /// The server description shown in the welcome screen.
    pub fn channel_id(&mut self, id: u64) -> &mut Self {
        self.0.insert("channel_id", Value::String(id.to_string()));

        self
    }
}
