//! User information-related models.

use std::fmt;
#[cfg(feature = "model")]
use std::fmt::Write;

#[cfg(feature = "model")]
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};

use super::prelude::*;
#[cfg(feature = "model")]
use crate::builder::{CreateBotAuthParameters, CreateMessage, EditProfile};
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
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json;
#[cfg(feature = "model")]
use crate::json::json;
use crate::json::to_string;
#[cfg(feature = "model")]
use crate::model::application::oauth::Scope;
use crate::model::mention::Mentionable;

/// Used with `#[serde(with|deserialize_with|serialize_with)]`
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Deserialize, Serialize)]
/// struct A {
///     #[serde(with = "discriminator")]
///     id: u16,
/// }
///
/// #[derive(Deserialize)]
/// struct B {
///     #[serde(deserialize_with = "discriminator::deserialize")]
///     id: u16,
/// }
///
/// #[derive(Serialize)]
/// struct C {
///     #[serde(serialize_with = "discriminator::serialize")]
///     id: u16,
/// }
/// ```
pub(crate) mod discriminator {
    use std::convert::TryFrom;
    use std::fmt;

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
        deserializer.deserialize_any(DiscriminatorVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(value: &u16, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{:04}", value))
    }

    struct DiscriminatorVisitor;

    impl<'de> Visitor<'de> for DiscriminatorVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string or integer discriminator")
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            u16::try_from(value).map_err(Error::custom)
        }

        fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
            s.parse().map_err(Error::custom)
        }
    }

    /// Used with `#[serde(with|deserialize_with|serialize_with)]`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize, Serialize)]
    /// struct A {
    ///     #[serde(with = "discriminator::option")]
    ///     id: Option<u16>,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct B {
    ///     #[serde(deserialize_with = "discriminator::option::deserialize")]
    ///     id: Option<u16>,
    /// }
    ///
    /// #[derive(Serialize)]
    /// struct C {
    ///     #[serde(serialize_with = "discriminator::option::serialize")]
    ///     id: Option<u16>,
    /// }
    /// ```
    pub(crate) mod option {
        use std::fmt;

        use serde::de::{Error, Visitor};
        use serde::{Deserializer, Serializer};

        use super::DiscriminatorVisitor;

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<u16>, D::Error> {
            deserializer.deserialize_option(OptionalDiscriminatorVisitor)
        }

        #[allow(clippy::trivially_copy_pass_by_ref)]
        pub fn serialize<S: Serializer>(
            value: &Option<u16>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            match value {
                Some(value) => serializer.serialize_some(&format_args!("{:04}", value)),
                None => serializer.serialize_none(),
            }
        }

        struct OptionalDiscriminatorVisitor;

        impl<'de> Visitor<'de> for OptionalDiscriminatorVisitor {
            type Value = Option<u16>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("optional string or integer discriminator")
            }

            fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }

            fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }

            fn visit_some<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                Ok(Some(deserializer.deserialize_any(DiscriminatorVisitor)?))
            }
        }
    }
}

/// Information about the current user.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CurrentUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    #[serde(with = "discriminator")]
    pub discriminator: u16,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    #[serde(rename = "username")]
    pub name: String,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
    pub banner: Option<String>,
    #[cfg(feature = "utils")]
    pub accent_colour: Option<Colour>,
    #[cfg(not(feature = "utils"))]
    pub accent_colour: Option<u32>,
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
    /// # fn run() {
    /// # use serenity::cache::Cache;
    /// #
    /// # let cache = Cache::default();
    /// // assuming the cache has been unlocked
    /// let user = cache.current_user();
    ///
    /// match user.avatar_url() {
    ///     Some(url) => println!("{}'s avatar can be found at {}", user.name, url),
    ///     None => println!("{} does not have an avatar set.", user.name),
    /// }
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.id, self.avatar.as_ref())
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    #[must_use]
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
    /// #     let http = Http::new("token");
    /// #     let mut user = CurrentUser::default();
    /// let avatar = serenity::utils::read_image("./avatar.png")?;
    ///
    /// user.edit(&http, |p| p.avatar(Some(&avatar))).await;
    /// #     Ok(())
    /// # }
    /// ```
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
        map.insert("username", Value::from(self.name.clone()));

        if let Some(email) = self.email.as_ref() {
            map.insert("email", Value::from(email.clone()));
        }

        let mut edit_profile = EditProfile(map);
        f(&mut edit_profile);
        let map = json::hashmap_to_json_map(edit_profile.0);

        *self = http.as_ref().edit_profile(&map).await?;

        Ok(())
    }

    /// Retrieves the URL to the current user's avatar, falling back to the
    /// default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it
    /// then falls back to [`Self::default_avatar_url`].
    #[inline]
    #[must_use]
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
    /// #     let http = Http::new("token");
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
                    Some(&GuildPagination::After(
                        guilds.last().map_or(GuildId(1), |g: &GuildInfo| g.id),
                    )),
                    Some(100),
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
    /// Only the `bot` scope is used, if you wish to use more, such as slash commands, see
    /// [`Self::invite_url_with_oauth2_scopes`]
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
    /// #     let http = Http::new("token");
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
    /// assert_eq!(
    ///     url,
    ///     "https://discordapp.com/api/oauth2/authorize? \
    ///                  client_id=249608697955745802&scope=bot"
    /// );
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
    /// #     let http = Http::new("token");
    /// use serenity::model::Permissions;
    ///
    /// // assuming the user has been bound
    /// let permissions =
    ///     Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS;
    /// let url = match user.invite_url(&http, permissions).await {
    ///     Ok(v) => v,
    ///     Err(why) => {
    ///         println!("Error getting invite url: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    ///
    /// assert_eq!(
    ///     url,
    ///     "https://discordapp.
    /// com/api/oauth2/authorize?client_id=249608697955745802&scope=bot&permissions=19456"
    /// );
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(Unauthorized)`][`HttpError::UnsuccessfulRequest`]
    /// If the user is not authorized for this end point.
    ///
    /// Should never return [`Error::Url`] as all the data is controlled over.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    pub async fn invite_url(
        &self,
        http: impl AsRef<Http>,
        permissions: Permissions,
    ) -> Result<String> {
        self.invite_url_with_oauth2_scopes(http, permissions, &[Scope::Bot]).await
    }

    /// Generate an invite url, but with custom scopes.
    ///
    /// # Examples
    ///
    /// Get the invite url with no permissions set and slash commands support:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() {
    /// #     let user = CurrentUser::default();
    /// #     let http = Http::new("token");
    /// use serenity::model::application::oauth::Scope;
    /// use serenity::model::Permissions;
    ///
    /// let scopes = vec![Scope::Bot, Scope::ApplicationsCommands];
    ///
    /// // assuming the user has been bound
    /// let url = match user.invite_url_with_oauth2_scopes(&http, Permissions::empty(), &scopes).await {
    ///     Ok(v) => v,
    ///     Err(why) => {
    ///         println!("Error getting invite url: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    ///
    /// assert_eq!(
    ///     url,
    ///     "https://discordapp.com/api/oauth2/authorize? \
    ///                  client_id=249608697955745802&scope=bot%20applications.commands"
    /// );
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(Unauthorized)`][`HttpError::UnsuccessfulRequest`]
    /// If the user is not authorized for this end point.
    ///
    /// Should never return [`Error::Url`] as all the data is controlled over.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    pub async fn invite_url_with_oauth2_scopes(
        &self,
        http: impl AsRef<Http>,
        permissions: Permissions,
        scopes: &[Scope],
    ) -> Result<String> {
        let mut builder = CreateBotAuthParameters::default();

        builder.permissions(permissions);
        builder.auto_client_id(http).await?;
        builder.scopes(scopes);

        Ok(builder.build())
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
    ///     None => println!("Could not get static avatar for {}.", user.name),
    /// }
    /// # }
    /// ```
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }
}

/// An enum that represents a default avatar.
///
/// The default avatar is calculated via the result of `discriminator % 5`.
///
/// The has of the avatar can be retrieved via calling [`Self::name`] on the enum.
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
    #[must_use]
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
    /// the same [`Self::name`]. The name+discriminator pair is always unique.
    #[serde(with = "discriminator")]
    pub discriminator: u16,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique.
    #[serde(rename = "username")]
    pub name: String,
    /// The public flags on a user's account
    pub public_flags: Option<UserPublicFlags>,
    /// Optional banner hash.
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API,
    /// e.g. with [`Http::get_user`].
    pub banner: Option<String>,
    /// The user's banner colour encoded as an integer representation of
    /// hexadecimal colour code
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API,
    /// e.g. with [`Http::get_user`].
    #[cfg(feature = "utils")]
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<Colour>,
    #[cfg(not(feature = "utils"))]
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<u32>,
}

bitflags! {
    /// User's public flags
    #[derive(Default)]
    pub struct UserPublicFlags: u32 {
        /// User's flag as discord employee
        const DISCORD_EMPLOYEE = 1 << 0;
        /// User's flag as partnered server owner
        const PARTNERED_SERVER_OWNER = 1 << 1;
        /// User's flag as hypesquad events
        const HYPESQUAD_EVENTS = 1 << 2;
        /// User's flag as bug hunter level 1
        const BUG_HUNTER_LEVEL_1 = 1 << 3;
        /// User's flag as house bravery
        const HOUSE_BRAVERY = 1 << 6;
        /// User's flag as house brilliance
        const HOUSE_BRILLIANCE = 1 << 7;
        /// User's flag as house balance
        const HOUSE_BALANCE = 1 << 8;
        /// User's flag as early supporter
        const EARLY_SUPPORTER = 1 << 9;
        /// User's flag as team user
        const TEAM_USER = 1 << 10;
        /// User's flag as system
        const SYSTEM = 1 << 12;
        /// User's flag as bug hunter level 2
        const BUG_HUNTER_LEVEL_2 = 1 << 14;
        /// User's flag as verified bot
        const VERIFIED_BOT = 1 << 16;
        /// User's flag as early verified bot developer
        const EARLY_VERIFIED_BOT_DEVELOPER = 1 << 17;
        /// User's flag as discord certified moderator
        const DISCORD_CERTIFIED_MODERATOR = 1 << 18;
        /// Bot's running with HTTP interactions
        const BOT_HTTP_INTERACTIONS = 1 << 19;
    }
}

impl Default for User {
    /// Initializes a [`User`] with default values. Setting the following:
    /// - **id** to `UserId(210)`
    /// - **avatar** to `Some("abc")`
    /// - **bot** to `true`.
    /// - **discriminator** to `1432`.
    /// - **name** to `"test"`.
    /// - **public_flags** to [`None`].
    fn default() -> Self {
        User {
            id: UserId(210),
            avatar: Some("abc".to_string()),
            bot: true,
            discriminator: 1432,
            name: "test".to_string(),
            public_flags: None,
            banner: None,
            accent_colour: None,
        }
    }
}

use std::hash::{Hash, Hasher};

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
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.id, self.avatar.as_ref())
    }

    /// Returns the formatted URL of the user's banner, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF banner.
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API,
    /// e.g. with [`Http::get_user`].
    #[inline]
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        banner_url(self.id, self.banner.as_ref())
    }

    /// Creates a direct message channel between the [current user] and the
    /// user. This can also retrieve the channel if one already exists.
    ///
    /// [current user]: CurrentUser
    ///
    /// # Errors
    ///
    /// See [`UserId::create_dm_channel`] for what errors may be returned.
    #[inline]
    pub async fn create_dm_channel(&self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        if self.bot {
            return Err(Error::Model(ModelError::MessagingBot));
        }

        self.id.create_dm_channel(cache_http).await
    }

    /// Retrieves the time that this user was created at.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        self.id.created_at()
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[inline]
    #[must_use]
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
    ///             let url =
    ///                 match ctx.cache.current_user().invite_url(&ctx, Permissions::empty()).await {
    ///                     Ok(v) => v,
    ///                     Err(why) => {
    ///                         println!("Error creating invite url: {:?}", why);
    ///
    ///                         return;
    ///                     },
    ///                 };
    ///
    ///             let help = format!("Helpful info here. Invite me with this link: <{}>", url,);
    ///
    ///             let dm = msg.author.direct_message(&ctx, |m| m.content(&help)).await;
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
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
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
    pub async fn direct_message<'a, F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.create_dm_channel(&cache_http).await?.send_message(&cache_http.http(), f).await
    }

    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn dm<'a, F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
    {
        self.direct_message(cache_http, f).await
    }

    /// Retrieves the URL to the user's avatar, falling back to the default
    /// avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it
    /// then falls back to [`Self::default_avatar_url`].
    #[must_use]
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
    /// Returns an [`Error::Http`] if the given [`Guild`] is unavailable,
    /// if that [`Role`] does not exist in the given [`Guild`], or if the
    /// given [`User`] is not in that [`Guild`].
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
                            if let Some(member) = cache.member(guild_id, self.id) {
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
    #[inline]
    pub async fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        *self = self.id.to_user(cache_http).await?;

        Ok(())
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    #[inline]
    #[must_use]
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
    /// use serenity::utils::ContentModifier::Bold;
    /// use serenity::utils::MessageBuilder;
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
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }

    /// Returns the user's nickname in the given `guild_id`.
    ///
    /// If none is used, it returns [`None`].
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
                if let Some(guild) = guild_id.to_guild_cached(cache) {
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
    pub fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> CollectReply {
        CollectReply::new(shard_messenger).author_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages sent by this user.
    #[cfg(feature = "collector")]
    pub fn await_replies(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> MessageCollectorBuilder {
        MessageCollectorBuilder::new(shard_messenger).author_id(self.id.0)
    }

    /// Await a single reaction by this user.
    #[cfg(feature = "collector")]
    pub fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> CollectReaction {
        CollectReaction::new(shard_messenger).author_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent by this user.
    #[cfg(feature = "collector")]
    pub fn await_reactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ReactionCollectorBuilder {
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
    /// Returns [`Error::Http`] if a [`User`] with that [`UserId`] does not exist,
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
                for channel_entry in cache.private_channels().iter() {
                    let channel = channel_entry.value();

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
        cache.as_ref().user(self)
    }

    /// First attempts to find a [`User`] by its Id in the cache,
    /// upon failure requests it via the REST API.
    ///
    /// **Note**: If the cache is not enabled, REST API will be used only.
    ///
    /// **Note**: If the cache is enabled, you might want to enable the `temp_cache` feature to
    /// cache user data retrieved by this function for a short duration.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if a [`User`] with that [`UserId`] does not exist,
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
                if let Some(user) = cache.user(self) {
                    return Ok(user);
                }
            }
        }

        let user = cache_http.http().get_user(self.0).await?;

        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        {
            if let Some(cache) = cache_http.cache() {
                cache.temp_users.insert(user.id, user.clone());
            }
        }

        Ok(user)
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
            banner: user.banner,
            accent_colour: user.accent_colour,
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
            public_flags: user.public_flags,
            banner: user.banner.clone(),
            accent_colour: user.accent_colour,
        }
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a [`CurrentUser`] struct.
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl<'a> From<&'a CurrentUser> for UserId {
    /// Gets the Id of a [`CurrentUser`] struct.
    fn from(current_user: &CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a [`Member`].
    fn from(member: Member) -> UserId {
        member.user.id
    }
}

impl<'a> From<&'a Member> for UserId {
    /// Gets the Id of a [`Member`].
    fn from(member: &Member) -> UserId {
        member.user.id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a [`User`].
    fn from(user: User) -> UserId {
        user.id
    }
}

impl<'a> From<&'a User> for UserId {
    /// Gets the Id of a [`User`].
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
fn banner_url(user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "webp" };

        cdn!("/banners/{}/{}.{}?size=1024", user_id.0, hash, ext)
    })
}

#[cfg(feature = "model")]
fn tag(name: &str, discriminator: u16) -> String {
    // 32: max length of username
    // 1: `#`
    // 4: max length of discriminator
    let mut tag = String::with_capacity(37);
    tag.push_str(name);
    tag.push('#');
    write!(tag, "{:04}", discriminator).unwrap();

    tag
}

#[cfg(test)]
mod test {
    #[test]
    fn test_discriminator_serde() {
        use serde::{Deserialize, Serialize};
        use serde_test::{assert_de_tokens, assert_tokens, Token};

        use super::discriminator;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct User {
            #[serde(with = "discriminator")]
            discriminator: u16,
        }
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct UserOpt {
            #[serde(with = "discriminator::option")]
            discriminator: Option<u16>,
        }

        let user = User {
            discriminator: 123,
        };
        assert_tokens(&user, &[
            Token::Struct {
                name: "User",
                len: 1,
            },
            Token::Str("discriminator"),
            Token::Str("0123"),
            Token::StructEnd,
        ]);
        assert_de_tokens(&user, &[
            Token::Struct {
                name: "User",
                len: 1,
            },
            Token::Str("discriminator"),
            Token::U16(123),
            Token::StructEnd,
        ]);

        let user = UserOpt {
            discriminator: Some(123),
        };
        assert_tokens(&user, &[
            Token::Struct {
                name: "UserOpt",
                len: 1,
            },
            Token::Str("discriminator"),
            Token::Some,
            Token::Str("0123"),
            Token::StructEnd,
        ]);
        assert_de_tokens(&user, &[
            Token::Struct {
                name: "UserOpt",
                len: 1,
            },
            Token::Str("discriminator"),
            Token::Some,
            Token::U16(123),
            Token::StructEnd,
        ]);
    }

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
