#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditThread<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitable: Option<bool>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditThread<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits the thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        channel_id: ChannelId,
    ) -> Result<GuildChannel> {
        http.as_ref().edit_thread(channel_id, &self, self.audit_log_reason).await
    }

    /// The name of the thread.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: Can only be set to 60, 1440, 4320, 10080 currently.
    pub fn auto_archive_duration(mut self, duration: u16) -> Self {
        self.auto_archive_duration = Some(duration);
        self
    }

    /// The archive status of the thread.
    ///
    /// **Note**: A thread that is `locked` can only be unarchived if the user has the
    /// `MANAGE_THREADS` permission.
    pub fn archived(mut self, archived: bool) -> Self {
        self.archived = Some(archived);
        self
    }

    /// The lock status of the thread.
    pub fn locked(mut self, lock: bool) -> Self {
        self.locked = Some(lock);
        self
    }

    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    pub fn invitable(mut self, invitable: bool) -> Self {
        self.invitable = Some(invitable);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
