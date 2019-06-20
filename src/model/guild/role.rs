use crate::model::prelude::*;
use std::cmp::Ordering;

#[cfg(all(feature = "builder", feature = "cache", feature = "model"))]
use crate::builder::EditRole;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::model::misc::RoleParseError;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_role;
#[cfg(all(feature = "cache", feature = "http"))]
use crate::http::client::Http;

/// Information about a role within a guild. A role represents a set of
/// permissions, and can be attached to one or multiple users. A role has
/// various miscellaneous configurations, such as being assigned a colour. Roles
/// are unique per guild and do not cross over to other guilds in any way, and
/// can have channel-specific permission overrides in addition to guild-level
/// permissions.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Role {
    /// The Id of the role. Can be used to calculate the role's creation date.
    pub id: RoleId,
    /// The colour of the role. This is an ergonomic representation of the inner
    /// value.
    #[cfg(feature = "utils")]
    #[serde(rename = "color")]
    pub colour: Colour,
    /// The colour of the role.
    #[cfg(not(feature = "utils"))]
    #[serde(rename = "color")]
    pub colour: u32,
    /// Indicator of whether the role is pinned above lesser roles.
    ///
    /// In the client, this causes [`Member`]s in the role to be seen above
    /// those in roles with a lower [`position`].
    ///
    /// [`Member`]: struct.Member.html
    /// [`position`]: #structfield.position
    pub hoist: bool,
    /// Indicator of whether the role is managed by an integration service.
    pub managed: bool,
    /// Indicator of whether the role can be mentioned, similar to mentioning a
    /// specific member or `@everyone`.
    ///
    /// Only members of the role will be notified if a role is mentioned with
    /// this set to `true`.
    #[serde(default)]
    pub mentionable: bool,
    /// The name of the role.
    pub name: String,
    /// A set of permissions that the role has been assigned.
    ///
    /// See the [`permissions`] module for more information.
    ///
    /// [`permissions`]: ../permissions/index.html
    pub permissions: Permissions,
    /// The role's position in the position list. Roles are considered higher in
    /// hierarchy if their position is higher.
    ///
    /// The `@everyone` role is usually either `-1` or `0`.
    pub position: i64,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "cache", feature = "http"))]
    #[inline]
    pub fn delete<T>(&mut self, cache_and_http: T) -> Result<()>
    where T: AsRef<CacheRwLock> + AsRef<Http> {
        AsRef::<Http>::as_ref(&cache_and_http)
            .delete_role(self.find_guild(&cache_and_http)?.0, self.id.0)
    }

    /// Edits a [`Role`], optionally setting its new fields.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted:
    ///
    /// ```rust,ignore
    /// # use serenity::model::id::RoleId;
    /// # let role = RoleId(7).to_role_cached(&cache).unwrap();
    /// // assuming a `role` has already been bound
    //
    /// role.edit(|mut r| {
    ///     r.hoist(true);
    ///
    ///     r
    /// });
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "builder", feature = "cache", feature = "http"))]
    pub fn edit<F: FnOnce(&mut EditRole) -> &mut EditRole, T>(&self, cache_and_http: T, f: F) -> Result<Role>
    where T: AsRef<CacheRwLock> + AsRef<Http> {
        self.find_guild(&cache_and_http)
            .and_then(|guild_id| guild_id.edit_role(&cache_and_http, self.id, f))
    }
    /// Searches the cache for the guild that owns the role.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if a guild is not in the cache
    /// that contains the role.
    ///
    /// [`ModelError::GuildNotFound`]: ../error/enum.Error.html#variant.GuildNotFound
    #[cfg(feature = "cache")]
    pub fn find_guild(&self, cache: impl AsRef<CacheRwLock>) -> Result<GuildId> {
        for guild in cache.as_ref().read().guilds.values() {
            let guild = guild.read();

            if guild.roles.contains_key(&RoleId(self.id.0)) {
                return Ok(guild.id);
            }
        }

        Err(Error::Model(ModelError::GuildNotFound))
    }

    /// Check that the role has the given permission.
    #[inline]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    /// Checks whether the role has all of the given permissions.
    ///
    /// The 'precise' argument is used to check if the role's permissions are
    /// precisely equivalent to the given permissions. If you need only check
    /// that the role has at least the given permissions, pass `false`.
    pub fn has_permissions(&self, permissions: Permissions, precise: bool) -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }
}

impl Display for Role {
    /// Format a mention for the role, pinging its members.
    // This is in the format of: `<@&ROLE_ID>`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { Display::fmt(&self.mention(), f) }
}

impl Eq for Role {}

impl Ord for Role {
    fn cmp(&self, other: &Role) -> Ordering {
        if self.position == other.position {
            self.id.cmp(&other.id)
        } else {
            self.position.cmp(&other.position)
        }
    }
}

impl PartialEq for Role {
    fn eq(&self, other: &Role) -> bool { self.id == other.id }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Role) -> Option<Ordering> { Some(self.cmp(other)) }
}

#[cfg(feature = "model")]
impl RoleId {
    /// Tries to find the [`Role`] by its Id in the cache.
    ///
    /// [`Role`]: ../guild/struct.Role.html
    #[cfg(feature = "cache")]
    pub fn to_role_cached(self, cache: impl AsRef<CacheRwLock>) -> Option<Role> {
        self._to_role_cached(&cache)
    }

    #[cfg(feature = "cache")]
    pub(crate) fn _to_role_cached(self, cache: impl AsRef<CacheRwLock>) -> Option<Role> {
        for guild in cache.as_ref().read().guilds.values() {
            let guild = guild.read();

            if !guild.roles.contains_key(&self) {
                continue;
            }

            if let Some(role) = guild.roles.get(&self) {
                return Some(role.clone());
            }
        }

        None
    }
}

impl From<Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: Role) -> RoleId { role.id }
}

impl<'a> From<&'a Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: &Role) -> RoleId { role.id }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStrAndCache for Role {
    type Err = RoleParseError;

    fn from_str(cache: impl AsRef<CacheRwLock>, s: &str) -> StdResult<Self, Self::Err> {
        match parse_role(s) {
            Some(x) => match RoleId(x).to_role_cached(&cache) {
                Some(role) => Ok(role),
                _ => Err(RoleParseError::NotPresentInCache),
            },
            _ => Err(RoleParseError::InvalidRole),
        }
    }
}
