use crate::model::{id::{StickerId, StickerPackId, GuildId}, user::User};

/// A sticker sent with a message.
///
/// Bots currently can only receive messages with stickers, not send.
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
    pub description: String,
    /// A unicode emoji representing the sticker's expression.
    pub tags: String,
    /// Previously the sticker asset hash, now an empty string.
    #[deprecated]
    pub asset: String,
    /// The type of sticker format.
    pub format_type: StickerFormatType,
    /// Whether or not the sticker is available.
    #[serde(default)]
    pub available: bool,
    /// Id of the guild that owns this sticker.
    pub guild_id: Option<GuildId>,
    /// User that uploaded the sticker.
    pub user: Option<User>,
    /// A sticker's sort order within a pack.
    pub sort_value: Option<u64>,
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
