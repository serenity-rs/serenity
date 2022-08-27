#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to specify the fields to edit in a [`GuildWidget`].
///
/// [`GuildWidget`]: crate::model::guild::GuildWidget
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildWidget<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditGuildWidget<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
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
    pub async fn execute(self, http: impl AsRef<Http>, guild_id: GuildId) -> Result<GuildWidget> {
        http.as_ref().edit_guild_widget(guild_id.into(), &self, self.audit_log_reason).await
    }

    /// Whether the widget is enabled or not.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// The server description shown in the welcome screen.
    pub fn channel_id(mut self, id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
