#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::guild::{ScheduledEventMetadata, ScheduledEventStatus, ScheduledEventType};
use crate::model::id::ChannelId;
use crate::model::Timestamp;
#[cfg(feature = "model")]
use crate::utils::encode_image;

#[derive(Clone, Debug, Default, Serialize)]
pub struct EditScheduledEvent {
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
}

impl EditScheduledEvent {
    /// Sets the channel id of the scheduled event. If the [`kind`] of the event is changed from
    /// [`External`] to either [`StageInstance`] or [`Voice`], then this field is also required.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    /// [`External`]: ScheduledEventType::External
    pub fn channel_id<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self.channel_id = Some(Some(channel_id.into()));
        self
    }

    /// Sets the name of the scheduled event.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event.
    #[inline]
    pub fn start_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
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
    pub fn end_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
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
    pub fn kind(&mut self, kind: ScheduledEventType) -> &mut Self {
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
    pub fn status(&mut self, status: ScheduledEventStatus) -> &mut Self {
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
    pub fn location(&mut self, location: impl Into<String>) -> &mut Self {
        self.entity_metadata = Some(Some(ScheduledEventMetadata {
            location: location.into(),
        }));
        self
    }

    /// Sets the cover image for the scheduled event.
    ///
    /// # Errors
    ///
    /// May error if a URL is given and the HTTP request fails, or if a path is given to a file
    /// that does not exist.
    #[cfg(feature = "model")]
    pub async fn image<'a>(
        &mut self,
        http: impl AsRef<Http>,
        image: impl Into<AttachmentType<'a>>,
    ) -> Result<&mut Self> {
        let image_data = image.into().data(&http.as_ref().client).await?;
        self.image = Some(encode_image(&image_data));
        Ok(self)
    }

    /// Sets the cover image for the scheduled event. Requires the input be a base64-encoded image
    /// that is in either JPG, GIF, or PNG format.
    #[cfg(not(feature = "model"))]
    pub fn image(&mut self, image: String) -> &mut Self {
        self.image = Some(image);
        self
    }
}
