pub mod sticker;
pub mod sticker_id;
pub mod sticker_item;
pub mod sticker_pack;

pub use self::{sticker::*, sticker_id::*, sticker_item::*, sticker_pack::*};
#[cfg(feature = "model")]
use crate::model::prelude::*;

#[cfg(feature = "model")]
fn sticker_url(sticker_id: StickerId, sticker_format_type: StickerFormatType) -> Option<String> {
    let ext = match sticker_format_type {
        StickerFormatType::Png | StickerFormatType::Apng => "png",
        StickerFormatType::Lottie => "json",
        StickerFormatType::Unknown => return None,
    };

    Some(cdn!("/stickers/{}.{}", sticker_id.0, ext))
}
