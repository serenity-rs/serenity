use crate::http::CacheHttp;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::channel::AttachmentType;
use crate::model::prelude::*;

/// A builder to create a [`Sticker`] for use via a number of model methods.
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
#[derive(Clone, Debug)]
pub struct CreateSticker<'a> {
    id: GuildId,
    fields: CreateStickerFields,
    file: Option<AttachmentType<'a>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateStickerFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl<'a> CreateSticker<'a> {
    pub(crate) fn new(id: GuildId) -> Self {
        Self {
            id,
            fields: CreateStickerFields::default(),
            file: None,
        }
    }

    /// The name of the sticker to set.
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.fields.name = Some(name.into());
        self
    }

    /// The description of the sticker.
    ///
    /// **Note**: If not empty, must be between 2 and 100 characters long.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.fields.description = Some(description.into());
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Must be between 2 and 200 characters long.
    #[must_use]
    pub fn tags(mut self, tags: impl Into<String>) -> Self {
        self.fields.tags = Some(tags.into());
        self
    }

    /// The sticker file.
    ///
    /// **Note**: Must be a PNG, APNG, or Lottie JSON file, max 500 KB.
    #[must_use]
    pub fn file<T: Into<AttachmentType<'a>>>(mut self, file: T) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Executes the request to create a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<Sticker> {
        let file = self.file.ok_or(Error::Model(ModelError::NoStickerFileSet))?;

        let mut map = Vec::with_capacity(3);
        if let Some(name) = self.fields.name {
            map.push(("name", name));
        }
        if let Some(tags) = self.fields.tags {
            map.push(("tags", tags));
        }
        if let Some(description) = self.fields.description {
            map.push(("description", description));
        }

        http.as_ref().create_sticker(self.id.into(), map, file, None).await
    }

    /// Executes the request to create a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise, returns [`Error::Http`] - see [`Self::execute`].
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn execute_with_cache(self, cache_http: impl CacheHttp) -> Result<Sticker> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(self.id) {
                    let req = Permissions::MANAGE_EMOJIS_AND_STICKERS;

                    if !guild.has_perms(&cache_http, req).await {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        self.execute(cache_http.http()).await
    }
}
