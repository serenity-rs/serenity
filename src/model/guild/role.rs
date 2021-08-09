use std::cmp::Ordering;

#[cfg(feature = "model")]
use serde::de::{Deserialize, Deserializer, Error as DeError};

#[cfg(feature = "model")]
use crate::builder::EditRole;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::cache::FromStrAndCache;
#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::model::misc::RoleParseError;
use crate::model::prelude::*;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_role;

/// Information about a role within a guild. A role represents a set of
/// permissions, and can be attached to one or multiple users. A role has
/// various miscellaneous configurations, such as being assigned a colour. Roles
/// are unique per guild and do not cross over to other guilds in any way, and
/// can have channel-specific permission overrides in addition to guild-level
/// permissions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Role {
    /// The Id of the role. Can be used to calculate the role's creation date.
    pub id: RoleId,
    /// The Id of the Guild the Role is in.
    pub guild_id: GuildId,
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
    /// those in roles with a lower [`Self::position`].
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
    /// [`permissions`]: super::permissions
    pub permissions: Permissions,
    /// The role's position in the position list. Roles are considered higher in
    /// hierarchy if their position is higher.
    ///
    /// The `@everyone` role is usually either `-1` or `0`.
    pub position: i64,
    /// The tags this role has. It can be used to determine if this role is a special role in this guild
    /// such as guild subscriber role, or if the role is linked to an [`Integration`] or a bot.
    ///
    /// [`Integration`]: super::Integration
    #[serde(default)]
    pub tags: RoleTags,
}

#[cfg(feature = "model")]
impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the curent user lacks permission to
    /// delete this role.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn delete(&mut self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_role(self.guild_id.0, self.id.0).await
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not
    /// have permission to Manage Roles.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn edit(
        &self,
        http: impl AsRef<Http>,
        f: impl FnOnce(&mut EditRole) -> &mut EditRole,
    ) -> Result<Role> {
        self.guild_id.edit_role(http, self.id, f).await
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
    #[inline]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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

#[cfg(feature = "model")]
impl RoleId {
    /// Tries to find the [`Role`] by its Id in the cache.
    #[cfg(feature = "cache")]
    pub fn to_role_cached(self, cache: impl AsRef<Cache>) -> Option<Role> {
        for guild_entry in cache.as_ref().guilds.iter() {
            let guild = guild_entry.value();

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
    fn from(role: Role) -> RoleId {
        role.id
    }
}

impl<'a> From<&'a Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: &Role) -> RoleId {
        role.id
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl FromStrAndCache for Role {
    type Err = RoleParseError;

    fn from_str<CRL>(cache: CRL, s: &str) -> StdResult<Self, Self::Err>
    where
        CRL: AsRef<Cache> + Send + Sync,
    {
        match parse_role(s) {
            Some(x) => match RoleId(x).to_role_cached(&cache) {
                Some(role) => Ok(role),
                None => Err(RoleParseError::NotPresentInCache),
            },
            None => Err(RoleParseError::InvalidRole),
        }
    }
}

/// The tags of a [`Role`].
#[derive(Clone, Debug, Default, Serialize)]
#[non_exhaustive]
pub struct RoleTags {
    /// The Id of the bot the [`Role`] belongs to.
    pub bot_id: Option<UserId>,
    /// The Id of the integration the [`Role`] belongs to.
    pub integration_id: Option<IntegrationId>,
    /// Whether this is the guild's premium subscriber role.
    #[serde(default)]
    pub premium_subscriber: bool,
}

impl<'de> Deserialize<'de> for RoleTags {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let bot_id = match map.contains_key("bot_id") {
            true => Some(
                map.remove("bot_id")
                    .ok_or_else(|| DeError::custom("expected bot_id"))
                    .and_then(UserId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let integration_id = match map.contains_key("integration_id") {
            true => Some(
                map.remove("integration_id")
                    .ok_or_else(|| DeError::custom("expected integration_id"))
                    .and_then(IntegrationId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let premium_subscriber = map.contains_key("premium_subscriber");

        Ok(Self {
            bot_id,
            integration_id,
            premium_subscriber,
        })
    }
}
