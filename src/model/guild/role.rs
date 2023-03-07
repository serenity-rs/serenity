use std::cmp::Ordering;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt;

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
use crate::model::prelude::*;
use crate::model::utils::is_false;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::parse_role;

/// Information about a role within a guild. A role represents a set of
/// permissions, and can be attached to one or multiple users. A role has
/// various miscellaneous configurations, such as being assigned a colour. Roles
/// are unique per guild and do not cross over to other guilds in any way, and
/// can have channel-specific permission overrides in addition to guild-level
/// permissions.
///
/// [Discord docs](https://discord.com/developers/docs/topics/permissions#role-object).
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
    /// Role icon image hash.
    ///
    /// `role-icons/<role_id>/<hash>.png` - PNG, JPEG, WEBP
    /// `role-icons/<role_id>/a_<hash>.gif` - GIF, Animated WEBP
    pub icon: Option<String>,
    /// Role unicoded image.
    pub unicode_emoji: Option<String>,
}

/// Helper for deserialization without a `GuildId` but then later updated to the correct `GuildId`.
///
/// The only difference to `Role` is `#[serde(default)]` on `guild_id`.
#[derive(Deserialize)]
pub(crate) struct InterimRole {
    pub id: RoleId,
    #[serde(default)]
    pub guild_id: GuildId,
    #[cfg(feature = "utils")]
    #[serde(rename = "color")]
    pub colour: Colour,
    #[cfg(not(feature = "utils"))]
    #[serde(rename = "color")]
    pub colour: u32,
    pub hoist: bool,
    pub managed: bool,
    #[serde(default)]
    pub mentionable: bool,
    pub name: String,
    pub permissions: Permissions,
    pub position: i64,
    #[serde(default)]
    pub tags: RoleTags,
}

impl From<InterimRole> for Role {
    fn from(r: InterimRole) -> Self {
        Self {
            id: r.id,
            guild_id: r.guild_id,
            colour: r.colour,
            hoist: r.hoist,
            managed: r.managed,
            mentionable: r.mentionable,
            name: r.name,
            permissions: r.permissions,
            position: r.position,
            tags: r.tags,
            icon: None,
            unicode_emoji: None,
        }
    }
}

#[cfg(feature = "model")]
impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
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
    /// role.edit(|r| r.hoist(true));
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
    #[must_use]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    /// Checks whether the role has all of the given permissions.
    ///
    /// The 'precise' argument is used to check if the role's permissions are
    /// precisely equivalent to the given permissions. If you need only check
    /// that the role has at least the given permissions, pass `false`.
    #[inline]
    #[must_use]
    pub fn has_permissions(&self, permissions: Permissions, precise: bool) -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }
}

impl fmt::Display for Role {
    /// Format a mention for the role, pinging its members.
    // This is in the format of: `<@&ROLE_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
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
#[derive(Debug)]
pub enum RoleParseError {
    NotPresentInCache,
    InvalidRole,
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl fmt::Display for RoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPresentInCache => f.write_str("not present in cache"),
            Self::InvalidRole => f.write_str("invalid role"),
        }
    }
}

#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
impl StdError for RoleParseError {}

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
///
/// [Discord docs](https://discord.com/developers/docs/topics/permissions#role-object-role-tags-structure).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[non_exhaustive]
pub struct RoleTags {
    /// The Id of the bot the [`Role`] belongs to.
    pub bot_id: Option<UserId>,
    /// The Id of the integration the [`Role`] belongs to.
    pub integration_id: Option<IntegrationId>,
    /// Whether this is the guild's premium subscriber role.
    #[serde(default, skip_serializing_if = "is_false", with = "premium_subscriber")]
    pub premium_subscriber: bool,
}

/// A premium subscriber role is reported with the field present and the value `null`.
mod premium_subscriber {
    use std::fmt;

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        deserializer.deserialize_option(NullValueVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(_: &bool, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }

    struct NullValueVisitor;

    impl<'de> Visitor<'de> for NullValueVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("null value")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(true)
        }

        /// Called by the `simd_json` crate
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::RoleTags;

    #[test]
    fn premium_subscriber_role_serde() {
        let value = RoleTags {
            bot_id: None,
            integration_id: None,
            premium_subscriber: true,
        };

        assert_tokens(&value, &[
            Token::Struct {
                name: "RoleTags",
                len: 3,
            },
            Token::Str("bot_id"),
            Token::None,
            Token::Str("integration_id"),
            Token::None,
            Token::Str("premium_subscriber"),
            Token::None,
            Token::StructEnd,
        ]);
    }

    #[test]
    fn non_premium_subscriber_role_serde() {
        let value = RoleTags {
            bot_id: None,
            integration_id: None,
            premium_subscriber: false,
        };

        assert_tokens(&value, &[
            Token::Struct {
                name: "RoleTags",
                len: 2,
            },
            Token::Str("bot_id"),
            Token::None,
            Token::Str("integration_id"),
            Token::None,
            Token::StructEnd,
        ]);
    }
}
