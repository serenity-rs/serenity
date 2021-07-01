use std::collections::HashMap;

use crate::http::AttachmentType;
use crate::internal::prelude::*;

/// A builder to create or edit a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_sticker`]
/// - [`Guild::create_sticker`]
/// - [`GuildId::create_sticker`]
///
/// [`Sticker`]: crate::model::sticker::Sticker
/// [`PartialGuild::create_sticker`]: crate::model::guild::PartialGuild::create_sticker
/// [`Guild::create_sticker`]: crate::model::guild::Guild::create_sticker
/// [`GuildId::create_sticker`]: crate::model::id::GuildId::create_sticker
#[derive(Clone, Debug, Default)]
pub struct CreateSticker<'a>(pub HashMap<&'static str, Value>, pub Option<AttachmentType<'a>>);

impl<'a> CreateSticker<'a> {
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

    /// The sticker file.
    pub fn file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.1 = Some(file.into());
        self
    }
}
