use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::model::channel::ReactionType;
use crate::model::prelude::ButtonStyle;
use crate::utils;

/// A builder for creating several [`ActionRow`]s.
///
/// [`ActionRow`]: crate::model::interactions::ActionRow
#[derive(Clone, Debug, Default)]
pub struct CreateComponents(pub Vec<Value>);

impl CreateComponents {
    /// Creates an action row.
    pub fn create_action_row<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateActionRow) -> &mut CreateActionRow,
    {
        let mut data = CreateActionRow::default();
        f(&mut data);

        self.add_action_row(data);

        self
    }

    /// Adds an action row.
    pub fn add_action_row(&mut self, mut row: CreateActionRow) -> &mut Self {
        self.0.push(row.build());

        self
    }

    /// Sets all the action rows.
    pub fn set_action_rows(&mut self, rows: Vec<CreateActionRow>) -> &mut Self {
        let new_rows = rows.into_iter().map(|mut f| f.build()).collect::<Vec<Value>>();

        for row in new_rows {
            self.0.push(row);
        }

        self
    }
}

/// A builder for creating an [`ActionRow`].
///
/// [`ActionRow`]: crate::model::interactions::ActionRow
#[derive(Clone, Debug, Default)]
pub struct CreateActionRow(pub HashMap<&'static str, Value>);

impl CreateActionRow {
    /// Creates a button.
    pub fn create_button<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateButton) -> &mut CreateButton,
    {
        let mut data = CreateButton::default();
        f(&mut data);

        self.add_button(data);

        self
    }

    /// Adds a button.
    pub fn add_button(&mut self, button: CreateButton) -> &mut Self {
        let components = self.0.entry("components").or_insert_with(|| Value::Array(Vec::new()));
        let components_array = components.as_array_mut().expect("Must be an array");

        components_array.push(button.build());

        self
    }

    /// Creates a button.
    pub fn create_select_menu<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenu) -> &mut CreateSelectMenu,
    {
        let mut data = CreateSelectMenu::default();
        f(&mut data);

        self.add_select_menu(data);

        self
    }

    /// Adds a button.
    pub fn add_select_menu(&mut self, menu: CreateSelectMenu) -> &mut Self {
        let components = self.0.entry("components").or_insert_with(|| Value::Array(Vec::new()));
        let components_array = components.as_array_mut().expect("Must be an array");

        components_array.push(menu.build());

        self
    }

    pub fn build(&mut self) -> Value {
        self.0.insert("type", Value::Number(serde_json::Number::from(1 as u8)));

        utils::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::interactions::Button
#[derive(Clone, Debug, Default)]
pub struct CreateButton(pub HashMap<&'static str, Value>);

impl CreateButton {
    /// Sets the style of the button.
    pub fn style(&mut self, kind: ButtonStyle) -> &mut Self {
        self.0.insert("style", Value::Number(serde_json::Number::from(kind as u8)));
        self
    }

    /// The label of the button.
    pub fn label<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("label", Value::String(label.to_string()));
        self
    }

    /// Sets the custom id of the button, a developer-defined identifier.
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::String(id.to_string()));
        self
    }

    /// The url for url style button.
    pub fn url<D: ToString>(&mut self, url: D) -> &mut Self {
        self.0.insert("url", Value::String(url.to_string()));
        self
    }

    /// Sets emoji of the button.
    pub fn emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".to_string(), Value::String(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".to_string(), Value::Bool(animated));
                map.insert("id".to_string(), Value::String(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".to_string(), Value::String(name));
                }
            },
        };

        self.0.insert("emoji", Value::Object(map));
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.0.insert("disabled", Value::Bool(disabled));
        self
    }

    pub fn build(mut self) -> Value {
        self.0.insert("type", Value::Number(serde_json::Number::from(2 as u8)));

        utils::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::interactions::SelectMenu
#[derive(Clone, Debug, Default)]
pub struct CreateSelectMenu(pub HashMap<&'static str, Value>);

impl CreateSelectMenu {
    /// The placeholder of the select menu.
    pub fn placeholder<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("placeholder", Value::String(label.to_string()));
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier.
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::String(id.to_string()));
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(&mut self, min: u64) -> &mut Self {
        self.0.insert("min_values", Value::Number(Number::from(min)));
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(&mut self, max: u64) -> &mut Self {
        self.0.insert("max_values", Value::Number(Number::from(max)));
        self
    }

    pub fn options<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenuOptions) -> &mut CreateSelectMenuOptions,
    {
        let mut data = CreateSelectMenuOptions::default();
        f(&mut data);

        self.0.insert("options", Value::Array(data.0));

        self
    }

    pub fn build(mut self) -> Value {
        self.0.insert("type", Value::Number(serde_json::Number::from(3 as u8)));

        utils::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating several [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::interactions::SelectMenuOption
#[derive(Clone, Debug, Default)]
pub struct CreateSelectMenuOptions(pub Vec<Value>);

impl CreateSelectMenuOptions {
    /// Creates an option.
    pub fn create_option<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenuOption) -> &mut CreateSelectMenuOption,
    {
        let mut data = CreateSelectMenuOption::default();
        f(&mut data);

        self.add_option(data);

        self
    }

    /// Adds an option.
    pub fn add_option(&mut self, option: CreateSelectMenuOption) -> &mut Self {
        let data = utils::hashmap_to_json_map(option.0);

        self.0.push(data.into());

        self
    }

    /// Sets all the options.
    pub fn set_options(&mut self, options: Vec<CreateSelectMenuOption>) -> &mut Self {
        let new_options = options
            .into_iter()
            .map(|option| utils::hashmap_to_json_map(option.0).into())
            .collect::<Vec<Value>>();

        for option in new_options {
            self.0.push(option);
        }

        self
    }
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::interactions::SelectMenuOption
#[derive(Clone, Debug, Default)]
pub struct CreateSelectMenuOption(pub HashMap<&'static str, Value>);

impl CreateSelectMenuOption {
    /// Sets the label of this option.
    pub fn label<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("label", Value::String(label.to_string()));
        self
    }

    /// Sets the value of this option.
    pub fn value<D: ToString>(&mut self, value: D) -> &mut Self {
        self.0.insert("value", Value::String(value.to_string()));
        self
    }

    /// Sets the description shown on this option.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("value", Value::String(description.to_string()));
        self
    }

    /// Sets emoji of the option.
    pub fn emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".to_string(), Value::String(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".to_string(), Value::Bool(animated));
                map.insert("id".to_string(), Value::String(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".to_string(), Value::String(name));
                }
            },
        };

        self.0.insert("emoji", Value::Object(map));
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(&mut self, disabled: bool) -> &mut Self {
        self.0.insert("default", Value::Bool(disabled));
        self
    }
}
