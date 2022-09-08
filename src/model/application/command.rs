use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "http")]
use crate::builder::{CreateApplicationCommand, CreateApplicationCommands};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::json::Value;
#[cfg(feature = "http")]
use crate::json::{self, JsonMap};
use crate::model::channel::ChannelType;
use crate::model::id::{
    ApplicationId,
    CommandId,
    CommandPermissionId,
    CommandVersionId,
    GuildId,
    RoleId,
    UserId,
};
use crate::model::Permissions;

/// The base command model that belongs to an application.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Command {
    /// The command Id.
    pub id: CommandId,
    /// The application command kind.
    #[serde(rename = "type")]
    pub kind: CommandType,
    /// The parent application Id.
    pub application_id: ApplicationId,
    /// The command guild Id, if it is a guild command.
    ///
    /// **Note**: It may only be present if it is received through the gateway.
    pub guild_id: Option<GuildId>,
    /// The command name.
    pub name: String,
    /// The localized command name of the selected locale.
    ///
    /// If the name is localized, either this field or [`Self::name_localizations`]
    /// is set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub name_localized: Option<String>,
    /// All localized command names.
    ///
    /// If the name is localized, either this field or [`Self::name_localized`]
    /// is set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub name_localizations: Option<HashMap<String, String>>,
    /// The command description.
    pub description: String,
    /// The localized command description of the selected locale.
    ///
    /// If the description is localized, either this field or [`Self::description_localizations`]
    /// is set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub description_localized: Option<String>,
    /// All localized command descriptions.
    ///
    /// If the description is localized, either this field or [`Self::description_localized`]
    /// is set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub description_localizations: Option<HashMap<String, String>>,
    /// The parameters for the command.
    #[serde(default)]
    pub options: Vec<CommandOption>,
    /// The default permissions required to execute the command.
    pub default_member_permissions: Option<Permissions>,
    /// Indicates whether the command is available in DMs with the app, only for globally-scoped commands.
    /// By default, commands are visible.
    #[serde(default)]
    pub dm_permission: Option<bool>,
    /// Whether the command is enabled by default when
    /// the application is added to a guild.
    #[serde(default = "default_permission_value")]
    #[deprecated(note = "replaced by `default_member_permissions`")]
    pub default_permission: bool,
    /// An autoincremented version identifier updated during substantial record changes.
    pub version: CommandVersionId,
}

fn default_permission_value() -> bool {
    true
}

#[cfg(feature = "http")]
impl Command {
    /// Creates a global [`Command`],
    /// overriding an existing one with the same name if it exists.
    ///
    /// When a created [`Command`] is used, the [`InteractionCreate`] event will be emitted.
    ///
    /// **Note**: Global commands may take up to an hour to be updated in the user slash commands
    /// list. If an outdated command data is sent by a user, discord will consider it as an error
    /// and then will instantly update that command.
    ///
    /// As such, it is recommended that guild application commands be used for testing purposes.
    ///
    /// # Examples
    ///
    /// Create a simple ping command:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::new("token"));
    /// use serenity::model::application::command::Command;
    /// use serenity::model::id::ApplicationId;
    ///
    /// let _ = Command::create_global_application_command(&http, |command| {
    ///     command.name("ping").description("A simple ping command")
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// Create a command that echoes what is inserted:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::new("token"));
    /// use serenity::model::application::command::{Command, CommandOptionType};
    /// use serenity::model::id::ApplicationId;
    ///
    /// let _ = Command::create_global_application_command(&http, |command| {
    ///     command.name("echo").description("Makes the bot send a message").create_option(|option| {
    ///         option
    ///             .name("message")
    ///             .description("The message to send")
    ///             .kind(CommandOptionType::String)
    ///             .required(true)
    ///     })
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the [`Command`] is illformed,
    /// such as if more than 10 [`choices`] are set. See the [API Docs] for further details.
    ///
    /// Can also return an [`Error::Json`] if there is an error in deserializing
    /// the response.
    ///
    /// [`InteractionCreate`]: crate::client::EventHandler::interaction_create
    /// [API Docs]: https://discord.com/developers/docs/interactions/slash-commands
    /// [`choices`]: CommandOption::choices
    pub async fn create_global_application_command<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Command>
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let map = Command::build_application_command(f);
        http.as_ref().create_global_application_command(&Value::from(map)).await
    }

    /// Overrides all global application commands.
    ///
    /// [`create_global_application_command`]: Self::create_global_application_command
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn set_global_application_commands<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Vec<Command>>
    where
        F: FnOnce(&mut CreateApplicationCommands) -> &mut CreateApplicationCommands,
    {
        let mut array = CreateApplicationCommands::default();

        f(&mut array);

        http.as_ref().create_global_application_commands(&Value::from(array.0)).await
    }

    /// Edits a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn edit_global_application_command<F>(
        http: impl AsRef<Http>,
        command_id: CommandId,
        f: F,
    ) -> Result<Command>
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let map = Command::build_application_command(f);
        http.as_ref().edit_global_application_command(command_id.into(), &Value::from(map)).await
    }

    /// Gets all global commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_global_application_commands(http: impl AsRef<Http>) -> Result<Vec<Command>> {
        http.as_ref().get_global_application_commands().await
    }

    /// Gets a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<Command> {
        http.as_ref().get_global_application_command(command_id.into()).await
    }

    /// Deletes a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<()> {
        http.as_ref().delete_global_application_command(command_id.into()).await
    }
}

#[cfg(feature = "http")]
impl Command {
    #[inline]
    pub(crate) fn build_application_command<F>(f: F) -> JsonMap
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let mut create_application_command = CreateApplicationCommand::default();
        f(&mut create_application_command);
        json::hashmap_to_json_map(create_application_command.0)
    }
}

/// The type of an application command.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-types).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum CommandType {
    ChatInput = 1,
    User = 2,
    Message = 3,
    Unknown = !0,
}

enum_number!(CommandType {
    ChatInput,
    User,
    Message
});

/// The parameters for an [`Command`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandOption {
    /// The option type.
    #[serde(rename = "type")]
    pub kind: CommandOptionType,
    /// The option name.
    pub name: String,
    /// Localizations of the option name with locale as the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<std::collections::HashMap<String, String>>,
    /// The option description.
    pub description: String,
    /// Localizations of the option description with locale as the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_localizations: Option<std::collections::HashMap<String, String>>,
    /// Whether the parameter is optional or required.
    #[serde(default)]
    pub required: bool,
    /// Choices the user can pick from.
    ///
    /// **Note**: Only available for [`String`] and [`Integer`] [`CommandOptionType`].
    ///
    /// [`String`]: CommandOptionType::String
    /// [`Integer`]: CommandOptionType::Integer
    #[serde(default)]
    pub choices: Vec<CommandOptionChoice>,
    /// The nested options.
    ///
    /// **Note**: Only available for [`SubCommand`] or [`SubCommandGroup`].
    ///
    /// [`SubCommand`]: CommandOptionType::SubCommand
    /// [`SubCommandGroup`]: CommandOptionType::SubCommandGroup
    #[serde(default)]
    pub options: Vec<CommandOption>,
    /// If the option is a [`Channel`], it will only be able to show these types.
    ///
    /// [`Channel`]: CommandOptionType::Channel
    #[serde(default)]
    pub channel_types: Vec<ChannelType>,
    /// Minimum permitted value for Integer or Number options
    #[serde(default)]
    pub min_value: Option<serde_json::Number>,
    /// Maximum permitted value for Integer or Number options
    #[serde(default)]
    pub max_value: Option<serde_json::Number>,
    /// Minimum permitted length for String options
    #[serde(default)]
    pub min_length: Option<u16>,
    /// Maximum permitted length for String options
    #[serde(default)]
    pub max_length: Option<u16>,
    #[serde(default)]
    pub autocomplete: bool,
}

/// The type of an [`CommandOption`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum CommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
    Mentionable = 9,
    Number = 10,
    Attachment = 11,
    Unknown = !0,
}

enum_number!(CommandOptionType {
    SubCommand,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
    Attachment
});

/// The only valid values a user can pick in an [`CommandOption`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandOptionChoice {
    /// The choice name.
    pub name: String,
    /// Localizations of the choice name, with locale as key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<std::collections::HashMap<String, String>>,
    /// The choice value.
    pub value: Value,
}

/// An [`Command`] permission.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-guild-application-command-permissions-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandPermission {
    /// The id of the command.
    pub id: CommandId,
    /// The id of the application the command belongs to.
    pub application_id: ApplicationId,
    /// The id of the guild.
    pub guild_id: GuildId,
    /// The permissions for the command in the guild.
    pub permissions: Vec<CommandPermissionData>,
}

/// The [`CommandPermission`] data.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-application-command-permissions-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandPermissionData {
    /// The [`RoleId`] or [`UserId`], depends on `kind` value.
    pub id: CommandPermissionId,
    /// The type of data this permissions applies to.
    #[serde(rename = "type")]
    pub kind: CommandPermissionType,
    /// Whether or not the provided data can use the command or not.
    pub permission: bool,
}

/// The type of an [`CommandPermissionData`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-application-command-permission-type).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum CommandPermissionType {
    Role = 1,
    User = 2,
    Channel = 3,
    Unknown = !0,
}

enum_number!(CommandPermissionType {
    Role,
    User,
    Channel
});

impl CommandPermissionId {
    /// Converts this [`CommandPermissionId`] to [`UserId`].
    #[must_use]
    pub fn to_user_id(self) -> UserId {
        self.0.into()
    }

    /// Converts this [`CommandPermissionId`] to [`RoleId`].
    #[must_use]
    pub fn to_role_id(self) -> RoleId {
        self.0.into()
    }
}

impl From<RoleId> for CommandPermissionId {
    fn from(id: RoleId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a RoleId> for CommandPermissionId {
    fn from(id: &RoleId) -> Self {
        Self(id.0)
    }
}

impl From<UserId> for CommandPermissionId {
    fn from(id: UserId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a UserId> for CommandPermissionId {
    fn from(id: &UserId) -> Self {
        Self(id.0)
    }
}

impl From<CommandPermissionId> for RoleId {
    fn from(id: CommandPermissionId) -> Self {
        Self(id.0)
    }
}

impl From<CommandPermissionId> for UserId {
    fn from(id: CommandPermissionId) -> Self {
        Self(id.0)
    }
}
