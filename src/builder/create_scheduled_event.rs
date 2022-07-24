#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::encode_image;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateScheduledEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_type: Option<ScheduledEventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_metadata: Option<ScheduledEventMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,

    privacy_level: u8,
}

impl CreateScheduledEvent {
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
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(guild_id) {
                    let req = Permissions::MANAGE_EVENTS;

                    if !guild.has_perms(&cache_http, req).await {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        self._execute(cache_http.http(), guild_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, guild_id: GuildId) -> Result<ScheduledEvent> {
        http.create_scheduled_event(guild_id.into(), &self, None).await
    }

    /// Sets the channel id of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::StageInstance`] or [`ScheduledEventType::Voice`].
    pub fn channel_id<C: Into<ChannelId>>(mut self, channel_id: C) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Sets the name of the scheduled event. Required to be set for event creation.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event. Required to be set for event creation.
    pub fn start_time<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.scheduled_start_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the end time of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::External`].
    pub fn end_time<T: Into<Timestamp>>(mut self, timestamp: T) -> Self {
        self.scheduled_end_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the entity type of the scheduled event. Required to be set for event creation.
    pub fn kind(mut self, kind: ScheduledEventType) -> Self {
        self.entity_type = Some(kind);
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
    pub async fn image<'a>(
        mut self,
        http: impl AsRef<Http>,
        image: impl Into<AttachmentType<'a>>,
    ) -> Result<Self> {
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
}

impl Default for CreateScheduledEvent {
    /// Creates a builder with default values, setting the `privacy_level` to `GUILD_ONLY`. As this
    /// is the only possible value of this field, it's only used at event creation, and we don't
    /// even parse it into the `ScheduledEvent` struct.
    fn default() -> Self {
        Self {
            privacy_level: 2,

            name: None,
            image: None,
            channel_id: None,
            description: None,
            entity_type: None,
            entity_metadata: None,
            scheduled_end_time: None,
            scheduled_start_time: None,
        }
    }
}
