/// Edits a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditStageInstance {
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
}

impl EditStageInstance {
    /// Sets the topic of the stage channel instance.
    pub fn topic(&mut self, topic: impl Into<String>) -> &mut Self {
        self.topic = Some(topic.into());
        self
    }
}
