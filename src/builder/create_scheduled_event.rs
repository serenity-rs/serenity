#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::encode_image;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateScheduledEvent<'a> {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    scheduled_start_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_end_time: Option<String>,
    entity_type: ScheduledEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_metadata: Option<ScheduledEventMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,

    privacy_level: u8,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateScheduledEvent<'a> {
    /// Creates a builder with the provided kind, name, and start time, leaving all other fields
    /// empty.
    pub fn new(
        kind: ScheduledEventType,
        name: impl Into<String>,
        scheduled_start_time: impl Into<Timestamp>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type: kind,
            scheduled_start_time: scheduled_start_time.into().to_string(),

            image: None,
            channel_id: None,
            description: None,
            entity_metadata: None,
            scheduled_end_time: None,

            // Set the privacy level to `GUILD_ONLY`. As this is the only possible value of this
            // field, it's onlyu used at event creation, and we don't even parse it into the
            // `ScheduledEvent` struct.
            privacy_level: 2,

            audit_log_reason: None,
        }
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    #[cfg(feature = "http")]
    #[inline]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
    ) -> Result<ScheduledEvent> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_EVENTS)
            .await?;

        self._execute(cache_http.http(), guild_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, guild_id: GuildId) -> Result<ScheduledEvent> {
        http.create_scheduled_event(guild_id.into(), &self, self.audit_log_reason).await
    }

    /// Sets the channel id of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::StageInstance`] or [`ScheduledEventType::Voice`].
    pub fn channel_id<C: Into<ChannelId>>(mut self, channel_id: C) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Sets the name of the scheduled event, replacing the current value as set in [`Self::new`].
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event, replacing the current value as set in
    /// [`Self::new`].
    pub fn start_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_start_time = timestamp.into().to_string();
        self
    }

    /// Sets the end time of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::External`].
    pub fn end_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_end_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the entity type of the scheduled event, replacing the current value as set in
    /// [`Self::new`].
    pub fn kind(mut self, kind: ScheduledEventType) -> Self {
        self.entity_type = kind;
        self
    }

    /// Sets the location of the scheduled event. Required to be set and non-empty if
    /// [`Self::kind`] is [`ScheduledEventType::External`].
    ///
    /// [`External`]: ScheduledEventType::External
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.entity_metadata = Some(ScheduledEventMetadata {
            location: location.into(),
        });
        self
    }

    /// Sets the cover image for the scheduled event.
    ///
    /// # Errors
    ///
    /// May error if the input is a URL and the HTTP request fails, or if it is a path to a file
    /// that does not exist.
    #[cfg(feature = "http")]
    pub async fn image(
        mut self,
        http: impl AsRef<Http>,
        image: impl Into<AttachmentType<'a>>,
    ) -> Result<CreateScheduledEvent<'a>> {
        let image_data = image.into().data(&http.as_ref().client).await?;
        self.image = Some(encode_image(&image_data));
        Ok(self)
    }

    /// Sets the cover image for the scheduled event. Requires the input be a base64-encoded image
    /// that is in either JPG, GIF, or PNG format.
    #[cfg(not(feature = "http"))]
    pub fn image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}
