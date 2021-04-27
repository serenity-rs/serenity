use crate::model::id::{StickerId, StickerPackId};

/// A sticker sent with a message.
///
/// Bots currently can only receive messages with stickers, not send.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Sticker {
    /// The unique ID given to this sticker.
    pub id: StickerId,
    /// The unique ID of the pack the sticker is from.
    pub pack_id: StickerPackId,
    /// The name of the sticker.
    pub name: String,
    /// Description of the sticker
    pub description: String,
    /// A comma-separated list of tags for the sticker.
    pub tags: Option<String>,
    /// The sticker asset hash.
    pub asset: String,
    /// The sticker preview asset hash.
    pub preview_asset: Option<String>,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
}

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
