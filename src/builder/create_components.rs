use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::json::{self, from_number, Value};
use crate::model::application::component::{ButtonStyle, InputTextStyle};
use crate::model::channel::ReactionType;

/// A builder for creating several [`ActionRow`]s.
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
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

    /// Set a single action row.
    /// Calling this will overwrite all action rows.
    pub fn set_action_row(&mut self, mut row: CreateActionRow) -> &mut Self {
        self.0 = vec![row.build()];

        self
    }

    /// Sets all the action rows.
    pub fn set_action_rows(&mut self, rows: Vec<CreateActionRow>) -> &mut Self {
        let new_rows = rows.into_iter().map(|mut f| f.build());

        for row in new_rows {
            self.0.push(row);
        }

        self
    }
}

/// A builder for creating an [`ActionRow`].
///
/// [`ActionRow`]: crate::model::application::component::ActionRow
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
        let components =
            self.0.entry("components").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let components_array = components.as_array_mut().expect("Must be an array");

        components_array.push(button.build());

        self
    }

    /// Creates a select menu.
    pub fn create_select_menu<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenu) -> &mut CreateSelectMenu,
    {
        let mut data = CreateSelectMenu::default();
        f(&mut data);

        self.add_select_menu(data);

        self
    }

    /// Adds a select menu.
    pub fn add_select_menu(&mut self, menu: CreateSelectMenu) -> &mut Self {
        let components =
            self.0.entry("components").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let components_array = components.as_array_mut().expect("Must be an array");

        components_array.push(menu.build());

        self
    }

    /// Creates an input text.
    pub fn create_input_text<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateInputText) -> &mut CreateInputText,
    {
        let mut data = CreateInputText::default();
        f(&mut data);

        self.add_input_text(data);

        self
    }

    /// Adds an input text.
    pub fn add_input_text(&mut self, input_text: CreateInputText) -> &mut Self {
        let components =
            self.0.entry("components").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let components_array = components.as_array_mut().expect("Must be an array");

        components_array.push(input_text.build());

        self
    }

    pub fn build(&mut self) -> Value {
        self.0.insert("type", from_number(1_u8));

        json::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating a [`Button`].
///
/// [`Button`]: crate::model::application::component::Button
#[derive(Clone, Debug)]
pub struct CreateButton(pub HashMap<&'static str, Value>);

impl Default for CreateButton {
    /// Creates a primary button.
    fn default() -> Self {
        let mut btn = Self(HashMap::new());
        btn.style(ButtonStyle::Primary);
        btn
    }
}

impl CreateButton {
    /// Sets the style of the button.
    pub fn style(&mut self, kind: ButtonStyle) -> &mut Self {
        self.0.insert("style", from_number(kind as u8));
        self
    }

    /// The label of the button.
    pub fn label<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("label", Value::from(label.to_string()));
        self
    }

    /// Sets the custom id of the button, a developer-defined identifier.
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::from(id.to_string()));
        self
    }

    /// The url for url style button.
    pub fn url<D: ToString>(&mut self, url: D) -> &mut Self {
        self.0.insert("url", Value::from(url.to_string()));
        self
    }

    /// Sets emoji of the button.
    pub fn emoji<R: Into<ReactionType>>(&mut self, emoji: R) -> &mut Self {
        self._emoji(emoji.into())
    }

    fn _emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".to_string(), Value::from(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".to_string(), Value::from(animated));
                map.insert("id".to_string(), Value::from(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".to_string(), Value::from(name));
                }
            },
        };

        self.0.insert("emoji", Value::from(map));
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.0.insert("disabled", Value::from(disabled));
        self
    }

    #[must_use]
    pub fn build(mut self) -> Value {
        self.0.insert("type", from_number(2_u8));

        json::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating a [`SelectMenu`].
///
/// [`SelectMenu`]: crate::model::application::component::SelectMenu
#[derive(Clone, Debug, Default)]
pub struct CreateSelectMenu(pub HashMap<&'static str, Value>);

impl CreateSelectMenu {
    /// The placeholder of the select menu.
    pub fn placeholder<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("placeholder", Value::from(label.to_string()));
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier.
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::from(id.to_string()));
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(&mut self, min: u64) -> &mut Self {
        self.0.insert("min_values", from_number(min));
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(&mut self, max: u64) -> &mut Self {
        self.0.insert("max_values", from_number(max));
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.0.insert("disabled", Value::from(disabled));
        self
    }

    pub fn options<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateSelectMenuOptions) -> &mut CreateSelectMenuOptions,
    {
        let mut data = CreateSelectMenuOptions::default();
        f(&mut data);

        self.0.insert("options", Value::from(data.0));

        self
    }

    #[must_use]
    pub fn build(mut self) -> Value {
        self.0.insert("type", from_number(3_u8));

        json::hashmap_to_json_map(self.0.clone()).into()
    }
}

/// A builder for creating several [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
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
        let data = json::hashmap_to_json_map(option.0);

        self.0.push(data.into());

        self
    }

    /// Sets all the options.
    pub fn set_options(&mut self, options: Vec<CreateSelectMenuOption>) -> &mut Self {
        let new_options =
            options.into_iter().map(|option| json::hashmap_to_json_map(option.0).into());

        for option in new_options {
            self.0.push(option);
        }

        self
    }
}

/// A builder for creating a [`SelectMenuOption`].
///
/// [`SelectMenuOption`]: crate::model::application::component::SelectMenuOption
#[derive(Clone, Debug, Default)]
pub struct CreateSelectMenuOption(pub HashMap<&'static str, Value>);

impl CreateSelectMenuOption {
    /// Creates an option.
    pub fn new<L: ToString, V: ToString>(label: L, value: V) -> Self {
        let mut opt = Self::default();
        opt.label(label).value(value);
        opt
    }

    /// Sets the label of this option.
    pub fn label<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("label", Value::from(label.to_string()));
        self
    }

    /// Sets the value of this option.
    pub fn value<D: ToString>(&mut self, value: D) -> &mut Self {
        self.0.insert("value", Value::from(value.to_string()));
        self
    }

    /// Sets the description shown on this option.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    /// Sets emoji of the option.
    pub fn emoji<R: Into<ReactionType>>(&mut self, emoji: R) -> &mut Self {
        self._emoji(emoji.into())
    }

    fn _emoji(&mut self, emoji: ReactionType) -> &mut Self {
        let mut map = JsonMap::new();

        match emoji {
            ReactionType::Unicode(u) => {
                map.insert("name".to_string(), Value::from(u));
            },
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                map.insert("animated".to_string(), Value::from(animated));
                map.insert("id".to_string(), Value::from(id.to_string()));

                if let Some(name) = name {
                    map.insert("name".to_string(), Value::from(name));
                }
            },
        };

        self.0.insert("emoji", Value::from(map));
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(&mut self, disabled: bool) -> &mut Self {
        self.0.insert("default", Value::from(disabled));
        self
    }
}

/// A builder for creating an [`InputText`].
///
/// [`InputText`]: crate::model::application::component::InputText
#[derive(Clone, Debug, Default)]
pub struct CreateInputText(pub HashMap<&'static str, Value>);

impl CreateInputText {
    /// Sets the custom id of the input text, a developer-defined identifier.
    pub fn custom_id<D: ToString>(&mut self, id: D) -> &mut Self {
        self.0.insert("custom_id", Value::from(id.to_string()));
        self
    }

    /// Sets the style of this input text
    pub fn style(&mut self, kind: InputTextStyle) -> &mut Self {
        self.0.insert("style", from_number(kind as u8));
        self
    }

    /// Sets the label of this input text.
    pub fn label<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("label", Value::from(label.to_string()));
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder<D: ToString>(&mut self, label: D) -> &mut Self {
        self.0.insert("placeholder", Value::from(label.to_string()));
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(&mut self, min: u64) -> &mut Self {
        self.0.insert("min_length", from_number(min));
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(&mut self, max: u64) -> &mut Self {
        self.0.insert("max_length", from_number(max));
        self
    }

    /// Sets the value of this input text.
    pub fn value<D: ToString>(&mut self, value: D) -> &mut Self {
        self.0.insert("value", Value::from(value.to_string()));
        self
    }

    /// Sets if the input text is required
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.0.insert("required", Value::from(required));
        self
    }

    #[must_use]
    pub fn build(mut self) -> Value {
        self.0.insert("type", from_number(4_u8));

        json::hashmap_to_json_map(self.0.clone()).into()
    }
}
