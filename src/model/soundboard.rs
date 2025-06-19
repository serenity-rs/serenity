#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;

/// A representation of a soundboard sound, a kind of audio that users can play
/// in voice channels.
///
/// [Discord docs](https://discord.com/developers/docs/resources/soundboard#soundboard-resource).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Soundboard {
    /// The unique Id of the soundboard sound. Can be used to calculate the
    /// creation date of the soundboard sound.
    #[serde(rename = "sound_id")]
    pub id: SoundId,
    /// The name of this soundboard sound.
    pub name: String,
    /// Volume of this soundboard sound. The valid range is from `0` to `1`.
    pub volume: f64,
    /// Id of the emoji for this soundboard sound.
    pub emoji_id: Option<EmojiId>,
    /// Unicode character of the emoji for this soundboard sound.
    pub emoji_name: Option<String>,
    /// Id of the guild this soundboard sound belongs to.
    pub guild_id: Option<GuildId>,
    /// Whether this soundboard sound may be used. Can be `false` if the guild
    /// lost Nitro boosts, for instance.
    pub available: bool,
    /// User who created this soundboard sound.
    pub user: Option<User>,
}

#[cfg(feature = "model")]
impl SoundId {
    /// Performs a HTTP request to fetch soundboard sound data from a guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    pub async fn to_soundboard(
        self,
        http: impl AsRef<Http>,
        guild_id: GuildId,
    ) -> Result<Soundboard> {
        guild_id.get_soundboard(http, self).await
    }
}
