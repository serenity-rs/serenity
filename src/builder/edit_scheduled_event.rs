use std::collections::HashMap;

#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::json::{json, Value, NULL};
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::guild::{ScheduledEventStatus, ScheduledEventType};
use crate::model::id::ChannelId;
use crate::model::Timestamp;
#[cfg(feature = "model")]
use crate::utils::encode_image;

#[derive(Clone, Debug, Default)]
pub struct EditScheduledEvent(pub HashMap<&'static str, Value>);

impl EditScheduledEvent {
    /// Sets the channel id of the scheduled event. If the [`kind`] of the event is changed from
    /// [`External`] to either [`StageInstance`] or [`Voice`], then this field is also required.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    /// [`External`]: ScheduledEventType::External
    pub fn channel_id<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self.0.insert("channel_id", Value::from(channel_id.into().0));
        self
    }

    /// Sets the name of the scheduled event.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description<S: ToString>(&mut self, description: S) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    /// Sets the start time of the scheduled event.
    #[inline]
    pub fn start_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self._timestamp("scheduled_start_time", timestamp.into());
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
        self._timestamp("scheduled_end_time", timestamp.into());
        self
    }

    fn _timestamp(&mut self, field: &'static str, timestamp: Timestamp) {
        self.0.insert(field, Value::from(timestamp.to_string()));
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
        match kind {
            ScheduledEventType::External => self.0.insert("channel_id", NULL),
            _ => self.0.insert("entity_metadata", NULL),
        };
        self.0.insert("entity_type", Value::from(kind.num()));
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
        self.0.insert("status", Value::from(status.num()));
        self
    }

    /// Sets the location of the scheduled event.
    ///
    /// If the [`kind`] of the event is changed to [`External`], then this field is also required
    /// to be set and non-empty.
    ///
    /// [`kind`]: EditScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    pub fn location<S: ToString>(&mut self, location: S) -> &mut Self {
        let obj = json!({
            "location": location.to_string(),
        });
        self.0.insert("entity_metadata", obj);
        self
    }

    /// Sets the cover image for the scheduled event.
    ///
    /// # Errors
    ///
    /// May error if the icon is a URL and the HTTP request fails, or if the image is a file
    /// on a path that doesn't exist.
    #[cfg(feature = "model")]
    pub async fn image<'a>(
        &mut self,
        http: impl AsRef<Http>,
        image: impl Into<AttachmentType<'a>>,
    ) -> Result<&mut Self> {
        let image_data = image.into().data(&http.as_ref().client).await?;
        self.0.insert("image", Value::from(encode_image(&image_data)));
        Ok(self)
    }
}
