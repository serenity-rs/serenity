use crate::model::id::ChannelId;

/// A builder to specify the fields to edit in a [`GuildWidget`].
///
/// [`GuildWidget`]: crate::model::guild::GuildWidget
#[derive(Clone, Debug, Default, Serialize)]
pub struct EditGuildWidget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<ChannelId>,
}

impl EditGuildWidget {
    /// Whether the widget is enabled or not.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = Some(enabled);

        self
    }

    /// The server description shown in the welcome screen.
    pub fn channel_id(&mut self, id: impl Into<ChannelId>) -> &mut Self {
        self.channel_id = Some(id.into());

        self
    }
}
