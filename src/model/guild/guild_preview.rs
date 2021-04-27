use crate::model::guild::Emoji;
use crate::model::id::GuildId;

/// Preview [`Guild`] information.
///
/// [`Guild`]: super::Guild
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GuildPreview {
    /// The guild Id.
    pub id: GuildId,
    /// The guild name.
    pub name: String,
    /// The guild icon hash if it has one.
    pub icon: Option<String>,
    /// The guild splash hash if it has one.
    pub splash: Option<String>,
    /// The guild discovery splash hash it it has one.
    pub discovery_splash: Option<String>,
    /// The custom guild emojis.
    pub emojis: Vec<Emoji>,
    /// The guild features. See [`Guild::features`]
    ///
    /// [`Guild::features`]: super::Guild::features
    pub features: Vec<String>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: u64,
    /// Approximate number of online members in this guild.
    pub approximate_presence_count: u64,
    /// The description for the guild, if the guild has the `DISCOVERABLE` feature.
    pub description: Option<String>,
}
