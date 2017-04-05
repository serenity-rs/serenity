use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use ::model::*;

#[cfg(feature="cache")]
use ::client::{CACHE, rest};
#[cfg(feature="cache")]
use ::internal::prelude::*;
#[cfg(feature="cache")]
use ::utils::builder::EditRole;

impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    #[inline]
    pub fn delete(&self) -> Result<()> {
        rest::delete_role(self.find_guild()?.0, self.id.0)
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
    /// // assuming a `guild` and `role_id` have been bound
    //
    /// guild.edit_role(role_id, |r| r.hoist(true));
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn edit_role<F: FnOnce(EditRole) -> EditRole>(&self, f: F) -> Result<Role> {
        match self.find_guild() {
            Ok(guild_id) => guild_id.edit_role(self.id, f),
            Err(why) => Err(why),
        }
    }

    /// Searches the cache for the guild that owns the role.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if a guild is not in the cache
    /// that contains the role.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    #[cfg(feature="cache")]
    pub fn find_guild(&self) -> Result<GuildId> {
        for guild in CACHE.read().unwrap().guilds.values() {
            let guild = guild.read().unwrap();

            if guild.roles.contains_key(&RoleId(self.id.0)) {
                return Ok(guild.id);
            }
        }

        Err(Error::Client(ClientError::GuildNotFound))
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
    pub fn has_permissions(&self, permissions: Permissions, precise: bool)
        -> bool {
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
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.mention(), f)
    }
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
    fn eq(&self, other: &Role) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Role) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RoleId {
    /// Search the cache for the role.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Role> {
        let cache = CACHE.read().unwrap();

        for guild in cache.guilds.values() {
            let guild = guild.read().unwrap();

            if !guild.roles.contains_key(self) {
                continue;
            }

            if let Some(role) = guild.roles.get(self) {
                return Some(role.clone());
            }
        }

        None
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl From<Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: Role) -> RoleId {
        role.id
    }
}
