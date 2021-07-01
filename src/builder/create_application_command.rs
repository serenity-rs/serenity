use std::collections::HashMap;

use serde_json::{json, Value};

use crate::{model::interactions::ApplicationCommandOptionType, utils};

/// A builder for creating a new [`ApplicationCommandOption`].
///
/// [`Self::kind`], [`Self::name`], and [`Self::description`] are required fields.
///
/// [`ApplicationCommandOption`]: crate::model::interactions::ApplicationCommandOption
/// [`kind`]: Self::kind
/// [`name`]: Self::name
/// [`description`]: Self::description
#[derive(Clone, Debug, Default)]
pub struct CreateApplicationCommandOption(pub HashMap<&'static str, Value>);

impl CreateApplicationCommandOption {
    /// Sets the ApplicationCommandOptionType.
    pub fn kind(&mut self, kind: ApplicationCommandOptionType) -> &mut Self {
        self.0.insert("type", Value::Number(serde_json::Number::from(kind as u8)));
        self
    }

    /// Sets the name of the option.
    ///
    /// **Note**: The option name must be between 1 and 32 characters.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Sets the description for the option.
    ///
    /// **Note**: The description must be between 1 and 100 characters.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::String(description.to_string()));
        self
    }

    /// The first required option for the user to complete.
    ///
    /// **Note**: Only one option can be `default`.
    pub fn default_option(&mut self, default: bool) -> &mut Self {
        self.0.insert("default", Value::Bool(default));
        self
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`.
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.0.insert("required", Value::Bool(required));
        self
    }

    /// Application commands can optionally have a limited
    /// number of integer or string choices.
    ///
    /// **Note**: There can be no more than 10 choices set.
    pub fn add_int_choice<D: ToString>(&mut self, name: D, value: i32) -> &mut Self {
        let choice = json!({
            "name": name.to_string(),
            "value" : value
        });
        self.add_choice(choice)
    }

    pub fn add_string_choice<D: ToString, E: ToString>(&mut self, name: D, value: E) -> &mut Self {
        let choice = json!({
            "name": name.to_string(),
            "value": value.to_string()
        });
        self.add_choice(choice)
    }

    fn add_choice(&mut self, value: Value) -> &mut Self {
        let choices = self.0.entry("choices").or_insert_with(|| Value::Array(Vec::new()));
        let choices_arr = choices.as_array_mut().expect("Must be an array");
        choices_arr.push(value);

        self
    }

    /// If the option is a [`SubCommand`] or [`SubCommandGroup`] nested options are its parameters.
    ///
    /// [`SubCommand`]: crate::model::interactions::ApplicationCommandOptionType::SubCommand
    /// [`SubCommandGroup`]: crate::model::interactions::ApplicationCommandOptionType::SubCommandGroup
    pub fn create_sub_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption,
    {
        let mut data = CreateApplicationCommandOption::default();
        f(&mut data);
        self.add_sub_option(data)
    }

    pub fn add_sub_option(&mut self, sub_option: CreateApplicationCommandOption) -> &mut Self {
        let new_option = utils::hashmap_to_json_map(sub_option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::Object(new_option));

        self
    }
}

/// A builder for creating a new [`ApplicationCommand`].
///
/// [`Self::name`] and [`Self::description`] are required fields.
///
/// [`ApplicationCommand`]: crate::model::interactions::ApplicationCommand
#[derive(Clone, Debug, Default)]
pub struct CreateApplicationCommand(pub HashMap<&'static str, Value>);

impl CreateApplicationCommand {
    /// Specify the name of the application command.
    ///
    /// **Note**: Must be between 1 and 32 characters long,
    /// and cannot start with a space.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Specify if the command should not be usable by default
    ///
    /// **Note**: Setting it to false will disable it for anyone,
    /// including administrators and guild owners.
    pub fn default_permission(&mut self, default_permission: bool) -> &mut Self {
        self.0.insert("default_permission", Value::Bool(default_permission));

        self
    }

    /// Specify the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::String(description.to_string()));
        self
    }

    /// Create an application command option for the application command.
    ///
    /// **Note**: Application commands can only have up to 10 options.
    pub fn create_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption,
    {
        let mut data = CreateApplicationCommandOption::default();
        f(&mut data);
        self.add_option(data)
    }

    /// Add an application command option for the application command.
    ///
    /// **Note**: Application commands can only have up to 10 options.
    pub fn add_option(&mut self, option: CreateApplicationCommandOption) -> &mut Self {
        let new_option = utils::hashmap_to_json_map(option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::Object(new_option));

        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can only have up to 10 options.
    pub fn set_options(&mut self, options: Vec<CreateApplicationCommandOption>) -> &mut Self {
        let new_options = options
            .into_iter()
            .map(|f| Value::Object(utils::hashmap_to_json_map(f.0)))
            .collect::<Vec<Value>>();
        self.0.insert("options", Value::Array(new_options));
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
        let new_data = Value::Object(utils::hashmap_to_json_map(command.0));

        self.0.push(new_data);

        self
    }

    /// Sets all the application commands.
    pub fn set_application_commands(
        &mut self,
        commands: Vec<CreateApplicationCommand>,
    ) -> &mut Self {
        let new_application_command = commands
            .into_iter()
            .map(|f| Value::Object(utils::hashmap_to_json_map(f.0)))
            .collect::<Vec<Value>>();

        for application_command in new_application_command {
            self.0.push(application_command);
        }

        self
    }
}
