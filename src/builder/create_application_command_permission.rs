#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::application::CommandPermission;
use crate::model::application::CommandPermissionType;
use crate::model::id::CommandPermissionId;
#[cfg(feature = "http")]
use crate::model::id::{CommandId, GuildId};

/// A builder for creating several [`CommandPermissionData`].
///
/// [`CommandPermissionData`]: crate::model::application::CommandPermissionData
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateCommandPermissionsData {
    permissions: Vec<CreateCommandPermissionData>,
}

impl CreateCommandPermissionsData {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
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
        http: impl AsRef<Http>,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<CommandPermission> {
        http.as_ref().edit_guild_application_command_permissions(guild_id, command_id, &self).await
    }

    /// Adds a permission for the application command.
    pub fn add_permission(mut self, permission: CreateCommandPermissionData) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Sets permissions for the application command.
    pub fn set_permissions(mut self, permissions: Vec<CreateCommandPermissionData>) -> Self {
        self.permissions = permissions;
        self
    }
}

/// A builder for creating an [`CommandPermissionData`].
///
/// All fields are required.
///
/// [`CommandPermissionData`]: crate::model::application::CommandPermissionData
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateCommandPermissionData {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<CommandPermissionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<CommandPermissionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission: Option<bool>,
}

impl CreateCommandPermissionData {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `CommandPermissionType` for the [`CommandPermissionData`].
    ///
    /// [`CommandPermissionData`]: crate::model::application::CommandPermissionData
    pub fn kind(mut self, kind: CommandPermissionType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the CommandPermissionId for the [`CommandPermissionData`].
    ///
    /// [`CommandPermissionData`]: crate::model::application::CommandPermissionData
    pub fn id(mut self, id: CommandPermissionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the permission for the [`CommandPermissionData`].
    ///
    /// **Note**: Passing `false` will only grey-out the application command in the list, and will
    /// not fully hide it from the user.
    ///
    /// [`CommandPermissionData`]: crate::model::application::CommandPermissionData
    pub fn permission(mut self, permission: bool) -> Self {
        self.permission = Some(permission);
        self
    }
}
