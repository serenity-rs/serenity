use std::borrow::Cow;

use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#create-guild-scheduled-event)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateScheduledEvent<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_metadata: Option<CreateScheduledEventMetadata<'a>>,
    name: Cow<'a, str>,
    privacy_level: ScheduledEventPrivacyLevel,
    scheduled_start_time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_end_time: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    entity_type: ScheduledEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateScheduledEvent<'a> {
    /// Creates a builder with the provided kind, name, and start time, leaving all other fields
    /// empty.
    pub fn new(
        kind: ScheduledEventType,
        name: impl Into<Cow<'a, str>>,
        scheduled_start_time: impl Into<Timestamp>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type: kind,
            scheduled_start_time: scheduled_start_time.into(),

            image: None,
            channel_id: None,
            description: None,
            entity_metadata: None,
            scheduled_end_time: None,

            // Set the privacy level to `GUILD_ONLY`. As this is the only possible value of this
            // field, it's onlyu used at event creation, and we don't even parse it into the
            // `ScheduledEvent` struct.
            privacy_level: ScheduledEventPrivacyLevel::GuildOnly,

            audit_log_reason: None,
        }
    }

    /// Sets the channel id of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::StageInstance`] or [`ScheduledEventType::Voice`].
    pub fn channel_id(mut self, channel_id: ChannelId) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Sets the name of the scheduled event, replacing the current value as set in [`Self::new`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event, replacing the current value as set in
    /// [`Self::new`].
    pub fn start_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_start_time = timestamp.into();
        self
    }

    /// Sets the end time of the scheduled event. Required if [`Self::kind`] is
    /// [`ScheduledEventType::External`].
    pub fn end_time(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.scheduled_end_time = Some(timestamp.into());
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
    pub fn location(mut self, location: impl Into<Cow<'a, str>>) -> Self {
        self.entity_metadata = Some(CreateScheduledEventMetadata {
            location: Some(location.into()),
        });
        self
    }

    /// Sets the cover image for the scheduled event.
    pub fn image(mut self, image: &CreateAttachment<'_>) -> Self {
        self.image = Some(image.to_base64());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }

    /// Creates a new scheduled event in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Events] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, channel_id: GuildId) -> Result<ScheduledEvent> {
        http.create_scheduled_event(channel_id, &self, self.audit_log_reason).await
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub(crate) struct CreateScheduledEventMetadata<'a> {
    pub(crate) location: Option<Cow<'a, str>>,
}
