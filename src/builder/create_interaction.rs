use std::collections::HashMap;
use serde_json::{Value, json};

use crate::{
    model::interactions::ApplicationCommandOptionType,
    utils,
};

#[derive(Clone, Debug, Default)]
pub struct CreateInteractionOptionChoices(pub Vec<Value>);

impl CreateInteractionOptionChoices {
    #[inline]
    pub fn add_int<D: ToString>(&mut self, name: D, value: i32) -> &mut Self {
        self.add_value(name.to_string(), Value::Number(serde_json::Number::from(value)))
    }

    #[inline]
    pub fn add_string<D: ToString, E: ToString>(&mut self, name: D, value: E) -> &mut Self {
        self.add_value(name.to_string(), Value::String(value.to_string()))
    }

    fn add_value(&mut self, name: String, value: Value) -> &mut Self {
        let choice = json!({
            "name": name,
            "value": value,
        });
        self.0.push(choice);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreateInteractionOption(pub HashMap<&'static str, Value>);

impl CreateInteractionOption {
    /// Sets the `ApplicationCommandOptionType` for the `InteractionOption`.
    pub fn kind(&mut self, kind: ApplicationCommandOptionType) -> &mut Self {
        self.0.insert("type", Value::Number(serde_json::Number::from(kind as u8)));
        self
    }

    /// Sets the name of the option.
    ///
    /// **Note**: The option name must be no more than 32 unicode code points.
    #[inline]
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self._name(name.to_string())
    }

    fn _name(&mut self, name: String) -> &mut Self {
        self.0.insert("name", Value::String(name));
        self
    }

    /// Sets the description for the option.
    ///
    /// **Note**: The description cannot be longer than 100 unicode code points.
    #[inline]
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self._description(description.to_string())
    }

    fn _description(&mut self, description: String) -> &mut Self {
        self.0.insert("description", Value::String(description));
        self
    }

    /// Sets if this option is the first required option.
    ///
    /// **Note**: Only one option can be `default`.
    pub fn default_option(&mut self, default: bool) -> &mut Self {
        todo!();
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`
    pub fn required(&mut self, required: bool) -> &mut Self {
        todo!();
    }

    /// Sets the choices for the option
    pub fn choices<F>(&mut self, f: F) -> &mut Self
    where F: FnOnce(&mut CreateInteractionOptionChoices) -> &mut CreateInteractionOptionChoices {
        let mut choices = CreateInteractionOptionChoices::default();
        f(&mut choices);
        self.0.insert("choices", Value::Array(choices.0));
        self
    }

    pub fn add_sub_option(&mut self, sub_option: CreateInteractionOption) -> &mut Self {
        let new_option = utils::hashmap_to_json_map(sub_option.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        if let Some(opt_arr) = options.as_array_mut() {
            opt_arr.push(Value::Object(new_option));
        };
        self
    }

    pub fn create_sub_option<F>(&mut self, f: F) -> &mut Self
    where F: FnOnce(&mut CreateInteractionOption) -> &mut CreateInteractionOption {
        let mut data = CreateInteractionOption::default();
        f(&mut data);
        let new_option = utils::hashmap_to_json_map(data.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        if let Some(opt_arr) = options.as_array_mut() {
            opt_arr.push(Value::Object(new_option));
        };
        self
    }

}


#[derive(Clone, Debug)]
pub struct CreateInteraction(pub HashMap<&'static str, Value>);

impl CreateInteraction {
    pub fn new(name: String, description: String) -> Self {
        let mut map: HashMap<&'static str, Value> = HashMap::new();
        map.insert("name", Value::String(name));
        map.insert("description", Value::String(description));
        Self(map)
    }

    pub fn create_option<F>(&mut self, f: F) -> &mut Self
    where F: FnOnce(&mut CreateInteractionOption) -> &mut CreateInteractionOption {
        let mut data = CreateInteractionOption::default();
        f(&mut data);
        let new_option = utils::hashmap_to_json_map(data.0);
        let options = self.0.entry("options").or_insert_with(|| Value::Array(Vec::new()));
        if let Some(opt_arr) = options.as_array_mut() {
            opt_arr.push(Value::Object(new_option));
        };
        // should this be changed to have None be a panic?
        self

    }

}

