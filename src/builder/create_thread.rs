#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateThread<'a> {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<ChannelType>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateThread<'a> {
    /// Creates a builder with the given thread name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            auto_archive_duration: None,
            rate_limit_per_user: None,
            kind: None,
            audit_log_reason: None,
        }
    }

    /// Creates a thread, either private or public. Public threads require a message to connect the
    /// thread to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        channel_id: ChannelId,
        message_id: Option<MessageId>,
    ) -> Result<GuildChannel> {
        let http = http.as_ref();
        let id = channel_id.into();
        match message_id {
            Some(msg_id) => {
                http.create_public_thread(id, msg_id.into(), &self, self.audit_log_reason).await
            },
            None => http.create_private_thread(id, &self, self.audit_log_reason).await,
        }
    }

    /// The name of the thread. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: Can only be set to 60, 1440, 4320, 10080 currently.
    pub fn auto_archive_duration(mut self, duration: u16) -> Self {
        self.auto_archive_duration = Some(duration);
        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are
    /// exempt from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    ///
    /// [`MANAGE_MESSAGES`]: crate::model::permissions::Permissions::MANAGE_MESSAGES
    /// [`MANAGE_CHANNELS`]: crate::model::permissions::Permissions::MANAGE_CHANNELS
    #[doc(alias = "slowmode")]
    pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    /// The thread type, either [`ChannelType::PublicThread`] or [`ChannelType::PrivateThread`].
    ///
    /// **Note**: This defaults to [`ChannelType::PrivateThread`] in order to match the behavior
    /// when thread documentation was first published. This is a bit of a weird default though, and
    /// thus is highly likely to change in the future, so it is recommended to always explicitly
    /// setting it to avoid any breaking change.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
