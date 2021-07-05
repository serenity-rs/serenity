#[cfg(feature = "model")]
use super::sticker_url;
#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::{
    id::{GuildId, StickerId, StickerPackId},
    user::User,
    utils::{deserialize_comma_separated_string, serialize_comma_separated_string},
};

/// A sticker sent with a message.
///
/// Bots cannot send stickers.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Sticker {
    /// The unique ID given to this sticker.
    pub id: StickerId,
    /// The unique ID of the pack the sticker is from.
    pub pack_id: Option<StickerPackId>,
    /// The name of the sticker.
    pub name: String,
    /// Description of the sticker
    pub description: Option<String>,
    /// For guild stickers, the Discord name of a unicode emoji representing the
    /// sticker's expression. For standard stickers, a list of
    /// related expressions.
    #[serde(
        serialize_with = "serialize_comma_separated_string",
        deserialize_with = "deserialize_comma_separated_string"
    )]
    pub tags: Vec<String>,
    /// The type of sticker.
    #[serde(rename = "type")]
    pub kind: StickerType,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
    /// Whether or not this guild sticker can be used, may be false due to loss
    /// of Server Boosts.
    #[serde(default)]
    pub available: bool,
    /// Id of the guild that owns this sticker.
    pub guild_id: Option<GuildId>,
    /// User that uploaded the sticker. This will be `None` if the current user
    /// does not have the [Manage Emojis and Stickers] permission.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub user: Option<User>,
    /// A sticker's sort order within a pack.
    pub sort_value: Option<u64>,
}

#[cfg(feature = "model")]
impl Sticker {
    /// Deletes a [`Sticker`] by Id from the guild.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to delete the sticker.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        if let Some(guild_id) = self.guild_id {
            guild_id.delete_sticker(&http, self.id).await
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }

    /// Edits a sticker, optionally setting its fields.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,ignore
    /// guild.edit_sticker(&context, StickerId(7), |r| r.name("Bun bun meow"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit<F>(&self, http: impl AsRef<Http>, f: F) -> Result<Sticker>
    where
        F: FnOnce(&mut EditSticker) -> &mut EditSticker,
    {
        if let Some(guild_id) = self.guild_id {
            guild_id.edit_sticker(&http, self.id, f).await
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }

    /// Retrieves the URL to the sticker image.
    ///
    /// **Note**: This will only be `None` if the format_type is unknown.
    #[inline]
    pub fn image_url(&self) -> Option<String> {
        sticker_url(self.id, self.format_type)
    }
}

/// Differentiates between sticker types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum StickerType {
    /// An official sticker in a pack, part of Nitro or in a removed purchasable
    /// pack.
    Standard = 1,
    /// A sticker uploaded to a Boosted guild for the guild's members.
    Guild = 2,
    /// Unknown sticker type.
    Unknown = !0,
}

enum_number!(StickerType {
    Standard,
    Guild
});

/// Differentiates between sticker formats.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum StickerFormatType {
    /// A PNG format sticker.
    Png = 1,
    /// An APNG format animated sticker.
    Apng = 2,
    /// A LOTTIE format animated sticker.
    Lottie = 3,
    /// Unknown sticker format type.
    Unknown = !0,
}

enum_number!(StickerFormatType {
    Png,
    Apng,
    Lottie
});
