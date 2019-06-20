use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
    Write as FmtWrite
};
use super::super::id::{EmojiId, RoleId};

#[cfg(all(feature = "cache", feature = "model"))]
use serde_json::json;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use std::mem;
#[cfg(all(feature = "cache", feature = "model"))]
use super::super::ModelError;
#[cfg(all(feature = "cache", feature = "model"))]
use super::super::id::GuildId;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(all(feature = "cache", feature = "http"))]
use crate::http::client::Http;

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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Emoji {
    /// Deletes the emoji.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]:
    /// ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    ///
    /// # Examples
    ///
    /// Delete a given emoji:
    ///
    /// ```rust,no_run
    /// # extern crate serde_json;
    /// # extern crate serenity;
    /// #
    /// # use serde_json::json;
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::client::Context;
    /// # use serenity::model::prelude::{EmojiId, Emoji, Role};
    /// #
    /// # #[command]
    /// # fn example(ctx: &mut Context) -> CommandResult {
    /// #     let mut emoji = serde_json::from_value::<Emoji>(json!({
    /// #         "animated": false,
    /// #         "id": EmojiId(7),
    /// #         "name": "blobface",
    /// #         "managed": false,
    /// #         "require_colons": false,
    /// #         "roles": Vec::<Role>::new(),
    /// #     })).unwrap();
    /// #
    /// // assuming emoji has been set already
    /// match emoji.delete(&ctx) {
    ///     Ok(()) => println!("Emoji deleted."),
    ///     Err(_) => println!("Could not delete emoji.")
    /// }
    /// #    Ok(())
    /// # }
    /// #
    /// # fn main() { }
    /// ```
    #[cfg(all(feature = "cache", feature = "http"))]
    pub fn delete<T>(&self, cache_and_http: T) -> Result<()>
    where T: AsRef<CacheRwLock> + AsRef<Http> {
        match self.find_guild_id(&cache_and_http) {
            Some(guild_id) => AsRef::<Http>::as_ref(&cache_and_http)
                .delete_emoji(guild_id.0, self.id.0),
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Edits the emoji by updating it with a new name.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(all(feature = "cache", feature = "http"))]
    pub fn edit<T>(&mut self, cache_and_http: T, name: &str) -> Result<()>
    where T: AsRef<CacheRwLock> + AsRef<Http> {
        match self.find_guild_id(&cache_and_http) {
            Some(guild_id) => {
                let map = json!({
                    "name": name,
                });

                match AsRef::<Http>::as_ref(&cache_and_http)
                    .edit_emoji(guild_id.0, self.id.0, &map) {
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
    ///
    /// # Examples
    ///
    /// Print the guild id that owns this emoji:
    ///
    /// ```rust,no_run
    /// # extern crate serde_json;
    /// # extern crate serenity;
    /// #
    /// # use serde_json::json;
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::{guild::{Emoji, Role}, id::EmojiId}};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # fn main() {
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// #
    /// # let mut emoji = serde_json::from_value::<Emoji>(json!({
    /// #     "animated": false,
    /// #     "id": EmojiId(7),
    /// #     "name": "blobface",
    /// #     "managed": false,
    /// #     "require_colons": false,
    /// #     "roles": Vec::<Role>::new(),
    /// # })).unwrap();
    /// #
    /// // assuming emoji has been set already
    /// if let Some(guild_id) = emoji.find_guild_id(&cache) {
    ///     println!("{} is owned by {}", emoji.name, guild_id);
    /// }
    /// # }
    /// ```
    #[cfg(feature = "cache")]
    pub fn find_guild_id(&self, cache: impl AsRef<CacheRwLock>) -> Option<GuildId> {
        for guild in cache.as_ref().read().guilds.values() {
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
    /// # extern crate serde_json;
    /// # extern crate serenity;
    /// #
    /// # use serde_json::json;
    /// # use serenity::model::{guild::{Emoji, Role}, id::EmojiId};
    /// #
    /// # fn main() {
    /// # let mut emoji = serde_json::from_value::<Emoji>(json!({
    /// #     "animated": false,
    /// #     "id": EmojiId(7),
    /// #     "name": "blobface",
    /// #     "managed": false,
    /// #     "require_colons": false,
    /// #     "roles": Vec::<Role>::new(),
    /// # })).unwrap();
    /// #
    /// // assuming emoji has been set already
    /// println!("Direct link to emoji image: {}", emoji.url());
    /// # }
    /// ```
    #[inline]
    pub fn url(&self) -> String {
        let extension = if self.animated {"gif"} else {"png"};
        format!(cdn!("/emojis/{}.{}"), self.id, extension)
    }
}

impl Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to
    /// render the emoji.
    ///
    /// This is in the format of: `<:NAME:EMOJI_ID>`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
