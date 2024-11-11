use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(any(feature = "http", doc))]
use crate::model::prelude::*;

/// A builder to create or edit a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`GuildId::edit_sticker`]
/// - [`Sticker::edit`]
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#modify-guild-sticker)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditSticker<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Cow<'a, str>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditSticker<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The name of the sticker to set.
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The description of the sticker.
    ///
    /// **Note**: If not empty, must be between 2 and 100 characters long.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Must be between 2 and 200 characters long.
    pub fn tags(mut self, tags: impl Into<Cow<'a, str>>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Edits the sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: &Http,
        guild_id: GuildId,
        sticker_id: StickerId,
    ) -> Result<Sticker> {
        http.edit_sticker(guild_id, sticker_id, &self, self.audit_log_reason).await
    }
}
