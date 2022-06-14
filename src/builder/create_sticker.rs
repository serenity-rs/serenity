use std::borrow::Cow;

use crate::model::channel::AttachmentType;

type Field = (Cow<'static, str>, Cow<'static, str>);

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
pub struct CreateSticker<'a> {
    name: Option<String>,
    tags: Option<String>,
    description: Option<String>,

    pub(crate) file: Option<AttachmentType<'a>>,
}

impl<'a> CreateSticker<'a> {
    /// The name of the sticker to set.
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// The description of the sticker.
    ///
    /// **Note**: If not empty, must be between 2 and 100 characters long.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Must be between 2 and 200 characters long.
    pub fn tags(&mut self, tags: impl Into<String>) -> &mut Self {
        self.tags = Some(tags.into());
        self
    }

    /// The sticker file.
    ///
    /// **Note**: Must be a PNG, APNG, or Lottie JSON file, max 500 KB.
    pub fn file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.file = Some(file.into());
        self
    }

    pub(crate) fn build(self) -> Option<(Vec<Field>, AttachmentType<'a>)> {
        let file = self.file?;
        let mut buf = Vec::with_capacity(3);

        if let Some(name) = self.name {
            buf.push(("name".into(), name.into()));
        }

        if let Some(description) = self.description {
            buf.push(("description".into(), description.into()));
        }

        if let Some(tags) = self.tags {
            buf.push(("tags".into(), tags.into()));
        }

        Some((buf, file))
    }
}
