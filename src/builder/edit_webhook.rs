use crate::model::id::ChannelId;

#[derive(Debug, Default, Clone, Serialize)]
pub struct EditWebhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
}

impl EditWebhook {
    /// Set default name of the Webhook.
    ///
    /// This must be between 1-80 characters.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the channel to move the webhook to.
    pub fn channel_id(&mut self, channel_id: impl Into<ChannelId>) -> &mut Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Set default avatar of the webhook.
    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        self.avatar = Some(avatar);
        self
    }
}
