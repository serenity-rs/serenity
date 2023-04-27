//! User information-related models.

use std::fmt;
#[cfg(feature = "model")]
use std::fmt::Write;
#[cfg(feature = "temp_cache")]
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::prelude::*;
#[cfg(feature = "model")]
use crate::builder::{Builder, CreateMessage, EditProfile};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, UserRef};
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::CacheHttp;
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json::json;
use crate::json::to_string;
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
        serializer.collect_str(&format_args!("{value:04}"))
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
                Some(value) => serializer.serialize_some(&format_args!("{value:04}")),
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
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object).
pub type CurrentUser = User;

#[cfg(feature = "model")]
impl CurrentUser {
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
    /// # use serenity::builder::{EditProfile, CreateAttachment};
    /// # use serenity::http::Http;
    /// # use serenity::model::user::CurrentUser;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut user = CurrentUser::default();
    /// let avatar = CreateAttachment::path("./avatar.png").await?;
    /// user.edit(&http, EditProfile::new().avatar(&avatar)).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set. May also return an [`Error::Json`]
    /// if there is an error in deserializing the API response.
    pub async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditProfile) -> Result<()> {
        *self = builder.execute(cache_http, ()).await?;
        Ok(())
    }
}

/// An enum that represents a default avatar.
///
/// The default avatar is calculated via the result of `discriminator % 5`.
///
/// The has of the avatar can be retrieved via calling [`Self::name`] on the enum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
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
    pub fn name(self) -> Result<String> {
        to_string(&self).map_err(From::from)
    }
}

/// The representation of a user's status.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#update-presence-status-types).
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
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
    #[default]
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

/// Information about a user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object), existence of
/// additional partial member field documented [here](https://discord.com/developers/docs/topics/gateway-events#message-create).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct User {
    /// The unique Id of the user. Can be used to calculate the account's creation date.
    pub id: UserId,
    /// The account's username. Changing username will trigger a discriminator change if the
    /// username+discriminator pair becomes non-unique.
    #[serde(rename = "username")]
    pub name: String,
    /// The account's discriminator to differentiate the user from others with the same
    /// [`Self::name`]. The name+discriminator pair is always unique.
    #[serde(with = "discriminator")]
    pub discriminator: u16,
    /// Optional avatar hash.
    pub avatar: Option<String>,
    /// Indicator of whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
    /// Whether the user is an Official Discord System user (part of the urgent message system).
    #[serde(default)]
    pub system: bool,
    /// Whether the user has two factor enabled on their account
    #[serde(default)]
    pub mfa_enabled: bool,
    /// Optional banner hash.
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    pub banner: Option<String>,
    /// The user's banner colour encoded as an integer representation of hexadecimal colour code
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<Colour>,
    /// The user's chosen language option
    pub locale: Option<String>,
    /// Whether the email on this account has been verified
    ///
    /// Requires [`Scope::Email`]
    pub verified: Option<bool>,
    /// The user's email
    ///
    /// Requires [`Scope::Email`]
    pub email: Option<String>,
    /// The flags on a user's account
    #[serde(default)]
    pub flags: UserPublicFlags,
    /// The type of Nitro subscription on a user's account
    #[serde(default)]
    pub premium_type: PremiumType,
    /// The public flags on a user's account
    pub public_flags: Option<UserPublicFlags>,
    /// Only included in [`Message::mentions`] for messages from the gateway.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#message-create-message-create-extra-fields).
    // Box required to avoid infinitely recursive types
    pub member: Option<Box<PartialMember>>,
}

enum_number! {
    /// Premium types denote the level of premium a user has. Visit the [Nitro](https://discord.com/nitro)
    /// page to learn more about the premium plans Discord currently offers.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/user#user-object-premium-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PremiumType {
        #[default]
        None = 0,
        NitroClassic = 1,
        Nitro = 2,
        NitroBasic = 3,
        _ => Unknown(u8),
    }
}

bitflags! {
    /// User's public flags
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/user#user-object-user-flags).
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
        /// User's flag as active developer
        const ACTIVE_DEVELOPER = 1 << 22;
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
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    #[inline]
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        banner_url(self.id, self.banner.as_ref())
    }

    /// Creates a direct message channel between the [current user] and the user. This can also
    /// retrieve the channel if one already exists.
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

    /// Sends a message to a user through a direct message channel. This is a channel that can only
    /// be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// When a user sends a message with a content of `"~help"`, DM the author a help message, and
    /// then react with `'👌'` to verify message sending:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")] {
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// #
    /// use serenity::builder::{CreateBotAuthParameters, CreateMessage};
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    /// #   #[cfg(feature = "cache")]
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "~help" {
    ///             let url = match CreateBotAuthParameters::new()
    ///                 .permissions(Permissions::empty())
    ///                 .scopes(&[Scope::Bot])
    ///                 .auto_client_id(&ctx)
    ///                 .await
    ///             {
    ///                 Ok(v) => v.build(),
    ///                 Err(why) => {
    ///                     println!("Error creating invite url: {:?}", why);
    ///                     return;
    ///                 },
    ///             };
    ///
    ///             let help = format!("Helpful info here. Invite me with this link: <{}>", url);
    ///
    ///             let builder = CreateMessage::new().content(help);
    ///             let dm = msg.author.direct_message(&ctx, builder).await;
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
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessagingBot`] if the user being direct messaged is a bot user.
    ///
    /// May also return an [`Error::Http`] if the user cannot be sent a direct message.
    ///
    /// Returns an [`Error::Json`] if there is an error deserializing the API response.
    pub async fn direct_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message> {
        self.create_dm_channel(&cache_http).await?.send_message(cache_http, builder).await
    }

    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn dm(&self, cache_http: impl CacheHttp, builder: CreateMessage) -> Result<Message> {
        self.direct_message(cache_http, builder).await
    }

    /// Retrieves the URL to the user's avatar, falling back to the default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it then falls back
    /// to [`Self::default_avatar_url`].
    #[must_use]
    pub fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.default_avatar_url())
    }

    /// Check if a user has a [`Role`]. This will retrieve the [`Guild`] from the [`Cache`] if it
    /// is available, and then check if that guild has the given [`Role`].
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
    /// Returns an [`Error::Http`] if the given [`Guild`] is unavailable, if that [`Role`] does not
    /// exist in the given [`Guild`], or if the given [`User`] is not in that [`Guild`].
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the API response.
    #[inline]
    pub async fn has_role(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        role: impl Into<RoleId>,
    ) -> Result<bool> {
        guild_id.into().member(cache_http, self).await.map(|m| m.roles.contains(&role.into()))
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
    /// # Ok(())
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

        // This can't be removed because `GuildId::member` clones the entire `Member` struct if
        // it's present in the cache, which is expensive.
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

        // At this point we're guaranteed to do an API call.
        guild_id.member(cache_http, &self.id).await.ok().and_then(|member| member.nick)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages sent by
    /// this user.
    #[cfg(feature = "collector")]
    pub fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        MessageCollector::new(shard_messenger).author_id(self.id)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions sent by
    /// this user.
    #[cfg(feature = "collector")]
    pub fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).author_id(self.id)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ReactionCollector {
        self.await_reaction(shard_messenger)
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
    /// Creates a direct message channel between the [current user] and the user. This can also
    /// retrieve the channel if one already exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`User`] with that [`UserId`] does not exist, or is otherwise
    /// unavailable.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the channel data
    /// returned by the Discord API.
    ///
    /// [current user]: CurrentUser
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
    pub fn to_user_cached(self, cache: &impl AsRef<Cache>) -> Option<UserRef<'_>> {
        cache.as_ref().user(self)
    }

    /// First attempts to find a [`User`] by its Id in the cache, upon failure requests it via the
    /// REST API.
    ///
    /// **Note**: If the cache is not enabled, REST API will be used only.
    ///
    /// **Note**: If the cache is enabled, you might want to enable the `temp_cache` feature to
    /// cache user data retrieved by this function for a short duration.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if a [`User`] with that [`UserId`] does not exist, or
    /// otherwise cannot be fetched.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the user.
    #[inline]
    pub async fn to_user(self, cache_http: impl CacheHttp) -> Result<User> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(user) = cache.user(self) {
                    return Ok(user.clone());
                }
            }
        }

        let user = cache_http.http().get_user(self).await?;

        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        {
            if let Some(cache) = cache_http.cache() {
                cache.temp_users.insert(user.id, Arc::new(user.clone()));
            }
        }

        Ok(user)
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
    write!(tag, "{discriminator:04}").unwrap();

    tag
}

#[cfg(test)]
mod test {
    #[test]
    fn test_discriminator_serde() {
        use serde::{Deserialize, Serialize};

        use super::discriminator;
        use crate::json::{assert_json, json};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct User {
            #[serde(with = "discriminator")]
            discriminator: u16,
        }
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct UserOpt {
            #[serde(
                default,
                skip_serializing_if = "Option::is_none",
                with = "discriminator::option"
            )]
            discriminator: Option<u16>,
        }

        let user = User {
            discriminator: 123,
        };
        assert_json(&user, json!({"discriminator": "0123"}));

        let user = UserOpt {
            discriminator: Some(123),
        };
        assert_json(&user, json!({"discriminator": "0123"}));

        let user_no_discriminator = UserOpt {
            discriminator: None,
        };
        assert_json(&user_no_discriminator, json!({}));
    }

    #[cfg(feature = "model")]
    mod model {
        use crate::model::id::UserId;
        use crate::model::user::User;

        #[test]
        fn test_core() {
            let mut user = User {
                id: UserId::new(210),
                avatar: Some("abc".to_string()),
                discriminator: 1432,
                name: "test".to_string(),
                ..Default::default()
            };

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
