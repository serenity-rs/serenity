//! User information-related models.

use std::fmt;
#[cfg(feature = "model")]
use std::fmt::Write;
use std::num::NonZeroU16;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
#[cfg(feature = "model")]
use serde_json::json;

use super::prelude::*;
#[cfg(feature = "model")]
use crate::builder::{CreateMessage, EditProfile};
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::avatar_url;

/// Used with `#[serde(with|deserialize_with|serialize_with)]`
///
/// # Examples
///
/// ```rust,ignore
/// use std::num::NonZeroU16;
///
/// #[derive(Deserialize, Serialize)]
/// struct A {
///     #[serde(with = "discriminator")]
///     id: Option<NonZeroU16>,
/// }
///
/// #[derive(Deserialize)]
/// struct B {
///     #[serde(deserialize_with = "discriminator::deserialize")]
///     id: Option<NonZeroU16>,
/// }
///
/// #[derive(Serialize)]
/// struct C {
///     #[serde(serialize_with = "discriminator::serialize")]
///     id: Option<NonZeroU16>,
/// }
/// ```
pub(crate) mod discriminator {
    use std::fmt;

    use serde::de::{Error, Visitor};

    struct DiscriminatorVisitor;

    impl Visitor<'_> for DiscriminatorVisitor {
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

    use std::num::NonZeroU16;

    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<NonZeroU16>, D::Error> {
        deserializer.deserialize_option(OptionalDiscriminatorVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
    pub fn serialize<S: Serializer>(
        value: &Option<NonZeroU16>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_some(&format_args!("{value:04}")),
            None => serializer.serialize_none(),
        }
    }

    struct OptionalDiscriminatorVisitor;

    impl<'de> Visitor<'de> for OptionalDiscriminatorVisitor {
        type Value = Option<NonZeroU16>;

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
            deserializer.deserialize_any(DiscriminatorVisitor).map(NonZeroU16::new)
        }
    }
}

/// Information about the current user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CurrentUser(User);

impl Deref for CurrentUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurrentUser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<CurrentUser> for User {
    fn from(user: CurrentUser) -> Self {
        user.0
    }
}

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
    pub async fn edit(&mut self, http: &Http, builder: EditProfile<'_>) -> Result<()> {
        *self = builder.execute(http).await?;
        Ok(())
    }
}

/// The representation of a user's status.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#update-presence-status-types).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct User {
    /// The unique Id of the user. Can be used to calculate the account's creation date.
    pub id: UserId,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique. Unless the account has
    /// migrated to a next generation username, which does not have a discriminant.
    #[serde(rename = "username")]
    pub name: FixedString<u8>,
    /// The account's discriminator to differentiate the user from others with
    /// the same [`Self::name`]. The name+discriminator pair is always unique.
    /// If the discriminator is not present, then this is a next generation username
    /// which is implicitly unique.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
    pub discriminator: Option<NonZeroU16>,
    /// The account's display name, if it is set.
    /// For bots this is the application name.
    pub global_name: Option<FixedString<u8>>,
    /// Optional avatar hash.
    pub avatar: Option<ImageHash>,
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
    pub banner: Option<ImageHash>,
    /// The user's banner colour encoded as an integer representation of hexadecimal colour code
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<Colour>,
    /// The user's chosen language option
    pub locale: Option<FixedString>,
    /// Whether the email on this account has been verified
    ///
    /// Requires [`Scope::Email`]
    pub verified: Option<bool>,
    /// The user's email
    ///
    /// Requires [`Scope::Email`]
    pub email: Option<FixedString>,
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum PremiumType {
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
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
        /// User's flag for suspected spam activity.
        #[cfg(feature = "unstable_discord_api")]
        const SPAMMER = 1 << 20;
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
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(None, self.id, self.avatar.as_ref())
    }

    /// Returns the formatted URL of the user's banner, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF banner.
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
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
    pub async fn create_dm_channel(&self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        if self.bot() {
            return Err(Error::Model(ModelError::MessagingBot));
        }

        self.id.create_dm_channel(cache_http).await
    }

    /// Retrieves the time that this user was created at.
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        self.id.created_at()
    }

    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[must_use]
    pub fn default_avatar_url(&self) -> String {
        default_avatar_url(self)
    }

    /// Sends a message to a user through a direct message channel. This is a channel that can only
    /// be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// See [`UserId::direct_message`] for examples.
    ///
    /// # Errors
    ///
    /// See [`UserId::direct_message`] for errors.
    pub async fn direct_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
        self.id.direct_message(cache_http, builder).await
    }

    /// Calculates the user's display name.
    ///
    /// The global name takes priority over the user's username if it exists.
    ///
    /// Note: Guild specific information is not included as this is only available on the [Member].
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.global_name.as_deref().unwrap_or(&self.name)
    }

    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    pub async fn dm(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
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

    /// Retrieves the URL to the static version of the user's avatar, falling back to the default
    /// avatar if needed.
    ///
    /// This will call [`Self::static_avatar_url`] first, and if that returns [`None`], it then
    /// falls back to [`Self::default_avatar_url`].
    #[must_use]
    pub fn static_face(&self) -> String {
        self.static_avatar_url().unwrap_or_else(|| self.default_avatar_url())
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
    pub async fn has_role(
        &self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        role: RoleId,
    ) -> Result<bool> {
        guild_id.member(cache_http, self.id).await.map(|m| m.roles.contains(&role))
    }

    /// Refreshes the information about the user.
    ///
    /// Replaces the instance with the data retrieved over the REST API.
    ///
    /// # Errors
    ///
    /// See [`UserId::to_user`] for what errors may be returned.
    pub async fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        *self = self.id.to_user(cache_http).await?;

        Ok(())
    }

    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
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
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: &Context, msg: &Message) {
    ///         if msg.content == "!mytag" {
    ///             let content = format!("Your tag is: {}", msg.author.tag());
    ///             let _ = msg.channel_id.say(&context.http, &content).await;
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }

    /// Returns the user's nickname in the given `guild_id`.
    ///
    /// If none is used, it returns [`None`].
    pub async fn nick_in(&self, cache_http: impl CacheHttp, guild_id: GuildId) -> Option<String> {
        // This can't be removed because `GuildId::member` clones the entire `Member` struct if
        // it's present in the cache, which is expensive.
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = guild_id.to_guild_cached(cache) {
                    if let Some(member) = guild.members.get(&self.id) {
                        return member.nick.clone().map(Into::into);
                    }
                }
            }
        }

        // At this point we're guaranteed to do an API call.
        guild_id
            .member(cache_http, self.id)
            .await
            .ok()
            .and_then(|member| member.nick)
            .map(Into::into)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages sent by
    /// this user.
    #[cfg(feature = "collector")]
    pub fn await_reply(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).author_id(self.id)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions sent by
    /// this user.
    #[cfg(feature = "collector")]
    pub fn await_reaction(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).author_id(self.id)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
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
        #[cfg(feature = "temp_cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(private_channel) = cache.temp_private_channels.get(&self) {
                return Ok(PrivateChannel::clone(&private_channel));
            }
        }

        let map = json!({
            "recipient_id": self,
        });

        let channel = cache_http.http().create_private_channel(&map).await?;

        #[cfg(feature = "temp_cache")]
        if let Some(cache) = cache_http.cache() {
            use crate::cache::MaybeOwnedArc;

            let cached_channel = MaybeOwnedArc::new(channel.clone());
            cache.temp_private_channels.insert(self, cached_channel);
        }

        Ok(channel)
    }

    /// Sends a message to a user through a direct message channel. This is a channel that can only
    /// be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// When a user sends a message with a content of `"~help"`, DM the author a help message
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// # struct Handler;
    /// use serenity::builder::CreateMessage;
    ///
    /// #[serenity::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: &Context, msg: &Message) {
    ///         if msg.content == "~help" {
    ///             let builder = CreateMessage::new().content("Helpful info here.");
    ///
    ///             if let Err(why) = msg.author.id.direct_message(&ctx, builder).await {
    ///                 println!("Err sending help: {why:?}");
    ///                 let _ = msg.reply(&ctx, "There was an error DMing you help.").await;
    ///             };
    ///         }
    ///     }
    /// }
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
        self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
        self.create_dm_channel(&cache_http).await?.send_message(cache_http, builder).await
    }

    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    pub async fn dm(
        self,
        cache_http: impl CacheHttp,
        builder: CreateMessage<'_>,
    ) -> Result<Message> {
        self.direct_message(cache_http, builder).await
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
    pub async fn to_user(self, cache_http: impl CacheHttp) -> Result<User> {
        #[cfg(feature = "temp_cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(user) = cache.temp_users.get(&self) {
                    return Ok(User::clone(&user));
                }
            }
        }

        let user = cache_http.http().get_user(self).await?;

        #[cfg(feature = "temp_cache")]
        {
            if let Some(cache) = cache_http.cache() {
                use crate::cache::MaybeOwnedArc;

                let cached_user = MaybeOwnedArc::new(user.clone());
                cache.temp_users.insert(cached_user.id, cached_user);
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

impl From<&Member> for UserId {
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

impl From<&User> for UserId {
    /// Gets the Id of a [`User`].
    fn from(user: &User) -> UserId {
        user.id
    }
}

#[cfg(feature = "model")]
fn default_avatar_url(user: &User) -> String {
    let avatar_id = if let Some(discriminator) = user.discriminator {
        discriminator.get() % 5 // Legacy username system
    } else {
        ((user.id.get() >> 22) % 6) as u16 // New username system
    };

    cdn!("/embed/avatars/{}.png", avatar_id)
}

#[cfg(feature = "model")]
fn static_avatar_url(user_id: UserId, hash: Option<&ImageHash>) -> Option<String> {
    hash.map(|hash| cdn!("/avatars/{}/{}.webp?size=1024", user_id, hash))
}

#[cfg(feature = "model")]
fn banner_url(user_id: UserId, hash: Option<&ImageHash>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.is_animated() { "gif" } else { "webp" };
        cdn!("/banners/{}/{}.{}?size=1024", user_id, hash, ext)
    })
}

#[cfg(feature = "model")]
fn tag(name: &str, discriminator: Option<NonZeroU16>) -> String {
    // 32: max length of username
    // 1: `#`
    // 4: max length of discriminator
    let mut tag = String::with_capacity(37);
    tag.push_str(name);
    if let Some(discriminator) = discriminator {
        tag.push('#');
        write!(tag, "{discriminator:04}").unwrap();
    }
    tag
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU16;

    #[test]
    fn test_discriminator_serde() {
        use serde::{Deserialize, Serialize};
        use serde_json::json;

        use super::discriminator;
        use crate::model::utils::assert_json;

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct User {
            #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
            discriminator: Option<NonZeroU16>,
        }

        let user = User {
            discriminator: NonZeroU16::new(123),
        };
        assert_json(&user, json!({"discriminator": "0123"}));

        let user_no_discriminator = User {
            discriminator: None,
        };
        assert_json(&user_no_discriminator, json!({}));
    }

    #[cfg(feature = "model")]
    mod model {
        use std::num::NonZeroU16;
        use std::str::FromStr;

        use small_fixed_array::FixedString;

        use crate::model::id::UserId;
        use crate::model::misc::ImageHash;
        use crate::model::user::User;

        #[test]
        fn test_core() {
            let mut user = User {
                id: UserId::new(210),
                avatar: Some(ImageHash::from_str("fb211703bcc04ee612c88d494df0272f").unwrap()),
                discriminator: NonZeroU16::new(1432),
                name: FixedString::from_static_trunc("test"),
                ..Default::default()
            };

            let expected = "/avatars/210/fb211703bcc04ee612c88d494df0272f.webp?size=1024";
            assert!(user.avatar_url().unwrap().ends_with(expected));
            assert!(user.static_avatar_url().unwrap().ends_with(expected));

            user.avatar = Some(ImageHash::from_str("a_fb211703bcc04ee612c88d494df0272f").unwrap());
            let expected = "/avatars/210/a_fb211703bcc04ee612c88d494df0272f.gif?size=1024";
            assert!(user.avatar_url().unwrap().ends_with(expected));
            let expected = "/avatars/210/a_fb211703bcc04ee612c88d494df0272f.webp?size=1024";
            assert!(user.static_avatar_url().unwrap().ends_with(expected));

            user.avatar = None;
            assert!(user.avatar_url().is_none());

            assert_eq!(user.tag(), "test#1432");
        }

        #[test]
        fn default_avatars() {
            let mut user = User {
                discriminator: None,
                id: UserId::new(737323631117598811),
                ..Default::default()
            };

            // New username system
            assert!(user.default_avatar_url().ends_with("5.png"));

            // Legacy username system
            user.discriminator = NonZeroU16::new(1);
            assert!(user.default_avatar_url().ends_with("1.png"));
            user.discriminator = NonZeroU16::new(2);
            assert!(user.default_avatar_url().ends_with("2.png"));
            user.discriminator = NonZeroU16::new(3);
            assert!(user.default_avatar_url().ends_with("3.png"));
            user.discriminator = NonZeroU16::new(4);
            assert!(user.default_avatar_url().ends_with("4.png"));
        }
    }
}
