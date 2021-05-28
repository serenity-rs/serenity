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
