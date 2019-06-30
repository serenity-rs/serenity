//! Models relating to guilds and types that it owns.

mod emoji;
mod guild_id;
mod integration;
mod member;
mod partial_guild;
mod role;
mod audit_log;
mod premium_tier;

#[cfg(feature = "http")]
use crate::http::CacheHttp;
pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::role::*;
pub use self::audit_log::*;
pub use self::premium_tier::*;

use chrono::{DateTime, FixedOffset};
use crate::model::prelude::*;
use serde::de::Error as DeError;
use super::utils::*;

#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(all(feature = "cache", feature = "model"))]
use parking_lot::RwLock;
#[cfg(all(feature = "http", feature = "model"))]
use serde_json::json;
#[cfg(all(feature = "cache", feature = "model"))]
use std::sync::Arc;
#[cfg(feature = "model")]
use crate::builder::{CreateChannel, EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use crate::constants::LARGE_THRESHOLD;
#[cfg(feature = "model")]
use log::{error, warn};
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "http")]
use crate::http::Http;

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
    #[serde(serialize_with = "serialize_gen_locked_map")]
    pub channels: HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
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
    ///
    /// [Discord Partnership]: https://discordapp.com/partners
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
    pub joined_at: DateTime<FixedOffset>,
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
    /// [`ReadyEvent`]: ../event/struct.ReadyEvent.html
    /// [`member_count`]: #structfield.member_count
    #[serde(serialize_with = "serialize_gen_map")]
    pub members: HashMap<UserId, Member>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    ///
    /// [`Role`]: struct.Role.html
    /// [`User`]: ../user/struct.User.html
    pub mfa_level: MfaLevel,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    ///
    /// [`User`]: ../user/struct.User.html
    pub owner_id: UserId,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// [`User`]: ../user/struct.User.html
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
    ///
    /// [`User`]: ../user/struct.User.html
    #[serde(serialize_with = "serialize_gen_map")]
    pub voice_states: HashMap<UserId, VoiceState>,
    /// The server's description
    pub description: Option<String>,
    /// The server's premium boosting level
    #[serde(default)]
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server
    #[serde(default)]
    pub premium_subscription_count: u64,
    /// The server's banner
    pub banner: Option<String>,
    /// The vanity url code for the guild
    pub vanity_url_code: Option<String>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Guild {
    #[cfg(feature = "cache")]
    fn check_hierarchy(&self, cache: impl AsRef<CacheRwLock>, other_user: UserId) -> Result<()> {
        let current_id = cache.as_ref().read().user.id;

        if let Some(higher) = self.greater_member_hierarchy(&cache, other_user, current_id) {
            if higher != current_id {
                return Err(Error::Model(ModelError::Hierarchy));
            }
        }

        Ok(())
    }

    /// Returns the "default" channel of the guild for the passed user id.
    /// (This returns the first channel that can be read by the user, if there isn't one,
    /// returns `None`)
    #[cfg(feature = "http")]
    pub fn default_channel(&self, uid: UserId) -> Option<Arc<RwLock<GuildChannel>>> {
        for (cid, channel) in &self.channels {
            if self.permissions_in(*cid, uid).read_messages() {
                return Some(Arc::clone(channel));
            }
        }

        None
    }

    /// Returns the guaranteed "default" channel of the guild.
    /// (This returns the first channel that can be read by everyone, if there isn't one,
    /// returns `None`)
    /// Note however that this is very costy if used in a server with lots of channels,
    /// members, or both.
    pub fn default_channel_guaranteed(&self) -> Option<Arc<RwLock<GuildChannel>>> {
        for (cid, channel) in &self.channels {
            for memid in self.members.keys() {
                if self.permissions_in(*cid, *memid).read_messages() {
                    return Some(Arc::clone(channel));
                }
            }
        }

        None
    }

    #[cfg(feature = "cache")]
    fn has_perms(&self, cache: impl AsRef<CacheRwLock>, mut permissions: Permissions) -> bool {
        let user_id = cache.as_ref().read().user.id;

        let perms = self.member_permissions(user_id);
        permissions.remove(perms);

        permissions.is_empty()
    }

    #[cfg(feature = "cache")]
    pub fn channel_id_from_name(&self, cache: impl AsRef<CacheRwLock>, name: impl AsRef<str>) -> Option<ChannelId> {
        let name = name.as_ref();
        let cache = cache.as_ref().read();
        let guild = cache.guilds.get(&self.id)?.read();

        guild.channels
            .iter()
            .find_map(|(id, c)| {
                if c.read().name == name {
                    Some(*id)
                } else {
                    None
                }
            })
    }

    /// Ban a [`User`] from the guild. All messages by the
    /// user within the last given number of days given will be deleted.
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
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ModelError::DeleteMessageDaysAmount`]: ../error/enum.Error.html#variant.DeleteMessageDaysAmount
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Guild::ban`]: ../guild/struct.Guild.html#method.ban
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "client")]
    #[inline]
    pub fn ban<U: Into<UserId>, BO: BanOptions>(&self, cache_http: impl CacheHttp, user: U, options: &BO) -> Result<()> {
        self._ban(cache_http, user.into(), options)
    }

    #[cfg(feature = "client")]
    fn _ban<BO: BanOptions>(&self, cache_http: impl CacheHttp, user: UserId, options: &BO) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }

                self.check_hierarchy(cache, user)?;
            }
        }

        self.id.ban(cache_http.http(), user, options)
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
    /// [`Ban`]: struct.Ban.html
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    pub fn bans(&self, cache_http: impl CacheHttp) -> Result<Vec<Ban>> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.bans(cache_http.http())
    }

    /// Retrieves a list of [`AuditLogs`] for the guild.
    ///
    /// [`AuditLogs`]: audit_log/struct.AuditLogs.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn audit_logs(&self, http: impl AsRef<Http>,
                             action_type: Option<u8>,
                             user_id: Option<UserId>,
                             before: Option<AuditLogEntryId>,
                             limit: Option<u8>) -> Result<AuditLogs> {
        self.id.audit_logs(&http, action_type, user_id, before, limit)
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn channels(&self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>> { self.id.channels(&http) }

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
    /// let _guild = Guild::create_guild("test", Region::UsWest, None);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`PartialGuild`]: struct.PartialGuild.html
    /// [`Shard`]: ../../gateway/struct.Shard.html
    /// [US West region]: enum.Region.html#variant.UsWest
    /// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
    #[cfg(feature = "http")]
    pub fn create(http: impl AsRef<Http>, name: &str, region: Region, icon: Option<&str>) -> Result<PartialGuild> {
        let map = json!({
            "icon": icon,
            "name": name,
            "region": region.name(),
        });

        http.as_ref().create_guild(&map)
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
    /// let _ = guild.create_channel(|c| c.name("my-test-channel").kind(ChannelType::Text));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`Channel`]: ../channel/enum.Channel.html
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "client")]
    pub fn create_channel(&self, cache_http: impl CacheHttp, f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_CHANNELS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.create_channel(cache_http.http(), f)
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
    /// [`EditProfile::avatar`]: ../../builder/struct.EditProfile.html#method.avatar
    /// [`utils::read_image`]: ../../utils/fn.read_image.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_emoji(&self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji> {
        self.id.create_emoji(&http, name, image)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_integration<I>(&self, http: impl AsRef<Http>, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self.id.create_integration(&http, integration_id, kind)
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
    /// let role = guild.create_role(|r| r.hoist(true).name("role"));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "client")]
    pub fn create_role<F>(&self, cache_http: impl CacheHttp, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_ROLES;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.create_role(cache_http.http(), f)
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
    /// [`ModelError::InvalidUser`]: ../error/enum.Error.html#variant.InvalidUser
    #[cfg(feature = "http")]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.owner_id != cache.read().user.id {
                    let req = Permissions::MANAGE_GUILD;

                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.delete(cache_http.http())
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(&self, http: impl AsRef<Http>, emoji_id: E) -> Result<()> {
        self.id.delete_emoji(&http, emoji_id)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(&self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self.id.delete_integration(&http, integration_id)
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
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_role<R: Into<RoleId>>(&self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self.id.delete_role(&http, role_id)
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
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "client")]
    pub fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
        where F: FnOnce(&mut EditGuild) -> &mut EditGuild {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_GUILD;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        match self.id.edit(cache_http.http(), f) {
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
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_emoji<E: Into<EmojiId>>(&self, http: impl AsRef<Http>, emoji_id: E, name: &str) -> Result<Emoji> {
        self.id.edit_emoji(&http, emoji_id, name)
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
    /// guild.edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_member<F, U>(&self, http: impl AsRef<Http>, user_id: U, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember, U: Into<UserId> {
        self.id.edit_member(&http, user_id, f)
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
    #[cfg(feature = "client")]
    pub fn edit_nickname(&self, cache_http: impl CacheHttp, new_nickname: Option<&str>) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::CHANGE_NICKNAME;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.edit_nickname(cache_http.http(), new_nickname)
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
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_role<F, R>(&self, http: impl AsRef<Http>, role_id: R, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole, R: Into<RoleId> {
        self.id.edit_role(&http, role_id, f)
    }

    /// Edits the order of [`Role`]s
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change the order of a role:
    ///
    /// ```rust,ignore
    /// use serenity::model::RoleId;
    /// guild.edit_role_position(RoleId(8), 2);
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_role_position<R>(&self, http: impl AsRef<Http>, role_id: R, position: u64) -> Result<Vec<Role>>
        where R: Into<RoleId> {
        self.id.edit_role_position(&http, role_id, position)
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// Requires that the current user be in the guild.
    #[cfg(feature = "http")]
    #[inline]
    pub fn get<G: Into<GuildId>>(http: impl AsRef<Http>, guild_id: G) -> Result<PartialGuild> { guild_id.into().to_partial_guild(&http) }

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
    /// [`position`]: struct.Role.html#structfield.position
    #[cfg(feature = "cache")]
    #[inline]
    pub fn greater_member_hierarchy<T, U>(&self, cache: impl AsRef<CacheRwLock>, lhs_id: T, rhs_id: U)
        -> Option<UserId> where T: Into<UserId>, U: Into<UserId> {
        self._greater_member_hierarchy(&cache, lhs_id.into(), rhs_id.into())
    }

    #[cfg(feature = "cache")]
    fn _greater_member_hierarchy(
        &self,
        cache: impl AsRef<CacheRwLock>,
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

        let lhs = self.members.get(&lhs_id)?
            .highest_role_info(&cache)
            .unwrap_or((RoleId(0), 0));
        let rhs = self.members.get(&rhs_id)?
            .highest_role_info(&cache)
            .unwrap_or((RoleId(0), 0));

        // If LHS and RHS both have no top position or have the same role ID,
        // then no one wins.
        if (lhs.1 == 0 && rhs.1 == 0) || (lhs.0 == rhs.0) {
            return None;
        }

        // If LHS's top position is higher than RHS, then LHS wins.
        if lhs.1 > rhs.1 {
            return Some(lhs_id)
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
    pub fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> { self.id.integrations(&http) }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    pub fn invites(&self, cache_http: impl CacheHttp) -> Result<Vec<RichInvite>> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_GUILD;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.invites(cache_http.http())
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    #[inline]
    pub fn is_large(&self) -> bool { self.members.len() > LARGE_THRESHOLD as usize }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, http: impl AsRef<Http>, user_id: U) -> Result<()> { self.id.kick(&http, user_id) }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self, http: impl AsRef<Http>) -> Result<()> { self.id.leave(&http) }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[inline]
    #[cfg(feature = "client")]
    pub fn member<U: Into<UserId>>(&self, cache_http: impl CacheHttp, user_id: U) -> Result<Member> {
        self.id.member(cache_http, user_id)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: ../user/struct.User.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn members<U>(&self, http: impl AsRef<Http>, limit: Option<u64>, after: U) -> Result<Vec<Member>>
        where U: Into<Option<UserId>> {
        self.id.members(&http, limit, after)
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus) -> Vec<&Member> {
        let mut members = vec![];

        for (&id, member) in &self.members {
            match self.presences.get(&id) {
                Some(presence) => if status == presence.status {
                    members.push(member);
                },
                None => continue,
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
    /// [`Member`]: struct.Member.html
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (name, discrim) = if let Some(pos) = name.rfind('#') {
            let split = name.split_at(pos + 1);

            let split2 = (
                match split.0.get(0..split.0.len() - 1) {
                    Some(s) => s,
                    None => "",
                },
                split.1,
            );

            match split2.1.parse::<u16>() {
                Ok(discrim_int) => (split2.0, Some(discrim_int)),
                Err(_) => (name, None),
            }
        } else {
            (&name[..], None)
        };

        self.members
            .values()
            .find(|member| {
                let name_matches = member.user.read().name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.read().discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                self.members
                    .values()
                    .find(|member| member.nick.as_ref().map_or(false, |nick| nick == name))
            })
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
    /// [`Member`]: struct.Member.html
    pub fn members_starting_with(&self, prefix: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|

                if case_sensitive {
                    member.user.read().name.starts_with(prefix)
                } else {
                    starts_with_case_insensitive(&member.user.read().name, prefix)
                }

                || member.nick.as_ref()
                    .map_or(false, |nick|

                    if case_sensitive {
                        nick.starts_with(prefix)
                    } else {
                        starts_with_case_insensitive(nick, prefix)
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.read().name[..], prefix) {
                                Cow::Owned(a.user.read().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(a.user.read().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.read().name[..], prefix) {
                                Cow::Owned(b.user.read().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(b.user.read().name.clone()),
                    };

                    closest_to_origin(prefix, &name_a[..], &name_b[..])
                });
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
    /// [`Member`]: struct.Member.html
    pub fn members_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|

                if case_sensitive {
                    member.user.read().name.contains(substring)
                } else {
                    contains_case_insensitive(&member.user.read().name, substring)
                }

                || member.nick.as_ref()
                    .map_or(false, |nick| {

                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.read().name[..], substring) {
                                Cow::Owned(a.user.read().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(a.user.read().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.read().name[..], substring) {
                                Cow::Owned(b.user.read().name.clone())
                            } else {
                                Cow::Borrowed(nick)
                            }
                        },
                        None => Cow::Owned(b.user.read().name.clone()),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
    /// their username.
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
    /// [`Member`]: struct.Member.html
    pub fn members_username_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member| {
                if case_sensitive {
                    member.user.read().name.contains(substring)
                } else {
                    contains_case_insensitive(&member.user.read().name, substring)
                }
            }).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = &a.user.read().name;
                    let name_b = &b.user.read().name;
                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
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
    /// [`Member`]: struct.Member.html
    pub fn members_nick_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|
                member.nick.as_ref()
                    .map_or(false, |nick| {

                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            Cow::Borrowed(nick)
                        },
                        None => Cow::Owned(a.user.read().name.clone()),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                                Cow::Borrowed(nick)
                            },
                        None => Cow::Owned(b.user.read().name.clone()),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// [`Member`]: struct.Member.html
    #[inline]
    pub fn member_permissions<U>(&self, user_id: U) -> Permissions
        where U: Into<UserId> {
        self._member_permissions(user_id.into())
    }

    fn _member_permissions(&self, user_id: UserId) -> Permissions {
        if user_id == self.owner_id {
            return Permissions::all();
        }

        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name,
                );

                return Permissions::empty();
            },
        };

        let member = match self.members.get(&user_id) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        let mut permissions = everyone.permissions;

        for role in &member.roles {
            if let Some(role) = self.roles.get(role) {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Permissions::all();
                }

                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.read().id,
                    self.id,
                    role,
                );
            }
        }

        permissions
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../permissions/struct.Permissions.html#associatedconstant.MOVE_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn move_member<C, U>(&self, http: impl AsRef<Http>, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self.id.move_member(&http, user_id, channel_id)
    }

    /// Calculate a [`User`]'s permissions in a given channel in the guild.
    ///
    /// [`User`]: ../user/struct.User.html
    #[inline]
    pub fn permissions_in<C, U>(&self, channel_id: C, user_id: U) -> Permissions
        where C: Into<ChannelId>, U: Into<UserId> {
        self._permissions_in(channel_id.into(), user_id.into())
    }

    fn _permissions_in(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> Permissions {
        // The owner has all permissions in all cases.
        if user_id == self.owner_id {
            return Permissions::all();
        }

        // Start by retrieving the @everyone role's permissions.
        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name
                );

                return Permissions::empty();
            },
        };

        // Create a base set of permissions, starting with `@everyone`s.
        let mut permissions = everyone.permissions;

        let member = match self.members.get(&user_id) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        for &role in &member.roles {
            if let Some(role) = self.roles.get(&role) {
                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.read().id,
                    self.id,
                    role
                );
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(channel) = self.channels.get(&channel_id) {
            let channel = channel.read();

            // If this is a text channel, then throw out voice permissions.
            if channel.kind == ChannelType::Text {
                permissions &= !(Permissions::CONNECT
                    | Permissions::SPEAK
                    | Permissions::MUTE_MEMBERS
                    | Permissions::DEAFEN_MEMBERS
                    | Permissions::MOVE_MEMBERS
                    | Permissions::USE_VAD);
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
                    if role.0 != self.id.0 && !member.roles.contains(&role) {
                        continue;
                    }

                    if let Some(role) = self.roles.get(&role) {
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
                if PermissionOverwriteType::Member(user_id) != overwrite.kind {
                    continue;
                }

                permissions = (permissions & !overwrite.deny) | overwrite.allow;
            }
        } else {
            warn!(
                "(╯°□°）╯︵ ┻━┻ Guild {} does not contain channel {}",
                self.id,
                channel_id
            );
        }

        // The default channel is always readable.
        if channel_id.0 == self.id.0 {
            permissions |= Permissions::READ_MESSAGES;
        }

        // No SEND_MESSAGES => no message-sending-related actions
        // If the member does not have the `SEND_MESSAGES` permission, then
        // throw out message-able permissions.
        if !permissions.contains(Permissions::SEND_MESSAGES) {
            permissions &= !(Permissions::SEND_TTS_MESSAGES
                | Permissions::MENTION_EVERYONE
                | Permissions::EMBED_LINKS
                | Permissions::ATTACH_FILES);
        }

        // If the member does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(Permissions::READ_MESSAGES) {
            permissions &= Permissions::KICK_MEMBERS
                | Permissions::BAN_MEMBERS
                | Permissions::ADMINISTRATOR
                | Permissions::MANAGE_GUILD
                | Permissions::CHANGE_NICKNAME
                | Permissions::MANAGE_NICKNAMES;
        }

        permissions
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
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "client")]
    pub fn prune_count(&self, cache_http: impl CacheHttp, days: u16) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::KICK_MEMBERS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.prune_count(cache_http.http(), days)
    }

    /// Re-orders the channels of the guild.
    ///
    /// Although not required, you should specify all channels' positions,
    /// regardless of whether they were updated. Otherwise, positioning can
    /// sometimes get weird.
    #[cfg(feature = "http")]
    pub fn reorder_channels<It>(&self, http: impl AsRef<Http>, channels: It) -> Result<()>
        where It: IntoIterator<Item = (ChannelId, u64)> {
        self.id.reorder_channels(&http, channels)
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
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    pub fn shard_id(&self, cache: impl AsRef<CacheRwLock>) -> u64 { self.id.shard_id(&cache) }

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
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }

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
    pub fn start_integration_sync<I: Into<IntegrationId>>(&self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self.id.start_integration_sync(&http, integration_id)
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
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "client")]
    pub fn start_prune(&self, cache_http: impl CacheHttp, days: u16) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::KICK_MEMBERS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.start_prune(cache_http.http(), days)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "client")]
    pub fn unban<U: Into<UserId>>(&self, cache_http: impl CacheHttp, user_id: U) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::BAN_MEMBERS;

                if !self.has_perms(cache, req) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.id.unban(&cache_http.http(), user_id)
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(&http)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[cfg(feature = "http")]
    #[inline]
    pub fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> { self.id.webhooks(&http) }

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
    /// # fn main() {
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if let Some(arc) = msg.guild_id.unwrap().to_guild_cached(&ctx.cache) {
    ///             if let Some(role) = arc.read().role_by_name("role_name") {
    ///                 println!("{:?}", role);
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "cache", feature = "client")))]
    /// # fn main() {}
    /// ```
    ///
    /// [`Role`]: ../guild/struct.Role.html
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("channels").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(channel) = value.as_object_mut() {
                        channel
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }

            if let Some(array) = map.get_mut("members").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(member) = value.as_object_mut() {
                        member
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
        let application_id = match map.remove("application_id") {
            Some(v) => serde_json::from_value::<Option<ApplicationId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let channels = map.remove("channels")
            .ok_or_else(|| DeError::custom("expected guild channels"))
            .and_then(deserialize_guild_channels)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(DefaultMessageNotificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let explicit_content_filter = map.remove("explicit_content_filter")
            .ok_or_else(|| DeError::custom(
                "expected guild explicit_content_filter"
            ))
            .and_then(ExplicitContentFilter::deserialize)
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
        let joined_at = map.remove("joined_at")
            .ok_or_else(|| DeError::custom("expected guild joined_at"))
            .and_then(DateTime::deserialize)
            .map_err(DeError::custom)?;
        let large = map.remove("large")
            .ok_or_else(|| DeError::custom("expected guild large"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let member_count = map.remove("member_count")
            .ok_or_else(|| DeError::custom("expected guild member_count"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let members = map.remove("members")
            .ok_or_else(|| DeError::custom("expected guild members"))
            .and_then(deserialize_members)
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
        let presences = map.remove("presences")
            .ok_or_else(|| DeError::custom("expected guild presences"))
            .and_then(deserialize_presences)
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
        let system_channel_id = match map.remove("system_channel_id") {
            Some(v) => Option::<ChannelId>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let voice_states = map.remove("voice_states")
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
            _nonexhaustive: (),
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
pub enum GuildContainer {
    /// A guild which can have its contents directly searched.
    Guild(PartialGuild),
    /// A guild's id, which can be used to search the cache for a guild.
    Id(GuildId),
    #[doc(hidden)]
    __Nonexhaustive,
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
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }
}

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer { GuildContainer::Guild(guild) }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer { GuildContainer::Id(guild_id) }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer { GuildContainer::Id(GuildId(id)) }
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }
}

/// Data for an unavailable guild.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GuildUnavailable {
    /// The Id of the [`Guild`] that is unavailable.
    ///
    /// [`Guild`]: struct.Guild.html
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    ///
    /// This should always be `true`.
    pub unavailable: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GuildStatus {
    OnlinePartialGuild(PartialGuild),
    OnlineGuild(Guild),
    Offline(GuildUnavailable),
    #[doc(hidden)]
    __Nonexhaustive,
}

#[cfg(feature = "model")]
impl GuildStatus {
    /// Retrieves the Id of the inner [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn id(&self) -> GuildId {
        match *self {
            GuildStatus::Offline(offline) => offline.id,
            GuildStatus::OnlineGuild(ref guild) => guild.id,
            GuildStatus::OnlinePartialGuild(ref partial_guild) => partial_guild.id,
            GuildStatus::__Nonexhaustive => unreachable!(),
        }
    }
}

/// Default message notification level for a guild.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum DefaultMessageNotificationLevel {
    /// Receive notifications for everything.
    All = 0,
    /// Receive only mentions.
    Mentions = 1,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    DefaultMessageNotificationLevel {
        All,
        Mentions,
    }
);

impl DefaultMessageNotificationLevel {
    pub fn num(self) -> u64 {
        match self {
            DefaultMessageNotificationLevel::All => 0,
            DefaultMessageNotificationLevel::Mentions => 1,
            DefaultMessageNotificationLevel::__Nonexhaustive => unreachable!(),
        }
    }
}

/// Setting used to filter explicit messages from members.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ExplicitContentFilter {
    /// Don't scan any messages.
    None = 0,
    /// Scan messages from members without a role.
    WithoutRole = 1,
    /// Scan messages sent by all members.
    All = 2,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    ExplicitContentFilter {
        None,
        WithoutRole,
        All,
    }
);

impl ExplicitContentFilter {
    pub fn num(self) -> u64 {
        match self {
            ExplicitContentFilter::None => 0,
            ExplicitContentFilter::WithoutRole => 1,
            ExplicitContentFilter::All => 2,
            ExplicitContentFilter::__Nonexhaustive => unreachable!(),
        }
    }
}

/// Multi-Factor Authentication level for guild moderators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum MfaLevel {
    /// MFA is disabled.
    None = 0,
    /// MFA is enabled.
    Elevated = 1,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    MfaLevel {
        None,
        Elevated,
    }
);

impl MfaLevel {
    pub fn num(self) -> u64 {
        match self {
            MfaLevel::None => 0,
            MfaLevel::Elevated => 1,
            MfaLevel::__Nonexhaustive => unreachable!(),
        }
    }
}

/// The name of a region that a voice server can be located in.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum Region {
    #[serde(rename = "amsterdam")] Amsterdam,
    #[serde(rename = "brazil")] Brazil,
    #[serde(rename = "eu-central")] EuCentral,
    #[serde(rename = "eu-west")] EuWest,
    #[serde(rename = "frankfurt")] Frankfurt,
    #[serde(rename = "hongkong")] HongKong,
    #[serde(rename = "japan")] Japan,
    #[serde(rename = "london")] London,
    #[serde(rename = "russia")] Russia,
    #[serde(rename = "singapore")] Singapore,
    #[serde(rename = "sydney")] Sydney,
    #[serde(rename = "us-central")] UsCentral,
    #[serde(rename = "us-east")] UsEast,
    #[serde(rename = "us-south")] UsSouth,
    #[serde(rename = "us-west")] UsWest,
    #[serde(rename = "vip-amsterdam")] VipAmsterdam,
    #[serde(rename = "vip-us-east")] VipUsEast,
    #[serde(rename = "vip-us-west")] VipUsWest,
    #[doc(hidden)]
    __Nonexhaustive,
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
            Region::__Nonexhaustive => unreachable!(),
        }
    }
}

#[doc="The level to set as criteria prior to a user being able to send
    messages in a [`Guild`].

    [`Guild`]: struct.Guild.html"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    VerificationLevel {
        None,
        Low,
        Medium,
        High,
        Higher,
    }
);

impl VerificationLevel {
    pub fn num(self) -> u64 {
        match self {
            VerificationLevel::None => 0,
            VerificationLevel::Low => 1,
            VerificationLevel::Medium => 2,
            VerificationLevel::High => 3,
            VerificationLevel::Higher => 4,
            VerificationLevel::__Nonexhaustive => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "model")]
    mod model {
        use chrono::prelude::*;
        use crate::model::prelude::*;
        use std::collections::*;
        use std::sync::Arc;

        fn gen_user() -> User {
            User {
                id: UserId(210),
                avatar: Some("abc".to_string()),
                bot: true,
                discriminator: 1432,
                name: "test".to_string(),
                _nonexhaustive: (),
            }
        }

        fn gen_member() -> Member {
            let dt: DateTime<FixedOffset> = FixedOffset::east(5 * 3600)
                .ymd(2016, 11, 08)
                .and_hms(0, 0, 0);
            let vec1 = Vec::new();
            let u = Arc::new(RwLock::new(gen_user()));

            Member {
                deaf: false,
                guild_id: GuildId(1),
                joined_at: Some(dt),
                mute: false,
                nick: Some("aaaa".to_string()),
                roles: vec1,
                user: u,
                _nonexhaustive: (),
            }
        }

        fn gen() -> Guild {
            let u = gen_user();
            let m = gen_member();

            let hm1 = HashMap::new();
            let hm2 = HashMap::new();
            let vec1 = Vec::new();
            let dt: DateTime<FixedOffset> = FixedOffset::east(5 * 3600)
                .ymd(2016, 11, 08)
                .and_hms(0, 0, 0);
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
                _nonexhaustive: (),
            }
        }


        #[test]
        fn member_named_username() {
            let guild = gen();
            let lhs = guild
                .member_named("test#1432")
                .unwrap()
                .display_name();

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
