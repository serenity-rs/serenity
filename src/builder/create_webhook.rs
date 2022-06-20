#[derive(Debug, Default, Clone, Serialize)]
pub struct CreateWebhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
}

impl CreateWebhook {
    /// Set default name of the Webhook.
    ///
    /// This must be between 1-80 characters.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set default avatar of the webhook.
    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        self.avatar = Some(avatar);
        self
    }
}
