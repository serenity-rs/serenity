use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::model::channel::AttachmentType;

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
    /// The name of the sticker to set.
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// The description of the sticker.
    ///
    /// **Note**: If not empty, must be between 2 and 100 characters long.
    pub fn description<S: ToString>(&mut self, description: S) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Must be between 2 and 200 characters long.
    pub fn tags<S: ToString>(&mut self, tags: S) -> &mut Self {
        self.0.insert("tags", Value::from(tags.to_string()));
        self
    }

    /// The sticker file.
    ///
    /// **Note**: Must be a PNG, APNG, or Lottie JSON file, max 500 KB.
    pub fn file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.1 = Some(file.into());
        self
    }
}
