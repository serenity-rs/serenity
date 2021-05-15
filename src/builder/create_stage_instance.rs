use std::collections::HashMap;

use serde_json::Value;

use crate::internal::prelude::*;

/// Creates a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default)]
pub struct CreateStageInstance(pub HashMap<&'static str, Value>);

impl CreateStageInstance {
    // Sets the stage channel id of the stage channel instance.
    pub fn channel_id(&mut self, id: u64) -> &mut Self {
        self.0.insert("channel_id", Value::Number(Number::from(id)));
        self
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic<D: ToString>(&mut self, topic: D) -> &mut Self {
        self.0.insert("topic", Value::String(topic.to_string()));

        self
    }
}
