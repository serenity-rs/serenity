#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::application::command::CommandPermission;
use crate::model::application::command::CommandPermissionType;
use crate::model::prelude::*;

/// A builder for creating several [`CommandPermission`].
#[derive(Clone, Debug, Default, Serialize)]
#[deprecated(note = "use `CreateApplicationCommandPermissionsData`")]
#[must_use]
pub struct CreateApplicationCommandsPermissions(Vec<CreateApplicationCommandPermissions>);

impl CreateApplicationCommandsPermissions {
    /// Overwrite permissions for all application commands in the guild.
    ///
    /// **Note**: Per [Discord's docs], this endpoint has been disabled and will always return an
    /// error. Use [`CreateApplicationCommandPermissionsData`] instead to update permissions one
    /// command at a time.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Discord's docs]: https://discord.com/developers/docs/interactions/application-commands#batch-edit-application-command-permissions
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        guild_id: GuildId,
    ) -> Result<Vec<CommandPermission>> {
        http.as_ref().edit_guild_application_commands_permissions(guild_id.into(), &self).await
    }

    /// Adds a new application command.
    pub fn add_application_command(
        mut self,
        application_command: CreateApplicationCommandPermissions,
    ) -> Self {
        self.0.push(application_command);
        self
    }

    /// Sets all the application commands.
    pub fn set_application_commands(
        mut self,
        application_commands: Vec<CreateApplicationCommandPermissions>,
    ) -> Self {
        self.0 = application_commands;
        self
    }
}

/// A builder for creating an [`CommandPermission`].
///
/// [`CommandPermission`]: crate::model::application::command::CommandPermission
#[derive(Clone, Debug, Default, Serialize)]
#[deprecated(note = "use `CreateApplicationCommandPermissionsData`")]
#[must_use]
pub struct CreateApplicationCommandPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<CommandId>,
    permissions: Vec<CreateApplicationCommandPermissionData>,
}

impl CreateApplicationCommandPermissions {
    /// The [`CommandId`] these permissions belong to.
    ///
    /// [`CommandId`]: crate::model::id::CommandId
    pub fn id(mut self, application_command_id: impl Into<CommandId>) -> Self {
        self.id = Some(application_command_id.into());
        self
    }

    /// Adds permission for the application command.
    pub fn add_permissions(mut self, permission: CreateApplicationCommandPermissionData) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Sets permissions for the application command.
    pub fn set_permissions(
        mut self,
        permissions: Vec<CreateApplicationCommandPermissionData>,
    ) -> Self {
        self.permissions = permissions;
        self
    }
}

/// A builder for creating several [`CommandPermissionData`].
///
/// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateApplicationCommandPermissionsData {
    permissions: Vec<CreateApplicationCommandPermissionData>,
}

impl CreateApplicationCommandPermissionsData {
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
        http.as_ref()
            .edit_guild_application_command_permissions(guild_id.into(), command_id.into(), &self)
            .await
    }

    /// Adds a permission for the application command.
    pub fn add_permission(mut self, permission: CreateApplicationCommandPermissionData) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Sets permissions for the application command.
    pub fn set_permissions(
        mut self,
        permissions: Vec<CreateApplicationCommandPermissionData>,
    ) -> Self {
        self.permissions = permissions;
        self
    }
}

/// A builder for creating an [`CommandPermissionData`].
///
/// All fields are required.
///
/// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateApplicationCommandPermissionData {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<CommandPermissionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission: Option<bool>,
}

impl CreateApplicationCommandPermissionData {
    /// Sets the `CommandPermissionType` for the [`CommandPermissionData`].
    ///
    /// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
    pub fn kind(mut self, kind: CommandPermissionType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the CommandPermissionId for the [`CommandPermissionData`].
    ///
    /// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Sets the permission for the [`CommandPermissionData`].
    ///
    /// **Note**: Passing `false` will only grey-out the application command in the list, and will
    /// not fully hide it from the user.
    ///
    /// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
    pub fn permission(mut self, permission: bool) -> Self {
        self.permission = Some(permission);
        self
    }
}
