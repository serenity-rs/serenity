use std::collections::HashMap;

use serde::Serialize;

use super::{InstallationContext, InteractionContext};
#[cfg(feature = "model")]
use crate::builder::{CreateCommand, EditCommand};
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;

/// The base command model that belongs to an application.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
    pub name: FixedString<u8>,
    /// The localized command name of the selected locale.
    ///
    /// If the name is localized, either this field or [`Self::name_localizations`] is set,
    /// depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub name_localized: Option<FixedString<u8>>,
    /// All localized command names.
    ///
    /// If the name is localized, either this field or [`Self::name_localized`] is set, depending
    /// on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub name_localizations: Option<HashMap<String, String>>,
    /// The command description.
    pub description: FixedString<u16>,
    /// The localized command description of the selected locale.
    ///
    /// If the description is localized, either this field or [`Self::description_localizations`]
    /// is set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub description_localized: Option<FixedString<u16>>,
    /// All localized command descriptions.
    ///
    /// If the description is localized, either this field or [`Self::description_localized`] is
    /// set, depending on which endpoint this data was retrieved from
    /// ([source](https://discord.com/developers/docs/interactions/application-commands#retrieving-localized-commands)).
    pub description_localizations: Option<HashMap<String, String>>,
    /// The parameters for the command.
    #[serde(default)]
    pub options: FixedArray<CommandOption>,
    /// The default permissions required to execute the command.
    pub default_member_permissions: Option<Permissions>,
    /// Indicates whether the command is available in DMs with the app, only for globally-scoped
    /// commands. By default, commands are visible.
    #[serde(default)]
    #[cfg(not(feature = "unstable"))]
    pub dm_permission: Option<bool>,
    /// Indicates whether the command is [age-restricted](https://discord.com/developers/docs/interactions/application-commands#agerestricted-commands),
    /// defaults to false.
    #[serde(default)]
    pub nsfw: bool,
    /// Installation context(s) where the command is available, only for globally-scoped commands.
    ///
    /// Defaults to [`InstallationContext::Guild`] and [`InstallationContext::User`].
    #[serde(default)]
    pub integration_types: Vec<InstallationContext>,
    /// Interaction context(s) where the command can be used, only for globally-scoped commands.
    ///
    /// By default, all interaction context types are included.
    pub contexts: Option<Vec<InteractionContext>>,
    /// An autoincremented version identifier updated during substantial record changes.
    pub version: CommandVersionId,
    /// Only present for commands of type [`PrimaryEntryPoint`].
    ///
    /// [`PrimaryEntryPoint`]: CommandType::PrimaryEntryPoint
    pub handler: Option<EntryPointHandlerType>,
}

#[cfg(feature = "model")]
impl Command {
    /// Create a global [`Command`], overriding an existing one with the same name if it exists.
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
    /// # let http: Arc<Http> = unimplemented!();
    /// use serenity::builder::CreateCommand;
    /// use serenity::model::application::Command;
    /// use serenity::model::id::ApplicationId;
    ///
    /// let builder = CreateCommand::new("ping").description("A simple ping command");
    /// let _ = Command::create_global_command(&http, builder).await;
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
    /// # let http: Arc<Http> = unimplemented!();
    /// use serenity::builder::{CreateCommand, CreateCommandOption as CreateOption};
    /// use serenity::model::application::{Command, CommandOptionType};
    /// use serenity::model::id::ApplicationId;
    ///
    /// let builder =
    ///     CreateCommand::new("echo").description("Makes the bot send a message").add_option(
    ///         CreateOption::new(CommandOptionType::String, "message", "The message to send")
    ///             .required(true),
    ///     );
    /// let _ = Command::create_global_command(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`InteractionCreate`]: crate::gateway::client::EventHandler::interaction_create
    pub async fn create_global_command(http: &Http, builder: CreateCommand<'_>) -> Result<Command> {
        builder.execute(http, None).await
    }

    /// Override all global application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_global_command`].
    pub async fn set_global_commands(
        http: &Http,
        commands: &[CreateCommand<'_>],
    ) -> Result<Vec<Command>> {
        http.create_global_commands(&commands).await
    }

    /// Edit a global command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    pub async fn edit_global_command(
        http: &Http,
        command_id: CommandId,
        builder: EditCommand<'_>,
    ) -> Result<Command> {
        builder.execute(http, command_id, None).await
    }

    /// Gets all global commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_global_commands(http: &Http) -> Result<Vec<Command>> {
        http.get_global_commands().await
    }

    /// Gets all global commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_global_commands_with_localizations(http: &Http) -> Result<Vec<Command>> {
        http.get_global_commands_with_localizations().await
    }

    /// Gets a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn get_global_command(http: &Http, command_id: CommandId) -> Result<Command> {
        http.get_global_command(command_id).await
    }

    /// Deletes a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn delete_global_command(http: &Http, command_id: CommandId) -> Result<()> {
        http.delete_global_command(command_id).await
    }
}

enum_number! {
    /// The type of an application command.
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum CommandType {
        ChatInput = 1,
        User = 2,
        Message = 3,
        PrimaryEntryPoint = 4,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Signifies how the invocation of a command of type [`PrimaryEntryPoint`] should be handled.
    ///
    /// [`PrimaryEntryPoint`]: CommandType::PrimaryEntryPoint
    /// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-entry-point-command-handler-types)
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum EntryPointHandlerType {
        AppHandler = 1,
        DiscordLaunchActivity = 2,
        _ => Unknown(u8),
    }
}

/// The parameters for an [`Command`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct CommandOption {
    /// The option type.
    #[serde(rename = "type")]
    pub kind: CommandOptionType,
    /// The option name.
    pub name: FixedString<u8>,
    /// Localizations of the option name with locale as the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<HashMap<FixedString<u8>, FixedString<u8>>>,
    /// The option description.
    pub description: FixedString<u16>,
    /// Localizations of the option description with locale as the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_localizations: Option<HashMap<FixedString<u16>, FixedString<u16>>>,
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
    pub choices: FixedArray<CommandOptionChoice, u8>,
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
    pub channel_types: FixedArray<ChannelType>,
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

enum_number! {
    /// The type of an [`CommandOption`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
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
        _ => Unknown(u8),
    }
}

/// The only valid values a user can pick in an [`CommandOption`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandOptionChoice {
    /// The choice name.
    pub name: FixedString,
    /// Localizations of the choice name, with locale as key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<HashMap<String, String>>,
    /// The choice value.
    pub value: Value,
}

/// An [`Command`] permission.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-guild-application-command-permissions-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandPermissions {
    /// The id of the command.
    pub id: CommandId,
    /// The id of the application the command belongs to.
    pub application_id: ApplicationId,
    /// The id of the guild.
    pub guild_id: GuildId,
    /// The permissions for the command in the guild.
    pub permissions: FixedArray<CommandPermission>,
}

/// The [`CommandPermission`] data.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-application-command-permissions-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandPermission {
    /// The [`RoleId`] or [`UserId`], depends on `kind` value.
    pub id: CommandPermissionId,
    /// The type of data this permissions applies to.
    #[serde(rename = "type")]
    pub kind: CommandPermissionType,
    /// Whether or not the provided data can use the command or not.
    pub permission: bool,
}

enum_number! {
    /// The type of a [`CommandPermission`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-permissions-object-application-command-permission-type).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum CommandPermissionType {
        Role = 1,
        User = 2,
        Channel = 3,
        _ => Unknown(u8),
    }
}

impl CommandPermissionId {
    /// Converts this [`CommandPermissionId`] to [`UserId`].
    #[must_use]
    pub fn to_user_id(self) -> UserId {
        self.into()
    }

    /// Converts this [`CommandPermissionId`] to [`RoleId`].
    #[must_use]
    pub fn to_role_id(self) -> RoleId {
        self.into()
    }
}

impl From<RoleId> for CommandPermissionId {
    fn from(id: RoleId) -> Self {
        Self::new(id.get())
    }
}

impl From<UserId> for CommandPermissionId {
    fn from(id: UserId) -> Self {
        Self::new(id.get())
    }
}

impl From<CommandPermissionId> for RoleId {
    fn from(id: CommandPermissionId) -> Self {
        Self::new(id.get())
    }
}

impl From<CommandPermissionId> for UserId {
    fn from(id: CommandPermissionId) -> Self {
        Self::new(id.get())
    }
}
