#[cfg(feature = "http")]
use crate::http::CacheHttp;
use crate::{model::prelude::*};
use chrono::{DateTime, FixedOffset};
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult
};
use super::deserialize_sync_user;

#[cfg(all(feature = "builder", feature = "cache", feature = "model"))]
use crate::builder::EditMember;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::Colour;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::{cache::CacheRwLock, utils};
#[cfg(all(feature = "http", feature = "cache"))]
use crate::http::Http;

/// A trait for allowing both u8 or &str or (u8, &str) to be passed into the `ban` methods in `Guild` and `Member`.
pub trait BanOptions {
    fn dmd(&self) -> u8 { 0 }
    fn reason(&self) -> &str { "" }
}

impl BanOptions for u8 {
    fn dmd(&self) -> u8 { *self }
}

impl BanOptions for str {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for &'a str {
    fn reason(&self) -> &str { self }
}

impl BanOptions for String {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for (u8, &'a str) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { self.1 }
}

impl BanOptions for (u8, String) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { &self.1 }
}

/// Information about a member of a guild.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Member {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: GuildId,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`](struct.Role.html)s given to the member.
    pub roles: Vec<RoleId>,
    /// Attached User struct.
    #[serde(deserialize_with = "deserialize_sync_user",
            serialize_with = "serialize_sync_user")]
    pub user: Arc<RwLock<User>>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Member {
    /// Adds a [`Role`] to the member, editing its roles in-place if the request
    /// was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "cache", feature = "http"))]
    #[inline]
    pub fn add_role<R: Into<RoleId>>(&mut self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self._add_role(&http, role_id.into())
    }

    #[cfg(all(feature = "cache", feature = "http"))]
    fn _add_role(&mut self, http: impl AsRef<Http>, role_id: RoleId) -> Result<()> {
        if self.roles.contains(&role_id) {
            return Ok(());
        }

        match http.as_ref().add_member_role(self.guild_id.0, self.user.read().id.0, role_id.0) {
            Ok(()) => {
                self.roles.push(role_id);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Adds one or multiple [`Role`]s to the member, editing
    /// its roles in-place if the request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "cache", feature = "http"))]
    pub fn add_roles(&mut self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()> {
        self.roles.extend_from_slice(role_ids);

        let mut builder = EditMember::default();
        builder.roles(&self.roles);
        let map = utils::hashmap_to_json_map(builder.0);

        match http.as_ref().edit_member(self.guild_id.0, self.user.read().id.0, &map) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.roles.retain(|r| !role_ids.contains(r));

                Err(why)
            },
        }
    }

    /// Ban the member from its guild, deleting the last X number of
    /// days' worth of messages.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild could not be
    /// found.
    ///
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(all(feature = "cache", feature = "http"))]
    #[inline]
    pub fn ban<BO: BanOptions>(&self, http: impl AsRef<Http>, ban_options: &BO) -> Result<()> {
        self._ban(&http, ban_options.dmd(), ban_options.reason())
    }

    #[cfg(all(feature = "cache", feature = "http"))]
    fn _ban(&self, http: impl AsRef<Http>, dmd: u8, reason: &str) -> Result<()> {
        if dmd > 7 {
            return Err(Error::Model(ModelError::DeleteMessageDaysAmount(dmd)));
        }

        if reason.len() > 512 {
            return Err(Error::ExceededLimit(reason.to_string(), 512));
        }

        http.as_ref().ban_user(
            self.guild_id.0,
            self.user.read().id.0,
            dmd,
            &*reason,
        )
    }

    /// Determines the member's colour.
    #[cfg(all(feature = "cache", feature = "utils"))]
    pub fn colour(&self, cache: impl AsRef<CacheRwLock>) -> Option<Colour> {
        let cache = cache.as_ref().read();
        let guild = cache.guilds.get(&self.guild_id)?.read();

        let mut roles = self.roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .collect::<Vec<&Role>>();
        roles.sort_by(|a, b| b.cmp(a));

        let default = Colour::default();

        roles
            .iter()
            .find(|r| r.colour.0 != default.0)
            .map(|r| r.colour)
    }

    /// Returns the "default channel" of the guild for the member.
    /// (This returns the first channel that can be read by the member, if there isn't
    /// one returns `None`)
    #[cfg(feature = "cache")]
    pub fn default_channel(&self, cache: impl AsRef<CacheRwLock>) -> Option<Arc<RwLock<GuildChannel>>> {
        let guild = match self.guild_id.to_guild_cached(&cache) {
            Some(guild) => guild,
            None => return None,
        };

        let reader = guild.read();

        for (cid, channel) in &reader.channels {
            if reader.user_permissions_in(*cid, self.user.read().id).read_messages() {
                return Some(Arc::clone(channel));
            }
        }

        None
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[inline]
    pub fn display_name(&self) -> Cow<'_, String> {
        self.nick
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(self.user.read().name.clone()))
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[inline]
    pub fn distinct(&self) -> String {
        format!(
            "{}#{}",
            self.display_name(),
            self.user.read().discriminator
        )
    }

    /// Edits the member with the given data. See [`Guild::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// [`Guild::edit_member`]: struct.Guild.html#method.edit_member
    /// [`EditMember`]: ../../builder/struct.EditMember.html
    #[cfg(feature = "cache")]
    pub fn edit<F: FnOnce(&mut EditMember) -> &mut EditMember>(&self, http: impl AsRef<Http>, f: F) -> Result<()> {
        let mut edit_member = EditMember::default();
        f(&mut edit_member);
        let map = utils::hashmap_to_json_map(edit_member.0);

        http.as_ref().edit_member(self.guild_id.0, self.user.read().id.0, &map)
    }

    /// Retrieves the ID and position of the member's highest role in the
    /// hierarchy, if they have one.
    ///
    /// This _may_ return `None` if:
    ///
    /// - the user has roles, but they are not present in the cache for cache
    /// inconsistency reasons
    /// - you already have a write lock to the member's guild
    ///
    /// The "highest role in hierarchy" is defined as the role with the highest
    /// position. If two or more roles have the same highest position, then the
    /// role with the lowest ID is the highest.
    #[cfg(feature = "cache")]
    pub fn highest_role_info(&self, cache: impl AsRef<CacheRwLock>) -> Option<(RoleId, i64)> {
        let guild = self.guild_id.to_guild_cached(&cache)?;
        let reader = guild.try_read()?;

        let mut highest = None;

        for role_id in &self.roles {
            if let Some(role) = reader.roles.get(&role_id) {
                // Skip this role if this role in iteration has:
                //
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some((id, pos)) = highest {
                    if role.position < pos || (role.position == pos && role.id > id) {
                        continue;
                    }
                }

                highest = Some((role.id, role.position));
            }
        }

        highest
    }

    /// Kick the member from the guild.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kick a member from its guild:
    ///
    /// ```rust,ignore
    /// // assuming a `member` has already been bound
    /// match member.kick() {
    ///     Ok(()) => println!("Successfully kicked member"),
    ///     Err(Error::Model(ModelError::GuildNotFound)) => {
    ///         println!("Couldn't determine guild of member");
    ///     },
    ///     Err(Error::Model(ModelError::InvalidPermissions(missing_perms))) => {
    ///         println!("Didn't have permissions; missing: {:?}", missing_perms);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the Id of the member's guild
    /// could not be determined.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform the kick.
    ///
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    pub fn kick(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let locked_cache = cache.read();

                if let Some(guild) = locked_cache.guilds.get(&self.guild_id) {
                    let req = Permissions::KICK_MEMBERS;
                    let reader = guild.read();

                    if !reader.has_perms(cache, req) {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }

                    reader.check_hierarchy(cache, self.user.read().id)?;
                }
            }
        }

        self.guild_id.kick(cache_http.http(), self.user.read().id)
    }

    /// Returns the guild-level permissions for the member.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming there's a `member` variable gotten from anything.
    /// println!("The permission bits for the member are: {}",
    /// member.permissions().expect("permissions").bits);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild the member's in could not be
    /// found in the cache.
    ///
    /// And/or returns [`ModelError::ItemMissing`] if the "default channel" of the guild is not
    /// found.
    ///
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    /// [`ModelError::ItemMissing`]: ../error/enum.Error.html#variant.ItemMissing
    #[cfg(feature = "cache")]
    pub fn permissions(&self, cache: impl AsRef<CacheRwLock>) -> Result<Permissions> {
        let guild = match self.guild_id.to_guild_cached(&cache) {
            Some(guild) => guild,
            None => return Err(From::from(ModelError::GuildNotFound)),
        };

        let reader = guild.read();

        Ok(reader.member_permissions(self.user.read().id))
    }

    /// Removes a [`Role`] from the member, editing its roles in-place if the
    /// request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "cache", feature = "http"))]
    #[inline]
    pub fn remove_role<R: Into<RoleId>>(&mut self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self._remove_role(&http, role_id.into())
    }

    #[cfg(all(feature = "cache", feature = "http"))]
    fn _remove_role(&mut self, http: impl AsRef<Http>, role_id: RoleId) -> Result<()> {
        if !self.roles.contains(&role_id) {
            return Ok(());
        }

        match http.as_ref().remove_member_role(self.guild_id.0, self.user.read().id.0, role_id.0) {
            Ok(()) => {
                self.roles.retain(|r| r.0 != role_id.0);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Removes one or multiple [`Role`]s from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "cache", feature = "http"))]
    pub fn remove_roles(&mut self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()> {
        self.roles.retain(|r| !role_ids.contains(r));

        let mut builder = EditMember::default();
        builder.roles(&self.roles);
        let map = utils::hashmap_to_json_map(builder.0);

        match http.as_ref().edit_member(self.guild_id.0, self.user.read().id.0, &map) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.roles.extend_from_slice(role_ids);

                Err(why)
            },
        }
    }

    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the Cache.
    ///
    /// If role data can not be found for the member, then `None` is returned.
    #[cfg(feature = "cache")]
    pub fn roles(&self, cache: impl AsRef<CacheRwLock>) -> Option<Vec<Role>> {
        self
            .guild_id
            .to_guild_cached(&cache)
            .map(|g| g
                .read()
                .roles
                .values()
                .filter(|role| self.roles.contains(&role.id))
                .cloned()
                .collect())
    }

    /// Unbans the [`User`] from the guild.
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
    #[cfg(all(feature = "cache", feature = "http"))]
    pub fn unban(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().remove_ban(self.guild_id.0, self.user.read().id.0)
    }

    /// Retrieves the member's user ID.
    ///
    /// This is a shortcut for accessing the [`user`] structfield, retrieving a
    /// reader guard, and then copying its ID.
    ///
    /// # Deadlocking
    ///
    /// This function can deadlock while retrieving a read guard to the user
    /// object if your application infinitely holds a write lock elsewhere.
    #[cfg(feature = "cache")]
    pub fn user_id(&self) -> UserId {
        self.user.read().id
    }
}

impl Display for Member {
    /// Mentions the user so that they receive a notification.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assumes a `member` has already been bound
    /// println!("{} is a member!", member);
    /// ```
    ///
    // This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.user.read().mention(), f)
    }
}

/// A partial amount of data for a member.
///
/// This is used in [`Message`]s from [`Guild`]s.
///
/// [`Guild`]: struct.Guild.html
/// [`Message`]: ../channel/struct.Message.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialMember {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}
