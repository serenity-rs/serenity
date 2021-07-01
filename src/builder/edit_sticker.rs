use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::json::from_number;
use crate::model::{guild::Role, Permissions};

/// A builder to create or edit a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_sticker`]
/// - [`Guild::create_sticker`]
/// - [`Guild::edit_sticker`]
/// - [`GuildId::create_sticker`]
/// - [`GuildId::edit_sticker`]
/// - [`Sticker::edit`]
///
/// [`PartialGuild::create_role`]: crate::model::guild::PartialGuild::create_role
/// [`Guild::create_role`]: crate::model::guild::Guild::create_role
/// [`Guild::edit_role`]: crate::model::guild::Guild::edit_role
/// [`GuildId::create_role`]: crate::model::id::GuildId::create_role
/// [`GuildId::edit_role`]: crate::model::id::GuildId::edit_role
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
