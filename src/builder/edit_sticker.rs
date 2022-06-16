/// A builder to create or edit a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`Guild::edit_sticker`]
/// - [`PartialGuild::edit_sticker`]
/// - [`GuildId::edit_sticker`]
/// - [`Sticker::edit`]
///
/// [`Sticker`]: crate::model::sticker::Sticker
/// [`PartialGuild::edit_sticker`]: crate::model::guild::PartialGuild::edit_sticker
/// [`Guild::edit_sticker`]: crate::model::guild::Guild::edit_sticker
/// [`GuildId::edit_sticker`]: crate::model::id::GuildId::edit_sticker
/// [`Sticker::edit`]: crate::model::sticker::Sticker::edit
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditSticker {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<String>,
}

impl EditSticker {
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
}
