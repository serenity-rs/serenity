use std::collections::HashMap;

use crate::json::Value;

/// Edits a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default)]
pub struct EditStageInstance(pub HashMap<&'static str, Value>);

impl EditStageInstance {
    /// Sets the topic of the stage channel instance.
    pub fn topic(&mut self, topic: impl Into<String>) -> &mut Self {
        self.0.insert("topic", Value::String(topic.into()));

        self
    }
}
