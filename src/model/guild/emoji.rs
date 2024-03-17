use std::fmt;

use crate::internal::prelude::*;
use crate::model::id::{EmojiId, RoleId};
use crate::model::user::User;
use crate::model::utils::default_true;

/// Represents a custom guild emoji, which can either be created using the API, or via an
/// integration. Emojis created using the API only work within the guild it was created in.
///
/// [Discord docs](https://discord.com/developers/docs/resources/emoji#emoji-object).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Emoji {
    /// Whether the emoji is animated.
    #[serde(default)]
    pub animated: bool,
    /// Whether the emoji can be used. This may be false when the guild loses boosts, reducing the
    /// emoji limit.
    #[serde(default = "default_true")]
    pub available: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can only contain
    /// alphanumeric characters and underscores.
    pub name: FixedString,
    /// Whether the emoji is managed via an [`Integration`] service.
    ///
    /// [`Integration`]: super::Integration
    #[serde(default)]
    pub managed: bool,
    /// Whether the emoji name needs to be surrounded by colons in order to be used by the client.
    #[serde(default)]
    pub require_colons: bool,
    /// A list of [`Role`]s that are allowed to use the emoji. If there are no roles specified,
    /// then usage is unrestricted.
    ///
    /// [`Role`]: super::Role
    #[serde(default)]
    pub roles: FixedArray<RoleId>,
    /// The user who created the emoji.
    pub user: Option<User>,
}

#[cfg(feature = "model")]
impl Emoji {
    /// Generates a URL to the emoji's image.
    ///
    /// # Examples
    ///
    /// Print the direct link to the given emoji:
    ///
    /// ```rust,no_run
    /// # use serenity::model::guild::Emoji;
    /// #
    /// # fn run(emoji: Emoji) {
    /// // assuming emoji has been set already
    /// println!("Direct link to emoji image: {}", emoji.url());
    /// # }
    /// ```
    #[must_use]
    pub fn url(&self) -> String {
        let extension = if self.animated() { "gif" } else { "png" };
        cdn!("/emojis/{}.{}", self.id, extension)
    }
}

impl fmt::Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to render the emoji.
    ///
    /// This is in the format of either `<:NAME:EMOJI_ID>` for normal emojis, or
    /// `<a:NAME:EMOJI_ID>` for animated emojis.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.animated() {
            f.write_str("<a:")?;
        } else {
            f.write_str("<:")?;
        }
        f.write_str(&self.name)?;
        fmt::Write::write_char(f, ':')?;
        fmt::Display::fmt(&self.id, f)?;
        fmt::Write::write_char(f, '>')
    }
}

impl ExtractKey<EmojiId> for Emoji {
    fn extract_key(&self) -> &EmojiId {
        &self.id
    }
}

impl From<Emoji> for EmojiId {
    /// Gets the Id of an [`Emoji`].
    fn from(emoji: Emoji) -> EmojiId {
        emoji.id
    }
}

impl From<&Emoji> for EmojiId {
    /// Gets the Id of an [`Emoji`].
    fn from(emoji: &Emoji) -> EmojiId {
        emoji.id
    }
}
