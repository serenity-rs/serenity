//! Models relating to guilds and types that it owns.

pub mod audit_log;
pub mod automod;
mod emoji;
mod guild_id;
mod guild_preview;
mod integration;
mod member;
mod partial_guild;
mod premium_tier;
mod role;
mod scheduled_event;
mod system_channel;
mod welcome_screen;

#[cfg(feature = "model")]
use std::borrow::Cow;

#[cfg(feature = "model")]
use tracing::error;
#[cfg(all(feature = "model", feature = "cache"))]
use tracing::warn;

pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::guild_preview::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::premium_tier::*;
pub use self::role::*;
pub use self::scheduled_event::*;
pub use self::system_channel::*;
pub use self::welcome_screen::*;
use super::utils::*;
#[cfg(feature = "model")]
use crate::builder::{
    AddMember,
    CreateChannel,
    CreateCommand,
    CreateCommandPermissionsData,
    CreateScheduledEvent,
    CreateSticker,
    EditAutoModRule,
    EditGuild,
    EditGuildWelcomeScreen,
    EditGuildWidget,
    EditMember,
    EditRole,
    EditScheduledEvent,
    EditSticker,
};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "model")]
use crate::constants::LARGE_THRESHOLD;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, UserPagination};
#[cfg(feature = "model")]
use crate::json::prelude::json;
#[cfg(feature = "model")]
use crate::model::application::command::{Command, CommandPermission};
#[cfg(feature = "model")]
use crate::model::guild::automod::Rule;
use crate::model::prelude::*;
use crate::model::utils::{emojis, presences, roles, stickers};
use crate::model::Timestamp;

/// A representation of a banning of a user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#ban-object).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<String>,
    /// The user that was banned.
    pub user: User,
}

/// Information about a Discord guild, such as channels, emojis, etc.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object) plus
/// [extension](https://discord.com/developers/docs/topics/gateway#guild-create).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Guild {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: Option<ChannelId>,
    /// The amount of seconds a user can not show any activity in a voice
    /// channel before being moved to an AFK channel -- if one exists.
    pub afk_timeout: u64,
    /// Application ID of the guild creator if it is bot-created.
    pub application_id: Option<ApplicationId>,
    /// All voice and text channels contained within a guild.
    ///
    /// This contains all channels regardless of permissions (i.e. the ability
    /// of the bot to read from or connect to them).
    #[serde(serialize_with = "serialize_map_values")]
    #[serde(deserialize_with = "deserialize_guild_channels")]
    pub channels: HashMap<ChannelId, GuildChannel>,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// All of the guild's custom emojis.
    #[serde(with = "emojis")]
    pub emojis: HashMap<EmojiId, Emoji>,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// The guild features. More information available at
    /// [`discord documentation`].
    ///
    /// The following is a list of known features:
    ///
    /// - `ANIMATED_ICON`
    /// - `BANNER`
    /// - `COMMERCE`
    /// - `COMMUNITY`
    /// - `DISCOVERABLE`
    /// - `FEATURABLE`
    /// - `INVITE_SPLASH`
    /// - `MEMBER_VERIFICATION_GATE_ENABLED`
    /// - `MONETIZATION_ENABLED`
    /// - `MORE_STICKERS`
    /// - `NEWS`
    /// - `PARTNERED`
    /// - `PREVIEW_ENABLED`
    /// - `PRIVATE_THREADS`
    /// - `ROLE_ICONS`
    /// - `SEVEN_DAY_THREAD_ARCHIVE`
    /// - `THREE_DAY_THREAD_ARCHIVE`
    /// - `TICKETED_EVENTS_ENABLED`
    /// - `VANITY_URL`
    /// - `VERIFIED`
    /// - `VIP_REGIONS`
    /// - `WELCOME_SCREEN_ENABLED`
    /// - `THREE_DAY_THREAD_ARCHIVE`
    /// - `SEVEN_DAY_THREAD_ARCHIVE`
    /// - `PRIVATE_THREADS`
    ///
    ///
    /// [`discord documentation`]: https://discord.com/developers/docs/resources/guild#guild-object-guild-features
    pub features: Vec<String>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivalent to the Id of the default role (`@everyone`).
    pub id: GuildId,
    /// The date that the current user joined the guild.
    pub joined_at: Timestamp,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received
    /// if the [`Self::member_count`] is greater than the [`LARGE_THRESHOLD`] set by
    /// the library.
    #[serde(serialize_with = "serialize_map_values")]
    #[serde(deserialize_with = "deserialize_members")]
    pub members: HashMap<UserId, Member>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    pub mfa_level: MfaLevel,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: UserId,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// **Note**: This will be empty unless the "guild presences" privileged
    /// intent is enabled.
    #[serde(with = "presences")]
    pub presences: HashMap<UserId, Presence>,
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
    /// A mapping of [`User`]s to their current voice state.
    #[serde(serialize_with = "serialize_map_values")]
    #[serde(deserialize_with = "deserialize_voice_states")]
    pub voice_states: HashMap<UserId, VoiceState>,
    /// The server's description, if it has one.
    pub description: Option<String>,
    /// The server's premium boosting level.
    #[serde(default)]
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    #[serde(default)]
    pub premium_subscription_count: u64,
    /// The guild's banner, if it has one.
    pub banner: Option<String>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<String>,
    /// The preferred locale of this guild only set if guild has the "DISCOVERABLE"
    /// feature, defaults to en-US.
    pub preferred_locale: String,
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
    /// Whether or not the guild widget is enabled.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or null if set to no invite
    pub widget_channel_id: Option<ChannelId>,
    /// The stage instances in this guild.
    #[serde(default)]
    pub stage_instances: Vec<StageInstance>,
    /// All active threads in this guild that current user has permission to view.
    #[serde(default)]
    pub threads: Vec<GuildChannel>,
    /// All of the guild's custom stickers.
    #[serde(with = "stickers")]
    pub stickers: HashMap<StickerId, Sticker>,
}

#[cfg(feature = "model")]
impl Guild {
    /// Gets all auto moderation [`Rule`]s of this guild via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the guild is unavailable.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn automod_rules(&self, http: impl AsRef<Http>) -> Result<Vec<Rule>> {
        self.id.automod_rules(http).await
    }

    /// Gets an auto moderation [`Rule`] of this guild by its ID via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if a rule with the given ID does not exist.
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

    #[cfg(feature = "cache")]
    fn check_hierarchy(&self, cache: impl AsRef<Cache>, other_user: UserId) -> Result<()> {
        let current_id = cache.as_ref().current_user().id;

        if let Some(higher) = self.greater_member_hierarchy(&cache, other_user, current_id) {
            if higher != current_id {
                return Err(Error::Model(ModelError::Hierarchy));
            }
        }

        Ok(())
    }

    /// Returns the "default" channel of the guild for the passed user id.
    /// (This returns the first channel that can be read by the user, if there isn't one,
    /// returns [`None`])
    #[must_use]
    pub fn default_channel(&self, uid: UserId) -> Option<&GuildChannel> {
        let member = self.members.get(&uid)?;
        self.channels.values().find(|&channel| {
            channel.kind != ChannelType::Category
                && self
                    .user_permissions_in(channel, member)
                    .map_or(false, Permissions::view_channel)
        })
    }

    /// Returns the guaranteed "default" channel of the guild.
    /// (This returns the first channel that can be read by everyone, if there isn't one,
    /// returns [`None`])
    ///
    /// **Note**: This is very costly if used in a server with lots of channels,
    /// members, or both.
    #[must_use]
    pub fn default_channel_guaranteed(&self) -> Option<&GuildChannel> {
        self.channels.values().find(|&channel| {
            channel.kind != ChannelType::Category
                && self
                    .members
                    .values()
                    .filter_map(|member| self.user_permissions_in(channel, member).ok())
                    .all(Permissions::view_channel)
        })
    }

    #[cfg(feature = "cache")]
    pub(crate) async fn has_perms(
        &self,
        cache_http: impl CacheHttp,
        mut permissions: Permissions,
    ) -> bool {
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

    /// Ban a [`User`] from the guild, deleting a number of
    /// days' worth of messages (`dmd`) between the range 0 and 7.
    ///
    /// Refer to the documentation for [`Guild::ban`] for more information.
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
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans, or may
    /// return a [`ModelError::Hierarchy`] if the member to be banned has a
    /// higher role than the current user.
    ///
    /// Otherwise returns [`Error::Http`] if the member cannot be banned.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn ban(
        &self,
        cache_http: impl CacheHttp,
        user: impl Into<UserId>,
        dmd: u8,
    ) -> Result<()> {
        self._ban_with_reason(cache_http, user.into(), dmd, "").await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`Self::ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the possible reasons [`Self::ban`] may return an error, an [`Error::ExceededLimit`]
    /// may also be returned if the reason is too long.
    #[inline]
    pub async fn ban_with_reason(
        &self,
        cache_http: impl CacheHttp,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self._ban_with_reason(cache_http, user.into(), dmd, reason.as_ref()).await
    }

    async fn _ban_with_reason(
        &self,
        cache_http: impl CacheHttp,
        user: UserId,
        dmd: u8,
        reason: &str,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }

                self.check_hierarchy(cache, user)?;
            }
        }

        self.id.ban_with_reason(cache_http.http(), user, dmd, reason).await
    }

    /// Returns the formatted URL of the guild's banner image, if one exists.
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp?size=1024", self.id, banner))
    }

    /// Retrieves a list of [`Ban`]s for the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn bans(&self, cache_http: impl CacheHttp) -> Result<Vec<Ban>> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.bans(cache_http.http()).await
    }

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[inline]
    pub async fn add_member(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        builder: AddMember,
    ) -> Result<Option<Member>> {
        self.id.add_member(http, user_id, builder).await
    }

    /// Retrieves a list of [`AuditLogs`] for the guild.
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission
    /// to view the audit log, or if an invalid value is given.
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
    /// Returns [`Error::Http`] if the guild is currently unavailable.
    #[inline]
    pub async fn channels(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.channels(http).await
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full
    /// [`Guild`] will be received over a [`Shard`].
    ///
    /// **Note**: This endpoint is usually only available for user accounts.
    /// Refer to Discord's information for the endpoint [here][whitelist] for
    /// more information. If you require this as a bot, re-think what you are
    /// doing and if it _really_ needs to be doing this.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region] with no icon:
    ///
    /// ```rust,ignore
    /// use serenity::model::Guild;
    ///
    /// let _guild = Guild::create_guild(&http, "test", None).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user cannot create a Guild.
    ///
    /// [`Shard`]: crate::gateway::Shard
    /// [whitelist]: https://discord.com/developers/docs/resources/guild#create-guild
    pub async fn create(
        http: impl AsRef<Http>,
        name: &str,
        icon: Option<&str>,
    ) -> Result<PartialGuild> {
        let map = json!({
            "icon": icon,
            "name": name,
        });

        http.as_ref().create_guild(&map).await
    }

    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::guild::Guild;
    /// # use serenity::model::id::GuildId;
    /// use serenity::builder::CreateChannel;
    /// use serenity::model::channel::ChannelType;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let guild = Guild::get(&http, GuildId::new(7)).await?;
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

    /// Creates an emoji in the guild with a name and base64-encoded image. The
    /// [`CreateAttachment`] builder is provided for you as a simple method to
    /// read an image and encode it into base64, if you are reading from the
    /// filesystem.
    ///
    /// The name of the emoji must be at least 2 characters long and can only
    /// contain alphanumeric characters and underscores.
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
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [`CreateAttachment`]: crate::builder::CreateAttachment
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
    pub async fn create_integration<I>(
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
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// See the documentation for [`EditRole`] for details.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn create_role(
        &self,
        cache_http: impl CacheHttp,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        self.id.create_role(cache_http, builder).await
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn create_scheduled_event(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        self.id.create_scheduled_event(cache_http, builder).await
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
        self.id.create_sticker(cache_http.http(), builder).await
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidUser`]
    /// if the current user is not the guild owner.
    ///
    /// Otherwise returns [`Error::Http`] if the current user is not the
    /// owner of the guild.
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.owner_id != cache.current_user().id {
                    let req = Permissions::MANAGE_GUILD;

                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.delete(cache_http.http()).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
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
    /// Returns an [`Error::Http`] if the current user lacks permission,
    /// or if an Integration with that Id does not exist.
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
    /// Returns [`Error::Http`] if the current user lacks permission
    /// to delete the role.
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

    /// Deletes a [Scheduled Event] by Id from the guild.
    ///
    /// Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete the scheduled event.
    ///
    /// [Scheduled Event]: ScheduledEvent
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    #[inline]
    pub async fn delete_scheduled_event(
        &self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
    ) -> Result<()> {
        self.id.delete_scheduled_event(http, event_id).await
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
    /// # Examples
    ///
    /// Change a guild's icon using a file named "icon.png":
    ///
    /// ```rust,no_run
    /// # use serenity::builder::{EditGuild, CreateAttachment};
    /// # use serenity::{http::Http, model::id::GuildId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::new("token");
    /// #     let mut guild = GuildId::new(1).to_partial_guild(&http).await?;
    /// let base64_icon = CreateAttachment::path("./icon.png").await?.to_base64();
    ///
    /// // assuming a `guild` has already been bound
    /// let builder = EditGuild::new().icon(Some(base64_icon));
    /// guild.edit(&http, builder).await?;
    /// #     Ok(())
    /// # }
    /// ```
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
    /// Also see [`Emoji::edit`] if you have the `cache` and `model` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis and Stickers] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
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
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    pub async fn edit_nickname(
        &self,
        cache_http: impl CacheHttp,
        new_nickname: Option<&str>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::CHANGE_NICKNAME;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.edit_nickname(cache_http.http(), new_nickname).await
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
    /// guild.edit_role_position(&context, RoleId::new(8), 2);
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

    /// Modifies a scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn edit_scheduled_event(
        &self,
        cache_http: impl CacheHttp,
        event_id: impl Into<ScheduledEventId>,
        builder: EditScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        self.id.edit_scheduled_event(cache_http, event_id, builder).await
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
    /// # use serenity::model::guild::Guild;
    /// # use serenity::model::id::GuildId;
    /// use serenity::builder::EditSticker;
    /// use serenity::model::id::StickerId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::new("token");
    /// # let guild = Guild::get(&http, GuildId::new(7)).await?;
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
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
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

        let lhs =
            self.members.get(&lhs_id)?.highest_role_info(&cache).unwrap_or((RoleId::new(1), 0));
        let rhs =
            self.members.get(&rhs_id)?.highest_role_info(&cache).unwrap_or((RoleId::new(1), 0));

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

    /// Returns the formatted URL of the guild's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the guild has a GIF icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            let ext = if icon.starts_with("a_") { "gif" } else { "webp" };

            cdn!("/icons/{}/{}.{}", self.id, icon, ext)
        })
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable
    #[inline]
    pub async fn emojis(&self, http: impl AsRef<Http>) -> Result<Vec<Emoji>> {
        self.id.emojis(http).await
    }

    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an emoji with that Id does not exist
    /// in the guild, or if the guild is unavailable.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing
    /// the API response.
    #[inline]
    pub async fn emoji(&self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<Emoji> {
        self.id.emoji(http, emoji_id).await
    }

    /// Gets all integration of the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission
    /// to see integrations.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing
    /// the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        self.id.integrations(http).await
    }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to see invites.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have
    /// permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn invites(&self, cache_http: impl CacheHttp) -> Result<Vec<RichInvite>> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::MANAGE_GUILD;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.invites(cache_http.http()).await
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    #[inline]
    #[must_use]
    pub fn is_large(&self) -> bool {
        self.members.len() > LARGE_THRESHOLD as usize
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked by
    /// the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn kick(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.kick(http, user_id).await
    }

    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error,
    /// may also return an error if the reason is too long.
    #[inline]
    pub async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        self.id.kick_with_reason(http, user_id, reason).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the current user
    /// cannot leave the guild, or currently is not in the guild.
    #[inline]
    pub async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(http).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled [`Self::members`] will be checked
    /// first, if so, a reference to the member will be returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in
    /// the guild or if the guild is otherwise unavailable.
    #[inline]
    pub async fn member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Cow<'_, Member>> {
        let user_id = user_id.into();

        if let Some(member) = self.members.get(&user_id) {
            Ok(Cow::Borrowed(member))
        } else {
            cache_http.http().get_member(self.id, user_id).await.map(Cow::Owned)
        }
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
    /// Returns an [`Error::Http`] if the API returns an error, may also
    /// return [`Error::NotInRange`] if the input is not within range.
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

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    #[must_use]
    pub fn members_with_status(&self, status: OnlineStatus) -> Vec<&Member> {
        let mut members = vec![];

        for (&id, member) in &self.members {
            if let Some(presence) = self.presences.get(&id) {
                if status == presence.status {
                    members.push(member);
                }
            }
        }

        members
    }

    /// Retrieves the first [`Member`] found that matches the name - with an
    /// optional discriminator - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup,
    /// as no two people can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator,
    /// then a search will be done for the nickname. When searching by nickname,
    /// the hash (`#`) and everything after it is included in the search.
    ///
    /// The following are valid types of searches:
    ///
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    ///
    /// **Note**: This will only search members that are cached. If you want to
    /// search all members in the guild via the Http API, use
    /// [`Self::search_members`].
    #[must_use]
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (username, discrim) = match crate::utils::parse_user_tag(name) {
            Some((username, discrim)) => (username, Some(discrim)),
            None => (name, None),
        };

        for member in self.members.values() {
            if member.user.name == username
                && discrim.map_or(true, |d| member.user.discriminator == d)
            {
                return Some(member);
            }
        }

        self.members.values().find(|member| member.nick.as_ref().map_or(false, |nick| nick == name))
    }

    /// Retrieves all [`Member`] that start with a given [`String`].
    ///
    /// `sorted` decides whether the best early match of the `prefix`
    /// should be the criteria to sort the result.
    /// For the `prefix` "zey" and the unsorted result:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    /// It would be sorted:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// **Locking**:
    /// First collects a [`Member`]'s [`User`]-name by read-locking all inner
    /// [`User`]s, and then sorts. This ensures that no name is being changed
    /// after being sorted in the originally correct position.
    /// However, since the read-locks are dropped after borrowing the name,
    /// the names might have been changed by the user, the sorted list cannot
    /// account for this.
    ///
    /// **Note**: This will only search members that are cached. If you want to
    /// search all members in the guild via the Http API, use
    /// [`Self::search_members`].
    #[must_use]
    pub fn members_starting_with(
        &self,
        prefix: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        fn starts_with(name: &str, prefix: &str, case_sensitive: bool) -> bool {
            if case_sensitive {
                name.starts_with(prefix)
            } else {
                name.to_lowercase().starts_with(&prefix.to_lowercase())
            }
        }

        let mut members = self
            .members
            .values()
            .filter_map(|member| {
                let username = &member.user.name;

                if starts_with(username, prefix, case_sensitive) {
                    Some((member, username.clone()))
                } else {
                    match &member.nick {
                        Some(nick) => starts_with(nick, prefix, case_sensitive)
                            .then(|| (member, nick.clone())),
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(prefix, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] as
    /// either username or nick, with a priority on username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sorted` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// It will look at the account name first, if that does not fit the
    /// search-criteria `substring`, the display-name will be considered.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Due to two fields of a [`Member`] being candidates for
    /// the searched field, setting `sorted` to `true` will result in an overhead,
    /// as both fields have to be considered again for sorting.
    ///
    /// **Locking**:
    /// First collects a [`Member`]'s [`User`]-name by read-locking all inner
    /// [`User`]s, and then sorts. This ensures that no name is being changed
    /// after being sorted in the originally correct position.
    /// However, since the read-locks are dropped after borrowing the name,
    /// the names might have been changed by the user, the sorted list cannot
    /// account for this.
    ///
    /// **Note**: This will only search members that are cached. If you want to
    /// search all members in the guild via the Http API, use
    /// [`Self::search_members`].
    #[must_use]
    pub fn members_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .values()
            .filter_map(|member| {
                let username = &member.user.name;

                if contains(username, substring, case_sensitive) {
                    Some((member, username.clone()))
                } else {
                    match &member.nick {
                        Some(nick) => contains(nick, substring, case_sensitive)
                            .then(|| (member, nick.clone())),
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves a tuple of [`Member`]s containing a given [`String`] in
    /// their username as the first field and the name used for sorting
    /// as the second field.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Locking**:
    /// First collects a [`Member`]'s [`User`]-name by read-locking all inner
    /// [`User`]s, and then sorts. This ensures that no name is being changed
    /// after being sorted in the originally correct position.
    /// However, since the read-locks are dropped after borrowing the name,
    /// the names might have been changed by the user, the sorted list cannot
    /// account for this.
    ///
    /// **Note**: This will only search members that are cached. If you want to
    /// search all members in the guild via the Http API, use
    /// [`Self::search_members`].
    #[must_use]
    pub fn members_username_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .values()
            .filter_map(|member| {
                let name = &member.user.name;
                contains(name, substring, case_sensitive).then(|| (member, name.clone()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] in
    /// their nick.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Instead of panicking, when sorting does not find
    /// a nick, the username will be used (this should never happen).
    ///
    /// **Locking**:
    /// First collects a [`Member`]'s nick directly or by read-locking all inner
    /// [`User`]s (in case of no nick, see note above), and then sorts.
    /// This ensures that no name is being changed after being sorted in the
    /// originally correct position.
    /// However, since the read-locks are dropped after borrowing the name,
    /// the names might have been changed by the user, the sorted list cannot
    /// account for this.
    ///
    /// **Note**: This will only search members that are cached. If you want to
    /// search all members in the guild via the Http API, use
    /// [`Self::search_members`].
    #[must_use]
    pub fn members_nick_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .values()
            .filter_map(|member| {
                let nick = member.nick.as_ref().unwrap_or(&member.user.name);
                contains(nick, substring, case_sensitive).then(|| (member, nick.clone()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
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

        let member = self.member(cache_http, &user_id).await?;

        Ok(self._member_permission_from_member(&member))
    }

    /// Helper function that's used for getting a [`Member`]'s permissions.
    #[cfg(feature = "cache")]
    pub(crate) fn _member_permission_from_member(&self, member: &Member) -> Permissions {
        if member.user.id == self.owner_id {
            return Permissions::all();
        }

        let everyone = if let Some(everyone) = self.roles.get(&RoleId(self.id.0)) {
            everyone
        } else {
            error!("@everyone role ({}) missing in '{}'", self.id, self.name);

            return Permissions::empty();
        };

        let mut permissions = everyone.permissions;

        for role in &member.roles {
            if let Some(role) = self.roles.get(role) {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Permissions::all();
                }

                permissions |= role.permissions;
            } else {
                warn!("{} on {} has non-existent role {:?}", member.user.id, self.id, role,);
            }
        }

        permissions
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user
    /// lacks permission, or if the member is not currently
    /// in a voice channel for this [`Guild`].
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
    /// Returns [`Error::Model`] if the [`Member`] has a non-existent role
    /// for some reason.
    #[inline]
    pub fn user_permissions_in(
        &self,
        channel: &GuildChannel,
        member: &Member,
    ) -> Result<Permissions> {
        Self::_user_permissions_in(channel, member, &self.roles, self.owner_id, self.id)
    }

    /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn _user_permissions_in(
        channel: &GuildChannel,
        member: &Member,
        roles: &HashMap<RoleId, Role>,
        owner_id: UserId,
        guild_id: GuildId,
    ) -> Result<Permissions> {
        // The owner has all permissions in all cases.
        if member.user.id == owner_id {
            return Ok(Self::remove_unnecessary_voice_permissions(channel, Permissions::all()));
        }

        // Start by retrieving the @everyone role's permissions.
        let everyone = if let Some(everyone) = roles.get(&RoleId(guild_id.0)) {
            everyone
        } else {
            error!("@everyone role missing in {}", guild_id,);
            return Err(Error::Model(ModelError::RoleNotFound));
        };

        // Create a base set of permissions, starting with `@everyone`s.
        let mut permissions = everyone.permissions;

        for &role in &member.roles {
            if let Some(role) = roles.get(&role) {
                permissions |= role.permissions;
            } else {
                error!("{} on {} has non-existent role {:?}", member.user.id, guild_id, role);
                return Err(Error::Model(ModelError::RoleNotFound));
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Ok(Self::remove_unnecessary_voice_permissions(channel, Permissions::all()));
        }

        // Apply the permission overwrites for the channel for each of the
        // overwrites that - first - applies to the member's roles, and then
        // the member itself.
        //
        // First apply the denied permission overwrites for each, then apply
        // the allowed.

        let mut data = Vec::with_capacity(member.roles.len());

        // Roles
        for overwrite in &channel.permission_overwrites {
            if let PermissionOverwriteType::Role(role) = overwrite.kind {
                if role.0 != guild_id.0 && !member.roles.contains(&role) {
                    continue;
                }

                if let Some(role) = roles.get(&role) {
                    data.push((role.position, overwrite.deny, overwrite.allow));
                }
            }
        }

        data.sort_by(|a, b| a.0.cmp(&b.0));

        for overwrite in data {
            permissions = (permissions & !overwrite.1) | overwrite.2;
        }

        // Member
        for overwrite in &channel.permission_overwrites {
            if PermissionOverwriteType::Member(member.user.id) != overwrite.kind {
                continue;
            }

            permissions = (permissions & !overwrite.deny) | overwrite.allow;
        }

        // The default channel is always readable.
        if channel.id.0 == guild_id.0 {
            permissions |= Permissions::VIEW_CHANNEL;
        }

        Self::remove_unusable_permissions(&mut permissions);

        Ok(permissions)
    }

    /// Calculate a [`Role`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Will return an [`Error::Model`] if the [`Role`] or [`Channel`] is not from this [`Guild`].
    #[inline]
    pub fn role_permissions_in(&self, channel: &GuildChannel, role: &Role) -> Result<Permissions> {
        Self::_role_permissions_in(channel, role, self.id)
    }

    /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn _role_permissions_in(
        channel: &GuildChannel,
        role: &Role,
        guild_id: GuildId,
    ) -> Result<Permissions> {
        // Fail if the role or channel is not from this guild.
        if role.guild_id != guild_id || channel.guild_id != guild_id {
            return Err(Error::Model(ModelError::WrongGuild));
        }

        let mut permissions = role.permissions;

        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Ok(Self::remove_unnecessary_voice_permissions(channel, Permissions::all()));
        }

        for overwrite in &channel.permission_overwrites {
            if let PermissionOverwriteType::Role(permissions_role_id) = overwrite.kind {
                if permissions_role_id == role.id {
                    permissions = (permissions & !overwrite.deny) | overwrite.allow;

                    break;
                }
            }
        }

        Self::remove_unusable_permissions(&mut permissions);

        Ok(permissions)
    }

    /// Retrieves the count of the number of [`Member`]s that would be pruned
    /// with the number of given days.
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
    /// Otherwise may return [`Error::Http`] if the current user does not have permission.
    /// Can also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn prune_count(&self, cache_http: impl CacheHttp, days: u8) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::KICK_MEMBERS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.prune_count(cache_http.http(), days).await
    }

    pub(crate) fn remove_unusable_permissions(permissions: &mut Permissions) {
        // No SEND_MESSAGES => no message-sending-related actions
        // If the member does not have the `SEND_MESSAGES` permission, then
        // throw out message-able permissions.
        if !permissions.contains(Permissions::SEND_MESSAGES) {
            *permissions &= !(Permissions::SEND_TTS_MESSAGES
                | Permissions::MENTION_EVERYONE
                | Permissions::EMBED_LINKS
                | Permissions::ATTACH_FILES);
        }

        // If the permission does not have the `VIEW_CHANNEL` permission, then
        // throw out actionable permissions.
        if !permissions.contains(Permissions::VIEW_CHANNEL) {
            *permissions &= !(Permissions::KICK_MEMBERS
                | Permissions::BAN_MEMBERS
                | Permissions::ADMINISTRATOR
                | Permissions::MANAGE_GUILD
                | Permissions::CHANGE_NICKNAME
                | Permissions::MANAGE_NICKNAMES);
        }
    }

    pub(crate) fn remove_unnecessary_voice_permissions(
        channel: &GuildChannel,
        mut permissions: Permissions,
    ) -> Permissions {
        // If this is a text channel, then throw out voice permissions.
        if channel.kind == ChannelType::Text {
            permissions &= !(Permissions::CONNECT
                | Permissions::SPEAK
                | Permissions::MUTE_MEMBERS
                | Permissions::DEAFEN_MEMBERS
                | Permissions::MOVE_MEMBERS
                | Permissions::USE_VAD
                | Permissions::STREAM);
        }

        permissions
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

    /// Fetches a specified scheduled event in the guild, by Id. If `with_user_count` is set to
    /// `true`, then the `user_count` field will be populated, indicating the number of users
    /// interested in the event.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event(
        &self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        with_user_count: bool,
    ) -> Result<ScheduledEvent> {
        self.id.scheduled_event(http, event_id, with_user_count).await
    }

    /// Fetches a list of all scheduled events in the guild. If `with_user_count` is set to `true`,
    /// then each event returned will have its `user_count` field populated.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_events(
        &self,
        http: impl AsRef<Http>,
        with_user_count: bool,
    ) -> Result<Vec<ScheduledEvent>> {
        self.id.scheduled_events(http, with_user_count).await
    }

    /// Fetches a list of interested users for the specified event.
    ///
    /// If `limit` is left unset, by default at most 100 users are returned.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event_users(
        &self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        limit: Option<u64>,
    ) -> Result<Vec<ScheduledEventUser>> {
        self.id.scheduled_event_users(http, event_id, limit).await
    }

    /// Fetches a list of interested users for the specified event, with additional options and
    /// filtering. See [`Http::get_scheduled_event_users`] for details.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn scheduled_event_users_optioned(
        &self,
        http: impl AsRef<Http>,
        event_id: impl Into<ScheduledEventId>,
        limit: Option<u64>,
        target: Option<UserPagination>,
        with_member: Option<bool>,
    ) -> Result<Vec<ScheduledEventUser>> {
        self.id.scheduled_event_users_optioned(http, event_id, limit, target, with_member).await
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
    pub fn shard_id(&self, cache: impl AsRef<Cache>) -> u32 {
        self.id.shard_id(&cache)
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
    /// Returns an [`Error::Http`] if the current user does not have permission,
    /// or if an [`Integration`] with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[inline]
    pub async fn start_integration_sync(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.start_integration_sync(http, integration_id).await
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

    /// Unbans the given [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// Otherwise will return an [`Error::Http`] if the current user does not
    /// have permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn unban(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.unban(cache_http.http(), user_id).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    ///
    /// # Errors
    ///
    /// Will return [`Error::Http`] if the current user is lacking permissions.
    /// Can also return an [`Error::Json`] if there is an error deserializing
    /// the API response.
    #[inline]
    pub async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(http).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    ///
    /// # Errors
    ///
    /// Will return an [`Error::Http`] if the current user is lacking permissions.
    /// Can also return an [`Error::Json`] if there is an error deserializing
    /// the API response.
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
    /// # #[cfg(all(feature = "cache", feature = "client"))]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if let Some(guild_id) = msg.guild_id {
    ///             if let Some(guild) = guild_id.to_guild_cached(&ctx) {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("{:?}", role);
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

/// Checks if a `&str` contains another `&str`.
#[cfg(feature = "model")]
fn contains(haystack: &str, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        haystack.contains(needle)
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}

/// Takes a `&str` as `origin` and tests if either
/// `word_a` or `word_b` is closer.
///
/// **Note**: Normally `word_a` and `word_b` are
/// expected to contain `origin` as substring.
/// If not, using `closest_to_origin` would sort these
/// the end.
#[cfg(feature = "model")]
fn closest_to_origin(origin: &str, word_a: &str, word_b: &str) -> std::cmp::Ordering {
    let value_a = match word_a.find(origin) {
        Some(value) => value + word_a.len(),
        None => return std::cmp::Ordering::Greater,
    };

    let value_b = match word_b.find(origin) {
        Some(value) => value + word_b.len(),
        None => return std::cmp::Ordering::Less,
    };

    value_a.cmp(&value_b)
}

/// A [`Guild`] widget.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-widget-settings-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildWidget {
    /// Whether the widget is enabled.
    pub enabled: bool,
    /// The widget channel id.
    pub channel_id: Option<ChannelId>,
}

/// Representation of the number of members that would be pruned by a guild
/// prune operation.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#get-guild-prune-count).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Basic information about a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object), subset undocumented (closest thing is
/// [this](https://discord.com/developers/docs/topics/rpc#getguilds-get-guilds-response-structure)).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<String>,
    /// The name of the guild.
    pub name: String,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
}

#[cfg(any(feature = "model", feature = "utils"))]
impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    ///
    /// This will produce a WEBP image URL, or GIF if the guild has a GIF icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            let ext = if icon.starts_with("a_") { "gif" } else { "webp" };

            cdn!("/icons/{}/{}.{}", self.id, icon, ext)
        })
    }
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[must_use]
    pub fn splash_url(&self) -> Option<String> {
        self.splash.as_ref().map(|splash| cdn!("/splashes/{}/{}.webp?size=4096", self.id, splash))
    }
}

/// Data for an unavailable guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#unavailable-guild-object).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct UnavailableGuild {
    /// The Id of the [`Guild`] that may be unavailable.
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    #[serde(default)]
    pub unavailable: bool,
}

enum_number! {
    /// Default message notification level for a guild.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-default-message-notification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum DefaultMessageNotificationLevel {
        /// Receive notifications for everything.
        #[default]
        All = 0,
        /// Receive only mentions.
        Mentions = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Setting used to filter explicit messages from members.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-explicit-content-filter-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ExplicitContentFilter {
        /// Don't scan any messages.
        #[default]
        None = 0,
        /// Scan messages from members without a role.
        WithoutRole = 1,
        /// Scan messages sent by all members.
        All = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Multi-Factor Authentication level for guild moderators.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-mfa-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum MfaLevel {
        /// MFA is disabled.
        #[default]
        None = 0,
        /// MFA is enabled.
        Elevated = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The level to set as criteria prior to a user being able to send
    /// messages in a [`Guild`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-verification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum VerificationLevel {
        /// Does not require any verification.
        #[default]
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The [`Guild`] nsfw level.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-guild-nsfw-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum NsfwLevel {
        /// The nsfw level is not specified.
        #[default]
        Default = 0,
        /// The guild is considered as explicit.
        Explicit = 1,
        /// The guild is considered as safe.
        Safe = 2,
        /// The guild is age restricted.
        AgeRestricted = 3,
        _ => Unknown(u8),
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use std::collections::*;

        use crate::model::prelude::*;

        fn gen_member() -> Member {
            Member {
                nick: Some("aaaa".to_string()),
                user: User {
                    name: "test".into(),
                    discriminator: 1432,
                    ..User::default()
                },
                ..Default::default()
            }
        }

        fn gen() -> Guild {
            let m = gen_member();

            Guild {
                members: HashMap::from([(m.user.id, m)]),
                ..Default::default()
            }
        }

        #[test]
        fn member_named_username() {
            let guild = gen();
            let lhs = guild.member_named("test#1432").unwrap().display_name();

            assert_eq!(lhs, gen_member().display_name());
        }

        #[test]
        fn member_named_nickname() {
            let guild = gen();
            let lhs = guild.member_named("aaaa").unwrap().display_name();

            assert_eq!(lhs, gen_member().display_name());
        }
    }
}
