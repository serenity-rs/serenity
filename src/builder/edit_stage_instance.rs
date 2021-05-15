use std::collections::HashMap;

use serde_json::Value;

use crate::internal::prelude::*;

/// Edits a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default)]
pub struct EditStageInstance(pub HashMap<&'static str, Value>);

impl EditStageInstance {
    /// Sets the topic of the stage channel instance.
    pub fn topic<D: ToString>(&mut self, topic: D) -> &mut Self {
        self.0.insert("topic", Value::String(topic.to_string()));

        self
    }
}
