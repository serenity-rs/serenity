use std::collections::HashMap;

use crate::json::{from_number, Value};

/// Creates a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default)]
pub struct CreateStageInstance(pub HashMap<&'static str, Value>);

impl CreateStageInstance {
    // Sets the stage channel id of the stage channel instance.
    pub fn channel_id(&mut self, id: u64) -> &mut Self {
        self.0.insert("channel_id", from_number(id));
        self
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic<D: ToString>(&mut self, topic: D) -> &mut Self {
        self.0.insert("topic", Value::from(topic.to_string()));

        self
    }
}
