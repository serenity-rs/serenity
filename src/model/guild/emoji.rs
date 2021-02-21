use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};

#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
use crate::http::Http;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::json::json;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::model::id::GuildId;
use crate::model::id::{EmojiId, RoleId};
use crate::model::user::User;
use crate::model::utils::default_true;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::model::ModelError;

/// Represents a custom guild emoji, which can either be created using the API,
/// or via an integration. Emojis created using the API only work within the
/// guild it was created in.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Emoji {
    /// Whether the emoji is animated.
    #[serde(default)]
    pub animated: bool,
    /// Whether the emoji can be used. This may be false when the guild loses boosts,
    /// reducing the emoji limit.
    #[serde(default = "default_true")]
    pub available: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can
    /// only contain alphanumeric characters and underscores.
    pub name: String,
    /// Whether the emoji is managed via an [`Integration`] service.
    ///
    /// [`Integration`]: super::Integration
    #[serde(default)]
    pub managed: bool,
    /// Whether the emoji name needs to be surrounded by colons in order to be
    /// used by the client.
    #[serde(default)]
    pub require_colons: bool,
    /// A list of [`Role`]s that are allowed to use the emoji. If there are no
    /// roles specified, then usage is unrestricted.
    ///
    /// [`Role`]: super::Role
    #[serde(default)]
    pub roles: Vec<RoleId>,
    /// The user who created the emoji.
    pub user: Option<User>,
}

#[cfg(feature = "model")]
impl Emoji {
    /// Deletes the emoji.
    /// This method requires the cache to fetch the guild ID.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    ///
    /// # Examples
    ///
    /// Delete a given emoji:
    ///
    /// ```rust,no_run
    /// # use serde_json::json;
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::client::Context;
    /// # use serenity::model::prelude::{EmojiId, Emoji, Role};
    /// #
    /// # #[command]
    /// # async fn example(ctx: &Context) -> CommandResult {
    /// #     let mut emoji = serde_json::from_value::<Emoji>(json!({
    /// #         "animated": false,
    /// #         "id": EmojiId(7),
    /// #         "name": "blobface",
    /// #         "managed": false,
    /// #         "require_colons": false,
    /// #         "roles": Vec::<Role>::new(),
    /// #     }))?;
    /// #
    /// // assuming emoji has been set already
    /// match emoji.delete(&ctx).await {
    ///     Ok(()) => println!("Emoji deleted."),
    ///     Err(_) => println!("Could not delete emoji.")
    /// }
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or may return [`ModelError::ItemMissing`] if the emoji is not
    /// in the cache.
    ///
    /// [Manage Emojis]: crate::model::permissions::Permissions::MANAGE_EMOJIS
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn delete<T: AsRef<Cache> + AsRef<Http>>(&self, cache_http: T) -> Result<()> {
        match self.find_guild_id(&cache_http).await {
            Some(guild_id) => {
                AsRef::<Http>::as_ref(&cache_http).delete_emoji(guild_id.0, self.id.0).await
            },
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Edits the emoji by updating it with a new name.
    /// This method requires the cache to fetch the guild ID.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid name is given.
    ///
    /// [Manage Emojis]: crate::model::permissions::Permissions::MANAGE_EMOJIS
    #[cfg(feature = "cache")]
    pub async fn edit<T: AsRef<Cache> + AsRef<Http>>(
        &mut self,
        cache_http: T,
        name: &str,
    ) -> Result<()> {
        match self.find_guild_id(&cache_http).await {
            Some(guild_id) => {
                let map = json!({
                    "name": name,
                });

                *self = AsRef::<Http>::as_ref(&cache_http)
                    .edit_emoji(guild_id.0, self.id.0, &map)
                    .await?;

                Ok(())
            },
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Finds the [`Guild`] that owns the emoji by looking through the Cache.
    ///
    /// [`Guild`]: super::Guild
    ///
    /// # Examples
    ///
    /// Print the guild id that owns this emoji:
    ///
    /// ```rust,no_run
    /// # use serde_json::json;
    /// # use serenity::{cache::Cache, model::{guild::{Emoji, Role}, id::EmojiId}};
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let cache = Cache::default();
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
    /// if let Some(guild_id) = emoji.find_guild_id(&cache).await {
    ///     println!("{} is owned by {}", emoji.name, guild_id);
    /// }
    /// # }
    /// ```
    #[cfg(feature = "cache")]
    pub async fn find_guild_id(&self, cache: impl AsRef<Cache>) -> Option<GuildId> {
        for guild in cache.as_ref().guilds.read().await.values() {
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
        let extension = if self.animated { "gif" } else { "png" };
        format!(cdn!("/emojis/{}.{}"), self.id, extension)
    }
}

impl Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to
    /// render the emoji.
    ///
    /// This is in the format of either `<:NAME:EMOJI_ID>` for normal emojis,
    /// or `<a:NAME:EMOJI_ID>` for animated emojis.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.animated {
            f.write_str("<a:")?;
        } else {
            f.write_str("<:")?;
        }
        f.write_str(&self.name)?;
        FmtWrite::write_char(f, ':')?;
        Display::fmt(&self.id, f)?;
        FmtWrite::write_char(f, '>')
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
