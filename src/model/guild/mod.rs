//! Models relating to guilds and types that it owns.

mod audit_log;
mod emoji;
mod guild_id;
mod integration;
mod member;
mod partial_guild;
mod premium_tier;
mod role;
mod system_channel;

use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use serde::de::Error as DeError;
#[cfg(feature = "simd-json")]
use simd_json::StaticNode;
#[cfg(feature = "model")]
use tracing::{error, warn};

pub use self::audit_log::*;
pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::premium_tier::*;
pub use self::role::*;
pub use self::system_channel::*;
use super::utils::*;
#[cfg(feature = "model")]
use crate::builder::{CreateChannel, EditGuild, EditMember, EditRole};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::constants::LARGE_THRESHOLD;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
#[cfg(all(feature = "http", feature = "model"))]
use crate::json::json;
#[cfg(all(feature = "model", feature = "unstable_discord_api"))]
use crate::{
    builder::CreateInteraction,
    model::interactions::{ApplicationCommand, Interaction},
};
use crate::{
    json::{from_number, from_value, prelude::*},
    model::prelude::*,
};

/// A representation of a banning of a user.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash, Serialize)]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<String>,
    /// The user that was banned.
    pub user: User,
}

/// Information about a Discord guild, such as channels, emojis, etc.
#[derive(Clone, Debug, Serialize)]
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
    #[serde(serialize_with = "serialize_gen_map")]
    pub channels: HashMap<ChannelId, GuildChannel>,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: DefaultMessageNotificationLevel,
    /// All of the guild's custom emojis.
    #[serde(serialize_with = "serialize_gen_map")]
    pub emojis: HashMap<EmojiId, Emoji>,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// VIP features enabled for the guild. Can be obtained through the
    /// [Discord Partnership] website.
    ///
    /// The following is a list of known features:
    ///
    /// - `INVITE_SPLASH`
    /// - `VANITY_URL`
    /// - `VERIFIED`
    /// - `VIP_REGIONS`
    /// - `PARTNERED`
    /// - `MORE_EMOJI`
    /// - `DISCOVERABLE`
    /// - `FEATURABLE`
    /// - `COMMERCE`
    /// - `PUBLIC`
    /// - `NEWS`
    /// - `BANNER`
    /// - `ANIMATED_ICON`
    /// - `PUBLIC_DISABLED`
    /// - `COMMUNITY`
    /// - `WELCOME_SCREEN_ENABLED`
    ///
    /// [Discord Partnership]: https://discord.com/partners
    pub features: Vec<String>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivilant to the Id of the default role (`@everyone`) and also
    /// that of the default channel (typically `#general`).
    pub id: GuildId,
    /// The date that the current user joined the guild.
    pub joined_at: DateTime<Utc>,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received
    /// if the [`member_count`] is greater than the `LARGE_THRESHOLD` set by
    /// the library.
    ///
    /// [`member_count`]: Self::member_count
    #[serde(serialize_with = "serialize_gen_map")]
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
    #[serde(serialize_with = "serialize_gen_map")]
    pub presences: HashMap<UserId, Presence>,
    /// The region that the voice servers that the guild uses are located in.
    pub region: String,
    /// A mapping of the guild's roles.
    #[serde(serialize_with = "serialize_gen_map")]
    pub roles: HashMap<RoleId, Role>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the [`"InviteSplash"`] feature is enabled, this can be used to generate
    /// a URL to a splash image.
    pub splash: Option<String>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<ChannelId>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// A mapping of [`User`]s to their current voice state.
    #[serde(serialize_with = "serialize_gen_map")]
    pub voice_states: HashMap<UserId, VoiceState>,
    /// The server's description
    pub description: Option<String>,
    /// The server's premium boosting level.
    #[serde(default)]
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    #[serde(default)]
    pub premium_subscription_count: u64,
    /// The server's banner.
    pub banner: Option<String>,
    /// The vanity url code for the guild.
    pub vanity_url_code: Option<String>,
    /// The preferred locale of this guild only set if guild has the "DISCOVERABLE"
    /// feature, defaults to en-US.
    pub preferred_locale: String,
}

#[cfg(feature = "model")]
impl Guild {
    #[cfg(feature = "cache")]
    async fn check_hierarchy(&self, cache: impl AsRef<Cache>, other_user: UserId) -> Result<()> {
        let current_id = cache.as_ref().current_user().await.id;

        if let Some(higher) = self.greater_member_hierarchy(&cache, other_user, current_id).await {
            if higher != current_id {
                return Err(Error::Model(ModelError::Hierarchy));
            }
        }

        Ok(())
    }

    /// Returns the "default" channel of the guild for the passed user id.
    /// (This returns the first channel that can be read by the user, if there isn't one,
    /// returns `None`)
    pub async fn default_channel(&self, uid: UserId) -> Option<&GuildChannel> {
        let member = self.members.get(&uid)?;
        for channel in self.channels.values() {
            if self.user_permissions_in(channel, member).ok()?.read_messages() {
                return Some(channel);
            }
        }

        None
    }

    /// Returns the guaranteed "default" channel of the guild.
    /// (This returns the first channel that can be read by everyone, if there isn't one,
    /// returns `None`)
    ///
    /// **Note**: This is very costly if used in a server with lots of channels,
    /// members, or both.
    pub async fn default_channel_guaranteed(&self) -> Option<&GuildChannel> {
        for channel in self.channels.values() {
            for member in self.members.values() {
                if self.user_permissions_in(channel, member).ok()?.read_messages() {
                    return Some(channel);
                }
            }
        }

        None
    }

    #[cfg(feature = "cache")]
    async fn has_perms(&self, cache_http: impl CacheHttp, mut permissions: Permissions) -> bool {
        if let Some(cache) = cache_http.cache() {
            let user_id = cache.current_user().await.id;

            if let Ok(perms) = self.member_permissions(&cache_http, user_id).await {
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
    pub async fn channel_id_from_name(
        &self,
        cache: impl AsRef<Cache>,
        name: impl AsRef<str>,
    ) -> Option<ChannelId> {
        let name = name.as_ref();
        let guild_channels = cache.as_ref().guild_channels(&self.id).await?;

        for (id, channel) in guild_channels.iter() {
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

    /// Ban a [`User`] from the guild with a reason. Refer to [`ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the possible reasons `ban` may return an error, an [`Error::ExceededLimit`]
    /// may also be returned if the reason is too long.
    ///
    /// [`ban`]: Self::ban
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

                self.check_hierarchy(cache, user).await?;
            }
        }

        self.id.ban_with_reason(cache_http.http(), user, dmd, reason).await
    }

    /// Returns the formatted URL of the guild's banner image, if one exists.
    pub fn banner_url(&self) -> Option<String> {
        self.banner
            .as_ref()
            .map(|banner| format!(cdn!("/banners/{}/{}.webp?size=1024"), self.id, banner))
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
        self.id.audit_logs(&http, action_type, user_id, before, limit).await
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
        self.id.channels(&http).await
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
    /// use serenity::model::{Guild, Region};
    ///
    /// let _guild = Guild::create_guild(&http, "test", Region::UsWest, None).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user cannot create a Guild.
    ///
    /// [`Shard`]: crate::gateway::Shard
    /// [US West region]: Region::UsWest
    /// [whitelist]: https://discord.com/developers/docs/resources/guild#create-guild
    pub async fn create(
        http: impl AsRef<Http>,
        name: &str,
        region: Region,
        icon: Option<&str>,
    ) -> Result<PartialGuild> {
        let map = json!({
            "icon": icon,
            "name": name,
            "region": region.name(),
        });

        http.as_ref().create_guild(&map).await
    }

    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelType;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let _ = guild
    ///     .create_channel(&http, |c| c.name("my-test-channel").kind(ChannelType::Text))
    ///     .await;
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to manage channels.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_channel(
        &self,
        cache_http: impl CacheHttp,
        f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel,
    ) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::MANAGE_CHANNELS;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.create_channel(cache_http.http(), f).await
    }

    /// Creates an emoji in the guild with a name and base64-encoded image. The
    /// [`utils::read_image`] function is provided for you as a simple method to
    /// read an image and encode it into base64, if you are reading from the
    /// filesystem.
    ///
    /// The name of the emoji must be at least 2 characters long and can only
    /// contain alphanumeric characters and underscores.
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
    /// Returns [`Error::Http`] if the current user lacks permission.
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
    pub async fn create_integration<I>(
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
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Create a role which can be mentioned, with the name 'test':
    ///
    /// ```rust,ignore
    /// // assuming a `guild` has been bound
    ///
    /// let role = guild.create_role(&http, |r| r.hoist(true).name("role")).await;
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to manage roles.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does
    /// not have permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn create_role<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Role>
    where
        F: FnOnce(&mut EditRole) -> &mut EditRole,
    {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::MANAGE_ROLES;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.create_role(cache_http.http(), f).await
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
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.owner_id != cache.current_user().await.id {
                    let req = Permissions::MANAGE_GUILD;

                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.delete(cache_http.http()).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
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
        self.id.delete_role(&http, role_id).await
    }

    /// Edits the current guild with new data where specified.
    ///
    /// Refer to `EditGuild`'s documentation for a full list of methods.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// # Examples
    ///
    /// Change a guild's icon using a file name "icon.png":
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // We are using read_image helper function from utils.
    /// let base64_icon = utils::read_image("./icon.png")
    ///     .expect("Failed to read image");
    ///
    /// guild.edit(|g| g.icon(base64_icon));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to edit the guild.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have
    /// permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    pub async fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
    where
        F: FnOnce(&mut EditGuild) -> &mut EditGuild,
    {
        #[cfg(feature = "cache")]
        {
            if cache_http.cache().is_some() {
                let req = Permissions::MANAGE_GUILD;

                if !self.has_perms(&cache_http, req).await {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        match self.id.edit(cache_http.http(), f).await {
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
    /// Also see [`Emoji::edit`] if you have the `cache` and `model` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Emojis]: Permissions::MANAGE_EMOJIS
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// nicknaming them. Returns the new member.
    ///
    /// Refer to [`EditMember`]'s documentation for a full list of methods and
    /// permission restrictions.
    ///
    /// # Examples
    ///
    /// Mute a member and set their roles to just one role with a predefined Id:
    ///
    /// ```rust,ignore
    /// guild.edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks the necessary permissions.
    ///
    /// [`EditMember`]: crate::builder::EditMember
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted:
    ///
    /// ```rust,ignore
    /// guild.edit_role(&context, RoleId(7), |r| r.hoist(true));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit_role<F>(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        f: F,
    ) -> Result<Role>
    where
        F: FnOnce(&mut EditRole) -> &mut EditRole,
    {
        self.id.edit_role(&http, role_id, f).await
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
    /// guild.edit_role_position(&context, RoleId(8), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn edit_role_position(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        position: u64,
    ) -> Result<Vec<Role>> {
        self.id.edit_role_position(&http, role_id, position).await
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn get(http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<PartialGuild> {
        guild_id.into().to_partial_guild(&http).await
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest
    /// [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances
    /// is not present. Returns `None` if the users have the same hierarchy, as
    /// neither are greater than the other.
    ///
    /// If both user IDs are the same, `None` is returned. If one of the users
    /// is the guild owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn greater_member_hierarchy(
        &self,
        cache: impl AsRef<Cache>,
        lhs_id: impl Into<UserId>,
        rhs_id: impl Into<UserId>,
    ) -> Option<UserId> {
        self._greater_member_hierarchy(&cache, lhs_id.into(), rhs_id.into()).await
    }

    #[cfg(feature = "cache")]
    async fn _greater_member_hierarchy(
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
            self.members.get(&lhs_id)?.highest_role_info(&cache).await.unwrap_or((RoleId(0), 0));
        let rhs =
            self.members.get(&rhs_id)?.highest_role_info(&cache).await.unwrap_or((RoleId(0), 0));

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
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            let ext = if icon.starts_with("a_") { "gif" } else { "webp" };

            format!(cdn!("/icons/{}/{}.{}"), self.id, icon, ext)
        })
    }

    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable
    ///
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// May also return [`Error::Json`] if there is an error in deserialzing
    /// the API response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
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
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        self.id.integrations(&http).await
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
    /// [`Error::Http`]: crate::error::Error::Http
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn kick(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.kick(&http, user_id).await
    }

    #[inline]
    /// # Errors
    ///
    /// In addition to the reasons `kick` may return an error,
    /// may also return an error if the reason is too long.
    pub async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        self.id.kick_with_reason(&http, user_id, reason).await
    }

    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the current user
    /// cannot leave the guild, or currently is not in the guild.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(&http).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the user is not in
    /// the guild or if the guild is otherwise unavailable.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// Returns an [`Error::Http`] if the API returns an error, may also
    /// return [`Error::NotInRange`] if the input is not within range.
    ///
    /// [`User`]: crate::model::user::User
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::NotInRange`]: crate::error::Error::NotInRange
    #[inline]
    pub async fn members(
        &self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>> {
        self.id.members(&http, limit, after).await
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
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
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (name, discrim) = if let Some(pos) = name.rfind('#') {
            let split = name.split_at(pos + 1);

            let split2 = (split.0.get(0..split.0.len() - 1).unwrap_or(""), split.1);

            match split2.1.parse::<u16>() {
                Ok(discrim_int) => (split2.0, Some(discrim_int)),
                Err(_) => (name, None),
            }
        } else {
            (&name[..], None)
        };

        for member in self.members.values() {
            let name_matches = member.user.name == name;

            let discrim_matches = match discrim {
                Some(discrim) => member.user.discriminator == discrim,
                None => true,
            };

            if name_matches && discrim_matches {
                return Some(member);
            }
        }

        self.members.values().find(|member| member.nick.as_ref().map_or(false, |nick| nick == name))
    }

    /// Retrieves all [`Member`] that start with a given `String`.
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
    pub async fn members_starting_with(
        &self,
        prefix: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        fn starts_with(prefix: &str, case_sensitive: bool, name: &str) -> bool {
            case_sensitive && name.starts_with(prefix)
                || !case_sensitive && starts_with_case_insensitive(name, prefix)
        }

        let mut members = futures::stream::iter(self.members.values())
            .filter_map(|member| async move {
                let username = &member.user.name;

                if starts_with(prefix, case_sensitive, username) {
                    Some((member, username.to_string()))
                } else {
                    match member.nick {
                        Some(ref nick) => {
                            if starts_with(prefix, case_sensitive, nick) {
                                Some((member, nick.to_string()))
                            } else {
                                None
                            }
                        },
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>()
            .await;

        if sorted {
            members.sort_by(|a, b| closest_to_origin(prefix, &a.1[..], &b.1[..]));
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` as
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
    /// **Note**: Due to two fields of a `Member` being candidates for
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
    pub async fn members_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        fn contains(substring: &str, case_sensitive: bool, name: &str) -> bool {
            case_sensitive && name.contains(substring)
                || !case_sensitive && contains_case_insensitive(name, substring)
        }

        let mut members = futures::stream::iter(self.members.values())
            .filter_map(|member| async move {
                let username = &member.user.name;

                if contains(substring, case_sensitive, username) {
                    Some((member, username.to_string()))
                } else {
                    match member.nick {
                        Some(ref nick) => {
                            if contains(substring, case_sensitive, nick) {
                                Some((member, nick.to_string()))
                            } else {
                                None
                            }
                        },
                        None => None,
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>()
            .await;

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
            members
        } else {
            members
        }
    }

    /// Retrieves a tuple of [`Member`]s containing a given `String` in
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
    pub async fn members_username_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = futures::stream::iter(self.members.values())
            .filter_map(|member| async move {
                if case_sensitive {
                    let name = &member.user.name;

                    if name.contains(substring) {
                        Some((member, name.to_string()))
                    } else {
                        None
                    }
                } else {
                    let name = &member.user.name;

                    if contains_case_insensitive(name, substring) {
                        Some((member, name.to_string()))
                    } else {
                        None
                    }
                }
            })
            .collect::<Vec<(&Member, String)>>()
            .await;

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
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
    /// **Note**: Instead of panicing, when sorting does not find
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
    pub async fn members_nick_containing(
        &self,
        substring: &str,
        case_sensitive: bool,
        sorted: bool,
    ) -> Vec<(&Member, String)> {
        let mut members = futures::stream::iter(self.members.values())
            .filter_map(|member| async move {
                let nick = match member.nick {
                    Some(ref nick) => nick.to_string(),
                    None => member.user.name.to_string(),
                };

                if case_sensitive && nick.contains(substring)
                    || !case_sensitive && contains_case_insensitive(&nick, substring)
                {
                    Some((member, nick))
                } else {
                    None
                }
            })
            .collect::<Vec<(&Member, String)>>()
            .await;

        if sorted {
            members.sort_by(|a, b| closest_to_origin(substring, &a.1[..], &b.1[..]));
            members
        } else {
            members
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

        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!("@everyone role ({}) missing in '{}'", self.id, self.name,);

                return Ok(Permissions::empty());
            },
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
    /// [`Error::Http`]: crate::error::Error::Http
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
    /// Returns [`Error::Model`] if the [`Member`] has a non-existent role
    /// for some reason.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    #[inline]
    pub fn user_permissions_in(
        &self,
        channel: &GuildChannel,
        member: &Member,
    ) -> Result<Permissions> {
        Self::_user_permissions_in(channel, member, &self.roles, self.owner_id, self.id)
    }

    /// Helper function that can also be used from `PartialGuild`.
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
        let everyone = match roles.get(&RoleId(guild_id.0)) {
            Some(everyone) => everyone,
            None => {
                error!("@everyone role missing in {}", guild_id,);
                return Err(Error::Model(ModelError::RoleNotFound));
            },
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
            permissions |= Permissions::READ_MESSAGES;
        }

        Self::remove_unusable_permissions(&mut permissions);

        Ok(permissions)
    }

    /// Calculate a [`Role`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Will return an [`Error::Model`] if the `Role` or `Channel` is not from this `Guild`.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    #[inline]
    pub fn role_permissions_in(&self, channel: &GuildChannel, role: &Role) -> Result<Permissions> {
        Self::_role_permissions_in(channel, role, self.id)
    }

    /// Helper function that can also be used from `PartialGuild`.
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
    pub async fn prune_count(&self, cache_http: impl CacheHttp, days: u16) -> Result<GuildPrune> {
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

        // If the permission does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(Permissions::READ_MESSAGES) {
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
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn reorder_channels<It>(&self, http: impl AsRef<Http>, channels: It) -> Result<()>
    where
        It: IntoIterator<Item = (ChannelId, u64)>,
    {
        self.id.reorder_channels(&http, channels).await
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
        self.id.shard_id(&cache).await
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
    /// Returns an [`Error::Http`] if the current user does not have permission,
    /// or if an `Integration` with that Id does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Error::Http`]: crate::error::Error::Http
    #[inline]
    pub async fn start_integration_sync(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.start_integration_sync(&http, integration_id).await
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
    pub async fn start_prune(&self, cache_http: impl CacheHttp, days: u16) -> Result<GuildPrune> {
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
    /// [`Error::Http`]: crate::error::Error::Http
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

        self.id.unban(&cache_http.http(), user_id).await
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
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    #[inline]
    pub async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(&http).await
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
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
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
    ///             if let Some(guild) = guild_id.to_guild_cached(&ctx).await {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("{:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::builder("token").event_handler(Handler).await?;
    ///
    /// client.start().await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id").and_then(|x| x.as_str()).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("channels").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(channel) = value.as_object_mut() {
                        channel.insert("guild_id".to_string(), from_number(guild_id));
                    }
                }
            }

            if let Some(array) = map.get_mut("members").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(member) = value.as_object_mut() {
                        member.insert("guild_id".to_string(), from_number(guild_id));
                    }
                }
            }

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
        let application_id = match map.remove("application_id") {
            Some(v) => from_value::<Option<ApplicationId>>(v).map_err(DeError::custom)?,
            None => None,
        };
        let channels = map
            .remove("channels")
            .ok_or_else(|| DeError::custom("expected guild channels"))
            .and_then(deserialize_guild_channels)
            .map_err(DeError::custom)?;
        let default_message_notifications = map
            .remove("default_message_notifications")
            .ok_or_else(|| DeError::custom("expected guild default_message_notifications"))
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map
            .remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let explicit_content_filter = map
            .remove("explicit_content_filter")
            .ok_or_else(|| DeError::custom("expected guild explicit_content_filter"))
            .and_then(ExplicitContentFilter::deserialize)
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
        let joined_at = map
            .remove("joined_at")
            .ok_or_else(|| DeError::custom("expected guild joined_at"))
            .and_then(DateTime::deserialize)
            .map_err(DeError::custom)?;
        let large = map
            .remove("large")
            .ok_or_else(|| DeError::custom("expected guild large"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let member_count = map
            .remove("member_count")
            .ok_or_else(|| DeError::custom("expected guild member_count"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let members = map
            .remove("members")
            .ok_or_else(|| DeError::custom("expected guild members"))
            .and_then(deserialize_members)
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
        let presences = map
            .remove("presences")
            .ok_or_else(|| DeError::custom("expected guild presences"))
            .and_then(deserialize_presences)
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
        let system_channel_id = match map.remove("system_channel_id") {
            Some(v) => Option::<ChannelId>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map
            .remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let voice_states = map
            .remove("voice_states")
            .ok_or_else(|| DeError::custom("expected guild voice_states"))
            .and_then(deserialize_voice_states)
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
        let preferred_locale = map
            .remove("preferred_locale")
            .ok_or_else(|| DeError::custom("expected preferred locale"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            afk_channel_id,
            application_id,
            afk_timeout,
            channels,
            default_message_notifications,
            emojis,
            explicit_content_filter,
            features,
            icon,
            id,
            joined_at,
            large,
            member_count,
            members,
            mfa_level,
            name,
            owner_id,
            presences,
            region,
            roles,
            splash,
            system_channel_id,
            verification_level,
            voice_states,
            description,
            premium_tier,
            premium_subscription_count,
            banner,
            vanity_url_code,
            preferred_locale,
        })
    }
}

/// Checks if a `&str` contains another `&str`.
#[cfg(feature = "model")]
fn contains_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().contains(&to_find.to_lowercase())
}

/// Checks if a `&str` starts with another `&str`.
#[cfg(feature = "model")]
fn starts_with_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().starts_with(&to_find.to_lowercase())
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

/// A container for guilds.
///
/// This is used to differentiate whether a guild itself can be used or whether
/// a guild needs to be retrieved from the cache.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(PartialGuild),
    /// A guild's id, which can be used to search the cache for a guild.
    Id(GuildId),
}

/// Information relating to a guild's widget embed.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildEmbed {
    /// The Id of the channel to show the embed for.
    pub channel_id: ChannelId,
    /// Whether the widget embed is enabled.
    pub enabled: bool,
}

/// Representation of the number of members that would be pruned by a guild
/// prune operation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Basic information about a guild.
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
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            let ext = if icon.starts_with("a_") { "gif" } else { "webp" };

            format!(cdn!("/icons/{}/{}.{}"), self.id, icon, ext)
        })
    }
}

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer {
        GuildContainer::Guild(guild)
    }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer {
        GuildContainer::Id(guild_id)
    }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer {
        GuildContainer::Id(GuildId(id))
    }
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.splash
            .as_ref()
            .map(|splash| format!(cdn!("/splashes/{}/{}.webp?size=4096"), self.id, splash))
    }
}

/// Data for an unavailable guild.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildUnavailable {
    /// The Id of the [`Guild`] that may be unavailable.
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    #[serde(default)]
    pub unavailable: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GuildStatus {
    OnlinePartialGuild(PartialGuild),
    OnlineGuild(Guild),
    Offline(GuildUnavailable),
}

#[cfg(feature = "model")]
impl GuildStatus {
    /// Retrieves the Id of the inner [`Guild`].
    pub fn id(&self) -> GuildId {
        match *self {
            GuildStatus::Offline(offline) => offline.id,
            GuildStatus::OnlineGuild(ref guild) => guild.id,
            GuildStatus::OnlinePartialGuild(ref partial_guild) => partial_guild.id,
        }
    }
}

/// Default message notification level for a guild.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DefaultMessageNotificationLevel {
    /// Receive notifications for everything.
    All = 0,
    /// Receive only mentions.
    Mentions = 1,
}

enum_number!(DefaultMessageNotificationLevel {
    All,
    Mentions
});

impl DefaultMessageNotificationLevel {
    pub fn num(self) -> u64 {
        match self {
            DefaultMessageNotificationLevel::All => 0,
            DefaultMessageNotificationLevel::Mentions => 1,
        }
    }
}

/// Setting used to filter explicit messages from members.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ExplicitContentFilter {
    /// Don't scan any messages.
    None = 0,
    /// Scan messages from members without a role.
    WithoutRole = 1,
    /// Scan messages sent by all members.
    All = 2,
}

enum_number!(ExplicitContentFilter {
    None,
    WithoutRole,
    All
});

impl ExplicitContentFilter {
    pub fn num(self) -> u64 {
        match self {
            ExplicitContentFilter::None => 0,
            ExplicitContentFilter::WithoutRole => 1,
            ExplicitContentFilter::All => 2,
        }
    }
}

/// Multi-Factor Authentication level for guild moderators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum MfaLevel {
    /// MFA is disabled.
    None = 0,
    /// MFA is enabled.
    Elevated = 1,
}

enum_number!(MfaLevel {
    None,
    Elevated
});

impl MfaLevel {
    pub fn num(self) -> u64 {
        match self {
            MfaLevel::None => 0,
            MfaLevel::Elevated => 1,
        }
    }
}

/// The name of a region that a voice server can be located in.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
pub enum Region {
    #[serde(rename = "amsterdam")]
    Amsterdam,
    #[serde(rename = "brazil")]
    Brazil,
    #[serde(rename = "eu-central")]
    EuCentral,
    #[serde(rename = "eu-west")]
    EuWest,
    #[serde(rename = "frankfurt")]
    Frankfurt,
    #[serde(rename = "hongkong")]
    HongKong,
    #[serde(rename = "japan")]
    Japan,
    #[serde(rename = "london")]
    London,
    #[serde(rename = "russia")]
    Russia,
    #[serde(rename = "singapore")]
    Singapore,
    #[serde(rename = "sydney")]
    Sydney,
    #[serde(rename = "us-central")]
    UsCentral,
    #[serde(rename = "us-east")]
    UsEast,
    #[serde(rename = "us-south")]
    UsSouth,
    #[serde(rename = "us-west")]
    UsWest,
    #[serde(rename = "vip-amsterdam")]
    VipAmsterdam,
    #[serde(rename = "vip-us-east")]
    VipUsEast,
    #[serde(rename = "vip-us-west")]
    VipUsWest,
}

impl Region {
    pub fn name(&self) -> &str {
        match *self {
            Region::Amsterdam => "amsterdam",
            Region::Brazil => "brazil",
            Region::EuCentral => "eu-central",
            Region::EuWest => "eu-west",
            Region::Frankfurt => "frankfurt",
            Region::HongKong => "hongkong",
            Region::Japan => "japan",
            Region::London => "london",
            Region::Russia => "russia",
            Region::Singapore => "singapore",
            Region::Sydney => "sydney",
            Region::UsCentral => "us-central",
            Region::UsEast => "us-east",
            Region::UsSouth => "us-south",
            Region::UsWest => "us-west",
            Region::VipAmsterdam => "vip-amsterdam",
            Region::VipUsEast => "vip-us-east",
            Region::VipUsWest => "vip-us-west",
        }
    }
}

/// The level to set as criteria prior to a user being able to send
/// messages in a [`Guild`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
}

enum_number!(VerificationLevel {
    None,
    Low,
    Medium,
    High,
    Higher
});

impl VerificationLevel {
    pub fn num(self) -> u64 {
        match self {
            VerificationLevel::None => 0,
            VerificationLevel::Low => 1,
            VerificationLevel::Medium => 2,
            VerificationLevel::High => 3,
            VerificationLevel::Higher => 4,
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use std::collections::*;

        use chrono::prelude::*;

        use crate::model::prelude::*;

        fn gen_user() -> User {
            User::default()
        }

        fn gen_member() -> Member {
            #[allow(clippy::zero_prefixed_literal)]
            let dt: DateTime<Utc> =
                FixedOffset::east(5 * 3600).ymd(2016, 11, 08).and_hms(0, 0, 0).with_timezone(&Utc);

            let vec1 = Vec::new();
            let u = gen_user();

            Member {
                deaf: false,
                guild_id: GuildId(1),
                joined_at: Some(dt),
                mute: false,
                nick: Some("aaaa".to_string()),
                roles: vec1,
                user: u,
                pending: false,
                premium_since: None,
                #[cfg(feature = "unstable_discord_api")]
                permissions: None,
            }
        }

        fn gen() -> Guild {
            let u = gen_user();
            let m = gen_member();

            let hm1 = HashMap::new();
            let hm2 = HashMap::new();
            let vec1 = Vec::new();

            #[allow(clippy::zero_prefixed_literal)]
            let dt: DateTime<Utc> =
                FixedOffset::east(5 * 3600).ymd(2016, 11, 08).and_hms(0, 0, 0).with_timezone(&Utc);

            let mut hm3 = HashMap::new();
            let hm4 = HashMap::new();
            let hm5 = HashMap::new();
            let hm6 = HashMap::new();

            hm3.insert(u.id, m);

            let notifications = DefaultMessageNotificationLevel::All;

            Guild {
                afk_channel_id: Some(ChannelId(0)),
                afk_timeout: 0,
                channels: hm1,
                default_message_notifications: notifications,
                emojis: hm2,
                features: vec1,
                icon: Some("/avatars/210/a_aaa.webp?size=1024".to_string()),
                id: GuildId(1),
                joined_at: dt,
                large: false,
                member_count: 1,
                members: hm3,
                mfa_level: MfaLevel::Elevated,
                name: "Spaghetti".to_string(),
                owner_id: UserId(210),
                presences: hm4,
                region: "NA".to_string(),
                roles: hm5,
                splash: Some("asdf".to_string()),
                verification_level: VerificationLevel::None,
                voice_states: hm6,
                description: None,
                premium_tier: PremiumTier::Tier1,
                application_id: Some(ApplicationId(0)),
                explicit_content_filter: ExplicitContentFilter::None,
                system_channel_id: Some(ChannelId(0)),
                premium_subscription_count: 12,
                banner: None,
                vanity_url_code: Some("bruhmoment".to_string()),
                preferred_locale: "en-US".to_string(),
            }
        }

        #[tokio::test]
        #[allow(clippy::unwrap_used)]
        async fn member_named_username() {
            let guild = gen();
            let lhs = guild.member_named("test#1432").unwrap().display_name();

            assert_eq!(lhs, gen_member().display_name());
        }

        #[tokio::test]
        #[allow(clippy::unwrap_used)]
        async fn member_named_nickname() {
            let guild = gen();
            let lhs = guild.member_named("aaaa").unwrap().display_name();

            assert_eq!(lhs, gen_member().display_name());
        }
    }
}
