use std::borrow::Cow;

use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

/// A builder to create a guild sticker
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#create-guild-sticker)
#[derive(Clone, Debug)]
#[must_use]
pub struct CreateSticker<'a> {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    tags: Cow<'static, str>,
    file: CreateAttachment<'a>,
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateSticker<'a> {
    /// Creates a new builder with the given data. All of this builder's fields are required.
    pub fn new(name: impl Into<Cow<'static, str>>, file: CreateAttachment<'a>) -> Self {
        Self {
            name: name.into(),
            tags: Cow::default(),
            description: Cow::default(),
            file,
            audit_log_reason: None,
        }
    }

    /// Set the name of the sticker, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the description of the sticker.
    ///
    /// **Note**: Must be empty or 2-100 characters.
    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Max 200 characters long.
    pub fn tags(mut self, tags: impl Into<Cow<'static, str>>) -> Self {
        self.tags = tags.into();
        self
    }

    /// Set the sticker file. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be a PNG, APNG, or Lottie JSON file, max 500 KB.
    pub fn file(mut self, file: CreateAttachment<'a>) -> Self {
        self.file = file;
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<Sticker> {
        let map = vec![
            ("name".into(), self.name),
            ("tags".into(), self.tags),
            ("description".into(), self.description),
        ];

        http.create_sticker(guild_id, map, self.file, self.audit_log_reason).await
    }
}
