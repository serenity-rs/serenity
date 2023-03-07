use std::collections::HashMap;

#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
use crate::json::{json, Value};
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::guild::ScheduledEventType;
use crate::model::id::ChannelId;
use crate::model::Timestamp;
#[cfg(feature = "model")]
use crate::utils::encode_image;

#[derive(Clone, Debug)]
pub struct CreateScheduledEvent(pub HashMap<&'static str, Value>);

impl CreateScheduledEvent {
    /// Sets the channel id of the scheduled event. Required if the [`kind`] of the event is
    /// [`StageInstance`] or [`Voice`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    pub fn channel_id<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self.0.insert("channel_id", Value::from(channel_id.into().0));
        self
    }

    /// Sets the name of the scheduled event. Required to be set for event creation.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description<S: ToString>(&mut self, description: S) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    /// Sets the start time of the scheduled event. Required to be set for event creation.
    #[inline]
    pub fn start_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self._timestamp("scheduled_start_time", timestamp.into());
        self
    }

    /// Sets the end time of the scheduled event. Required if the [`kind`] of the event is
    /// [`External`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    #[inline]
    pub fn end_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self._timestamp("scheduled_end_time", timestamp.into());
        self
    }

    fn _timestamp(&mut self, field: &'static str, timestamp: Timestamp) {
        self.0.insert(field, Value::from(timestamp.to_string()));
    }

    /// Sets the entity type of the scheduled event. Required to be set for event creation.
    pub fn kind(&mut self, kind: ScheduledEventType) -> &mut Self {
        self.0.insert("entity_type", Value::from(kind.num()));
        self
    }

    /// Sets the location of the scheduled event. Required to be set and non-empty if the
    /// [`kind`] of the event is [`External`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
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

impl Default for CreateScheduledEvent {
    /// Creates a builder with default values, setting the `privacy_level` to `GUILD_ONLY`. As this
    /// is the only possible value of this field, it's only used at event creation, and we don't
    /// even parse it into the `ScheduledEvent` struct.
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert("privacy_level", Value::from(2));

        CreateScheduledEvent(map)
    }
}
