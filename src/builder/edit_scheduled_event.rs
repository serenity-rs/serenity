#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#modify-guild-scheduled-event)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditScheduledEvent<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_metadata: Option<Option<ScheduledEventMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<ScheduledEventPrivacyLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
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

    /// The privacy level of the scheduled event
    pub fn privacy_level(mut self, privacy_level: ScheduledEventPrivacyLevel) -> Self {
        self.privacy_level = Some(privacy_level);
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

    /// Sets the entity type of the scheduled event.
    ///
    /// If changing to [`External`], then [`end_time`] and [`location`] must also be set.
    /// Otherwise, if changing to either [`StageInstance`] or [`Voice`], then [`channel_id`] is
    /// also required to be set.
    ///
    /// See the [Discord docs] for more details.
    ///
    /// [`channel_id`]: EditScheduledEvent::channel_id
    /// [`end_time`]: EditScheduledEvent::end_time
    /// [`location`]: EditScheduledEvent::location
    ///
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    /// [`External`]: ScheduledEventType::External
    /// [Discord docs]: https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-field-requirements-by-entity-type
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
            location: Some(location.into().into()),
        }));
        self
    }

    /// Sets the cover image for the scheduled event.
    pub fn image(mut self, image: &CreateAttachment) -> Self {
        self.image = Some(image.to_base64());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditScheduledEvent<'_> {
    type Context<'ctx> = (GuildId, ScheduledEventId);
    type Built = ScheduledEvent;

    /// Modifies a scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: If the event was created by the current user, requires either [Create Events] or
    /// the [Manage Events] permission. Otherwise, the [Manage Events] permission is required.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().edit_scheduled_event(ctx.0, ctx.1, &self, self.audit_log_reason).await
    }
}
