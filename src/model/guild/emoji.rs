use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use super::super::{EmojiId, ModelError, RoleId};

#[cfg(feature="cache")]
use std::mem;
#[cfg(feature="cache")]
use ::CACHE;
#[cfg(feature="cache")]
use ::internal::prelude::*;
#[cfg(feature="model")]
use ::http;
#[cfg(feature="cache")]
use super::super::GuildId;

/// Represents a custom guild emoji, which can either be created using the API,
/// or via an integration. Emojis created using the API only work within the
/// guild it was created in.
#[derive(Clone, Debug, Deserialize)]
pub struct Emoji {
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

#[cfg(feature="model")]
impl Emoji {
    /// Deletes the emoji.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[cfg(feature="cache")]
    pub fn delete(&self) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => http::delete_emoji(guild_id.0, self.id.0),
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Edits the emoji by updating it with a new name.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[cfg(feature="cache")]
    pub fn edit(&mut self, name: &str) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => {
                let map = json!({
                    "name": name,
                });

                match http::edit_emoji(guild_id.0, self.id.0, &map) {
                    Ok(emoji) => {
                        mem::replace(self, emoji);

                        Ok(())
                    },
                    Err(why) => Err(why),
                }
            },
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Finds the [`Guild`] that owns the emoji by looking through the Cache.
    ///
    /// [`Guild`]: struct.Guild.html
    #[cfg(feature="cache")]
    pub fn find_guild_id(&self) -> Option<GuildId> {
        for guild in CACHE.read().unwrap().guilds.values() {
            let guild = guild.read().unwrap();

            if guild.emojis.contains_key(&self.id) {
                return Some(guild.id);
            }
        }

        None
    }

    /// Generates a URL to the emoji's image.
    #[inline]
    pub fn url(&self) -> String {
        format!(cdn!("/emojis/{}.png"), self.id)
    }
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

impl Display for EmojiId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl From<Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: Emoji) -> EmojiId {
        emoji.id
    }
}

impl<'a> From<&'a Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: &Emoji) -> EmojiId {
        emoji.id
    }
}
