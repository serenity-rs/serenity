//! User information-related models.

#[cfg(feature = "http")]
use crate::http::CacheHttp;
use serde_json;
use std::fmt;
use super::utils::deserialize_u16;
use super::prelude::*;
use crate::{internal::prelude::*, model::misc::Mentionable};

#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditProfile};
#[cfg(feature = "model")]
use crate::http::GuildPagination;
#[cfg(all(feature = "cache", feature = "model"))]
use parking_lot::RwLock;
#[cfg(feature = "model")]
use std::fmt::Write;
#[cfg(feature = "model")]
use std::mem;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(all(
    feature = "model",
    any(
        all(feature = "builder", feature = "client"),
        feature = "http",
    ),
))]
use serde_json::json;
#[cfg(all(feature = "cache", feature = "model"))]
use std::sync::Arc;
#[cfg(feature = "model")]
use crate::utils;
#[cfg(feature = "http")]
use crate::http::Http;

/// Information about the current user.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct CurrentUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)] pub bot: bool,
    #[serde(deserialize_with = "deserialize_u16")] pub discriminator: u16,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    #[serde(rename = "username")] pub name: String,
    pub verified: bool,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl CurrentUser {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    ///
    /// # Examples
    ///
    /// Print out the current user's avatar url if one is set:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let cache = cache.read();
    /// // assuming the cache has been unlocked
    /// let user = &cache.user;
    ///
    /// match user.avatar_url() {
    ///     Some(url) => println!("{}'s avatar can be found at {}", user.name, url),
    ///     None => println!("{} does not have an avatar set.", user.name)
    /// }
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn avatar_url(&self) -> Option<String> { avatar_url(self.id, self.avatar.as_ref()) }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    pub fn default_avatar_url(&self) -> String { default_avatar_url(self.discriminator) }

    /// Edits the current user's profile settings.
    ///
    /// This mutates the current user in-place.
    ///
    /// Refer to [`EditProfile`]'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the avatar:
    ///
    /// ```rust,ignore
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// let avatar = serenity::utils::read_image("./avatar.png").unwrap();
    ///
    /// context.cache.write().user.edit(|p| p.avatar(Some(&avatar)));
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    ///
    /// [`EditProfile`]: ../../builder/struct.EditProfile.html
    #[cfg(feature = "http")]
    pub fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<()>
        where F: FnOnce(&mut EditProfile) -> &mut EditProfile {
        let mut map = HashMap::new();
        map.insert("username", Value::String(self.name.clone()));

        if let Some(email) = self.email.as_ref() {
            map.insert("email", Value::String(email.clone()));
        }

        let mut edit_profile = EditProfile(map);
        f(&mut edit_profile);
        let map = utils::hashmap_to_json_map(edit_profile.0);

        match http.as_ref().edit_profile(&map) {
            Ok(new) => {
                let _ = mem::replace(self, new);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Retrieves the URL to the current user's avatar, falling back to the
    /// default avatar if needed.
    ///
    /// This will call [`avatar_url`] first, and if that returns `None`, it
    /// then falls back to [`default_avatar_url`].
    ///
    /// [`avatar_url`]: #method.avatar_url
    /// [`default_avatar_url`]: #method.default_avatar_url
    pub fn face(&self) -> String {
        self.avatar_url()
            .unwrap_or_else(|| self.default_avatar_url())
    }

    /// Gets a list of guilds that the current user is in.
    ///
    /// # Examples
    ///
    /// Print out the names of all guilds the current user is in:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let cache = cache.read();
    /// # let http = Arc::new(Http::default());
    /// // assuming the cache has been unlocked
    /// let user = &cache.user;
    ///
    /// if let Ok(guilds) = user.guilds(&http) {
    ///     for (index, guild) in guilds.into_iter().enumerate() {
    ///         println!("{}: {}", index, guild.name);
    ///     }
    /// }
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    #[cfg(feature = "http")]
    pub fn guilds(&self, http: impl AsRef<Http>) -> Result<Vec<GuildInfo>> {
        http.as_ref().get_guilds(&GuildPagination::After(GuildId(1)), 100)
    }

    /// Returns the invite url for the bot with the given permissions.
    ///
    /// This queries the REST API for the client id.
    ///
    /// If the permissions passed are empty, the permissions part will be dropped.
    ///
    /// # Examples
    ///
    /// Get the invite url with no permissions set:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let mut cache = cache.write();
    /// # let http = Arc::new(Http::default());
    ///
    /// use serenity::model::Permissions;
    ///
    /// // assuming the cache has been unlocked
    /// let url = match cache.user.invite_url(&http, Permissions::empty()) {
    ///     Ok(v) => v,
    ///     Err(why) => {
    ///         println!("Error getting invite url: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    ///
    /// assert_eq!(url, "https://discordapp.com/api/oauth2/authorize? \
    ///                  client_id=249608697955745802&scope=bot");
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    ///
    /// Get the invite url with some basic permissions set:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, http::Http, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let mut cache = cache.write();
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::Permissions;
    ///
    /// // assuming the cache has been unlocked
    /// let url = match cache.user.invite_url(&http, Permissions::READ_MESSAGES | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS) {
    ///     Ok(v) => v,
    ///     Err(why) => {
    ///         println!("Error getting invite url: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    ///
    /// assert_eq!(url,
    /// "https://discordapp.
    /// com/api/oauth2/authorize?client_id=249608697955745802&scope=bot&permissions=19456");
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(Unauthorized)`][`HttpError::UnsuccessfulRequest`]
    /// If the user is not authorized for this end point.
    ///
    /// May return [`Error::Format`] while writing url to the buffer.
    ///
    /// [`Error::Format`]: ../../enum.Error.html#variant.Format
    /// [`HttpError::UnsuccessfulRequest`]: ../../http/enum.HttpError.html#variant.UnsuccessfulRequest
    #[cfg(feature = "http")]
    pub fn invite_url(&self, http: impl AsRef<Http>, permissions: Permissions) -> Result<String> {
        let bits = permissions.bits();
        let client_id = http.as_ref().get_current_application_info().map(|v| v.id)?;

        let mut url = format!(
            "https://discordapp.com/api/oauth2/authorize?client_id={}&scope=bot",
            client_id
        );

        if bits != 0 {
            write!(url, "&permissions={}", bits)?;
        }

        Ok(url)
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    ///
    /// # Examples
    ///
    /// Print out the current user's static avatar url if one is set:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let cache = cache.read();
    /// // assuming the cache has been unlocked
    /// let user = &cache.user;
    ///
    /// match user.static_avatar_url() {
    ///     Some(url) => println!("{}'s static avatar can be found at {}", user.name, url),
    ///     None => println!("Could not get static avatar for {}.", user.name)
    /// }
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn static_avatar_url(&self) -> Option<String> {
        static_avatar_url(self.id, self.avatar.as_ref())
    }

    /// Returns the tag of the current user.
    ///
    /// # Examples
    ///
    /// Print out the current user's distinct identifier (e.g., Username#1234):
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "cache")]
    /// # fn main() {
    /// # use serenity::{cache::{Cache, CacheRwLock}, model::prelude::*, prelude::*};
    /// # use parking_lot::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache: CacheRwLock = Arc::new(RwLock::new(Cache::default())).into();
    /// # let cache = cache.read();
    /// // assuming the cache has been unlocked
    /// println!("The current user's distinct identifier is {}", cache.user.tag());
    /// # }
    /// #
    /// # #[cfg(not(feature = "cache"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn tag(&self) -> String { tag(&self.name, self.discriminator) }
}

/// An enum that represents a default avatar.
///
/// The default avatar is calculated via the result of `discriminator % 5`.
///
/// The has of the avatar can be retrieved via calling [`name`] on the enum.
///
/// [`name`]: #method.name
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub enum DefaultAvatar {
    /// The avatar when the result is `0`.
    #[serde(rename = "6debd47ed13483642cf09e832ed0bc1b")]
    Blurple,
    /// The avatar when the result is `1`.
    #[serde(rename = "322c936a8c8be1b803cd94861bdfa868")]
    Grey,
    /// The avatar when the result is `2`.
    #[serde(rename = "dd4dbc0016779df1378e7812eabaa04d")]
    Green,
    /// The avatar when the result is `3`.
    #[serde(rename = "0e291f67c9274a1abdddeb3fd919cbaa")]
    Orange,
    /// The avatar when the result is `4`.
    #[serde(rename = "1cbd08c76f8af6dddce02c5138971129")]
    Red,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl DefaultAvatar {
    /// Retrieves the String hash of the default avatar.
    pub fn name(self) -> Result<String> { serde_json::to_string(&self).map_err(From::from) }
}

/// The representation of a user's status.
///
/// # Examples
///
/// - [`DoNotDisturb`];
/// - [`Invisible`].
///
/// [`DoNotDisturb`]: #variant.DoNotDisturb
/// [`Invisible`]: #variant.Invisible
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum OnlineStatus {
    #[serde(rename = "dnd")] DoNotDisturb,
    #[serde(rename = "idle")] Idle,
    #[serde(rename = "invisible")] Invisible,
    #[serde(rename = "offline")] Offline,
    #[serde(rename = "online")] Online,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl OnlineStatus {
    pub fn name(&self) -> &str {
        match *self {
            OnlineStatus::DoNotDisturb => "dnd",
            OnlineStatus::Idle => "idle",
            OnlineStatus::Invisible => "invisible",
            OnlineStatus::Offline => "offline",
            OnlineStatus::Online => "online",
            OnlineStatus::__Nonexhaustive => unreachable!(),
        }
    }
}

impl Default for OnlineStatus {
    fn default() -> OnlineStatus { OnlineStatus::Online }
}

/// Information about a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    /// The unique Id of the user. Can be used to calculate the account's
    /// creation date.
    pub id: UserId,
    /// Optional avatar hash.
    pub avatar: Option<String>,
    /// Indicator of whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
    /// The account's discriminator to differentiate the user from others with
    /// the same [`name`]. The name+discriminator pair is always unique.
    ///
    /// [`name`]: #structfield.name
    #[serde(deserialize_with = "deserialize_u16")]
    pub discriminator: u16,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique.
    #[serde(rename = "username")]
    pub name: String,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

use std::hash::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for User {}

impl Hash for User {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

#[cfg(feature = "model")]
impl User {
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    #[inline]
    pub fn avatar_url(&self) -> Option<String> { avatar_url(self.id, self.avatar.as_ref()) }

    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: struct.CurrentUser.html
    #[inline]
    #[cfg(feature = "http")]
    pub fn create_dm_channel(&self, http: impl AsRef<Http>) -> Result<PrivateChannel> { self.id.create_dm_channel(&http) }

    /// Retrieves the time that this user was created at.
    #[inline]
    pub fn created_at(&self) -> DateTime<FixedOffset> { self.id.created_at() }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    pub fn default_avatar_url(&self) -> String { default_avatar_url(self.discriminator) }

    /// Sends a message to a user through a direct message channel. This is a
    /// channel that can only be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// When a user sends a message with a content of `"~help"`, DM the author a
    /// help message, and then react with `'👌'` to verify message sending:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature="client")] {
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// use serenity::model::Permissions;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    /// #   #[cfg(feature = "cache")]
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "~help" {
    ///             let url = match ctx.cache.read().user.invite_url(&ctx, Permissions::empty()) {
    ///                 Ok(v) => v,
    ///                 Err(why) => {
    ///                     println!("Error creating invite url: {:?}", why);
    ///
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let help = format!(
    ///                 "Helpful info here. Invite me with this link: <{}>",
    ///                 url,
    ///             );
    ///
    ///             let dm = msg.author.direct_message(&ctx, |m| {
    ///                 m.content(&help)
    ///             });
    ///
    ///             match dm {
    ///                 Ok(_) => {
    ///                     let _ = msg.react(&ctx, '👌');
    ///                 },
    ///                 Err(why) => {
    ///                     println!("Err sending help: {:?}", why);
    ///
    ///                     let _ = msg.reply(&ctx, "There was an error DMing you help.");
    ///                 },
    ///             };
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler);
    /// # }
    /// ```
    ///
    /// # Examples
    ///
    /// Returns a [`ModelError::MessagingBot`] if the user being direct messaged
    /// is a bot user.
    ///
    /// [`ModelError::MessagingBot`]: ../error/enum.Error.html#variant.MessagingBot
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    /// [`User::dm`]: struct.User.html#method.dm
    // A tale with Clippy:
    //
    // A person named Clippy once asked you to unlock a box and take something
    // from it, but you never re-locked it, so you'll die and the universe will
    // implode because the box must remain locked unless you're there, and you
    // can't just borrow that item from it and take it with you forever.
    //
    // Instead what you do is unlock the box, take the item out of it, make a
    // copy of said item, and then re-lock the box, and take your copy of the
    // item with you.
    //
    // The universe is still fine, and nothing implodes.
    //
    // (AKA: Clippy is wrong and so we have to mark as allowing this lint.)
    #[allow(clippy::let_and_return)]
    #[cfg(all(feature = "builder", feature = "client"))]
    pub fn direct_message<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
        where for <'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
        if self.bot {
            return Err(Error::Model(ModelError::MessagingBot));
        }

        let mut private_channel_id = None;

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                private_channel_id = cache.read().private_channels
                    .values()
                    .map(|ch| ch.read())
                    .find(|ch| ch.recipient.read().id == self.id)
                    .map(|ch| ch.id);
            }
        }

        let private_channel_id = match private_channel_id {
            Some(id) => id,
            None => {
                let map = json!({
                    "recipient_id": self.id.0,
                });

                cache_http.http().create_private_channel(&map)?.id
            }
        };

        private_channel_id.send_message(&cache_http.http(), f)
    }

    /// This is an alias of [direct_message].
    ///
    /// # Examples
    ///
    /// Sending a message:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    ///
    /// let _ = message.author.dm("Hello!");
    /// ```
    ///
    /// # Examples
    ///
    /// Returns a [`ModelError::MessagingBot`] if the user being direct messaged
    /// is a bot user.
    ///
    /// [`ModelError::MessagingBot`]: ../error/enum.Error.html#variant.MessagingBot
    /// [direct_message]: #method.direct_message
    #[cfg(all(feature = "builder", feature = "client"))]
    #[inline]
    pub fn dm<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where for <'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
        self.direct_message(cache_http, f)
    }

    /// Retrieves the URL to the user's avatar, falling back to the default
    /// avatar if needed.
    ///
    /// This will call [`avatar_url`] first, and if that returns `None`, it
    /// then falls back to [`default_avatar_url`].
    ///
    /// [`avatar_url`]: #method.avatar_url
    /// [`default_avatar_url`]: #method.default_avatar_url
    pub fn face(&self) -> String {
        self.avatar_url()
            .unwrap_or_else(|| self.default_avatar_url())
    }

    /// Check if a user has a [`Role`]. This will retrieve the [`Guild`] from
    /// the [`Cache`] if it is available, and then check if that guild has the
    /// given [`Role`].
    ///
    /// Three forms of data may be passed in to the guild parameter: either a
    /// [`PartialGuild`], a [`GuildId`], or a `u64`.
    ///
    /// # Examples
    ///
    /// Check if a guild has a [`Role`] by Id:
    ///
    /// ```rust,ignore
    /// // Assumes a 'guild_id' and `role_id` have already been bound
    /// let _ = message.author.has_role(guild_id, role_id);
    /// ```
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`GuildId`]: ../id/struct.GuildId.html
    /// [`PartialGuild`]: ../guild/struct.PartialGuild.html
    /// [`Role`]: ../guild/struct.Role.html
    /// [`Cache`]: ../../cache/struct.Cache.html
    #[cfg(feature = "client")]
    pub fn has_role<G, R>(&self, cache_http: impl CacheHttp, guild: G, role: R) -> Result<bool>
        where G: Into<GuildContainer>, R: Into<RoleId> {
        self._has_role(cache_http, guild.into(), role.into())
    }

    #[cfg(feature = "client")]
    fn _has_role(
        &self,
        cache_http: impl CacheHttp,
        guild: GuildContainer,
        role: RoleId
    ) -> Result<bool> {
        match guild {
            GuildContainer::Guild(partial_guild) => {
                self._has_role(
                    cache_http,
                    GuildContainer::Id(partial_guild.id), role
                )
            },
            GuildContainer::Id(guild_id) => {
                let mut has_role = None;

                #[cfg(feature = "cache")]
                {
                    if let Some(cache) = cache_http.cache() {
                        has_role = Some(
                            cache.read()
                            .guilds
                            .get(&guild_id)
                            .map_or(false, |g| {
                                g.read().members.get(&self.id)
                                    .map_or(false, |m| m.roles.contains(&role))
                            })
                        );
                    }
                }

                #[cfg(feature = "http")]
                {
                    if let Some(has_role) = has_role {
                        Ok(has_role)
                    } else {
                        cache_http.http()
                            .get_member(guild_id.0, self.id.0)
                            .map(|m| m.roles.contains(&role))
                    }
                }
                #[cfg(not(feature = "http"))]
                {
                    Err(Error::Model(ModelError::ItemMissing))
                }
            },
            GuildContainer::__Nonexhaustive => unreachable!(),
        }
    }

    /// Refreshes the information about the user.
    ///
    /// Replaces the instance with the data retrieved over the REST API.
    #[cfg(feature = "http")]
    pub fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        self.id.to_user(cache_http).map(|replacement| {
            mem::replace(self, replacement);
        })
    }


    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    #[inline]
    pub fn static_avatar_url(&self) -> Option<String> {
        static_avatar_url(self.id, self.avatar.as_ref())
    }

    /// Returns the "tag" for the user.
    ///
    /// The "tag" is defined as "username#discriminator", such as "zeyla#5479".
    ///
    /// # Examples
    ///
    /// Make a command to tell the user what their tag is:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # fn main() {
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// use serenity::utils::MessageBuilder;
    /// use serenity::utils::ContentModifier::Bold;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, context: Context, msg: Message) {
    ///         if msg.content == "!mytag" {
    ///             let content = MessageBuilder::new()
    ///                 .push("Your tag is ")
    ///                 .push(Bold + msg.author.tag())
    ///                 .build();
    ///
    ///             let _ = msg.channel_id.say(&context.http, &content);
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// # }
    /// #
    /// # #[cfg(not(feature = "client"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn tag(&self) -> String { tag(&self.name, self.discriminator) }

    /// Returns the user's nickname in the given `guild_id`.
    ///
    /// If none is used, it returns `None`.
    #[inline]
    #[cfg(feature = "http")]
    pub fn nick_in<G>(&self, cache_http: impl CacheHttp, guild_id: G) -> Option<String>
    where G: Into<GuildId> {
        self._nick_in(cache_http, guild_id.into())
    }

    #[cfg(feature = "http")]
    fn _nick_in(&self, cache_http: impl CacheHttp, guild_id: GuildId) -> Option<String> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                return guild_id.to_guild_cached(cache).and_then(|guild| {
                    guild.read().members.get(&self.id).and_then(|member| member.nick.clone())
                });
            }
        }

        guild_id.member(cache_http, &self.id).ok().and_then(|member| member.nick.clone())
    }
}

impl fmt::Display for User {
    /// Formats a string which will mention the user.
    // This is in the format of: `<@USER_ID>`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

#[cfg(feature = "model")]
impl UserId {
    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: ../user/struct.CurrentUser.html
    #[cfg(feature = "http")]
    pub fn create_dm_channel(self, http: impl AsRef<Http>) -> Result<PrivateChannel> {
        let map = json!({
            "recipient_id": self.0,
        });

        http.as_ref().create_private_channel(&map)
    }

    /// Attempts to find a [`User`] by its Id in the cache.
    ///
    /// [`User`]: ../user/struct.User.html
    #[cfg(feature = "cache")]
    #[inline]
    pub fn to_user_cached(self, cache: impl AsRef<CacheRwLock>) -> Option<Arc<RwLock<User>>> {cache.as_ref().read().user(self) }

    /// First attempts to find a [`User`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the cache is not enabled,
    /// REST API will be used only.
    ///
    /// [`User`]: ../user/struct.User.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn to_user(self, cache_http: impl CacheHttp) -> Result<User> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if let Some(user) = cache.read().user(self) {
                    return Ok(user.read().clone());
                }
            }
        }

        cache_http.http().get_user(self.0)
    }
}

impl From<CurrentUser> for User {
    fn from(user: CurrentUser) -> Self {
        Self {
            avatar: user.avatar,
            bot: user.bot,
            discriminator: user.discriminator,
            id: user.id,
            name: user.name,
            _nonexhaustive: (),
        }
    }
}

impl<'a> From<&'a CurrentUser> for User {
    fn from(user: &'a CurrentUser) -> Self {
        Self {
            avatar: user.avatar.clone(),
            bot: user.bot,
            discriminator: user.discriminator,
            id: user.id,
            name: user.name.clone(),
            _nonexhaustive: (),
        }
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: CurrentUser) -> UserId { current_user.id }
}

impl<'a> From<&'a CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: &CurrentUser) -> UserId { current_user.id }
}

impl From<Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: Member) -> UserId { member.user.read().id }
}

impl<'a> From<&'a Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: &Member) -> UserId { member.user.read().id }
}

impl From<User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: User) -> UserId { user.id }
}

impl<'a> From<&'a User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: &User) -> UserId { user.id }
}

#[cfg(feature = "model")]
fn avatar_url(user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.starts_with("a_") {
            "gif"
        } else {
            "webp"
        };

        cdn!("/avatars/{}/{}.{}?size=1024", user_id.0, hash, ext)
    })
}

#[cfg(feature = "model")]
fn default_avatar_url(discriminator: u16) -> String {
    cdn!("/embed/avatars/{}.png", discriminator % 5u16)
}

#[cfg(feature = "model")]
fn static_avatar_url(user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| cdn!("/avatars/{}/{}.webp?size=1024", user_id, hash))
}

#[cfg(feature = "model")]
fn tag(name: &str, discriminator: u16) -> String {
    // 32: max length of username
    // 1: `#`
    // 4: max length of discriminator
    let mut tag = String::with_capacity(37);
    tag.push_str(name);
    tag.push('#');
    let _ = write!(tag, "{:04}", discriminator);

    tag
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use crate::model::id::UserId;
        use crate::model::user::User;

        fn gen() -> User {
            User {
                id: UserId(210),
                avatar: Some("abc".to_string()),
                bot: true,
                discriminator: 1432,
                name: "test".to_string(),
                _nonexhaustive: (),
            }
        }

        #[test]
        fn test_core() {
            let mut user = gen();

            assert!(
                user.avatar_url()
                    .unwrap()
                    .ends_with("/avatars/210/abc.webp?size=1024")
            );
            assert!(
                user.static_avatar_url()
                    .unwrap()
                    .ends_with("/avatars/210/abc.webp?size=1024")
            );

            user.avatar = Some("a_aaa".to_string());
            assert!(
                user.avatar_url()
                    .unwrap()
                    .ends_with("/avatars/210/a_aaa.gif?size=1024")
            );
            assert!(
                user.static_avatar_url()
                    .unwrap()
                    .ends_with("/avatars/210/a_aaa.webp?size=1024")
            );

            user.avatar = None;
            assert!(user.avatar_url().is_none());

            assert_eq!(user.tag(), "test#1432");
        }

        #[test]
        fn default_avatars() {
            let mut user = gen();

            user.discriminator = 0;
            assert!(user.default_avatar_url().ends_with("0.png"));
            user.discriminator = 1;
            assert!(user.default_avatar_url().ends_with("1.png"));
            user.discriminator = 2;
            assert!(user.default_avatar_url().ends_with("2.png"));
            user.discriminator = 3;
            assert!(user.default_avatar_url().ends_with("3.png"));
            user.discriminator = 4;
            assert!(user.default_avatar_url().ends_with("4.png"));
        }
    }
}
