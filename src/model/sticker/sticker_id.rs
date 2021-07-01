#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::utils;
use crate::{json::prelude::*, model::prelude::*};

#[cfg(feature = "model")]
impl StickerId {
    /// Delete a guild sticker.
    ///
    /// **Note**: The [Manage Emoji and Stickers] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Emoji and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJI_AND_STICKERS
    pub async fn delete(&self, http: impl AsRef<Http>, guild_id: u64) -> Result<()> {
        http.as_ref().delete_sticker(guild_id, self.0).await
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
    /// **Note**: The [Manage Emoji and Stickers] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if invalid edits are given.
    ///
    /// [Manage Emoji and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJI_AND_STICKERS
    pub async fn edit<F>(&self, http: impl AsRef<Http>, guild_id: u64, f: F) -> Result<Sticker>
    where
        F: FnOnce(&mut EditSticker) -> &mut EditSticker,
    {
        let mut edit_sticker = EditSticker::default();
        f(&mut edit_sticker);
        let map = utils::hashmap_to_json_map(edit_sticker.0);

        http.as_ref().edit_sticker(guild_id, self.0, &map).await
    }
}
