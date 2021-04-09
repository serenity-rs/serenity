//! User information-related models.

use std::fmt;
#[cfg(feature = "model")]
use std::fmt::Write;

use bitflags::__impl_bitflags;
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::prelude::*;
use super::utils::deserialize_u16;
#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditProfile};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectReaction,
    CollectReply,
    MessageCollectorBuilder,
    ReactionCollectorBuilder,
};
#[cfg(feature = "model")]
use crate::http::GuildPagination;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "model")]
use crate::json::json;
use crate::json::to_string;
#[cfg(feature = "model")]
use crate::utils;
use crate::{internal::prelude::*, model::misc::Mentionable};

/// Information about the current user.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CurrentUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    #[serde(deserialize_with = "deserialize_u16")]
    pub discriminator: u16,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    #[serde(rename = "username")]
    pub name: String,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
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
    /// # async fn run() {
    /// # use serenity::cache::Cache;
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache has been unlocked
    /// let user = cache.current_user().await;
    ///
    /// match user.avatar_url() {
    ///     Some(url) => println!("{}'s avatar can be found at {}", user.name, url),
    ///     None => println!("{} does not have an avatar set.", user.name)
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.id, self.avatar.as_ref())
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    pub fn default_avatar_url(&self) -> String {
        default_avatar_url(self.discriminator)
    }

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
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// #     let mut user = CurrentUser::default();
    /// let avatar = serenity::utils::read_image("./avatar.png")?;
    ///
    /// user.edit(&http, |p| p.avatar(Some(&avatar))).await;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EditProfile`]: crate::builder::EditProfile
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set.
    /// May also return an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut EditProfile) -> &mut EditProfile,
    {
        let mut map = HashMap::new();
        map.insert("username", Value::String(self.name.clone()));

        if let Some(email) = self.email.as_ref() {
            map.insert("email", Value::String(email.clone()));
        }

        let mut edit_profile = EditProfile(map);
        f(&mut edit_profile);
        let map = utils::hashmap_to_json_map(edit_profile.0);

        *self = http.as_ref().edit_profile(&map).await?;

        Ok(())
    }

    /// Retrieves the URL to the current user's avatar, falling back to the
    /// default avatar if needed.
    ///
    /// This will call [`avatar_url`] first, and if that returns `None`, it
    /// then falls back to [`default_avatar_url`].
    ///
    /// [`avatar_url`]: Self::avatar_url
    /// [`default_avatar_url`]: Self::default_avatar_url
    #[inline]
    pub fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.default_avatar_url())
    }

    /// Gets a list of guilds that the current user is in.
    ///
    /// # Examples
    ///
    /// Print out the names of all guilds the current user is in:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() {
    /// #     let user = CurrentUser::default();
    /// #     let http = Http::default();
    /// // assuming the user has been bound
    ///
    /// if let Ok(guilds) = user.guilds(&http).await {
    ///     for (index, guild) in guilds.into_iter().enumerate() {
    ///         println!("{}: {}", index, guild.name);
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the Discord API returns an error.
    /// Also can return [`Error::Json`] if there is an error in deserializing
    /// the data returned by the API.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn guilds(&self, http: impl AsRef<Http>) -> Result<Vec<GuildInfo>> {
        let mut guilds = Vec::new();
        loop {
            let mut pagination = http
                .as_ref()
                .get_guilds(
                    &GuildPagination::After(guilds.last().map_or(GuildId(1), |g: &GuildInfo| g.id)),
                    100,
                )
                .await?;
            let len = pagination.len();
            guilds.append(&mut pagination);
            if len != 100 {
                break;
            }
        }
        Ok(guilds)
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
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() {
    /// #     let user = CurrentUser::default();
    /// #     let http = Http::default();
    /// use serenity::model::Permissions;
    ///
    /// // assuming the user has been bound
    /// let url = match user.invite_url(&http, Permissions::empty()).await {
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
    /// ```
    ///
    /// Get the invite url with some basic permissions set:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() {
    /// #     let user = CurrentUser::default();
    /// #     let http = Http::default();
    /// use serenity::model::Permissions;
    ///
    /// // assuming the user has been bound
    /// let permissions = Permissions::READ_MESSAGES | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS;
    /// let url = match user.invite_url(&http, permissions).await {
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
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    pub async fn invite_url(
        &self,
        http: impl AsRef<Http>,
        permissions: Permissions,
    ) -> Result<String> {
        let bits = permissions.bits();
        let client_id = http.as_ref().get_current_application_info().await.map(|v| v.id)?;

        let mut url =
            format!("https://discord.com/api/oauth2/authorize?client_id={}&scope=bot", client_id);

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
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # fn run() {
    /// #     let user = CurrentUser::default();
    /// // assuming the user has been bound
    ///
    /// match user.static_avatar_url() {
    ///     Some(url) => println!("{}'s static avatar can be found at {}", user.name, url),
    ///     None => println!("Could not get static avatar for {}.", user.name)
    /// }
    /// # }
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
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # fn run() {
    /// #     let user = CurrentUser::default();
    /// // assuming the user has been bound
    ///
    /// println!("The current user's distinct identifier is {}", user.tag());
    /// # }
    /// ```
    #[inline]
    pub fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }
}

/// An enum that represents a default avatar.
///
/// The default avatar is calculated via the result of `discriminator % 5`.
///
/// The has of the avatar can be retrieved via calling [`name`] on the enum.
///
/// [`name`]: Self::name
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
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
}

impl DefaultAvatar {
    /// Retrieves the String hash of the default avatar.
    ///
    /// # Errors
    ///
    /// May return a [`Error::Json`] if there is a serialization error.
    ///
    /// [`Error::Json`]: crate::error::Error::Json
    pub fn name(self) -> Result<String> {
        to_string(&self).map_err(From::from)
    }
}

/// The representation of a user's status.
///
/// # Examples
///
/// - [`DoNotDisturb`];
/// - [`Invisible`].
///
/// [`DoNotDisturb`]: OnlineStatus::DoNotDisturb
/// [`Invisible`]: OnlineStatus::Invisible
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
pub enum OnlineStatus {
    #[serde(rename = "dnd")]
    DoNotDisturb,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "online")]
    Online,
}

impl OnlineStatus {
    pub fn name(&self) -> &str {
        match *self {
            OnlineStatus::DoNotDisturb => "dnd",
            OnlineStatus::Idle => "idle",
            OnlineStatus::Invisible => "invisible",
            OnlineStatus::Offline => "offline",
            OnlineStatus::Online => "online",
        }
    }
}

impl Default for OnlineStatus {
    fn default() -> OnlineStatus {
        OnlineStatus::Online
    }
}

/// Information about a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
    /// [`name`]: Self::name
    #[serde(deserialize_with = "deserialize_u16")]
    pub discriminator: u16,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique.
    #[serde(rename = "username")]
    pub name: String,
    /// the public flags on a user's account
    pub public_flags: Option<UserPublicFlags>,
}

/// User's public flags
#[derive(Clone, Copy)]
pub struct UserPublicFlags {
    pub bits: u32,
}

__impl_bitflags! {
    UserPublicFlags: u32 {
        /// User's flag as discord employee
        DISCORD_EMPLOYEE = 0b00000000_00000000_00000000_00000001;
        /// User's flag as partnered server owner
        PARTNERED_SERVER_OWNER = 0b00000000_00000000_00000000_00000010;
        /// User's flag as hypesquad events
        HYPESQUAD_EVENTS = 0b00000000_00000000_00000000_00000100;
        /// User's flag as bug hunter level 1
        BUG_HUNTER_LEVEL_1 = 0b00000000_00000000_00000000_00001000;
        /// User's flag as house bravery
        HOUSE_BRAVERY = 0b00000000_00000000_00000000_01000000;
        /// User's flag as house brilliance
        HOUSE_BRILLIANCE = 0b00000000_00000000_00000000_10000000;
        /// User's flag as house balance
        HOUSE_BALANCE = 0b00000000_00000000_00000001_00000000;
        /// User's flag as early supporter
        EARLY_SUPPORTER = 0b00000000_00000000_00000010_00000000;
        /// User's flag as team user
        TEAM_USER = 0b00000000_00000000_00000100_00000000;
        /// User's flag as system
        SYSTEM = 0b00000000_00000000_00010000_00000000;
        /// User's flag as bug hunter level 2
        BUG_HUNTER_LEVEL_2 = 0b00000000_00000000_01000000_00000000;
        /// User's flag as verified bot
        VERIFIED_BOT = 0b00000000_00000001_00000000_00000000;
        /// User's flag as early verified bot developer
        EARLY_VERIFIED_BOT_DEVELOPER = 0b00000000_00000010_00000000_00000000;
    }
}

impl<'de> Deserialize<'de> for UserPublicFlags {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(UserPublicFlags::from_bits_truncate(deserializer.deserialize_u32(U32Visitor)?))
    }
}

impl Serialize for UserPublicFlags {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl Default for User {
    /// Initializes a `User` with default values. Setting the following:
    /// - **id** to `UserId(210)`
    /// - **avatar** to `Some("abc")`
    /// - **bot** to `true`.
    /// - **discriminator** to `1432`.
    /// - **name** to `"test"`.
    /// - **public_flags** to `None`.
    fn default() -> Self {
        User {
            id: UserId(210),
            avatar: Some("abc".to_string()),
            bot: true,
            discriminator: 1432,
            name: "test".to_string(),
            public_flags: None,
        }
    }
}

use std::hash::{Hash, Hasher};

#[cfg(feature = "model")]
use chrono::{DateTime, Utc};

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
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.id, self.avatar.as_ref())
    }

    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: CurrentUser
    ///
    /// # Errors
    ///
    /// See [`UserId::create_dm_channel`] for what errors may be returned.
    ///
    /// [`UserId::create_dm_channel`]: crate::model::id::UserId::create_dm_channel
    #[inline]
    pub async fn create_dm_channel(&self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        if self.bot {
            return Err(Error::Model(ModelError::MessagingBot));
        }

        self.id.create_dm_channel(cache_http).await
    }

    /// Retrieves the time that this user was created at.
    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.id.created_at()
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    pub fn default_avatar_url(&self) -> String {
        default_avatar_url(self.discriminator)
    }

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
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    /// #   #[cfg(feature = "cache")]
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "~help" {
    ///             let url = match ctx.cache.current_user().await.invite_url(&ctx, Permissions::empty()).await {
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
    ///             })
    ///             .await;
    ///
    ///             match dm {
    ///                 Ok(_) => {
    ///                     let _ = msg.react(&ctx, '👌').await;
    ///                 },
    ///                 Err(why) => {
    ///                     println!("Err sending help: {:?}", why);
    ///
    ///                     let _ = msg.reply(&ctx, "There was an error DMing you help.").await;
    ///                 },
    ///             };
    ///         }
    ///     }
    /// }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client =Client::builder("token").event_handler(Handler).await?;
    /// #     Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessagingBot`] if the user being direct messaged
    /// is a bot user.
    ///
    /// May also return an [`Error::Http`] if the message was illformed, or if the
    /// user cannot be sent a direct message.
    ///
    /// [`Error::Json`] can also be returned if there is an error deserializing
    /// the API response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn direct_message<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where
        for<'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.create_dm_channel(&cache_http).await?.send_message(&cache_http.http(), f).await
    }

    /// This is an alias of [direct_message].
    ///
    /// [direct_message]: Self::direct_message
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn dm<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where
        for<'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.direct_message(cache_http, f).await
    }

    /// Retrieves the URL to the user's avatar, falling back to the default
    /// avatar if needed.
    ///
    /// This will call [`avatar_url`] first, and if that returns `None`, it
    /// then falls back to [`default_avatar_url`].
    ///
    /// [`avatar_url`]: Self::avatar_url
    /// [`default_avatar_url`]: Self::default_avatar_url
    pub fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.default_avatar_url())
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
    /// [`Cache`]: crate::cache::Cache
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the given `Guild` is unavailable,
    /// if that `Role` does not exist in the given `Guild`, or if the
    /// given `User` is not in that `Guild`.
    ///
    /// May also return an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    #[inline]
    pub async fn has_role(
        &self,
        cache_http: impl CacheHttp,
        guild: impl Into<GuildContainer>,
        role: impl Into<RoleId>,
    ) -> Result<bool> {
        self._has_role(&cache_http, guild.into(), role.into()).await
    }

    fn _has_role<'a>(
        &'a self,
        cache_http: &'a impl CacheHttp,
        guild: GuildContainer,
        role: RoleId,
    ) -> BoxFuture<'a, Result<bool>> {
        async move {
            match guild {
                GuildContainer::Guild(partial_guild) => {
                    self._has_role(cache_http, GuildContainer::Id(partial_guild.id), role).await
                },
                GuildContainer::Id(guild_id) => {
                    // Silences a warning when compiling without the `cache` feature.
                    #[allow(unused_mut)]
                    let mut has_role = None;

                    #[cfg(feature = "cache")]
                    {
                        if let Some(cache) = cache_http.cache() {
                            if let Some(member) = cache.member(guild_id, self.id).await {
                                has_role = Some(member.roles.contains(&role));
                            }
                        }
                    }

                    if let Some(has_role) = has_role {
                        Ok(has_role)
                    } else {
                        cache_http
                            .http()
                            .get_member(guild_id.0, self.id.0)
                            .await
                            .map(|m| m.roles.contains(&role))
                    }
                },
            }
        }
        .boxed()
    }

    /// Refreshes the information about the user.
    ///
    /// Replaces the instance with the data retrieved over the REST API.
    ///
    /// # Errors
    ///
    /// See [`UserId::to_user`] for what errors may be returned.
    ///
    /// [`UserId::to_user`]: crate::model::id::UserId::to_user
    #[inline]
    pub async fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        *self = self.id.to_user(cache_http).await?;

        Ok(())
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
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// use serenity::utils::MessageBuilder;
    /// use serenity::utils::ContentModifier::Bold;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         if msg.content == "!mytag" {
    ///             let content = MessageBuilder::new()
    ///                 .push("Your tag is ")
    ///                 .push(Bold + msg.author.tag())
    ///                 .build();
    ///
    ///             let _ = msg.channel_id.say(&context.http, &content).await;
    ///         }
    ///     }
    /// }
    /// let mut client =Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }

    /// Returns the user's nickname in the given `guild_id`.
    ///
    /// If none is used, it returns `None`.
    #[inline]
    pub async fn nick_in(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
    ) -> Option<String> {
        let guild_id = guild_id.into();

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = guild_id.to_guild_cached(cache).await {
                    if let Some(member) = guild.members.get(&self.id) {
                        return member.nick.clone();
                    }
                }
            }
        }

        guild_id.member(cache_http, &self.id).await.ok().and_then(|member| member.nick)
    }

    /// Returns a future that will await one message by this user.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reply<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).author_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages sent by this user.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_replies<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).author_id(self.id.0)
    }

    /// Await a single reaction by this user.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reaction<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).author_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent by this user.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reactions<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).author_id(self.id.0)
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
    /// [current user]: CurrentUser
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a `User` with that `UserId` does not exist,
    /// or is otherwise unavailable.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing
    /// the channel data returned by the Discord API.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_dm_channel(self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                for channel in cache.private_channels().await.values() {
                    if channel.recipient.id == self {
                        return Ok(channel.clone());
                    }
                }
            }
        }

        let map = json!({
            "recipient_id": self.0,
        });

        cache_http.http().create_private_channel(&map).await
    }

    /// Attempts to find a [`User`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn to_user_cached(self, cache: impl AsRef<Cache>) -> Option<User> {
        cache.as_ref().user(self).await
    }

    /// First attempts to find a [`User`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the cache is not enabled,
    /// REST API will be used only.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if a `User` with that `UserId` does not exist,
    /// or otherwise cannot be fetched.
    ///
    /// May also return an [`Error::Json`] if there is an error in
    /// deserializing the user.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    #[inline]
    pub async fn to_user(self, cache_http: impl CacheHttp) -> Result<User> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(user) = cache.user(self).await {
                    return Ok(user);
                }
            }
        }

        cache_http.http().get_user(self.0).await
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
            public_flags: user.public_flags,
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
            public_flags: user.public_flags.clone(),
        }
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl<'a> From<&'a CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: &CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: Member) -> UserId {
        member.user.id
    }
}

impl<'a> From<&'a Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: &Member) -> UserId {
        member.user.id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: User) -> UserId {
        user.id
    }
}

impl<'a> From<&'a User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: &User) -> UserId {
        user.id
    }
}

#[cfg(feature = "model")]
fn avatar_url(user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "webp" };

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

    #[allow(clippy::let_underscore_must_use)]
    let _ = write!(tag, "{:04}", discriminator);

    tag
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use crate::model::user::User;

        #[test]
        fn test_core() {
            let mut user = User::default();

            assert!(user.avatar_url().unwrap().ends_with("/avatars/210/abc.webp?size=1024"));
            assert!(user.static_avatar_url().unwrap().ends_with("/avatars/210/abc.webp?size=1024"));

            user.avatar = Some("a_aaa".to_string());
            assert!(user.avatar_url().unwrap().ends_with("/avatars/210/a_aaa.gif?size=1024"));
            assert!(user
                .static_avatar_url()
                .unwrap()
                .ends_with("/avatars/210/a_aaa.webp?size=1024"));

            user.avatar = None;
            assert!(user.avatar_url().is_none());

            assert_eq!(user.tag(), "test#1432");
        }

        #[test]
        fn default_avatars() {
            let mut user = User {
                discriminator: 0,
                ..Default::default()
            };

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
