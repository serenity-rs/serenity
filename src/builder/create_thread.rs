#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::channel::ChannelType;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateThread {
    #[cfg(feature = "http")]
    #[serde(skip)]
    channel_id: ChannelId,
    #[cfg(feature = "http")]
    #[serde(skip)]
    message_id: Option<MessageId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<ChannelType>,
}

impl CreateThread {
    pub fn new(
        #[cfg(feature = "http")] channel_id: ChannelId,
        #[cfg(feature = "http")] message_id: Option<MessageId>,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            channel_id,
            #[cfg(feature = "http")]
            message_id,
            name: None,
            auto_archive_duration: None,
            rate_limit_per_user: None,
            kind: None,
        }
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

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the [`MANAGE_MESSAGES`] and/or [`MANAGE_CHANNELS`] permissions are exempt
    /// from this restriction.
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

    /// The thread type, which can be [`ChannelType::PublicThread`] or [`ChannelType::PrivateThread`].
    ///
    /// **Note**: This defaults to [`ChannelType::PrivateThread`] in order to match the behavior
    /// when thread documentation was first published. This is a bit of a weird default though,
    /// and thus is highly likely to change in the future, so it is recommended to always
    /// explicitly setting it to avoid any breaking change.
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Executes the request to create a thread, either private or public. Public threads require a
    /// message to connect the thread to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<GuildChannel> {
        match self.message_id {
            Some(msg_id) => {
                http.as_ref()
                    .create_public_thread(self.channel_id.into(), msg_id.into(), &self)
                    .await
            },
            None => http.as_ref().create_private_thread(self.channel_id.into(), &self).await,
        }
    }
}
