use crate::model::application::command::CommandPermissionType;
use crate::model::id::CommandId;

/// A builder for creating several [`CommandPermission`].
///
/// [`CommandPermission`]: crate::model::application::command::CommandPermission
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateApplicationCommandsPermissions(Vec<CreateApplicationCommandPermissions>);

impl CreateApplicationCommandsPermissions {
    /// Creates a new application command.
    pub fn create_application_command<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(
            &mut CreateApplicationCommandPermissions,
        ) -> &mut CreateApplicationCommandPermissions,
    {
        let mut data = CreateApplicationCommandPermissions::default();
        f(&mut data);

        self.add_application_command(data);

        self
    }

    /// Adds a new application command.
    pub fn add_application_command(
        &mut self,
        application_command: CreateApplicationCommandPermissions,
    ) -> &mut Self {
        self.0.push(application_command);

        self
    }

    /// Sets all the application commands.
    pub fn set_application_commands(
        &mut self,
        application_commands: Vec<CreateApplicationCommandPermissions>,
    ) -> &mut Self {
        self.0 = application_commands;

        self
    }
}

/// A builder for creating an [`CommandPermission`].
///
/// [`CommandPermission`]: crate::model::application::command::CommandPermission
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateApplicationCommandPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<CommandId>,
    permissions: Vec<CreateApplicationCommandPermissionData>,
}

impl CreateApplicationCommandPermissions {
    /// The [`CommandId`] these permissions belong to.
    ///
    /// [`CommandId`]: crate::model::id::CommandId
    pub fn id(&mut self, application_command_id: impl Into<CommandId>) -> &mut Self {
        self.id = Some(application_command_id.into());
        self
    }

    /// Creates permissions for the application command.
    pub fn create_permissions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(
            &mut CreateApplicationCommandPermissionData,
        ) -> &mut CreateApplicationCommandPermissionData,
    {
        let mut data = CreateApplicationCommandPermissionData::default();
        f(&mut data);

        self.add_permissions(data);

        self
    }

    /// Adds permission for the application command.
    pub fn add_permissions(
        &mut self,
        permission: CreateApplicationCommandPermissionData,
    ) -> &mut Self {
        self.permissions.push(permission);
        self
    }

    /// Sets permissions for the application command.
    pub fn set_permissions(
        &mut self,
        permissions: Vec<CreateApplicationCommandPermissionData>,
    ) -> &mut Self {
        self.permissions = permissions;
        self
    }
}

/// A builder for creating several [`CommandPermissionData`].
///
/// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateApplicationCommandPermissionsData {
    permissions: Vec<CreateApplicationCommandPermissionData>,
}

impl CreateApplicationCommandPermissionsData {
    /// Creates a permission for the application command.
    pub fn create_permission<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(
            &mut CreateApplicationCommandPermissionData,
        ) -> &mut CreateApplicationCommandPermissionData,
    {
        let mut data = CreateApplicationCommandPermissionData::default();
        f(&mut data);

        self.add_permission(data);

        self
    }

    /// Adds a permission for the application command.
    pub fn add_permission(
        &mut self,
        permission: CreateApplicationCommandPermissionData,
    ) -> &mut Self {
        self.permissions.push(permission);
        self
    }

    /// Sets permissions for the application command.
    pub fn set_permissions(
        &mut self,
        permissions: Vec<CreateApplicationCommandPermissionData>,
    ) -> &mut Self {
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
    pub fn kind(&mut self, kind: CommandPermissionType) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the CommandPermissionId for the [`CommandPermissionData`].
    ///
    /// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.id = Some(id.to_string());
        self
    }

    /// Sets the permission for the [`CommandPermissionData`].
    ///
    /// **Note**: Setting it to `false` will only grey the application command in the
    /// list, it will not fully hide it to the user.
    ///
    /// [`CommandPermissionData`]: crate::model::application::command::CommandPermissionData
    pub fn permission(&mut self, permission: bool) -> &mut Self {
        self.permission = Some(permission);
        self
    }
}
