#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::id::{GuildId, StickerId, StickerPackId};
#[cfg(feature = "model")]
use crate::model::prelude::*;
use crate::model::user::User;
use crate::model::utils::comma_separated_string;

pub mod sticker_id;
pub mod sticker_item;
pub mod sticker_pack;

pub use self::sticker_id::*;
pub use self::sticker_item::*;
pub use self::sticker_pack::*;

/// A sticker sent with a message.
///
/// Bots cannot send stickers.
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-object).
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
    #[serde(with = "comma_separated_string")]
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
            guild_id.delete_sticker(http, self.id).await
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }

    /// Edits the sticker.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::id::{GuildId, StickerId};
    /// use serenity::builder::EditSticker;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let mut sticker = GuildId::new(7).sticker(&http, StickerId::new(7)).await?;
    /// let builder = EditSticker::new().name("Bun bun meow");
    /// sticker.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit(&mut self, http: impl AsRef<Http>, builder: EditSticker<'_>) -> Result<()> {
        if let Some(guild_id) = self.guild_id {
            *self = self.id.edit(http, guild_id, builder).await?;
            Ok(())
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }

    /// Retrieves the URL to the sticker image.
    ///
    /// **Note**: This will only be `None` if the format_type is unknown.
    #[inline]
    #[must_use]
    pub fn image_url(&self) -> Option<String> {
        sticker_url(self.id, self.format_type)
    }
}

enum_number! {
    /// Differentiates between sticker types.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-object-sticker-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum StickerType {
        /// An official sticker in a pack, part of Nitro or in a removed purchasable
        /// pack.
        Standard = 1,
        /// A sticker uploaded to a Boosted guild for the guild's members.
        Guild = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Differentiates between sticker formats.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-object-sticker-format-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum StickerFormatType {
        /// A PNG format sticker.
        Png = 1,
        /// An APNG format animated sticker.
        Apng = 2,
        /// A LOTTIE format animated sticker.
        Lottie = 3,
        _ => Unknown(u8),
    }
}

#[cfg(feature = "model")]
fn sticker_url(sticker_id: StickerId, sticker_format_type: StickerFormatType) -> Option<String> {
    let ext = match sticker_format_type {
        StickerFormatType::Png | StickerFormatType::Apng => "png",
        StickerFormatType::Lottie => "json",
        StickerFormatType::Unknown(_) => return None,
    };

    Some(cdn!("/stickers/{}.{}", sticker_id, ext))
}
