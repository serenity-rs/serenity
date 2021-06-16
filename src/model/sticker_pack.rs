use crate::model::{
    id::{SkuId, StickerId, StickerPackBannerId, StickerPackId},
    sticker::Sticker,
};

/// A sticker sent with a message.
///
/// Bots currently can only receive messages with stickers, not send.
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
    pub cover_sticker_id: StickerId,
    /// Description of the sticker pack.
    pub description: String,
    /// The unique ID given to the sticker pack's banner image.
    pub banner_asset_id: StickerPackBannerId,
}
