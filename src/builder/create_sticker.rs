#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create a [`Sticker`] for use via a number of model methods.
///
/// These are:
///
/// - [`PartialGuild::create_sticker`]
/// - [`Guild::create_sticker`]
/// - [`GuildId::create_sticker`]
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct CreateSticker<'a> {
    name: Option<String>,
    tags: Option<String>,
    description: Option<String>,

    file: Option<AttachmentType<'a>>,
}

impl<'a> CreateSticker<'a> {
    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn execute(self, cache_http: impl CacheHttp, guild_id: GuildId) -> Result<Sticker> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(guild_id) {
                    let req = Permissions::MANAGE_EMOJIS_AND_STICKERS;

                    if !guild.has_perms(&cache_http, req).await {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        self._execute(cache_http.http(), guild_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, guild_id: GuildId) -> Result<Sticker> {
        let file = self.file.ok_or(Error::Model(ModelError::NoStickerFileSet))?;

        let mut map = Vec::with_capacity(3);
        if let Some(name) = self.name {
            map.push(("name".to_string(), name));
        }
        if let Some(tags) = self.tags {
            map.push(("tags".to_string(), tags));
        }
        if let Some(description) = self.description {
            map.push(("description".to_string(), description));
        }

        http.create_sticker(guild_id.into(), map, file, None).await
    }

    /// The name of the sticker to set.
    ///
    /// **Note**: Must be between 2 and 30 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The description of the sticker.
    ///
    /// **Note**: If not empty, must be between 2 and 100 characters long.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The Discord name of a unicode emoji representing the sticker's expression.
    ///
    /// **Note**: Must be between 2 and 200 characters long.
    pub fn tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    /// The sticker file.
    ///
    /// **Note**: Must be a PNG, APNG, or Lottie JSON file, max 500 KB.
    pub fn file<T: Into<AttachmentType<'a>>>(mut self, file: T) -> Self {
        self.file = Some(file.into());
        self
    }
}
