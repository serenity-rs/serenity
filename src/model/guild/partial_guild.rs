use serde::de::Error as DeError;
#[cfg(feature = "simd-json")]
use simd_json::StaticNode;

#[cfg(feature = "model")]
use crate::builder::{CreateChannel, EditGuild, EditMember, EditRole};
#[cfg(all(feature = "cache", feature = "utils", feature = "client"))]
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
use crate::http::{CacheHttp, Http};
#[cfg(all(feature = "model", feature = "unstable_discord_api"))]
use crate::{
    builder::CreateInteraction,
    model::interactions::{ApplicationCommand, Interaction},
};
use crate::{json::from_value, model::prelude::*};
use crate::{
    json::{from_number, prelude::*},
    model::utils::{deserialize_emojis, deserialize_roles},
};

/// Partial information about a [`Guild`]. This does not include information
/// like member data.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct PartialGuild {
    pub id: GuildId,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u64,
    pub default_message_notifications: DefaultMessageNotificationLevel,
    pub widget_channel_id: Option<ChannelId>,
    pub widget_enabled: bool,
    #[serde(serialize_with = "serialize_emojis", deserialize_with = "deserialize_emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.
    ///
    /// Refer to [`Guild::features`] for more information.
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub mfa_level: MfaLevel,
    pub name: String,
    pub owner_id: UserId,
    pub region: String,
    #[serde(serialize_with = "serialize_roles", deserialize_with = "deserialize_roles")]
    pub roles: HashMap<RoleId, Role>,
    pub splash: Option<String>,
    pub verification_level: VerificationLevel,
    pub description: Option<String>,
    pub premium_tier: PremiumTier,
    // In some cases Discord returns `null` rather than 0.
    #[serde(deserialize_with = "deserialize_u64_or_zero")]
    pub premium_subscription_count: u64,
    pub banner: Option<String>,
    pub vanity_url_code: Option<String>,
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Ban a [`User`] from the guild, deleting a number of
    /// days' worth of messages (`dmd`) between the range 0 and 7.
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
    /// Also may return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn ban(
        &self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
    ) -> Result<()> {
        self.ban_with_reason(&http, user, dmd, "").await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the reasons `ban` may return an error,
    /// can also return an error if the reason is too long.
    ///
    /// [`ban`]: Self::ban
    #[inline]
    pub async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self.id.ban_with_reason(&http, user, dmd, reason).await
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn bans(&self, http: impl AsRef<Http>) -> Result<Vec<Ban>> {
        self.id.bans(&http).await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in
    /// the guild or if the guild is otherwise unavailable.
    #[inline]
    pub async fn channels(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.channels(&http).await
    }

    /// Creates a [`GuildChannel`] in the guild.
    ///
    /// Refer to [`Http::create_channel`] for more information.
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if invalid data was given, such as the channel name being
    /// too long.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn create_channel(
        &self,
        http: impl AsRef<Http>,
        f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel,
    ) -> Result<GuildChannel> {
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// if the emoji name is too long, or if the image is too large.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [`utils::read_image`]: crate::utils::read_image
    /// [Manage Emojis]: Permissions::MANAGE_EMOJIS
    #[inline]
    pub async fn create_emoji(
        &self,
        http: impl AsRef<Http>,
        name: &str,
        image: &str,
    ) -> Result<Emoji> {
        self.id.create_emoji(&http, name, image).await
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn create_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
        kind: &str,
    ) -> Result<()> {
        self.id.create_integration(&http, integration_id, kind).await
    }

    /// Creates a new [`ApplicationCommand`] for the guild.
    ///
    /// See the documentation for [`Interaction::create_global_application_command`] on how to use this.
    ///
    /// **Note**: `application_id` is usually the bot's id, unless it's a very old bot.
    ///
    /// [`ApplicationCommand`]: crate::model::interactions::ApplicationCommand
    #[cfg(feature = "unstable_discord_api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable_discord_api")))]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    pub async fn create_application_command<F>(
        &self,
        http: impl AsRef<Http>,
        application_id: u64,
        f: F,
    ) -> Result<ApplicationCommand>
    where
        F: FnOnce(&mut CreateInteraction) -> &mut CreateInteraction,
    {
        Interaction::create_guild_application_command(http, self.id, application_id, f).await
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value was set.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn create_role<F>(&self, http: impl AsRef<Http>, f: F) -> Result<Role>
    where
        F: FnOnce(&mut EditRole) -> &mut EditRole,
    {
        self.id.create_role(&http, f).await
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not the owner of
    /// the guild.
    #[inline]
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        self.id.delete(&http).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an emoji with that Id does not exist in the guild.
    ///
    /// [Manage Emojis]: Permissions::MANAGE_EMOJIS
    #[inline]
    pub async fn delete_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
    ) -> Result<()> {
        self.id.delete_emoji(&http, emoji_id).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an integration with that Id does not exist in the guild.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn delete_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.delete_integration(&http, integration_id).await
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `model` features
    /// enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a Role with that Id does not exist in the Guild.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn delete_role(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        self.id.delete_role(&http, role_id).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if an invalid value is set, or if the current user
    /// lacks permission to edit the guild.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut EditGuild) -> &mut EditGuild,
    {
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an emoji with that Id does not exist in the guild.
    ///
    /// [Manage Emojis]: Permissions::MANAGE_EMOJIS
    #[inline]
    pub async fn edit_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
        name: &str,
    ) -> Result<Emoji> {
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
    /// GuildId(7).edit_member(user_id, |m| m.mute(true).roles(&vec![role_id])).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks the necessary permissions.
    #[inline]
    pub async fn edit_member<F>(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        f: F,
    ) -> Result<Member>
    where
        F: FnOnce(&mut EditMember) -> &mut EditMember,
    {
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
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to change their nickname.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    #[inline]
    pub async fn edit_nickname(
        &self,
        http: impl AsRef<Http>,
        new_nickname: Option<&str>,
    ) -> Result<()> {
        self.id.edit_nickname(&http, new_nickname).await
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not
    /// in the guild.
    #[inline]
    pub async fn get(http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<PartialGuild> {
        guild_id.into().to_partial_guild(&http).await
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked
    /// by the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn kick(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.kick(&http, user_id).await
    }

    #[inline]
    /// # Errors
    ///
    /// In addition to the reasons `kick` may return an error,
    /// can also return an error if the reason is too long.
    pub async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        self.id.kick_with_reason(&http, user_id, reason).await
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable.
    #[inline]
    pub async fn emojis(&self, http: impl AsRef<Http>) -> Result<Vec<Emoji>> {
        self.id.emojis(http).await
    }

    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if an `Emoji` with the given Id does
    /// not exist for the guild.
    #[inline]
    pub async fn emoji(&self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<Emoji> {
        self.id.emoji(http, emoji_id).await
    }

    /// Gets all integration of the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        self.id.integrations(&http).await
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        self.id.invites(&http).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is unable to
    /// leave the Guild, or currently is not in the guild.
    #[inline]
    pub async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(&http).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not in the Guild,
    /// or if the Guild is otherwise unavailable.
    #[inline]
    pub async fn member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Member> {
        self.id.member(cache_http, user_id).await
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results.
    /// Minimum value is 1, maximum and default value is 1000.
    ///
    /// Optionally pass in `after` to offset the results by a [`User`]'s Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error,
    /// may also return [`Error::NotInRange`] if the input is
    /// not within range.
    ///
    /// [`User`]: ../user/struct.User.html
    #[inline]
    pub async fn members(
        &self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>> {
        self.id.members(&http, limit, after).await
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission,
    /// or if the member is not currently in a voice channel for this Guild.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    #[inline]
    pub async fn move_member(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        channel_id: impl Into<ChannelId>,
    ) -> Result<Member> {
        self.id.move_member(&http, user_id, channel_id).await
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the Member has a non-existent `Role`
    /// for some reason.
    #[inline]
    pub fn user_permissions_in(
        &self,
        channel: &GuildChannel,
        member: &Member,
    ) -> Result<Permissions> {
        Guild::_user_permissions_in(channel, member, &self.roles, self.owner_id, self.id)
    }

    /// Calculate a [`Role`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the `Role` or `Channel` is not from this `Guild`.
    #[inline]
    pub fn role_permissions_in(&self, channel: &GuildChannel, role: &Role) -> Result<Permissions> {
        Guild::_role_permissions_in(channel, role, self.id)
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// See [`Guild::prune_count`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [`Guild::prune_count`]: crate::model::guild::Guild::prune_count
    #[inline]
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
    /// [`utils::shard_id`]: crate::utils::shard_id
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    pub async fn shard_id(&self, cache: impl AsRef<Cache>) -> u64 {
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
    #[inline]
    pub fn splash_url(&self) -> Option<String> {
        self.splash
            .as_ref()
            .map(|splash| format!(cdn!("/splashes/{}/{}.webp?size=4096"), self.id, splash))
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::start_integration_sync`].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Guild::start_integration_sync`]: crate::model::guild::Guild::start_integration_sync
    #[inline]
    pub async fn start_integration_sync(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.start_integration_sync(&http, integration_id).await
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::unban`].
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    /// [`Guild::unban`]: crate::model::guild::Guild::unban
    #[inline]
    pub async fn unban(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.unban(&http, user_id).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::vanity_url`].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Guild::vanity_url`]: crate::model::guild::Guild::vanity_url
    #[inline]
    pub async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(&http).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::webhooks`].
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    /// [`Guild::webhooks`]: crate::model::guild::Guild::webhooks
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
    /// # #[cfg(all(feature = "client", feature = "cache"))]
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
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("Obtained role's reference: {:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client =Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #    Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }

    /// Returns a future that will await one message sent in this guild.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reply<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).guild_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages in this guild.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_replies<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).guild_id(self.id.0)
    }

    /// Await a single reaction in this guild.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reaction<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).guild_id(self.id.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent in this guild.
    #[cfg(feature = "collector")]
    #[cfg_attr(docsrs, doc(cfg(feature = "collector")))]
    pub fn await_reactions<'a>(
        &self,
        shard_messenger: &'a impl AsRef<ShardMessenger>,
    ) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).guild_id(self.id.0)
    }
}

impl<'de> Deserialize<'de> for PartialGuild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id").and_then(|x| x.as_str()).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("roles").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(role) = value.as_object_mut() {
                        role.insert("guild_id".to_string(), from_number(guild_id));
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => from_value::<Option<ChannelId>>(v).map_err(DeError::custom)?,
            None => None,
        };
        let afk_timeout = map
            .remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let default_message_notifications = map
            .remove("default_message_notifications")
            .ok_or_else(|| DeError::custom("expected guild default_message_notifications"))
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let widget_channel_id = match map.remove("widget_channel_id") {
            Some(e) => Option::<ChannelId>::deserialize(e).map_err(DeError::custom)?,
            None => None,
        };
        let widget_enabled = map
            .remove("widget_enabled")
            .ok_or_else(|| DeError::custom("expected guild widget_enabled"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map
            .remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let features = map
            .remove("features")
            .ok_or_else(|| Error::Other("expected guild features"))
            .and_then(from_value::<Vec<String>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map
            .remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let mfa_level = map
            .remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(MfaLevel::deserialize)
            .map_err(DeError::custom)?;
        let name = map
            .remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map
            .remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let region = map
            .remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map
            .remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map
            .remove("verification_level")
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
            #[cfg(not(feature = "simd-json"))]
            Some(Value::Null) | None => 0,
            #[cfg(feature = "simd-json")]
            Some(Value::Static(StaticNode::Null)) | None => 0,
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
            widget_channel_id,
            widget_enabled,
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
        })
    }
}
