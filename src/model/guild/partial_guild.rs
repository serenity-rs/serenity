use serde::de::Error as DeError;
#[cfg(feature = "cache")]
use tracing::{error, warn};

#[cfg(feature = "model")]
use crate::builder::{
    CreateChannel,
    CreateCommand,
    CreateCommandPermissionsData,
    CreateSticker,
    EditAutoModRule,
    EditGuild,
    EditGuildWelcomeScreen,
    EditGuildWidget,
    EditMember,
    EditRole,
    EditSticker,
};
#[cfg(all(feature = "cache", feature = "utils", feature = "client"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::json::prelude::*;
#[cfg(feature = "model")]
use crate::model::application::command::{Command, CommandPermission};
#[cfg(feature = "model")]
use crate::model::guild::automod::Rule;
use crate::model::prelude::*;
use crate::model::utils::{
    add_guild_id_to_map,
    emojis,
    remove_from_map,
    remove_from_map_opt,
    roles,
    stickers,
};

/// Partial information about a [`Guild`]. This does not include information
/// like member data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct PartialGuild {
    /// Application ID of the guild creator if it is bot-created.
    pub application_id: Option<ApplicationId>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivalent to the Id of the default role (`@everyone`).
    pub id: GuildId,
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: Option<ChannelId>,
    /// The amount of seconds a user can not show any activity in a voice
    /// channel before being moved to an AFK channel -- if one exists.
    pub afk_timeout: u64,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// Whether or not the guild widget is enabled.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or null if set to no invite
    pub widget_channel_id: Option<ChannelId>,
    /// All of the guild's custom emojis.
    #[serde(with = "emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.
    ///
    /// Refer to [`Guild::features`] for more information.
    pub features: Vec<String>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    pub mfa_level: MfaLevel,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: UserId,
    /// Whether or not the user is the owner of the guild.
    #[serde(default)]
    pub owner: bool,
    /// A mapping of the guild's roles.
    #[serde(with = "roles")]
    pub roles: HashMap<RoleId, Role>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the `InviteSplash` feature is enabled, this can be used to generate
    /// a URL to a splash image.
    pub splash: Option<String>,
    /// An identifying hash of the guild discovery's splash icon.
    ///
    /// **Note**: Only present for guilds with the `DISCOVERABLE` feature.
    pub discovery_splash: Option<String>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<ChannelId>,
    /// System channel flags.
    pub system_channel_flags: SystemChannelFlags,
    /// The id of the channel where rules and/or guidelines are displayed.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub rules_channel_id: Option<ChannelId>,
    /// The id of the channel where admins and moderators of Community guilds
    /// receive notices from Discord.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub public_updates_channel_id: Option<ChannelId>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// The guild's description, if it has one.
    pub description: Option<String>,
    /// The server's premium boosting level.
    #[serde(default)]
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    pub premium_subscription_count: u64,
    /// The guild's banner, if it has one.
    pub banner: Option<String>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<String>,
    /// The welcome screen of the guild.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub welcome_screen: Option<GuildWelcomeScreen>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<u64>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<u64>,
    /// The guild NSFW state. See [`discord support article`].
    ///
    /// [`discord support article`]: https://support.discord.com/hc/en-us/articles/1500005389362-NSFW-Server-Designation
    pub nsfw_level: NsfwLevel,
    /// The maximum amount of users in a video channel.
    pub max_video_channel_users: Option<u64>,
    /// The maximum number of presences for the guild. The default value is currently 25000.
    ///
    /// **Note**: It is in effect when it is `None`.
    pub max_presences: Option<u64>,
    /// The maximum number of members for the guild.
    pub max_members: Option<u64>,
    /// The user permissions in the guild.
    pub permissions: Option<String>,
    /// All of the guild's custom stickers.
    #[serde(with = "stickers")]
    pub stickers: HashMap<StickerId, Sticker>,
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Gets all auto moderation [`Rule`]s of this guild via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn automod_rules(self, http: impl AsRef<Http>) -> Result<Vec<Rule>> {
        self.id.automod_rules(http).await
    }

    /// Gets an auto moderation [`Rule`] of this guild by its ID via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a rule with the given ID does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<Rule> {
        self.id.automod_rule(http, rule_id).await
    }

    /// Creates an auto moderation [`Rule`] in the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Examples
    ///
    /// See [`GuildId::create_automod_rule`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn create_automod_rule(
        &self,
        http: impl AsRef<Http>,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule> {
        self.id.create_automod_rule(http, builder).await
    }

    /// Edit an auto moderation [`Rule`], given its Id.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn edit_automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule> {
        self.id.edit_automod_rule(http, rule_id, builder).await
    }

    /// Deletes an auto moderation [`Rule`] from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a rule with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn delete_automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<()> {
        self.id.delete_automod_rule(http, rule_id).await
    }

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
        self.ban_with_reason(http, user, dmd, "").await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`Self::ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::ban`] may return an error,
    /// can also return an error if the reason is too long.
    #[inline]
    pub async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self.id.ban_with_reason(http, user, dmd, reason).await
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
        self.id.bans(http).await
    }

    /// Gets a list of the guild's audit log entries
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an invalid value is given.
    ///
    /// [View Audit Log]: Permissions::VIEW_AUDIT_LOG
    #[inline]
    pub async fn audit_logs(
        &self,
        http: impl AsRef<Http>,
        action_type: Option<u8>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> Result<AuditLogs> {
        self.id.audit_logs(http, action_type, user_id, before, limit).await
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
        self.id.channels(http).await
    }

    #[cfg(feature = "cache")]
    pub fn channel_id_from_name(
        &self,
        cache: impl AsRef<Cache>,
        name: impl AsRef<str>,
    ) -> Option<ChannelId> {
        let name = name.as_ref();
        let guild_channels = cache.as_ref().guild_channels(self.id)?;

        for channel_entry in guild_channels.iter() {
            let (id, channel) = channel_entry.pair();

            if channel.name == name {
                return Some(*id);
            }
        }

        None
    }

    /// Creates a [`GuildChannel`] in the guild.
    ///
    /// Refer to [`Http::create_channel`] for more information.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::guild::PartialGuild;
    /// # use serenity::model::id::GuildId;
    /// use serenity::builder::CreateChannel;
    /// use serenity::model::channel::ChannelType;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let guild = PartialGuild::get(&http, GuildId::new(7)).await?;
    /// let builder = CreateChannel::new("my-test-channel").kind(ChannelType::Text);
    ///
    /// // assuming a `guild` has already been bound
    /// let _channel = guild.create_channel(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_channel(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateChannel<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_channel(cache_http, builder).await
    }

    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more
    /// information.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
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
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn create_emoji(
        &self,
        http: impl AsRef<Http>,
        name: &str,
        image: &str,
    ) -> Result<Emoji> {
        self.id.create_emoji(http, name, image).await
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
        self.id.create_integration(http, integration_id, kind).await
    }

    /// Create a guild specific application [`Command`].
    ///
    /// **Note**: Unlike global commands, guild commands will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    #[inline]
    pub async fn create_application_command(
        &self,
        http: impl AsRef<Http>,
        builder: CreateCommand,
    ) -> Result<Command> {
        self.id.create_application_command(http, builder).await
    }

    /// Override all guild application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_application_command`].
    pub async fn set_application_commands(
        &self,
        http: impl AsRef<Http>,
        commands: Vec<CreateCommand>,
    ) -> Result<Vec<Command>> {
        self.id.set_application_commands(http, commands).await
    }

    /// Create a guild specific [`CommandPermission`].
    ///
    /// **Note**: It will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommandPermissionsData::execute`] for a list of possible errors.
    pub async fn create_application_command_permission(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
        builder: CreateCommandPermissionsData,
    ) -> Result<CommandPermission> {
        self.id.create_application_command_permission(http, command_id, builder).await
    }

    /// Get all guild application commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_commands(&self, http: impl AsRef<Http>) -> Result<Vec<Command>> {
        self.id.get_application_commands(http).await
    }

    /// Get a specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_command(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<Command> {
        self.id.get_application_command(http, command_id).await
    }

    /// Edit a guild application command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    pub async fn edit_application_command(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
        builder: CreateCommand,
    ) -> Result<Command> {
        self.id.edit_application_command(http, command_id, builder).await
    }

    /// Delete guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_application_command(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<()> {
        self.id.delete_application_command(http, command_id).await
    }

    /// Get all guild application commands permissions only.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_commands_permissions(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<CommandPermission>> {
        self.id.get_application_commands_permissions(http).await
    }

    /// Get permissions for specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_application_command_permissions(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<CommandPermission> {
        self.id.get_application_command_permissions(http, command_id).await
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn create_role(
        &self,
        cache_http: impl CacheHttp,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        self.id.create_role(cache_http, builder).await
    }

    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    pub async fn create_sticker<'a>(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateSticker<'a>,
    ) -> Result<Sticker> {
        self.id.create_sticker(cache_http, builder).await
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
    pub async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.delete(http).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an emoji with that Id does not exist in the guild.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn delete_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
    ) -> Result<()> {
        self.id.delete_emoji(http, emoji_id).await
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
        self.id.delete_integration(http, integration_id).await
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
        self.id.delete_role(http, role_id).await
    }

    /// Deletes a [`Sticker`] by Id from the guild.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to delete the sticker.
    ///
    /// [Manage Emojis and Stickers]: crate::model::permissions::Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn delete_sticker(
        &self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
    ) -> Result<()> {
        self.id.delete_sticker(http, sticker_id).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditGuild<'_>) -> Result<()> {
        let guild = self.id.edit(cache_http, builder).await?;

        self.afk_channel_id = guild.afk_channel_id;
        self.afk_timeout = guild.afk_timeout;
        self.default_message_notifications = guild.default_message_notifications;
        self.emojis = guild.emojis;
        self.features = guild.features;
        self.icon = guild.icon;
        self.mfa_level = guild.mfa_level;
        self.name = guild.name;
        self.owner_id = guild.owner_id;
        self.roles = guild.roles;
        self.splash = guild.splash;
        self.verification_level = guild.verification_level;

        Ok(())
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if an emoji with that Id does not exist in the guild.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
        name: &str,
    ) -> Result<Emoji> {
        self.id.edit_emoji(http, emoji_id, name).await
    }

    /// Edits the properties a guild member, such as muting or nicknaming them. Returns the new
    /// member.
    ///
    /// Refer to the documentation of [`EditMember`] for a full list of methods and permission
    /// restrictions.
    ///
    /// # Examples
    ///
    /// See [`GuildId::edit_member`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[inline]
    pub async fn edit_member(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        builder: EditMember<'_>,
    ) -> Result<Member> {
        self.id.edit_member(http, user_id, builder).await
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass [`None`] to reset the nickname.
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
        self.id.edit_nickname(http, new_nickname).await
    }

    /// Edits a role, optionally setting its fields.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// See the documentation of [`GuildId::edit_role`] for details.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit_role(
        &self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        self.id.edit_role(cache_http, role_id, builder).await
    }

    /// Edits the order of [`Role`]s
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change the order of a role:
    ///
    /// ```rust,ignore
    /// use serenity::model::id::RoleId;
    /// partial_guild.edit_role_position(&context, RoleId::new(8), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit_role_position(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        position: u32,
    ) -> Result<Vec<Role>> {
        self.id.edit_role_position(http, role_id, position).await
    }

    /// Edits a sticker.
    ///
    /// **Note**: Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::guild::PartialGuild;
    /// # use serenity::model::id::GuildId;
    /// use serenity::builder::EditSticker;
    /// use serenity::model::id::StickerId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let guild = PartialGuild::get(&http, GuildId::new(7)).await?;
    /// let builder = EditSticker::new().name("Bun bun meow");
    /// guild.edit_sticker(&http, StickerId::new(7), builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Emojis and Stickers]: Permissions::MANAGE_EMOJIS_AND_STICKERS
    #[inline]
    pub async fn edit_sticker(
        &self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker> {
        self.id.edit_sticker(http, sticker_id, builder).await
    }

    /// Edits the guild's welcome screen.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit_welcome_screen(
        &self,
        http: impl AsRef<Http>,
        builder: EditGuildWelcomeScreen<'_>,
    ) -> Result<GuildWelcomeScreen> {
        self.id.edit_welcome_screen(http, builder).await
    }

    /// Edits the guild's widget.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit_widget(
        &self,
        http: impl AsRef<Http>,
        builder: EditGuildWidget<'_>,
    ) -> Result<GuildWidget> {
        self.id.edit_widget(http, builder).await
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not
    /// in the guild.
    #[inline]
    pub async fn get(
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
    ) -> Result<PartialGuild> {
        guild_id.into().to_partial_guild(cache_http).await
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest
    /// [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances
    /// is not present. Returns [`None`] if the users have the same hierarchy, as
    /// neither are greater than the other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users
    /// is the guild owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[cfg(feature = "cache")]
    #[inline]
    pub fn greater_member_hierarchy(
        &self,
        cache: impl AsRef<Cache>,
        lhs_id: impl Into<UserId>,
        rhs_id: impl Into<UserId>,
    ) -> Option<UserId> {
        self._greater_member_hierarchy(&cache, lhs_id.into(), rhs_id.into())
    }

    #[cfg(feature = "cache")]
    fn _greater_member_hierarchy(
        &self,
        cache: impl AsRef<Cache>,
        lhs_id: UserId,
        rhs_id: UserId,
    ) -> Option<UserId> {
        // Check that the IDs are the same. If they are, neither is greater.
        if lhs_id == rhs_id {
            return None;
        }

        // Check if either user is the guild owner.
        if lhs_id == self.owner_id {
            return Some(lhs_id);
        } else if rhs_id == self.owner_id {
            return Some(rhs_id);
        }

        let (lhs, rhs) = {
            let cache = cache.as_ref();
            let default = (RoleId::new(1), 0);

            // Clone is necessary, highest_role_info goes into cache.
            let (lhs, rhs) = {
                let guild = cache.guild(self.id)?;
                (guild.members.get(&lhs_id)?.clone(), guild.members.get(&rhs_id)?.clone())
            };

            (
                lhs.highest_role_info(cache).unwrap_or(default),
                rhs.highest_role_info(cache).unwrap_or(default),
            )
        };

        // If LHS and RHS both have no top position or have the same role ID,
        // then no one wins.
        if (lhs.1 == 0 && rhs.1 == 0) || (lhs.0 == rhs.0) {
            return None;
        }

        // If LHS's top position is higher than RHS, then LHS wins.
        if lhs.1 > rhs.1 {
            return Some(lhs_id);
        }

        // If RHS's top position is higher than LHS, then RHS wins.
        if rhs.1 > lhs.1 {
            return Some(rhs_id);
        }

        // If LHS and RHS both have the same position, but LHS has the lower
        // role ID, then LHS wins.
        //
        // If RHS has the higher role ID, then RHS wins.
        if lhs.1 == rhs.1 && lhs.0 < rhs.0 {
            Some(lhs_id)
        } else {
            Some(rhs_id)
        }
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// If member caching is enabled the cache will be checked
    /// first. If not found it will resort to an http request.
    ///
    /// Cache is still required to look up roles.
    ///
    /// # Errors
    ///
    /// See [`Guild::member`].
    #[inline]
    #[cfg(feature = "cache")]
    pub async fn member_permissions(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Permissions> {
        self._member_permissions(cache_http, user_id.into()).await
    }

    #[cfg(feature = "cache")]
    async fn _member_permissions(
        &self,
        cache_http: impl CacheHttp,
        user_id: UserId,
    ) -> Result<Permissions> {
        if user_id == self.owner_id {
            return Ok(Permissions::all());
        }

        let everyone = if let Some(everyone) = self.roles.get(&RoleId(self.id.0)) {
            everyone
        } else {
            error!("@everyone role ({}) missing in '{}'", self.id, self.name,);

            return Ok(Permissions::empty());
        };

        let member = self.member(cache_http, &user_id).await?;

        let mut permissions = everyone.permissions;

        for role in &member.roles {
            if let Some(role) = self.roles.get(role) {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Ok(Permissions::all());
                }

                permissions |= role.permissions;
            } else {
                warn!("{} on {} has non-existent role {:?}", member.user.id, self.id, role,);
            }
        }

        Ok(permissions)
    }

    /// Re-orders the channels of the guild.
    ///
    /// Although not required, you should specify all channels' positions,
    /// regardless of whether they were updated. Otherwise, positioning can
    /// sometimes get weird.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is lacking permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    #[inline]
    pub async fn reorder_channels<It>(&self, http: impl AsRef<Http>, channels: It) -> Result<()>
    where
        It: IntoIterator<Item = (ChannelId, u64)>,
    {
        self.id.reorder_channels(http, channels).await
    }

    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname
    /// starts with a provided string.
    ///
    /// Optionally pass in the `limit` to limit the number of results.
    /// Minimum value is 1, maximum and default value is 1000.
    ///
    /// **Note**: Queries are case insensitive.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    #[inline]
    pub async fn search_members(
        &self,
        http: impl AsRef<Http>,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Member>> {
        self.id.search_members(http, query, limit).await
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to kick members.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have
    /// permission.
    ///
    /// Can also return an [`Error::Json`] if there is an error deserializing
    /// the API response.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn start_prune(&self, cache_http: impl CacheHttp, days: u8) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::KICK_MEMBERS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.start_prune(cache_http.http(), days).await
    }

    #[cfg(feature = "cache")]
    async fn has_perms(&self, cache_http: impl CacheHttp, mut permissions: Permissions) -> bool {
        if let Some(cache) = cache_http.cache() {
            let user_id = cache.current_user().id;

            if let Ok(perms) = self.member_permissions(cache_http, user_id).await {
                permissions.remove(perms);

                permissions.is_empty()
            } else {
                false
            }
        } else {
            false
        }
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
        self.id.kick(http, user_id).await
    }

    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error,
    /// can also return an error if the reason is too long.
    #[inline]
    pub async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        self.id.kick_with_reason(http, user_id, reason).await
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| cdn!("/icons/{}/{}.webp", self.id, icon))
    }

    /// Returns a formatted URL of the guild's banner, if the guild has a banner.
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp", self.id, banner))
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
    /// Returns [`Error::Http`] if an [`Emoji`] with the given Id does
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
        self.id.integrations(http).await
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
        self.id.invites(http).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is unable to
    /// leave the Guild, or currently is not in the guild.
    #[inline]
    pub async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(http).await
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
    /// [`User`]: crate::model::user::User
    #[inline]
    pub async fn members(
        &self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>> {
        self.id.members(http, limit, after).await
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
        self.id.move_member(http, user_id, channel_id).await
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the Member has a non-existent [`Role`]
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
    /// Returns [`Error::Model`] if the [`Role`] or [`Channel`] is not from this [`Guild`].
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
    pub async fn prune_count(&self, http: impl AsRef<Http>, days: u8) -> Result<GuildPrune> {
        self.id.prune_count(http, days).await
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
    #[must_use]
    pub fn shard_id(&self, cache: impl AsRef<Cache>) -> u32 {
        self.id.shard_id(cache)
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
    #[must_use]
    pub fn shard_id(&self, shard_count: u32) -> u32 {
        self.id.shard_id(shard_count)
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[inline]
    #[must_use]
    pub fn splash_url(&self) -> Option<String> {
        self.splash.as_ref().map(|splash| cdn!("/splashes/{}/{}.webp?size=4096", self.id, splash))
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
        self.id.start_integration_sync(http, integration_id).await
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
        self.id.unban(http, user_id).await
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
        self.id.vanity_url(http).await
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
        self.id.webhooks(http).await
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
    ///             if let Some(guild) = guild_id.to_guild_cached(&context) {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("Obtained role's reference: {:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client =
    ///     Client::builder("token", GatewayIntents::default()).event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #    Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages in this guild.
    #[cfg(feature = "collector")]
    pub fn reply_collector(&self, shard_messenger: &ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).guild_id(self.id)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of reactions sent in this guild.
    #[cfg(feature = "collector")]
    pub fn reaction_collector(&self, shard_messenger: &ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).guild_id(self.id)
    }

    /// Gets the guild active threads.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or
    /// if the bot issuing the request is not in the guild.
    pub async fn get_active_threads(&self, http: impl AsRef<Http>) -> Result<ThreadsData> {
        self.id.get_active_threads(http).await
    }
}

impl<'de> Deserialize<'de> for PartialGuild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = remove_from_map(&mut map, "id")?;

        add_guild_id_to_map(&mut map, "roles", id);

        let emojis = map
            .remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(emojis::deserialize)
            .map_err(DeError::custom)?;

        let roles = map
            .remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(roles::deserialize)
            .map_err(DeError::custom)?;

        let premium_subscription_count = match map.remove("premium_subscription_count") {
            #[cfg(not(feature = "simd-json"))]
            Some(Value::Null) | None => 0,
            #[cfg(feature = "simd-json")]
            Some(Value::Static(StaticNode::Null)) | None => 0,
            Some(v) => u64::deserialize(v).map_err(DeError::custom)?,
        };

        let stickers = map
            .remove("stickers")
            .ok_or_else(|| DeError::custom("expected guild stickers"))
            .and_then(stickers::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            id,
            emojis,
            roles,
            premium_subscription_count,
            stickers,
            afk_channel_id: remove_from_map_opt(&mut map, "afk_channel_id")?.flatten(),
            afk_timeout: remove_from_map(&mut map, "afk_timeout")?,
            application_id: remove_from_map_opt(&mut map, "application_id")?.flatten(),
            default_message_notifications: remove_from_map(
                &mut map,
                "default_message_notifications",
            )?,
            features: remove_from_map(&mut map, "features")?,
            widget_enabled: remove_from_map_opt(&mut map, "widget_enabled")?.flatten(),
            widget_channel_id: remove_from_map_opt(&mut map, "widget_channel_id")?.flatten(),
            icon: remove_from_map_opt(&mut map, "icon")?.flatten(),
            mfa_level: remove_from_map(&mut map, "mfa_level")?,
            name: remove_from_map(&mut map, "name")?,
            owner_id: remove_from_map(&mut map, "owner_id")?,
            owner: remove_from_map_opt(&mut map, "owner")?.unwrap_or_default(),
            splash: remove_from_map_opt(&mut map, "splash")?.flatten(),
            discovery_splash: remove_from_map_opt(&mut map, "discovery_splash")?.flatten(),
            system_channel_id: remove_from_map_opt(&mut map, "system_channel_id")?.flatten(),
            system_channel_flags: remove_from_map(&mut map, "system_channel_flags")?,
            rules_channel_id: remove_from_map_opt(&mut map, "rules_channel_id")?.flatten(),
            public_updates_channel_id: remove_from_map_opt(&mut map, "public_updates_channel_id")?
                .flatten(),
            verification_level: remove_from_map(&mut map, "verification_level")?,
            description: remove_from_map_opt(&mut map, "description")?.flatten(),
            premium_tier: remove_from_map_opt(&mut map, "premium_tier")?.unwrap_or_default(),
            banner: remove_from_map_opt(&mut map, "banner")?.flatten(),
            vanity_url_code: remove_from_map_opt(&mut map, "vanity_url_code")?.flatten(),
            welcome_screen: remove_from_map_opt(&mut map, "welcome_screen")?,
            approximate_member_count: remove_from_map_opt(&mut map, "approximate_member_count")?,
            approximate_presence_count: remove_from_map_opt(
                &mut map,
                "approximate_presence_count",
            )?,
            nsfw_level: remove_from_map(&mut map, "nsfw_level")?,
            max_video_channel_users: remove_from_map(&mut map, "max_video_channel_users")?,
            max_presences: remove_from_map_opt(&mut map, "max_presences")?.flatten(),
            max_members: remove_from_map_opt(&mut map, "max_members")?.flatten(),
            permissions: remove_from_map_opt(&mut map, "permissions")?.unwrap_or_default(),
        })
    }
}

impl From<Guild> for PartialGuild {
    /// Converts this [`Guild`] instance into a [`PartialGuild`]
    ///
    /// [`PartialGuild`] is not a strict subset and contains some data specific to the current user
    /// that [`Guild`] does not contain. Therefore, this method needs access to cache and HTTP to
    /// generate the missing data
    fn from(guild: Guild) -> Self {
        Self {
            application_id: guild.application_id,
            id: guild.id,
            afk_channel_id: guild.afk_channel_id,
            afk_timeout: guild.afk_timeout,
            default_message_notifications: guild.default_message_notifications,
            widget_enabled: guild.widget_enabled,
            widget_channel_id: guild.widget_channel_id,
            emojis: guild.emojis,
            features: guild.features,
            icon: guild.icon,
            mfa_level: guild.mfa_level,
            name: guild.name,
            owner_id: guild.owner_id,
            // Not very good, but PartialGuild deserialization uses .unwrap_or_default() too
            owner: Default::default(),
            roles: guild.roles,
            splash: guild.splash,
            discovery_splash: guild.discovery_splash,
            system_channel_id: guild.system_channel_id,
            system_channel_flags: guild.system_channel_flags,
            rules_channel_id: guild.rules_channel_id,
            public_updates_channel_id: guild.public_updates_channel_id,
            verification_level: guild.verification_level,
            description: guild.description,
            premium_tier: guild.premium_tier,
            premium_subscription_count: guild.premium_subscription_count,
            banner: guild.banner,
            vanity_url_code: guild.vanity_url_code,
            welcome_screen: guild.welcome_screen,
            approximate_member_count: guild.approximate_member_count,
            approximate_presence_count: guild.approximate_presence_count,
            nsfw_level: guild.nsfw_level,
            max_video_channel_users: guild.max_video_channel_users,
            max_presences: guild.max_presences,
            max_members: guild.max_members,
            permissions: None,
            stickers: guild.stickers,
        }
    }
}
