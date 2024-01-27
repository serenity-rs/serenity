use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for creating several [`CommandPermission`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#edit-application-command-permissions).
// Cannot be replaced by a simple Vec<CreateCommandPermission> because we need the schema with
// the `permissions` field, and also to be forward compatible if a new field beyond just
// `permissions` is added to the HTTP endpoint
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditCommandPermissions<'a> {
    permissions: Cow<'a, [CreateCommandPermission]>,
}

impl<'a> EditCommandPermissions<'a> {
    pub fn new(permissions: impl Into<Cow<'a, [CreateCommandPermission]>>) -> Self {
        Self {
            permissions: permissions.into(),
        }
    }

    /// Create permissions for a guild application command. These will overwrite any existing
    /// permissions for that command.
    ///
    /// **Note**: The permissions will update instantly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given. See [Discord's docs] for more details.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Discord's docs]: https://discord.com/developers/docs/interactions/slash-commands
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: &Http,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<CommandPermissions> {
        http.edit_guild_command_permissions(guild_id, command_id, &self).await
    }
}

/// A builder for creating an [`CommandPermission`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-application-command-permissions-structure).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateCommandPermission(CommandPermission);

impl CreateCommandPermission {
    /// Creates a permission overwrite for a specific role
    pub fn role(id: RoleId, allow: bool) -> Self {
        Self(CommandPermission {
            id: id.into(),
            kind: CommandPermissionType::Role,
            permission: allow,
        })
    }

    /// Creates a permission overwrite for a specific user
    pub fn user(id: UserId, allow: bool) -> Self {
        Self(CommandPermission {
            id: id.into(),
            kind: CommandPermissionType::User,
            permission: allow,
        })
    }

    /// Creates a permission overwrite for a specific channel
    pub fn channel(id: ChannelId, allow: bool) -> Self {
        Self(CommandPermission {
            id: id.get().into(),
            kind: CommandPermissionType::Channel,
            permission: allow,
        })
    }

    /// Creates a permission overwrite for a everyone in a guild
    pub fn everyone(guild_id: GuildId, allow: bool) -> Self {
        Self(CommandPermission {
            id: guild_id.get().into(),
            kind: CommandPermissionType::User,
            permission: allow,
        })
    }

    /// Creates a permission overwrite for all channels in a guild
    pub fn all_channels(guild_id: GuildId, allow: bool) -> Self {
        Self(CommandPermission {
            id: CommandPermissionId::new(guild_id.get() - 1),
            kind: CommandPermissionType::Channel,
            permission: allow,
        })
    }
}
