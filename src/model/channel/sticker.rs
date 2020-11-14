use crate::model::id::{StickerId, StickerPackId};

/// A sticker sent with a message
///
/// Bots currently can only receive messages with stickers, not send
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// Differentiates between sticker formats
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum StickerFormatType {
    /// A PNG format sticker.
    PNG = 0,
    /// An APNG format animated sticker.
    APNG = 1,
    /// A LOTTIE format animated sticker.
    LOTTIE = 2,
}

enum_number!(
    StickerFormatType {
        PNG,
        APNG,
        LOTTIE,
    }
);

