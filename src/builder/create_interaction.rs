use std::collections::HashMap;

use crate::json::from_number;
use crate::json::prelude::*;
use crate::json::Value;
use crate::{model::interactions::ApplicationCommandOptionType, utils};

/// A builder for creating a new [`ApplicationCommandInteractionDataOption`].
///
/// [`kind`], [`name`], and [`description`] are required fields.
///
/// [`ApplicationCommandInteractionDataOption`]: crate::model::interactions::ApplicationCommandInteractionDataOption
/// [`kind`]: Self::kind
/// [`name`]: Self::name
/// [`description`]: Self::description
#[derive(Clone, Debug, Default)]
pub struct CreateInteractionOption(pub HashMap<&'static str, Value>);

impl CreateInteractionOption {
    /// Set the ApplicationCommandOptionType for the InteractionOption.
    pub fn kind(&mut self, kind: ApplicationCommandOptionType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));
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

    /// The first required option for the user to complete
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

    /// Interaction commands can optionally have a limited
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
        F: FnOnce(&mut CreateInteractionOption) -> &mut CreateInteractionOption,
    {
        let mut data = CreateInteractionOption::default();
        f(&mut data);
        self.add_sub_option(data)
    }

    pub fn add_sub_option(&mut self, sub_option: CreateInteractionOption) -> &mut Self {
        let new_option = utils::hashmap_to_json_map(sub_option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::from(new_option));

        self
    }
}

/// A builder for creating a new [`ApplicationCommand`].
///
/// [`name`] and [`description`] are required fields.
///
/// [`ApplicationCommand`]: crate::model::interactions::ApplicationCommand
/// [`name`]: Self::name
/// [`description`]: Self::description
#[derive(Clone, Debug, Default)]
pub struct CreateInteraction(pub HashMap<&'static str, Value>);

impl CreateInteraction {
    /// Specify the name of the Interaction.
    ///
    /// **Note**: Must be between 1 and 32 characters long,
    /// and cannot start with a space.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));
        self
    }

    /// Specify the description of the Interaction.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::String(description.to_string()));
        self
    }

    /// Create an interaction option for the interaction.
    ///
    /// **Note**: Interactions can only have up to 10 options.
    pub fn create_interaction_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateInteractionOption) -> &mut CreateInteractionOption,
    {
        let mut data = CreateInteractionOption::default();
        f(&mut data);
        self.add_interaction_option(data)
    }

    /// Add an interaction option for the interaction.
    ///
    /// **Note**: Interactions can only have up to 10 options.
    pub fn add_interaction_option(&mut self, option: CreateInteractionOption) -> &mut Self {
        let new_option = utils::hashmap_to_json_map(option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        let opt_arr = options.as_array_mut().expect("Must be an array");
        opt_arr.push(Value::from(new_option));

        self
    }

    /// Sets all the interaction options for the interaction.
    ///
    /// **Note**: Interactions can only have up to 10 options.
    pub fn set_interaction_options(&mut self, options: Vec<CreateInteractionOption>) -> &mut Self {
        let new_options = options
            .into_iter()
            .map(|f| Value::from(utils::hashmap_to_json_map(f.0)))
            .collect::<Vec<Value>>();
        self.0.insert("options", Value::Array(new_options));
        self
    }
}
