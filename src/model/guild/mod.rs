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

use nonmax::{NonMaxU16, NonMaxU64, NonMaxU8};
#[cfg(feature = "model")]
use tracing::{error, warn};

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
#[cfg(feature = "model")]
use crate::builder::{
    AddMember,
    CreateChannel,
    CreateCommand,
    CreateScheduledEvent,
    CreateSticker,
    EditAutoModRule,
    EditCommandPermissions,
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
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "model")]
use crate::constants::LARGE_THRESHOLD;
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, UserPagination};
use crate::internal::prelude::*;
use crate::model::prelude::*;
use crate::model::utils::*;

/// A representation of a banning of a user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#ban-object).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<FixedString>,
    /// The user that was banned.
    pub user: User,
}

/// The response from [`GuildId::bulk_ban`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#bulk-guild-ban).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BulkBanResponse {
    /// The users that were successfully banned.
    pub banned_users: Vec<UserId>,
    /// The users that were not successfully banned.
    pub failed_users: Vec<UserId>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AfkMetadata {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: ChannelId,
    /// The amount of seconds a user can not show any activity in a voice channel before being
    /// moved to an AFK channel -- if one exists.
    pub afk_timeout: AfkTimeout,
}

/// Information about a Discord guild, such as channels, emojis, etc.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object) plus
/// [extension](https://discord.com/developers/docs/topics/gateway-events#guild-create).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Guild {
    /// The unique Id identifying the guild.
    ///
    /// This is equivalent to the Id of the default role (`@everyone`).
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<ImageHash>,
    /// Icon hash, returned when in the template object
    pub icon_hash: Option<ImageHash>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the `InviteSplash` feature is enabled, this can be used to generate a URL to a splash
    /// image.
    pub splash: Option<ImageHash>,
    /// An identifying hash of the guild discovery's splash icon.
    ///
    /// **Note**: Only present for guilds with the `DISCOVERABLE` feature.
    pub discovery_splash: Option<ImageHash>,
    // Omitted `owner` field because only Http::get_guilds uses it, which returns GuildInfo
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: UserId,
    // Omitted `permissions` field because only Http::get_guilds uses it, which returns GuildInfo
    // Omitted `region` field because it is deprecated (see Discord docs)
    /// Information about the voice afk channel.
    #[serde(flatten)]
    pub afk_metadata: Option<AfkMetadata>,
    /// Whether or not the guild widget is enabled.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or null if set to no invite
    pub widget_channel_id: Option<ChannelId>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// A mapping of the guild's roles.
    pub roles: ExtractMap<RoleId, Role>,
    /// All of the guild's custom emojis.
    pub emojis: ExtractMap<EmojiId, Emoji>,
    /// The guild features. More information available at [`discord documentation`].
    ///
    /// The following is a list of known features:
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
    pub features: FixedArray<FixedString>,
    /// Indicator of whether the guild requires multi-factor authentication for [`Role`]s or
    /// [`User`]s with moderation permissions.
    pub mfa_level: MfaLevel,
    /// Application ID of the guild creator if it is bot-created.
    pub application_id: Option<ApplicationId>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<ChannelId>,
    /// System channel flags.
    pub system_channel_flags: SystemChannelFlags,
    /// The id of the channel where rules and/or guidelines are displayed.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub rules_channel_id: Option<ChannelId>,
    /// The maximum number of presences for the guild. The default value is currently 25000.
    ///
    /// **Note**: It is in effect when it is `None`.
    pub max_presences: Option<NonMaxU64>,
    /// The maximum number of members for the guild.
    pub max_members: Option<NonMaxU64>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<FixedString>,
    /// The server's description, if it has one.
    pub description: Option<FixedString>,
    /// The guild's banner, if it has one.
    pub banner: Option<FixedString>,
    /// The server's premium boosting level.
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    pub premium_subscription_count: Option<NonMaxU64>,
    /// The preferred locale of this guild only set if guild has the "DISCOVERABLE" feature,
    /// defaults to en-US.
    pub preferred_locale: FixedString,
    /// The id of the channel where admins and moderators of Community guilds receive notices from
    /// Discord.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub public_updates_channel_id: Option<ChannelId>,
    /// The maximum amount of users in a video channel.
    pub max_video_channel_users: Option<NonMaxU64>,
    /// The maximum amount of users in a stage video channel
    pub max_stage_video_channel_users: Option<NonMaxU64>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<NonMaxU64>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The welcome screen of the guild.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub welcome_screen: Option<GuildWelcomeScreen>,
    /// The guild NSFW state. See [`discord support article`].
    ///
    /// [`discord support article`]: https://support.discord.com/hc/en-us/articles/1500005389362-NSFW-Server-Designation
    pub nsfw_level: NsfwLevel,
    /// All of the guild's custom stickers.
    pub stickers: ExtractMap<StickerId, Sticker>,
    /// Whether the guild has the boost progress bar enabled
    pub premium_progress_bar_enabled: bool,

    // =======
    // From here on, all fields are from Guild Create Event's extra fields (see Discord docs)
    // =======
    /// The date that the current user joined the guild.
    pub joined_at: Timestamp,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// Whether this guild is unavailable due to an outage.
    #[serde(default)]
    pub unavailable: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// A mapping of [`User`]s to their current voice state.
    pub voice_states: ExtractMap<UserId, VoiceState>,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received if the
    /// [`Self::member_count`] is greater than the [`LARGE_THRESHOLD`] set by the library.
    pub members: ExtractMap<UserId, Member>,
    /// All voice and text channels contained within a guild.
    ///
    /// This contains all channels regardless of permissions (i.e. the ability of the bot to read
    /// from or connect to them).
    pub channels: ExtractMap<ChannelId, GuildChannel>,
    /// All active threads in this guild that current user has permission to view.
    ///
    /// A thread is guaranteed (for errors, not for panics) to be cached if a `MESSAGE_CREATE`
    /// event is fired in said thread, however an `INTERACTION_CREATE` may not have a private
    /// thread in cache.
    pub threads: FixedArray<GuildChannel>,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// **Note**: This will be empty unless the "guild presences" privileged intent is enabled.
    pub presences: ExtractMap<UserId, Presence>,
    /// The stage instances in this guild.
    pub stage_instances: FixedArray<StageInstance>,
    /// The stage instances in this guild.
    #[serde(rename = "guild_scheduled_events")]
    pub scheduled_events: FixedArray<ScheduledEvent>,
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
    pub async fn automod_rules(&self, http: &Http) -> Result<Vec<Rule>> {
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
    pub async fn automod_rule(&self, http: &Http, rule_id: RuleId) -> Result<Rule> {
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
    pub async fn create_automod_rule(
        &self,
        http: &Http,
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
    pub async fn edit_automod_rule(
        &self,
        http: &Http,
        rule_id: RuleId,
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
    /// Returns [`Error::Http`] if the current user lacks permission, or if a rule with that Id
    /// does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn delete_automod_rule(
        &self,
        http: &Http,
        rule_id: RuleId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_automod_rule(http, rule_id, reason).await
    }

    #[cfg(feature = "cache")]
    fn check_hierarchy(&self, cache: &Cache, other_user: UserId) -> Result<()> {
        let current_id = cache.current_user().id;

        if let Some(higher) = self.greater_member_hierarchy(other_user, current_id) {
            if higher != current_id {
                return Err(Error::Model(ModelError::Hierarchy));
            }
        }

        Ok(())
    }

    /// Returns the "default" channel of the guild for the passed user id. (This returns the first
    /// channel that can be read by the user, if there isn't one, returns [`None`])
    #[must_use]
    pub fn default_channel(&self, uid: UserId) -> Option<&GuildChannel> {
        let member = self.members.get(&uid)?;
        self.channels.iter().find(|&channel| {
            channel.kind != ChannelType::Category
                && self.user_permissions_in(channel, member).view_channel()
        })
    }

    /// Returns the guaranteed "default" channel of the guild. (This returns the first channel that
    /// can be read by everyone, if there isn't one, returns [`None`])
    ///
    /// **Note**: This is very costly if used in a server with lots of channels, members, or both.
    #[must_use]
    pub fn default_channel_guaranteed(&self) -> Option<&GuildChannel> {
        self.channels.iter().find(|&channel| {
            channel.kind != ChannelType::Category
                && self
                    .members
                    .iter()
                    .map(|member| self.user_permissions_in(channel, member))
                    .all(Permissions::view_channel)
        })
    }

    /// Intentionally not async. Retrieving anything from HTTP here is overkill/undesired
    #[cfg(feature = "cache")]
    pub(crate) fn require_perms(
        &self,
        cache: &Cache,
        required_permissions: Permissions,
    ) -> Result<(), Error> {
        if let Some(member) = self.members.get(&cache.current_user().id) {
            // This isn't used for any channel-specific permissions, but sucks still.
            #[allow(deprecated)]
            let bot_permissions = self.member_permissions(member);
            if !bot_permissions.contains(required_permissions) {
                return Err(Error::Model(ModelError::InvalidPermissions {
                    required: required_permissions,
                    present: bot_permissions,
                }));
            }
        }
        Ok(())
    }

    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
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
    /// let _ = guild.ban(user, 4, None);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the number of days' worth of messages
    /// to delete is over the maximum.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform bans, or may return a [`ModelError::Hierarchy`] if the
    /// member to be banned has a higher role than the current user.
    ///
    /// Otherwise returns [`Error::Http`] if the member cannot be banned.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn ban(
        &self,
        cache_http: impl CacheHttp,
        user: UserId,
        dmd: u8,
        reason: Option<&str>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::BAN_MEMBERS)?;

                self.check_hierarchy(cache, user)?;
            }
        }

        self.id.ban(cache_http.http(), user, dmd, reason).await
    }

    /// Bans multiple users from the guild, returning the users that were and weren't banned.
    ///
    /// # Errors
    ///
    /// See [`GuildId::bulk_ban`] for more information.
    pub async fn bulk_ban(
        &self,
        cache_http: impl CacheHttp,
        user_ids: &[UserId],
        delete_message_seconds: u32,
        reason: Option<&str>,
    ) -> Result<BulkBanResponse> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::BAN_MEMBERS & Permissions::MANAGE_GUILD)?;
            }
        }

        self.id.bulk_ban(cache_http.http(), user_ids, delete_message_seconds, reason).await
    }

    /// Returns the formatted URL of the guild's banner image, if one exists.
    #[must_use]
    pub fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp?size=1024", self.id, banner))
    }

    /// Gets a list of the guild's bans, with additional options and filtering. See
    /// [`Http::get_bans`] for details.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn bans(
        &self,
        cache_http: impl CacheHttp,
        target: Option<UserPagination>,
        limit: Option<NonMaxU16>,
    ) -> Result<Vec<Ban>> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::BAN_MEMBERS)?;
            }
        }

        self.id.bans(cache_http.http(), target, limit).await
    }

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    pub async fn add_member(
        &self,
        http: &Http,
        user_id: UserId,
        builder: AddMember<'_>,
    ) -> Result<Option<Member>> {
        self.id.add_member(http, user_id, builder).await
    }

    /// Retrieves a list of [`AuditLogs`] for the guild.
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission to view the audit log,
    /// or if an invalid value is given.
    ///
    /// [View Audit Log]: Permissions::VIEW_AUDIT_LOG
    pub async fn audit_logs(
        &self,
        http: &Http,
        action_type: Option<audit_log::Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<NonMaxU8>,
    ) -> Result<AuditLogs> {
        self.id.audit_logs(http, action_type, user_id, before, limit).await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is currently unavailable.
    pub async fn channels(&self, http: &Http) -> Result<ExtractMap<ChannelId, GuildChannel>> {
        self.id.channels(http).await
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full [`Guild`] will be received
    /// over a [`Shard`].
    ///
    /// **Note**: This endpoint is usually only available for user accounts. Refer to Discord's
    /// information for the endpoint [here][whitelist] for more information. If you require this as
    /// a bot, re-think what you are doing and if it _really_ needs to be doing this.
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
    pub async fn create(http: &Http, name: &str, icon: Option<ImageHash>) -> Result<PartialGuild> {
        #[derive(serde::Serialize)]
        struct CreateGuild<'a> {
            name: &'a str,
            icon: Option<ImageHash>,
        }

        let body = CreateGuild {
            name,
            icon,
        };

        http.create_guild(&body).await
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
    /// # let http: Http = unimplemented!();
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
    /// [`CreateAttachment`] builder is provided for you as a simple method to read an image and
    /// encode it into base64, if you are reading from the filesystem.
    ///
    /// The name of the emoji must be at least 2 characters long and can only contain alphanumeric
    /// characters and underscores.
    ///
    /// Requires the [Create Guild Expressions] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to how to read an image
    /// from the filesystem and encode it as base64. Most of the example can be applied similarly
    /// for this method.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [`CreateAttachment`]: crate::builder::CreateAttachment
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_emoji(
        &self,
        http: &Http,
        name: &str,
        image: &str,
        reason: Option<&str>,
    ) -> Result<Emoji> {
        self.id.create_emoji(http, name, image, reason).await
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
    pub async fn create_integration(
        &self,
        http: &Http,
        integration_id: IntegrationId,
        kind: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.create_integration(http, integration_id, kind, reason).await
    }

    /// Create a guild specific application [`Command`].
    ///
    /// **Note**: Unlike global commands, guild commands will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`CreateCommand::execute`]: ../../builder/struct.CreateCommand.html#method.execute
    pub async fn create_command(&self, http: &Http, builder: CreateCommand<'_>) -> Result<Command> {
        self.id.create_command(http, builder).await
    }

    /// Override all guild application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_command`].
    pub async fn set_commands(
        &self,
        http: &Http,
        commands: &[CreateCommand<'_>],
    ) -> Result<Vec<Command>> {
        self.id.set_commands(http, commands).await
    }

    /// Overwrites permissions for a specific command.
    ///
    /// **Note**: It will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommandPermissionsData::execute`] for a list of possible errors.
    ///
    /// [`CreateCommandPermissionsData::execute`]: ../../builder/struct.CreateCommandPermissionsData.html#method.execute
    pub async fn edit_command_permissions(
        &self,
        http: &Http,
        command_id: CommandId,
        builder: EditCommandPermissions<'_>,
    ) -> Result<CommandPermissions> {
        self.id.edit_command_permissions(http, command_id, builder).await
    }

    /// Get all guild application commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands(&self, http: &Http) -> Result<Vec<Command>> {
        self.id.get_commands(http).await
    }

    /// Get all guild application commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands_with_localizations(&self, http: &Http) -> Result<Vec<Command>> {
        self.id.get_commands_with_localizations(http).await
    }

    /// Get a specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_command(&self, http: &Http, command_id: CommandId) -> Result<Command> {
        self.id.get_command(http, command_id).await
    }

    /// Edit a guild application command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`CreateCommand::execute`]: ../../builder/struct.CreateCommand.html#method.execute
    pub async fn edit_command(
        &self,
        http: &Http,
        command_id: CommandId,
        builder: CreateCommand<'_>,
    ) -> Result<Command> {
        self.id.edit_command(http, command_id, builder).await
    }

    /// Delete guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_command(&self, http: &Http, command_id: CommandId) -> Result<()> {
        self.id.delete_command(http, command_id).await
    }

    /// Get all guild application commands permissions only.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_commands_permissions(&self, http: &Http) -> Result<Vec<CommandPermissions>> {
        self.id.get_commands_permissions(http).await
    }

    /// Get permissions for specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_command_permissions(
        &self,
        http: &Http,
        command_id: CommandId,
    ) -> Result<CommandPermissions> {
        self.id.get_command_permissions(http, command_id).await
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
    pub async fn create_role(&self, http: &Http, builder: EditRole<'_>) -> Result<Role> {
        self.id.create_role(http, builder).await
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    pub async fn create_scheduled_event(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        self.id.create_scheduled_event(cache_http, builder).await
    }

    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_sticker(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateSticker<'_>,
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
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidUser`] if the current user
    /// is not the guild owner.
    ///
    /// Otherwise returns [`Error::Http`] if the current user is not the owner of the guild.
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.owner_id != cache.current_user().id {
                    return Err(Error::Model(ModelError::InvalidUser));
                }
            }
        }

        self.id.delete(cache_http.http()).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist in the guild.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn delete_emoji(
        &self,
        http: &Http,
        emoji_id: EmojiId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_emoji(http, emoji_id, reason).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission, or if an Integration with
    /// that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn delete_integration(
        &self,
        http: &Http,
        integration_id: IntegrationId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_integration(http, integration_id, reason).await
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `model` features enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete the role.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn delete_role(
        &self,
        http: &Http,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_role(http, role_id, reason).await
    }

    /// Deletes a [`ScheduledEvent`] by id from the guild.
    ///
    /// **Note**: If the event was created by the current user, requires either [Create Events] or
    /// the [Manage Events] permission. Otherwise, the [Manage Events] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete the scheduled event.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn delete_scheduled_event(
        &self,
        http: &Http,
        event_id: ScheduledEventId,
    ) -> Result<()> {
        self.id.delete_scheduled_event(http, event_id).await
    }

    /// Deletes a [`Sticker`] by Id from the guild.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a sticker with that id
    /// does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn delete_sticker(
        &self,
        http: &Http,
        sticker_id: StickerId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_sticker(http, sticker_id, reason).await
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
    /// # use serenity::{http::Http, model::guild::Guild};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut guild: Guild = unimplemented!();
    /// let icon = CreateAttachment::path("./icon.png").await?;
    ///
    /// // assuming a `guild` has already been bound
    /// let builder = EditGuild::new().icon(Some(&icon));
    /// guild.edit(&http, builder).await?;
    /// # Ok(())
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

        self.afk_metadata = guild.afk_metadata;
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
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn edit_emoji(
        &self,
        http: &Http,
        emoji_id: EmojiId,
        name: &str,
        reason: Option<&str>,
    ) -> Result<Emoji> {
        self.id.edit_emoji(http, emoji_id, name, reason).await
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
    pub async fn edit_member(
        &self,
        http: &Http,
        user_id: UserId,
        builder: EditMember<'_>,
    ) -> Result<Member> {
        self.id.edit_member(http, user_id, builder).await
    }

    /// Edits the guild's MFA level. Returns the new level on success.
    ///
    /// Requires guild ownership.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    pub async fn edit_mfa_level(
        &self,
        http: &Http,
        mfa_level: MfaLevel,
        audit_log_reason: Option<&str>,
    ) -> Result<MfaLevel> {
        self.id.edit_mfa_level(http, mfa_level, audit_log_reason).await
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass [`None`] to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to change their own nickname.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    pub async fn edit_nickname(
        &self,
        cache_http: impl CacheHttp,
        new_nickname: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::CHANGE_NICKNAME)?;
            }
        }

        self.id.edit_nickname(cache_http.http(), new_nickname, reason).await
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
    pub async fn edit_role(
        &self,
        cache_http: impl CacheHttp,
        role_id: RoleId,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        self.id.edit_role(cache_http, role_id, builder).await
    }

    /// Edits the order of [`Role`]s. Requires the [Manage Roles] permission.
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
    pub async fn edit_role_position(
        &self,
        http: &Http,
        role_id: RoleId,
        position: i16,
        audit_log_reason: Option<&str>,
    ) -> Result<Vec<Role>> {
        self.id.edit_role_position(http, role_id, position, audit_log_reason).await
    }

    /// Modifies a scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: If the event was created by the current user, requires either [Create Events] or
    /// the [Manage Events] permission. Otherwise, the [Manage Events] permission is required.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn edit_scheduled_event(
        &self,
        http: &Http,
        event_id: ScheduledEventId,
        builder: EditScheduledEvent<'_>,
    ) -> Result<ScheduledEvent> {
        self.id.edit_scheduled_event(http, event_id, builder).await
    }

    /// Edits a sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
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
    /// # let http: Http = unimplemented!();
    /// # let guild: Guild = unimplemented!();
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
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    pub async fn edit_sticker(
        &self,
        http: &Http,
        sticker_id: StickerId,
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
        http: &Http,
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
        http: &Http,
        builder: EditGuildWidget<'_>,
    ) -> Result<GuildWidget> {
        self.id.edit_widget(http, builder).await
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send all data with a guild
    /// retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    pub async fn get(cache_http: impl CacheHttp, guild_id: GuildId) -> Result<PartialGuild> {
        guild_id.to_partial_guild(cache_http).await
    }

    /// Gets the highest role a [`Member`] of this Guild has.
    ///
    /// Returns None if the member has no roles or the member from this guild.
    #[must_use]
    pub fn member_highest_role(&self, member: &Member) -> Option<&Role> {
        let mut highest: Option<&Role> = None;

        for role_id in &member.roles {
            if let Some(role) = self.roles.get(role_id) {
                // Skip this role if this role in iteration has:
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some(highest) = highest {
                    if role.position < highest.position
                        || (role.position == highest.position && role.id > highest.id)
                    {
                        continue;
                    }
                }

                highest = Some(role);
            }
        }

        highest
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances is not present.
    /// Returns [`None`] if the users have the same hierarchy, as neither are greater than the
    /// other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users is the guild
    /// owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[must_use]
    pub fn greater_member_hierarchy(&self, lhs_id: UserId, rhs_id: UserId) -> Option<UserId> {
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

        let lhs = self
            .member_highest_role(self.members.get(&lhs_id)?)
            .map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        let rhs = self
            .member_highest_role(self.members.get(&rhs_id)?)
            .map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        // If LHS and RHS both have no top position or have the same role ID, then no one wins.
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

        // If LHS and RHS both have the same position, but LHS has the lower role ID, then LHS
        // wins.
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
        icon_url(self.id, self.icon.as_ref())
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable
    pub async fn emojis(&self, http: &Http) -> Result<Vec<Emoji>> {
        self.id.emojis(http).await
    }

    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an emoji with that id does not exist in the guild, or if the
    /// guild is unavailable.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    pub async fn emoji(&self, http: &Http, emoji_id: EmojiId) -> Result<Emoji> {
        self.id.emoji(http, emoji_id).await
    }

    /// Gets all integration of the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission to see integrations.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn integrations(&self, http: &Http) -> Result<Vec<Integration>> {
        self.id.integrations(http).await
    }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to see invites.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn invites(&self, cache_http: impl CacheHttp) -> Result<Vec<RichInvite>> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::MANAGE_GUILD)?;
            }
        }

        self.id.invites(cache_http.http()).await
    }

    /// Checks if the guild is 'large'.
    ///
    /// A guild is considered large if it has more than 250 members.
    #[must_use]
    #[deprecated = "Use Guild::large"]
    pub fn is_large(&self) -> bool {
        self.member_count > u64::from(LARGE_THRESHOLD)
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked by the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn kick(&self, http: &Http, user_id: UserId, reason: Option<&str>) -> Result<()> {
        self.id.kick(http, user_id, reason).await
    }

    /// Returns a guild [`Member`] object for the current user.
    ///
    /// See [`Http::get_current_user_guild_member`] for more.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild or the access token
    /// lacks the necessary scope.
    pub async fn current_user_member(&self, http: &Http) -> Result<Member> {
        self.id.current_user_member(http).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the current user cannot leave the guild, or currently is
    /// not in the guild.
    pub async fn leave(&self, http: &Http) -> Result<()> {
        self.id.leave(http).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled [`Self::members`] will be checked first, if so, a reference
    /// to the member will be returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in the guild or if the guild is otherwise
    /// unavailable.
    pub async fn member(&self, http: &Http, user_id: UserId) -> Result<Cow<'_, Member>> {
        if let Some(member) = self.members.get(&user_id) {
            Ok(Cow::Borrowed(member))
        } else {
            http.get_member(self.id, user_id).await.map(Cow::Owned)
        }
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// Optionally pass in `after` to offset the results by a [`User`]'s Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, may also return
    /// [`ModelError::TooSmall`] or [`ModelError::TooLarge`] if the limit is not within range.
    ///
    /// [`User`]: crate::model::user::User
    pub async fn members(
        &self,
        http: &Http,
        limit: Option<NonMaxU16>,
        after: Option<UserId>,
    ) -> Result<Vec<Member>> {
        self.id.members(http, limit, after).await
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(move |member| {
            self.presences.get(&member.user.id).is_some_and(|p| p.status == status)
        })
    }

    /// Retrieves the first [`Member`] found that matches the name - with an optional discriminator
    /// - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup, as no two people
    /// can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator, then a search will be
    /// done for the nickname. When searching by nickname, the hash (`#`) and everything after it
    /// is included in the search.
    ///
    /// The following are valid types of searches:
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`Self::search_members`].
    #[must_use]
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (username, discrim) = match crate::utils::parse_user_tag(name) {
            Some((username, discrim)) => (username, Some(discrim)),
            None => (name, None),
        };

        for member in &self.members {
            if &*member.user.name == username
                && discrim.map_or(true, |d| member.user.discriminator == d)
            {
                return Some(member);
            }
        }

        self.members.iter().find(|member| member.nick.as_deref().is_some_and(|nick| nick == name))
    }

    /// Retrieves all [`Member`] that start with a given [`String`].
    ///
    /// `sorted` decides whether the best early match of the `prefix` should be the criteria to
    /// sort the result.
    ///
    /// For the `prefix` "zey" and the unsorted result:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`Self::search_members`].
    #[must_use]
    pub fn members_starting_with(
        &self,
        prefix: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, &str)> {
        fn starts_with(name: &str, prefix: &str, case_sensitive: bool) -> bool {
            if case_sensitive {
                name.starts_with(prefix)
            } else {
                name.to_lowercase().starts_with(&prefix.to_lowercase())
            }
        }

        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let username = &member.user.name;

                if starts_with(username, prefix, case_sensitive) {
                    Some((member, username.as_str()))
                } else {
                    match &member.nick {
                        Some(nick) => starts_with(nick, prefix, case_sensitive)
                            .then(|| (member, nick.as_str())),
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, &str)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(prefix, a.1, b.1));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] as either username or nick, with a
    /// priority on username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sorted` decides whether the best early match of the search-term should be the criteria to
    /// sort the result. It will look at the account name first, if that does not fit the
    /// search-criteria `substring`, the display-name will be considered.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Due to two fields of a [`Member`] being candidates for the searched field,
    /// setting `sorted` to `true` will result in an overhead, as both fields have to be considered
    /// again for sorting.
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`Self::search_members`].
    #[must_use]
    pub fn members_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let username = &member.user.name;

                if contains(username, substring, case_sensitive) {
                    Some((member, username.clone().into()))
                } else {
                    match &member.nick {
                        Some(nick) => contains(nick, substring, case_sensitive)
                            .then(|| (member, nick.clone().into())),
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

    /// Retrieves a tuple of [`Member`]s containing a given [`String`] in their username as the
    /// first field and the name used for sorting as the second field.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term should be the criteria to
    /// sort the result.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`Self::search_members`].
    #[must_use]
    pub fn members_username_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let name = &member.user.name;
                contains(name, substring, case_sensitive).then(|| (member, name.clone().into()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Retrieves all [`Member`] containing a given [`String`] in their nick.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    ///
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term should be the criteria to
    /// sort the result.
    ///
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Instead of panicking, when sorting does not find a nick, the username will be
    /// used (this should never happen).
    ///
    /// **Note**: This will only search members that are cached. If you want to search all members
    /// in the guild via the Http API, use [`Self::search_members`].
    #[must_use]
    pub fn members_nick_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = self
            .members
            .iter()
            .filter_map(|member| {
                let nick = member.nick.as_ref().unwrap_or(&member.user.name);
                contains(nick, substring, case_sensitive).then(|| (member, nick.clone().into()))
            })
            .collect::<Vec<(&Member, String)>>();

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
        }

        members
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    #[must_use]
    #[deprecated = "Use Guild::user_permissions_in, as this doesn't consider permission overwrites"]
    pub fn member_permissions(&self, member: &Member) -> Permissions {
        Self::user_permissions_in_(
            None,
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Calculate a [`PartialMember`]'s permissions in the guild.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    #[deprecated = "Use Guild::partial_member_permissions_in, as this doesn't consider permission overwrites"]
    pub fn partial_member_permissions(
        &self,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Self::user_permissions_in_(
            None,
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission, or if the member is not
    /// currently in a voice channel for this [`Guild`].
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    pub async fn move_member(
        &self,
        http: &Http,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Member> {
        self.id.move_member(http, user_id, channel_id).await
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    #[must_use]
    pub fn user_permissions_in(&self, channel: &GuildChannel, member: &Member) -> Permissions {
        Self::user_permissions_in_(
            Some(channel),
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Calculate a [`PartialMember`]'s permissions in a given channel in a guild.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    pub fn partial_member_permissions_in(
        &self,
        channel: &GuildChannel,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }

        Self::user_permissions_in_(
            Some(channel),
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn user_permissions_in_(
        channel: Option<&GuildChannel>,
        member_user_id: UserId,
        member_roles: &[RoleId],
        guild_id: GuildId,
        guild_roles: &ExtractMap<RoleId, Role>,
        guild_owner_id: UserId,
    ) -> Permissions {
        let mut everyone_allow_overwrites = Permissions::empty();
        let mut everyone_deny_overwrites = Permissions::empty();
        let mut roles_allow_overwrites = Vec::new();
        let mut roles_deny_overwrites = Vec::new();
        let mut member_allow_overwrites = Permissions::empty();
        let mut member_deny_overwrites = Permissions::empty();

        if let Some(channel) = channel {
            for overwrite in &channel.permission_overwrites {
                match overwrite.kind {
                    PermissionOverwriteType::Member(user_id) => {
                        if member_user_id == user_id {
                            member_allow_overwrites = overwrite.allow;
                            member_deny_overwrites = overwrite.deny;
                        }
                    },
                    PermissionOverwriteType::Role(role_id) => {
                        if role_id.get() == guild_id.get() {
                            everyone_allow_overwrites = overwrite.allow;
                            everyone_deny_overwrites = overwrite.deny;
                        } else if member_roles.contains(&role_id) {
                            roles_allow_overwrites.push(overwrite.allow);
                            roles_deny_overwrites.push(overwrite.deny);
                        }
                    },
                }
            }
        }

        calculate_permissions(CalculatePermissions {
            is_guild_owner: member_user_id == guild_owner_id,
            everyone_permissions: if let Some(role) = guild_roles.get(&RoleId::new(guild_id.get()))
            {
                role.permissions
            } else {
                error!("@everyone role missing in {}", guild_id);
                Permissions::empty()
            },
            user_roles_permissions: member_roles
                .iter()
                .map(|role_id| {
                    if let Some(role) = guild_roles.get(role_id) {
                        role.permissions
                    } else {
                        warn!(
                            "{} on {} has non-existent role {:?}",
                            member_user_id, guild_id, role_id
                        );
                        Permissions::empty()
                    }
                })
                .collect(),
            everyone_allow_overwrites,
            everyone_deny_overwrites,
            roles_allow_overwrites,
            roles_deny_overwrites,
            member_allow_overwrites,
            member_deny_overwrites,
        })
    }

    /// Retrieves the count of the number of [`Member`]s that would be pruned with the number of
    /// given days.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to kick members.
    ///
    /// Otherwise may return [`Error::Http`] if the current user does not have permission. Can also
    /// return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn prune_count(&self, cache_http: impl CacheHttp, days: u8) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::KICK_MEMBERS)?;
            }
        }

        self.id.prune_count(cache_http.http(), days).await
    }

    /// Re-orders the channels of the guild.
    ///
    /// Although not required, you should specify all channels' positions, regardless of whether
    /// they were updated. Otherwise, positioning can sometimes get weird.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is lacking permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn reorder_channels(
        &self,
        http: &Http,
        channels: impl IntoIterator<Item = (ChannelId, u64)>,
    ) -> Result<()> {
        self.id.reorder_channels(http, channels).await
    }

    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname starts with a
    /// provided string.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// **Note**: Queries are case insensitive.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    pub async fn search_members(
        &self,
        http: &Http,
        query: &str,
        limit: Option<NonMaxU16>,
    ) -> Result<Vec<Member>> {
        self.id.search_members(http, query, limit).await
    }

    /// Fetches a specified scheduled event in the guild, by Id. If `with_user_count` is set to
    /// `true`, then the `user_count` field will be populated, indicating the number of users
    /// interested in the event.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event(
        &self,
        http: &Http,
        event_id: ScheduledEventId,
        with_user_count: bool,
    ) -> Result<ScheduledEvent> {
        self.id.scheduled_event(http, event_id, with_user_count).await
    }

    /// Fetches a list of all scheduled events in the guild. If `with_user_count` is set to `true`,
    /// then each event returned will have its `user_count` field populated.
    ///
    /// **Note**: Requires the [View Channel] permission at the guild level.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_events(
        &self,
        http: &Http,
        with_user_count: bool,
    ) -> Result<Vec<ScheduledEvent>> {
        self.id.scheduled_events(http, with_user_count).await
    }

    /// Fetches a list of interested users for the specified event.
    ///
    /// If `limit` is left unset, by default at most 100 users are returned.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event_users(
        &self,
        http: &Http,
        event_id: ScheduledEventId,
        limit: Option<NonMaxU8>,
    ) -> Result<Vec<ScheduledEventUser>> {
        self.id.scheduled_event_users(http, event_id, limit).await
    }

    /// Fetches a list of interested users for the specified event, with additional options and
    /// filtering. See [`Http::get_scheduled_event_users`] for details.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the provided Id is
    /// invalid.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn scheduled_event_users_optioned(
        &self,
        http: &Http,
        event_id: ScheduledEventId,
        limit: Option<NonMaxU8>,
        target: Option<UserPagination>,
        with_member: Option<bool>,
    ) -> Result<Vec<ScheduledEventUser>> {
        self.id.scheduled_event_users_optioned(http, event_id, limit, target, with_member).await
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// See the documentation for [`GuildId::shard_id`].
    #[must_use]
    #[cfg(feature = "utils")]
    pub fn shard_id(&self, shard_total: std::num::NonZeroU16) -> u16 {
        self.id.shard_id(shard_total)
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
    /// Returns an [`Error::Http`] if the current user does not have permission, or if an
    /// [`Integration`] with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn start_integration_sync(
        &self,
        http: &Http,
        integration_id: IntegrationId,
    ) -> Result<()> {
        self.id.start_integration_sync(http, integration_id).await
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires [Kick Members] and [Manage Guild] permissions.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to kick members.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have permission.
    ///
    /// Can also return an [`Error::Json`] if there is an error deserializing the API response.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn start_prune(
        &self,
        cache_http: impl CacheHttp,
        days: u8,
        reason: Option<&str>,
    ) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::KICK_MEMBERS | Permissions::MANAGE_GUILD)?;
            }
        }

        self.id.start_prune(cache_http.http(), days, reason).await
    }

    /// Unbans the given [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform bans.
    ///
    /// Otherwise will return an [`Error::Http`] if the current user does not have permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn unban(
        &self,
        cache_http: impl CacheHttp,
        user_id: UserId,
        reason: Option<&str>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                self.require_perms(cache, Permissions::BAN_MEMBERS)?;
            }
        }

        self.id.unban(cache_http.http(), user_id, reason).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    ///
    /// # Errors
    ///
    /// Will return [`Error::Http`] if the current user is lacking permissions. Can also return an
    /// [`Error::Json`] if there is an error deserializing the API response.
    pub async fn vanity_url(&self, http: &Http) -> Result<String> {
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
    /// Will return an [`Error::Http`] if the current user is lacking permissions. Can also return
    /// an [`Error::Json`] if there is an error deserializing the API response.
    pub async fn webhooks(&self, http: &Http) -> Result<Vec<Webhook>> {
        self.id.webhooks(http).await
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// # use serenity::model::prelude::*;
    /// # use serenity::prelude::*;
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// #[cfg(all(feature = "cache", feature = "client"))]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: &Context, msg: &Message) {
    ///         if let Some(guild_id) = msg.guild_id {
    ///             if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("{:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.iter().find(|role| role_name == &*role.name)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of messages in this
    /// guild.
    #[cfg(feature = "collector")]
    pub fn await_reply(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        MessageCollector::new(shard_messenger).guild_id(self.id)
    }

    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    pub fn await_replies(&self, shard_messenger: ShardMessenger) -> MessageCollector {
        self.await_reply(shard_messenger)
    }

    /// Returns a builder which can be awaited to obtain a message or stream of reactions sent in
    /// this guild.
    #[cfg(feature = "collector")]
    pub fn await_reaction(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).guild_id(self.id)
    }

    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    pub fn await_reactions(&self, shard_messenger: ShardMessenger) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }

    /// Gets the guild active threads.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    pub async fn get_active_threads(&self, http: &Http) -> Result<ThreadsData> {
        self.id.get_active_threads(http).await
    }
}

#[cfg(feature = "model")]
struct CalculatePermissions {
    /// Whether the guild member is the guild owner
    pub is_guild_owner: bool,
    /// Base permissions given to @everyone (guild level)
    pub everyone_permissions: Permissions,
    /// Permissions allowed to a user by their roles (guild level)
    pub user_roles_permissions: Vec<Permissions>,
    /// Overwrites that deny permissions for @everyone (channel level)
    pub everyone_allow_overwrites: Permissions,
    /// Overwrites that allow permissions for @everyone (channel level)
    pub everyone_deny_overwrites: Permissions,
    /// Overwrites that deny permissions for specific roles (channel level)
    pub roles_allow_overwrites: Vec<Permissions>,
    /// Overwrites that allow permissions for specific roles (channel level)
    pub roles_deny_overwrites: Vec<Permissions>,
    /// Member-specific overwrites that deny permissions (channel level)
    pub member_allow_overwrites: Permissions,
    /// Member-specific overwrites that allow permissions (channel level)
    pub member_deny_overwrites: Permissions,
}

#[cfg(feature = "model")]
impl Default for CalculatePermissions {
    fn default() -> Self {
        Self {
            is_guild_owner: false,
            everyone_permissions: Permissions::empty(),
            user_roles_permissions: Vec::new(),
            everyone_allow_overwrites: Permissions::empty(),
            everyone_deny_overwrites: Permissions::empty(),
            roles_allow_overwrites: Vec::new(),
            roles_deny_overwrites: Vec::new(),
            member_allow_overwrites: Permissions::empty(),
            member_deny_overwrites: Permissions::empty(),
        }
    }
}

/// Translated from the pseudo code at https://discord.com/developers/docs/topics/permissions#permission-overwrites
///
/// The comments within this file refer to the above link
#[cfg(feature = "model")]
fn calculate_permissions(data: CalculatePermissions) -> Permissions {
    if data.is_guild_owner {
        return Permissions::all();
    }

    // 1. Base permissions given to @everyone are applied at a guild level
    let mut permissions = data.everyone_permissions;
    // 2. Permissions allowed to a user by their roles are applied at a guild level
    for role_permission in data.user_roles_permissions {
        permissions |= role_permission;
    }

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }

    // 3. Overwrites that deny permissions for @everyone are applied at a channel level
    permissions &= !data.everyone_deny_overwrites;
    // 4. Overwrites that allow permissions for @everyone are applied at a channel level
    permissions |= data.everyone_allow_overwrites;

    // 5. Overwrites that deny permissions for specific roles are applied at a channel level
    let mut role_deny_permissions = Permissions::empty();
    for p in data.roles_deny_overwrites {
        role_deny_permissions |= p;
    }
    permissions &= !role_deny_permissions;

    // 6. Overwrites that allow permissions for specific roles are applied at a channel level
    let mut role_allow_permissions = Permissions::empty();
    for p in data.roles_allow_overwrites {
        role_allow_permissions |= p;
    }
    permissions |= role_allow_permissions;

    // 7. Member-specific overwrites that deny permissions are applied at a channel level
    permissions &= !data.member_deny_overwrites;
    // 8. Member-specific overwrites that allow permissions are applied at a channel level
    permissions |= data.member_allow_overwrites;

    permissions
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

/// Takes a `&str` as `origin` and tests if either `word_a` or `word_b` is closer.
///
/// **Note**: Normally `word_a` and `word_b` are expected to contain `origin` as substring. If not,
/// using `closest_to_origin` would sort these the end.
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

/// Representation of the number of members that would be pruned by a guild prune operation.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#get-guild-prune-count).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Variant of [`Guild`] returned from [`Http::get_guilds`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object),
/// [subset example](https://discord.com/developers/docs/resources/user#get-current-user-guilds-example-partial-guild).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<ImageHash>,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
    /// See [`Guild::features`].
    pub features: FixedArray<String>,
}

#[cfg(feature = "model")]
impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    ///
    /// This will produce a WEBP image URL, or GIF if the guild has a GIF icon.
    #[must_use]
    pub fn icon_url(&self) -> Option<String> {
        icon_url(self.id, self.icon.as_ref())
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
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum DefaultMessageNotificationLevel {
        /// Receive notifications for everything.
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ExplicitContentFilter {
        /// Don't scan any messages.
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum MfaLevel {
        /// MFA is disabled.
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum VerificationLevel {
        /// Does not require any verification.
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum NsfwLevel {
        /// The nsfw level is not specified.
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

enum_number! {
    /// The [`Guild`] AFK timeout length.
    ///
    /// See [AfkMetadata::afk_timeout].
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum AfkTimeout {
        OneMinute = 60,
        FiveMinutes = 300,
        FifteenMinutes = 900,
        ThirtyMinutes = 1800,
        OneHour = 3600,
        _ => Unknown(u16),
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use std::num::NonZeroU16;

        use crate::model::prelude::*;

        fn gen_member() -> Member {
            Member {
                nick: Some(FixedString::from_static_trunc("aaaa")),
                user: User {
                    name: FixedString::from_static_trunc("test"),
                    discriminator: NonZeroU16::new(1432),
                    ..User::default()
                },
                ..Default::default()
            }
        }

        fn gen() -> Guild {
            let m = gen_member();

            Guild {
                members: ExtractMap::from_iter([m]),
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
