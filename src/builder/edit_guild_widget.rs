use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// A builder to specify the fields to edit in a [`GuildWidget`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#modify-guild-widget)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildWidget<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,

    #[serde(skip)]
    audit_log_reason: Option<Cow<'a, str>>,
}

impl<'a> EditGuildWidget<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the widget is enabled or not.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// The server description shown in the welcome screen.
    pub fn channel_id(mut self, id: ChannelId) -> Self {
        self.channel_id = Some(id);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.audit_log_reason = Some(reason.into());
        self
    }

    pub fn into_owned(self) -> EditGuildWidget<'static> {
        let Self {
            enabled,
            channel_id,
            audit_log_reason,
        } = self;
        EditGuildWidget {
            enabled,
            channel_id,
            audit_log_reason: audit_log_reason.map(|r| Cow::Owned(r.into_owned())),
        }
    }

    /// Edits the guild's widget.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<GuildWidget> {
        http.edit_guild_widget(guild_id, &self, self.audit_log_reason.as_deref()).await
    }
}
