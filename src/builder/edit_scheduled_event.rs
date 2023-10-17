#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditScheduledEvent<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_metadata: Option<Option<ScheduledEventMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_type: Option<ScheduledEventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<ScheduledEventStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditScheduledEvent<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Modifies a scheduled event in the guild with the data set, if any.
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
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        event_id: ScheduledEventId,
    ) -> Result<ScheduledEvent> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, guild_id, Permissions::MANAGE_EVENTS)
            .await?;

        self._execute(cache_http.http(), guild_id, event_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(
        self,
        http: &Http,
        guild_id: GuildId,
        event_id: ScheduledEventId,
    ) -> Result<ScheduledEvent> {
        http.as_ref()
            .edit_scheduled_event(guild_id.into(), event_id.into(), &self, self.audit_log_reason)
            .await
    }

    /// Sets the channel id of the scheduled event. If the [`kind`] of the event is changed from
    /// [`External`] to either [`StageInstance`] or [`Voice`], then this field is also required.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`Voice`]: ScheduledEventType::Voice
    /// [`External`]: ScheduledEventType::External
    pub fn channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(Some(channel_id.into()));
        self
    }

    /// Sets the name of the scheduled event.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event.
    #[inline]
    pub fn start_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_start_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the end time of the scheduled event.
    ///
    /// If the [`kind`] of the event is changed to [`External`], then this field is also required.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    #[inline]
    pub fn end_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_end_time = Some(timestamp.into().to_string());
        self
    }

    // See https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-field-requirements-by-entity-type
    /// Sets the entity type of the scheduled event.
    ///
    /// If changing to [`External`], then [`end_time`] and [`location`] must also be set.
    /// Otherwise, if changing to either [`StageInstance`] or [`Voice`], then [`channel_id`] is
    /// also required to be set.
    ///
    /// [`channel_id`]: EditScheduledEvent::channel_id
    /// [`end_time`]: EditScheduledEvent::end_time
    /// [`location`]: EditScheduledEvent::location
    ///
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    /// [`External`]: ScheduledEventType::External
    pub fn kind(mut self, kind: ScheduledEventType) -> Self {
        if let ScheduledEventType::External = kind {
            self.channel_id = Some(None);
        } else {
            self.entity_metadata = Some(None);
        }

        self.entity_type = Some(kind);
        self
    }

    /// Sets the status of the scheduled event.
    ///
    /// Only the following transitions are valid:
    ///
    /// [`Scheduled`] -> [`Active`]
    ///
    /// [`Active`] -> [`Completed`]
    ///
    /// [`Scheduled`] -> [`Canceled`]
    ///
    /// Additionally, if the event's status is [`Completed`] or [`Canceled`], then it can no longer
    /// be updated.
    ///
    /// [`Scheduled`]: ScheduledEventStatus::Scheduled
    /// [`Active`]: ScheduledEventStatus::Active
    /// [`Completed`]: ScheduledEventStatus::Completed
    /// [`Canceled`]: ScheduledEventStatus::Canceled
    pub fn status(mut self, status: ScheduledEventStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the location of the scheduled event.
    ///
    /// If the [`kind`] of the event is changed to [`External`], then this field is also required
    /// to be set and non-empty.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.entity_metadata = Some(Some(ScheduledEventMetadata {
            location: location.into(),
        }));
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
    ) -> Result<EditScheduledEvent<'a>> {
        self.image = Some(image.into().to_base64(&http.as_ref().client).await?);
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
