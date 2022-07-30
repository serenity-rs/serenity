use std::collections::HashMap;

#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::application::command::Command;
use crate::model::application::command::{CommandOptionType, CommandType};
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct CommandOptionChoice {
    name: String,
    value: Value,
    name_localizations: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum Number {
    Float(f64),
    Integer(u64),
}

/// A builder for creating a new [`CommandOption`].
///
/// [`Self::kind`], [`Self::name`], and [`Self::description`] are required fields.
///
/// [`CommandOption`]: crate::model::application::command::CommandOption
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateApplicationCommandOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<CommandOptionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    name_localizations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    description_localizations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    autocomplete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_value: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_value: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_length: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_length: Option<u16>,

    channel_types: Vec<ChannelType>,
    choices: Vec<CommandOptionChoice>,
    options: Vec<CreateApplicationCommandOption>,
}

impl CreateApplicationCommandOption {
    /// Sets the `CommandOptionType`.
    pub fn kind(mut self, kind: CommandOptionType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the name of the option.
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Specifies a localized name of the option.
    ///
    /// ```rust
    /// # serenity::builder::CreateApplicationCommandOption::default()
    /// .name("age")
    /// .name_localized("zh-CN", "岁数")
    /// # ;
    /// ```
    pub fn name_localized(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_localizations.insert(locale.into(), name.into());
        self
    }

    /// Sets the description for the option.
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specifies a localized description of the option.
    ///
    /// ```rust
    /// # serenity::builder::CreateApplicationCommandOption::default()
    /// .description("Wish a friend a happy birthday")
    /// .description_localized("zh-CN", "祝你朋友生日快乐")
    /// # ;
    /// ```
    pub fn description_localized(
        mut self,
        locale: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.description_localizations.insert(locale.into(), description.into());
        self
    }

    /// The first required option for the user to complete.
    ///
    /// **Note**: Only one option can be `default`.
    pub fn default_option(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Adds an optional int-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice(self, name: impl Into<String>, value: i32) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: HashMap::new(),
        })
    }

    /// Adds a localized optional int-choice. See [`Self::add_int_choice`] for more info.
    pub fn add_int_choice_localized(
        self,
        name: impl Into<String>,
        value: i32,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
        })
    }

    /// Adds an optional string-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be up to 100 characters.
    pub fn add_string_choice(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::String(value.into()),
            name_localizations: HashMap::new(),
        })
    }

    /// Adds a localized optional string-choice. See [`Self::add_string_choice`] for more info.
    pub fn add_string_choice_localized(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::String(value.into()),
            name_localizations: locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
        })
    }

    /// Adds an optional number-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice(self, name: impl Into<String>, value: f64) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: HashMap::new(),
        })
    }

    /// Adds a localized optional number-choice. See [`Self::add_number_choice`] for more info.
    pub fn add_number_choice_localized(
        self,
        name: impl Into<String>,
        value: f64,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
        })
    }

    fn add_choice(mut self, value: CommandOptionChoice) -> Self {
        self.choices.push(value);
        self
    }

    /// Optionally enable/disable autocomplete interactions for this option.
    ///
    /// **Notes**:
    /// - May not be set to `true` if `choices` are set
    /// - Options using `autocomplete` are not confined to only use given choices
    pub fn set_autocomplete(mut self, value: bool) -> Self {
        self.autocomplete = Some(value);
        self
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group
    /// can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::command::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::command::CommandOptionType::SubCommand
    pub fn add_sub_option(mut self, sub_option: CreateApplicationCommandOption) -> Self {
        self.options.push(sub_option);
        self
    }

    /// If the option is a [`Channel`], it will only be able to show these types.
    ///
    /// [`Channel`]: crate::model::application::command::CommandOptionType::Channel
    pub fn channel_types(mut self, channel_types: Vec<ChannelType>) -> Self {
        self.channel_types = channel_types;
        self
    }

    /// Sets the minimum permitted value for this integer option
    pub fn min_int_value(mut self, value: u64) -> Self {
        self.min_value = Some(Number::Integer(value));
        self
    }

    /// Sets the maximum permitted value for this integer option
    pub fn max_int_value(mut self, value: u64) -> Self {
        self.max_value = Some(Number::Integer(value));
        self
    }

    /// Sets the minimum permitted value for this number option
    pub fn min_number_value(mut self, value: f64) -> Self {
        self.min_value = Some(Number::Float(value));
        self
    }

    /// Sets the maximum permitted value for this number option
    pub fn max_number_value(mut self, value: f64) -> Self {
        self.max_value = Some(Number::Float(value));
        self
    }

    /// Sets the minimum permitted length for this string option.
    ///
    /// The value of `min_length` must be greater or equal to `0`.
    pub fn min_length(&mut self, value: u16) -> &mut Self {
        self.min_length = Some(value);

        self
    }

    /// Sets the maximum permitted length for this string option.
    ///
    /// The value of `max_length` must be greater or equal to `1`.
    pub fn max_length(&mut self, value: u16) -> &mut Self {
        self.max_length = Some(value);

        self
    }
}

/// A builder for creating a new [`Command`].
///
/// [`Self::name`] and [`Self::description`] are required fields.
///
/// [`Command`]: crate::model::application::command::Command
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateApplicationCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<CommandType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    name_localizations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    description_localizations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_member_permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dm_permission: Option<bool>,

    options: Vec<CreateApplicationCommandOption>,
}

impl CreateApplicationCommand {
    /// Create a [`Command`], overriding an existing one with the same name if it exists.
    ///
    /// Providing a `command_id` will edit the corresponding command.
    ///
    /// Providing a `guild_id` will create a command in the corresponding [`Guild`]. Otherwise, a
    /// global command will be created.
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
        guild_id: Option<GuildId>,
        command_id: Option<CommandId>,
    ) -> Result<Command> {
        let http = http.as_ref();
        match (guild_id, command_id) {
            (Some(guild_id), Some(command_id)) => {
                http.edit_guild_application_command(guild_id.into(), command_id.into(), &self).await
            },
            (Some(guild_id), None) => {
                http.create_guild_application_command(guild_id.into(), &self).await
            },
            (None, Some(command_id)) => {
                http.edit_global_application_command(command_id.into(), &self).await
            },
            (None, None) => http.create_global_application_command(&self).await,
        }
    }

    /// Specifies the name of the application command.
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two
    /// global commands of the same app cannot have the same name. Two guild-specific commands of
    /// the same app cannot have the same name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Specifies a localized name of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateApplicationCommand::default()
    /// .name("birthday")
    /// .name_localized("zh-CN", "生日")
    /// .name_localized("el", "γενέθλια")
    /// # ;
    /// ```
    pub fn name_localized(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_localizations.insert(locale.into(), name.into());
        self
    }

    /// Specifies the type of the application command.
    pub fn kind(mut self, kind: CommandType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Specifies the default permissions required to execute the command.
    pub fn default_member_permissions(mut self, permissions: Permissions) -> Self {
        self.default_member_permissions = Some(permissions.bits().to_string());
        self
    }

    /// Specifies if the command is available in DMs.
    pub fn dm_permission(mut self, enabled: bool) -> Self {
        self.dm_permission = Some(enabled);

        self
    }

    /// Specifies the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specifies a localized description of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateApplicationCommand::default()
    /// .description("Wish a friend a happy birthday")
    /// .description_localized("zh-CN", "祝你朋友生日快乐")
    /// # ;
    /// ```
    pub fn description_localized(
        mut self,
        locale: impl Into<String>,

        description: impl Into<String>,
    ) -> Self {
        self.description_localizations.insert(locale.into(), description.into());
        self
    }

    /// Adds an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn add_option(mut self, option: CreateApplicationCommandOption) -> Self {
        self.options.push(option);
        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn set_options(mut self, options: Vec<CreateApplicationCommandOption>) -> Self {
        self.options = options;
        self
    }
}
