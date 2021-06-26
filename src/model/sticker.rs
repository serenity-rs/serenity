use crate::model::{
    id::{GuildId, StickerId, StickerPackId},
    user::User,
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
    pub description: String,
    /// For guild stickers, a unicode emoji representing the sticker's
    /// expression. For nitro stickers, a comma-separated list of related
    /// expressions.
    pub tags: String,
    /// The type of sticker.
    #[serde(rename = "type")]
    pub kind: StickerType,
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

/// The smallest amount of data required to render a sticker.
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
