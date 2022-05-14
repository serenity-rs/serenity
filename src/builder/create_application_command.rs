use std::collections::HashMap;

use crate::json;
use crate::json::prelude::*;
use crate::model::application::command::{CommandOptionType, CommandType};
use crate::model::channel::ChannelType;
use crate::model::Permissions;

/// A builder for creating a new [`CommandOption`].
///
/// [`Self::kind`], [`Self::name`], and [`Self::description`] are required fields.
///
/// [`CommandOption`]: crate::model::application::command::CommandOption
/// [`kind`]: Self::kind
/// [`name`]: Self::name
/// [`description`]: Self::description
#[derive(Clone, Debug, Default)]
pub struct CreateApplicationCommandOption(pub HashMap<&'static str, Value>);

impl CreateApplicationCommandOption {
    /// Sets the `CommandOptionType`.
    pub fn kind(&mut self, kind: CommandOptionType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));
        self
    }

    /// Sets the name of the option.
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.0.insert("name", Value::String(name.into()));
        self
    }

    /// Sets the description for the option.
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.0.insert("description", Value::String(description.into()));
        self
    }

    /// The first required option for the user to complete.
    ///
    /// **Note**: Only one option can be `default`.
    pub fn default_option(&mut self, default: bool) -> &mut Self {
        self.0.insert("default", Value::from(default));
        self
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`.
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.0.insert("required", Value::from(required));
        self
    }

    /// Adds an optional int-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice(&mut self, name: impl Into<String>, value: i32) -> &mut Self {
        let choice = json!({
            "name": name.into(),
            "value" : value
        });
        self.add_choice(choice)
    }

    /// Adds an optional string-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be up to 100 characters.
    pub fn add_string_choice(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        let choice = json!({
            "name": name.into(),
            "value": value.into()
        });
        self.add_choice(choice)
    }

    /// Adds an optional number-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100 characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice(&mut self, name: impl Into<String>, value: f64) -> &mut Self {
        let choice = json!({
            "name": name.into(),
            "value" : value
        });
        self.add_choice(choice)
    }

    fn add_choice(&mut self, value: Value) -> &mut Self {
        let choices = self.0.entry("choices").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let choices_arr = choices.as_array_mut().expect("Must be an array");
        choices_arr.push(value);

        self
    }

    /// Optionally enable/disable autocomplete interactions for this option.
    ///
    /// **Notes**:
    /// - May not be set to `true` if `choices` are set
    /// - Options using `autocomplete` are not confined to only use given choices
    pub fn set_autocomplete(&mut self, value: bool) -> &mut Self {
        self.0.insert("autocomplete", Value::from(value));

        self
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::command::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::command::CommandOptionType::SubCommand
    pub fn create_sub_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption,
    {
        let mut data = CreateApplicationCommandOption::default();
        f(&mut data);
        self.add_sub_option(data)
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::command::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::command::CommandOptionType::SubCommand
    pub fn add_sub_option(&mut self, sub_option: CreateApplicationCommandOption) -> &mut Self {
        let new_option = json::hashmap_to_json_map(sub_option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::from(new_option));

        self
    }

    /// If the option is a [`Channel`], it will only be able to show these types.
    ///
    /// [`Channel`]: crate::model::application::command::CommandOptionType::Channel
    pub fn channel_types(&mut self, channel_types: &[ChannelType]) -> &mut Self {
        self.0.insert(
            "channel_types",
            Value::from(channel_types.iter().map(|i| from_number(*i as u8)).collect::<Vec<_>>()),
        );

        self
    }

    /// Sets the minimum permitted value for this integer option
    pub fn min_int_value(&mut self, value: impl ToNumber) -> &mut Self {
        self.0.insert("min_value", value.to_number());

        self
    }

    /// Sets the maximum permitted value for this integer option
    pub fn max_int_value(&mut self, value: impl ToNumber) -> &mut Self {
        self.0.insert("max_value", value.to_number());

        self
    }

    /// Sets the minimum permitted value for this number option
    pub fn min_number_value(&mut self, value: f64) -> &mut Self {
        self.0.insert("min_value", Value::from(value));

        self
    }

    /// Sets the maximum permitted value for this number option
    pub fn max_number_value(&mut self, value: f64) -> &mut Self {
        self.0.insert("max_value", Value::from(value));

        self
    }
}

/// A builder for creating a new [`Command`].
///
/// [`Self::name`] and [`Self::description`] are required fields.
///
/// [`Command`]: crate::model::application::command::Command
#[derive(Clone, Debug, Default)]
pub struct CreateApplicationCommand(pub HashMap<&'static str, Value>);

impl CreateApplicationCommand {
    /// Specifies the name of the application command.
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two global commands of the same app cannot have the same name. Two guild-specific commands of the same app cannot have the same name.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.0.insert("name", Value::String(name.into()));
        self
    }

    /// Specifies the type of the application command.
    pub fn kind(&mut self, kind: CommandType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));
        self
    }

    /// Specifies the default permissions required to execute the command.
    pub fn default_member_permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.0.insert("default_member_permissions", Value::String(permissions.bits().to_string()));

        self
    }

    /// Specifies if the command is available in DMs.
    pub fn dm_permission(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("dm_permission", Value::from(enabled));

        self
    }

    /// Specifies the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.0.insert("description", Value::String(description.into()));
        self
    }

    /// Creates an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn create_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption,
    {
        let mut data = CreateApplicationCommandOption::default();
        f(&mut data);
        self.add_option(data)
    }

    /// Adds an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn add_option(&mut self, option: CreateApplicationCommandOption) -> &mut Self {
        let new_option = json::hashmap_to_json_map(option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::from(new_option));

        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn set_options(&mut self, options: Vec<CreateApplicationCommandOption>) -> &mut Self {
        let new_options = options
            .into_iter()
            .map(|f| Value::from(json::hashmap_to_json_map(f.0)))
            .collect::<Vec<Value>>();

        self.0.insert("options", Value::from(new_options));
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreateApplicationCommands(pub Vec<Value>);

impl CreateApplicationCommands {
    /// Creates a new application command.
    pub fn create_application_command<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let mut data = CreateApplicationCommand::default();
        f(&mut data);

        self.add_application_command(data);

        self
    }

    /// Adds a new application command.
    pub fn add_application_command(&mut self, command: CreateApplicationCommand) -> &mut Self {
        let new_data = Value::from(json::hashmap_to_json_map(command.0));

        self.0.push(new_data);

        self
    }

    /// Sets all the application commands.
    pub fn set_application_commands(
        &mut self,
        commands: Vec<CreateApplicationCommand>,
    ) -> &mut Self {
        let new_application_command =
            commands.into_iter().map(|f| Value::from(json::hashmap_to_json_map(f.0)));

        for application_command in new_application_command {
            self.0.push(application_command);
        }

        self
    }
}
