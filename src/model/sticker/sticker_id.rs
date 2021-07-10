#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::prelude::*;

#[cfg(feature = "model")]
impl StickerId {
    /// Delete a guild sticker.
    ///
    /// **Note**: The [Manage Emojis and Stickers] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn delete(&self, http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<()> {
        guild_id.into().delete_sticker(http, self.0).await
    }

    /// Requests the sticker via the REST API to get a [`Sticker`] with all
    /// details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does
    /// not exist, or is otherwise unavailable.
    pub async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker> {
        http.as_ref().get_sticker(self.0).await
    }

    /// Edits the sticker.
    ///
    /// **Note**: The [Manage Emojis and Stickers] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if invalid edits are given.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn edit<F>(
        &self,
        http: impl AsRef<Http>,
        guild_id: impl Into<GuildId>,
        f: F,
    ) -> Result<Sticker>
    where
        F: FnOnce(&mut EditSticker) -> &mut EditSticker,
    {
        guild_id.into().edit_sticker(http, self.0, f).await
    }
}
