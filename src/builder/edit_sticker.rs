use std::collections::HashMap;

use crate::internal::prelude::*;



/// A builder to create or edit a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`Guild::edit_sticker`]
/// - [`GuildId::edit_sticker`]
/// - [`Sticker::edit`]
#[derive(Clone, Debug, Default)]
pub struct EditSticker(pub HashMap<&'static str, Value>);

impl EditSticker {
    /// The name of the role to set.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// The set of permissions to assign the role.
    pub fn description<S: ToString>(&mut self, description: S) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    /// The name of a unicode emoji representing the sticker's expression
    pub fn tags<S: ToString>(&mut self, tags: S) -> &mut Self {
        self.0.insert("tags", Value::from(tags.to_string()));
        self
    }
}
