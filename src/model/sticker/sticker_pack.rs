use crate::model::id::{SkuId, StickerId, StickerPackBannerId, StickerPackId};
use crate::model::sticker::Sticker;

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
