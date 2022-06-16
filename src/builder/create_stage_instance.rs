use crate::model::id::ChannelId;

/// Creates a [`StageInstance`].
///
/// [`StageInstance`]: crate::model::channel::StageInstance
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateStageInstance {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
}

impl CreateStageInstance {
    // Sets the stage channel id of the stage channel instance.
    pub fn channel_id(&mut self, id: impl Into<ChannelId>) -> &mut Self {
        self.channel_id = Some(id.into());
        self
    }

    /// Sets the topic of the stage channel instance.
    pub fn topic(&mut self, topic: impl Into<String>) -> &mut Self {
        self.topic = Some(topic.into());
        self
    }
}
