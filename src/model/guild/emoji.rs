use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use super::super::id::{EmojiId, RoleId};

#[cfg(all(feature = "cache", feature = "model"))]
use internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use std::mem;
#[cfg(all(feature = "cache", feature = "model"))]
use super::super::ModelError;
#[cfg(all(feature = "cache", feature = "model"))]
use super::super::id::GuildId;
#[cfg(all(feature = "cache", feature = "model"))]
use {CACHE, http};

/// Represents a custom guild emoji, which can either be created using the API,
/// or via an integration. Emojis created using the API only work within the
/// guild it was created in.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Emoji {
    /// Whether the emoji is animated.
    #[serde(default)]
    pub animated: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can
    /// only contain alphanumeric characters and underscores.
    pub name: String,
    /// Whether the emoji is managed via an [`Integration`] service.
    ///
    /// [`Integration`]: struct.Integration.html
    pub managed: bool,
    /// Whether the emoji name needs to be surrounded by colons in order to be
    /// used by the client.
    pub require_colons: bool,
    /// A list of [`Role`]s that are allowed to use the emoji. If there are no
    /// roles specified, then usage is unrestricted.
    ///
    /// [`Role`]: struct.Role.html
    pub roles: Vec<RoleId>,
}

#[cfg(feature = "model")]
impl Emoji {
    /// Finds the [`Guild`] that owns the emoji by looking through the Cache.
    ///
    /// [`Guild`]: struct.Guild.html
    ///
    /// # Examples
    ///
    /// Print the guild id that owns this emoji:
    ///
    /// ```rust,no_run
    /// # use serenity::model::guild::Emoji;
    /// # use serenity::model::id::EmojiId;
    /// #
    /// # let mut emoji = Emoji {
    /// #     animated: false,
    /// #     id: EmojiId(7),
    /// #     name: String::from("blobface"),
    /// #     managed: false,
    /// #     require_colons: false,
    /// #     roles: vec![],
    /// # };
    /// #
    /// // assuming emoji has been set already
    /// if let Some(guild_id) = emoji.find_guild_id() {
    ///     println!("{} is owned by {}", emoji.name, guild_id);
    /// }
    /// ```
    #[cfg(feature = "cache")]
    pub fn find_guild_id(&self) -> Option<GuildId> {
        for guild in CACHE.read().guilds.values() {
            let guild = guild.read();

            if guild.emojis.contains_key(&self.id) {
                return Some(guild.id);
            }
        }

        None
    }

    /// Generates a URL to the emoji's image.
    ///
    /// # Examples
    ///
    /// Print the direct link to the given emoji:
    ///
    /// ```rust,no_run
    /// # use serenity::model::guild::Emoji;
    /// # use serenity::model::id::EmojiId;
    /// #
    /// # let mut emoji = Emoji {
    /// #     animated: false,
    /// #     id: EmojiId(7),
    /// #     name: String::from("blobface"),
    /// #     managed: false,
    /// #     require_colons: false,
    /// #     roles: vec![],
    /// # };
    /// #
    /// // assuming emoji has been set already
    /// println!("Direct link to emoji image: {}", emoji.url());
    /// ```
    #[inline]
    pub fn url(&self) -> String { format!(cdn!("/emojis/{}.png"), self.id) }
}

impl Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to
    /// render the emoji.
    ///
    /// This is in the format of: `<:NAME:EMOJI_ID>`.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("<:")?;
        f.write_str(&self.name)?;
        FmtWrite::write_char(f, ':')?;
        Display::fmt(&self.id, f)?;
        FmtWrite::write_char(f, '>')
    }
}

impl From<Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: Emoji) -> EmojiId { emoji.id }
}

impl<'a> From<&'a Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: &Emoji) -> EmojiId { emoji.id }
}
