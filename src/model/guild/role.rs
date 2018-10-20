use model::prelude::*;
use std::cmp::Ordering;

#[cfg(all(feature = "builder", feature = "cache", feature = "model"))]
use builder::EditRole;
#[cfg(all(feature = "cache", feature = "model"))]
use internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use {CACHE, Cache, http};

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use std::str::FromStr;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use model::misc::RoleParseError;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use utils::parse_role;

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
}

#[cfg(feature = "model")]
impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "cache")]
    #[inline]
    pub fn delete(&self) -> Result<()> { http::delete_role(self.find_guild()?.0, self.id.0) }

    /// Edits a [`Role`], optionally setting its new fields.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted:
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::RoleId;
    /// # let role = RoleId(7).to_role_cached().unwrap();
    /// // assuming a `role` has already been bound
    //
    /// role.edit(|r| r.hoist(true));
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(all(feature = "builder", feature = "cache"))]
    pub fn edit<F: FnOnce(EditRole) -> EditRole>(&self, f: F) -> Result<Role> {
        self.find_guild()
            .and_then(|guild_id| guild_id.edit_role(self.id, f))
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
    pub fn find_guild(&self) -> Result<GuildId> {
        for guild in CACHE.read().guilds.values() {
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
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.mention(), f) }
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
    /// Search the cache for the role.
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.5.8", note = "Use the `to_role_cached`-method instead.")]
    pub fn find(&self) -> Option<Role> {
        self.to_role_cached()
    }

    /// Tries to find the [`Role`] by its Id in the cache.
    ///
    /// [`Role`]: ../guild/struct.Role.html
    #[cfg(feature = "cache")]
    pub fn to_role_cached(self) -> Option<Role> {
        self._to_role_cached(&CACHE)
    }

    #[cfg(feature = "cache")]
    pub(crate) fn _to_role_cached(self, cache: &RwLock<Cache>) -> Option<Role> {
        for guild in cache.read().guilds.values() {
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
impl FromStr for Role {
    type Err = RoleParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match parse_role(s) {
            Some(x) => match RoleId(x).to_role_cached() {
                Some(role) => Ok(role),
                _ => Err(RoleParseError::NotPresentInCache),
            },
            _ => Err(RoleParseError::InvalidRole),
        }
    }
}
