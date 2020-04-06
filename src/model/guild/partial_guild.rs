use serde::de::Error as DeError;

#[cfg(feature = "http")]
use crate::http::CacheHttp;
use crate::{model::prelude::*};
use super::super::utils::{deserialize_emojis, deserialize_roles};

#[cfg(feature = "model")]
use crate::builder::{CreateChannel, EditGuild, EditMember, EditRole};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(all(feature = "cache", feature = "utils", feature = "client"))]
use crate::cache::CacheRwLock;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::MutexMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectAllReactions, CollectNReactions, CollectOneReaction,
    CollectAllReplies, CollectNReplies, CollectOneReply,
};

/// Partial information about a [`Guild`]. This does not include information
/// like member data.
///
/// [`Guild`]: struct.Guild.html
#[derive(Clone, Debug, Serialize)]
pub struct PartialGuild {
    pub id: GuildId,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u64,
    pub default_message_notifications: DefaultMessageNotificationLevel,
    pub embed_channel_id: Option<ChannelId>,
    pub embed_enabled: bool,
    #[serde(serialize_with = "serialize_emojis", deserialize_with = "deserialize_emojis")] pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.
    ///
    /// Refer to [`Guild::features`] for more information.
    ///
    /// [`Guild::features`]: struct.Guild.html#structfield.features
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub mfa_level: MfaLevel,
    pub name: String,
    pub owner_id: UserId,
    pub region: String,
    #[serde(serialize_with = "serialize_roles", deserialize_with = "deserialize_roles")] pub roles: HashMap<RoleId, Role>,
    pub splash: Option<String>,
    pub verification_level: VerificationLevel,
    pub description: Option<String>,
    pub premium_tier: PremiumTier,
    // In some cases Discord returns `null` rather than 0.
    #[serde(deserialize_with = "deserialize_u64_or_zero")]
    pub premium_subscription_count: u64,
    pub banner: Option<String>,
    pub vanity_url_code: Option<String>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Ban a [`User`] from the guild. All messages by the
    /// user within the last given number of days given will be deleted. This
    /// may be a range between `0` and `7`.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,ignore
    /// // assumes a `user` and `guild` have already been bound
    /// let _ = guild.ban(user, 4);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ModelError::DeleteMessageDaysAmount`]: ../error/enum.Error.html#variant.DeleteMessageDaysAmount
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    pub async fn ban<U: Into<UserId>>(&self, http: impl AsRef<Http>, user: U, delete_message_days: u8) -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Model(
                ModelError::DeleteMessageDaysAmount(delete_message_days),
            ));
        }

        self.id.ban(&http, user, &delete_message_days).await
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn bans(&self, http: impl AsRef<Http>) -> Result<Vec<Ban>> {
        self.id.bans(&http).await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[cfg(feature = "http")]
    #[inline]
    pub async fn channels(&self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.channels(&http).await
    }

    /// Creates a [`GuildChannel`] in the guild.
    ///
    /// Refer to [`http::create_channel`] for more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelType;
    ///
    /// guild.create_channel(|c| c.name("test").kind(ChannelType::Voice));
    /// ```
    ///
    /// [`GuildChannel`]: ../channel/struct.GuildChannel.html
    /// [`http::create_channel`]: ../../http/fn.create_channel.html
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_channel(&self, http: impl AsRef<Http>, f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel) -> Result<GuildChannel> {
        self.id.create_channel(&http, f).await
    }

    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more
    /// information.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to
    /// how to read an image from the filesystem and encode it as base64. Most
    /// of the example can be applied similarly for this method.
    ///
    /// [`EditProfile::avatar`]: ../../builder/struct.EditProfile.html#method.avatar
    /// [`Guild::create_emoji`]: struct.Guild.html#method.create_emoji
    /// [`utils::read_image`]: ../../utils/fn.read_image.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_emoji(&self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji> {
        self.id.create_emoji(&http, name, image).await
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_integration<I>(&self, http: impl AsRef<Http>, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self.id.create_integration(&http, integration_id, kind).await
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Guild::create_role`]: struct.Guild.html#method.create_role
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn create_role<F>(&self, http: impl AsRef<Http>, f: F) -> Result<Role>
    where F: FnOnce(&mut EditRole) -> &mut EditRole {
        self.id.create_role(&http, f).await
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        self.id.delete(&http).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn delete_emoji<E: Into<EmojiId>>(&self, http: impl AsRef<Http>, emoji_id: E) -> Result<()> {
        self.id.delete_emoji(&http, emoji_id).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn delete_integration<I: Into<IntegrationId>>(&self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self.id.delete_integration(&http, integration_id).await
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [`Role::delete`]: struct.Role.html#method.delete
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[inline]
    pub async fn delete_role<R: Into<RoleId>>(&self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self.id.delete_role(&http, role_id).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    pub async fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<()>
        where F: FnOnce(&mut EditGuild) -> &mut EditGuild {
        match self.id.edit(&http, f).await {
            Ok(guild) => {
                self.afk_channel_id = guild.afk_channel_id;
                self.afk_timeout = guild.afk_timeout;
                self.default_message_notifications = guild.default_message_notifications;
                self.emojis = guild.emojis;
                self.features = guild.features;
                self.icon = guild.icon;
                self.mfa_level = guild.mfa_level;
                self.name = guild.name;
                self.owner_id = guild.owner_id;
                self.region = guild.region;
                self.roles = guild.roles;
                self.splash = guild.splash;
                self.verification_level = guild.verification_level;

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Emoji::edit`]: struct.Emoji.html#method.edit
    /// [Manage Emojis]:
    /// ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn edit_emoji<E: Into<EmojiId>>(&self, http: impl AsRef<Http>, emoji_id: E, name: &str) -> Result<Emoji> {
        self.id.edit_emoji(&http, emoji_id, name).await
    }

    /// Edits the properties of member of the guild, such as muting or
    /// nicknaming them.
    ///
    /// Refer to `EditMember`'s documentation for a full list of methods and
    /// permission restrictions.
    ///
    /// # Examples
    ///
    /// Mute a member and set their roles to just one role with a predefined Id:
    ///
    /// ```rust,ignore
    /// use serenity::model::GuildId;
    ///
    /// GuildId(7).edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    #[cfg(feature = "http")]
    #[inline]
    pub async fn edit_member<F, U>(&self, http: impl AsRef<Http>, user_id: U, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember, U: Into<UserId> {
        self.id.edit_member(&http, user_id, f).await
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Change Nickname]: ../permissions/struct.Permissions.html#associatedconstant.CHANGE_NICKNAME
    #[cfg(feature = "http")]
    #[inline]
    pub async fn edit_nickname(&self, http: impl AsRef<Http>, new_nickname: Option<&str>) -> Result<()> {
        self.id.edit_nickname(&http, new_nickname).await
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// Requires that the current user be in the guild.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn get<G: Into<GuildId>>(http: impl AsRef<Http>, guild_id: G) -> Result<PartialGuild> {
        guild_id.into().to_partial_guild(&http).await
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn kick<U: Into<UserId>>(&self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        self.id.kick(&http, user_id).await
    }

    /// Kicks a [`Member`] from the guild and specifies a reason.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn kick_with_reason<U: Into<UserId>>(&self, http: impl AsRef<Http>, user_id: U, reason: &str) -> Result<()> {
        self.id.kick_with_reason(&http, user_id, reason).await
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        self.id.integrations(&http).await
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        self.id.invites(&http).await
    }

    /// Leaves the guild.
    #[cfg(feature = "http")]
    #[inline]
    pub async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(&http).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[cfg(feature = "client")]
    pub async fn member<U: Into<UserId>>(&self, cache_http: impl CacheHttp, user_id: U) -> Result<Member> {
        self.id.member(cache_http, user_id).await
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: ../user/struct.User.html
    #[cfg(feature = "http")]
    pub async fn members<U>(&self, http: impl AsRef<Http>, limit: Option<u64>, after: U) -> Result<Vec<Member>>
        where U: Into<Option<UserId>> {
        self.id.members(&http, limit, after).await
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../permissions/struct.Permissions.html#associatedconstant.MOVE_MEMBERS
    #[inline]
    #[cfg(feature = "http")]
    pub async fn move_member<C, U>(&self, http: impl AsRef<Http>, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self.id.move_member(&http, user_id, channel_id).await
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[inline]
    #[cfg(feature = "http")]
    pub async fn prune_count(&self, http: impl AsRef<Http>, days: u16) -> Result<GuildPrune> {
        self.id.prune_count(&http, days).await
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to
    /// retrieve the total number of shards in use. If you already have the
    /// total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: ../../utils/fn.shard_id.html
    #[cfg(all(feature = "cache", feature = "utils", feature = "client"))]
    #[inline]
    pub async fn shard_id(&self, cache: impl AsRef<CacheRwLock>) -> u64 {
        self.id.shard_id(cache).await
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used
    /// will need to be passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`,
    /// using 17 shards:
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub async fn shard_id(&self, shard_count: u64) -> u64 {
        self.id.shard_id(shard_count).await
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub async fn start_integration_sync<I: Into<IntegrationId>>(&self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self.id.start_integration_sync(&http, integration_id).await
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn unban<U: Into<UserId>>(&self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        self.id.unban(&http, user_id).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(&http).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        self.id.webhooks(&http).await
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of
    /// them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         if let Some(guild_id) = msg.guild_id {
    ///             if let Some(guild) = guild_id.to_guild_cached(&context).await {
    ///                 if let Some(role) = guild.read().await.role_by_name("role_name") {
    ///                     println!("Obtained role's reference: {:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).await?;
    ///
    /// client.start().await?;
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// [`Role`]: ../guild/struct.Role.html
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }

    /// Await a single reply sent in this guild.
    #[cfg(feature = "collector")]
    pub fn await_reply<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>) -> CollectOneReply<'a> {
        CollectOneReply::new(shard_messenger).guild_id(self.id.0)
    }

    /// Await `number` of replies in this guild.
    #[cfg(feature = "collector")]
    pub fn await_n_replies<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>, number: u32) -> CollectNReplies<'a> {
        CollectNReplies::new(shard_messenger).guild_id(self.id.0).collect_limit(number)
    }

    /// Await all replies in this guild. This won't stop unless a set limit is
    /// reached, there are no limits set by default.
    /// For example, a limit can be set by calling `timeout` or `collect_limit`.
    #[cfg(feature = "collector")]
    pub fn await_all_replies<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>) -> CollectAllReplies<'a> {
        CollectAllReplies::new(shard_messenger).guild_id(self.id.0)
    }

    /// Await a single reaction sent in this guild.
    #[cfg(feature = "collector")]
    pub fn await_reaction<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>) -> CollectOneReaction<'a> {
        CollectOneReaction::new(shard_messenger).guild_id(self.id.0)
    }

    /// Await `number` of reactions in this guild.
    #[cfg(feature = "collector")]
    pub fn await_n_reactions<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>, number: u32) -> CollectNReactions<'a> {
        CollectNReactions::new(shard_messenger).guild_id(self.id.0).collect_limit(number)
    }

    /// Await all reactions sent in this guild. This won't stop unless
    /// a set limit is reached, there are no limits set by default.
    /// For example, a limit can be set by calling `timeout` or `collect_limit`.
    #[cfg(feature = "collector")]
    pub fn await_all_reactions<'a>(self, shard_messenger: &'a impl AsRef<MutexMessenger>) -> CollectAllReactions<'a> {
        CollectAllReactions::new(shard_messenger).guild_id(self.id.0)
    }
}

impl<'de> Deserialize<'de> for PartialGuild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("roles").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(role) = value.as_object_mut() {
                        role
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => serde_json::from_value::<Option<ChannelId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let afk_timeout = map.remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let embed_channel_id = match map.remove("embed_channel_id") {
            Some(e) => Option::<ChannelId>::deserialize(e).map_err(DeError::custom)?,
            None => None,
        };
        let embed_enabled = map.remove("embed_enabled")
            .ok_or_else(|| DeError::custom("expected guild embed_enabled"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let features = map.remove("features")
            .ok_or_else(|| DeError::custom("expected guild features"))
            .and_then(serde_json::from_value::<Vec<String>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let mfa_level = map.remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(MfaLevel::deserialize)
            .map_err(DeError::custom)?;
        let name = map.remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map.remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let region = map.remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map.remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let description = match map.remove("description") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let premium_tier = match map.remove("premium_tier") {
            Some(v) => PremiumTier::deserialize(v).map_err(DeError::custom)?,
            None => PremiumTier::default(),
        };
        let premium_subscription_count = match map.remove("premium_subscription_count") {
            Some(Value::Null) | None => 0,
            Some(v) => u64::deserialize(v).map_err(DeError::custom)?,
        };
        let banner = match map.remove("banner") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let vanity_url_code = match map.remove("vanity_url_code") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };

        Ok(Self {
            afk_channel_id,
            afk_timeout,
            default_message_notifications,
            embed_channel_id,
            embed_enabled,
            emojis,
            features,
            icon,
            id,
            mfa_level,
            name,
            owner_id,
            region,
            roles,
            splash,
            verification_level,
            description,
            premium_tier,
            premium_subscription_count,
            banner,
            vanity_url_code,
            _nonexhaustive: (),
        })
    }
}

impl<'de> Deserialize<'de> for PartialGuild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("roles").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(role) = value.as_object_mut() {
                        role
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => serde_json::from_value::<Option<ChannelId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let afk_timeout = map.remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let embed_channel_id = match map.remove("embed_channel_id") {
            Some(e) => Option::<ChannelId>::deserialize(e).map_err(DeError::custom)?,
            None => None,
        };
        let embed_enabled = map.remove("embed_enabled")
            .ok_or_else(|| DeError::custom("expected guild embed_enabled"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let features = map.remove("features")
            .ok_or_else(|| DeError::custom("expected guild features"))
            .and_then(serde_json::from_value::<Vec<String>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let mfa_level = map.remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(MfaLevel::deserialize)
            .map_err(DeError::custom)?;
        let name = map.remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map.remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let region = map.remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map.remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let description = match map.remove("description") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let premium_tier = match map.remove("premium_tier") {
            Some(v) => PremiumTier::deserialize(v).map_err(DeError::custom)?,
            None => PremiumTier::default(),
        };
        let premium_subscription_count = match map.remove("premium_subscription_count") {
            Some(Value::Null) | None => 0,
            Some(v) => u64::deserialize(v).map_err(DeError::custom)?,
        };
        let banner = match map.remove("banner") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let vanity_url_code = match map.remove("vanity_url_code") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };

        Ok(Self {
            afk_channel_id,
            afk_timeout,
            default_message_notifications,
            embed_channel_id,
            embed_enabled,
            emojis,
            features,
            icon,
            id,
            mfa_level,
            name,
            owner_id,
            region,
            roles,
            splash,
            verification_level,
            description,
            premium_tier,
            premium_subscription_count,
            banner,
            vanity_url_code,
            _nonexhaustive: (),
        })
    }
}
