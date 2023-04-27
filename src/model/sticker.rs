#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::comma_separated_string;

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
    pub async fn delete(self, http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<()> {
        guild_id.into().delete_sticker(http, self).await
    }

    /// Requests the sticker via the REST API to get a [`Sticker`] with all details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does not exist, or is
    /// otherwise unavailable.
    pub async fn to_sticker(self, http: impl AsRef<Http>) -> Result<Sticker> {
        http.as_ref().get_sticker(self).await
    }

    /// Edits the sticker.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn edit(
        self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker> {
        guild_id.into().edit_sticker(cache_http, self, builder).await
    }
}

/// The smallest amount of data required to render a sticker.
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-item-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StickerItem {
    /// The unique ID given to this sticker.
    pub id: StickerId,
    /// The name of the sticker.
    pub name: String,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
}

#[cfg(feature = "model")]
impl StickerItem {
    /// Requests the sticker via the REST API to get a [`Sticker`] with all details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does
    /// not exist, or is otherwise unavailable.
    #[inline]
    pub async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker> {
        self.id.to_sticker(http).await
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

/// A sticker sent with a message.
///
/// Bots currently can only receive messages with stickers, not send.
///
/// [Discord docs](https://discord.com/developers/docs/resources/sticker#sticker-pack-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StickerPack {
    /// The unique ID given to this sticker sticker pack.
    pub id: StickerPackId,
    /// The stickers in the pack
    pub stickers: Vec<Sticker>,
    /// The name of the sticker pack
    pub name: String,
    /// The unique ID given to the pack's SKU.
    pub sku_id: SkuId,
    /// ID of a sticker in the pack which is shown as the pack's icon.
    pub cover_sticker_id: Option<StickerId>,
    /// Description of the sticker pack.
    pub description: String,
    /// The unique ID given to the sticker pack's banner image.
    pub banner_asset_id: StickerPackBannerId,
}

#[cfg(feature = "model")]
impl StickerPack {
    /// Returns the sticker that is shown as the pack's icon
    #[must_use]
    pub fn cover_sticker(&self) -> Option<&Sticker> {
        self.cover_sticker_id.and_then(|id| self.stickers.iter().find(|s| s.id == id))
    }

    #[must_use]
    pub fn banner_url(&self) -> String {
        banner_url(self.banner_asset_id)
    }
}

#[cfg(feature = "model")]
fn banner_url(banner_asset_id: StickerPackBannerId) -> String {
    cdn!("/app-assets/710982414301790216/store/{}.webp?size=1024", banner_asset_id.0)
}

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
    /// For guild stickers, the Discord name of a unicode emoji representing the sticker's
    /// expression. For standard stickers, a list of related expressions.
    #[serde(with = "comma_separated_string")]
    pub tags: Vec<String>,
    /// The type of sticker.
    #[serde(rename = "type")]
    pub kind: StickerType,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
    /// Whether or not this guild sticker can be used, may be false due to loss of Server Boosts.
    #[serde(default)]
    pub available: bool,
    /// Id of the guild that owns this sticker.
    pub guild_id: Option<GuildId>,
    /// User that uploaded the sticker. This will be `None` if the current user does not have the
    /// [Manage Emojis and Stickers] permission.
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
    /// Returns [`Error::Http`] if the current user lacks permission to delete the sticker.
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
    /// # use serenity::model::id::GuildId;
    /// # use serenity::model::sticker::Sticker;
    /// use serenity::builder::EditSticker;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut sticker: Sticker = unimplemented!();
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
    pub async fn edit(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditSticker<'_>,
    ) -> Result<()> {
        if let Some(guild_id) = self.guild_id {
            *self = self.id.edit(cache_http, guild_id, builder).await?;
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
        /// An official sticker in a pack, part of Nitro or in a removed purchasable pack.
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
        /// A GIF format animated sticker.
        Gif = 4,
        _ => Unknown(u8),
    }
}

#[cfg(feature = "model")]
fn sticker_url(sticker_id: StickerId, sticker_format_type: StickerFormatType) -> Option<String> {
    let ext = match sticker_format_type {
        StickerFormatType::Png | StickerFormatType::Apng => "png",
        StickerFormatType::Lottie => "json",
        StickerFormatType::Gif => "gif",
        StickerFormatType::Unknown(_) => return None,
    };

    Some(cdn!("/stickers/{}.{}", sticker_id, ext))
}
