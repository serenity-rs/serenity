#[cfg(feature = "model")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::guild::{ScheduledEventMetadata, ScheduledEventType};
use crate::model::id::ChannelId;
use crate::model::Timestamp;
#[cfg(feature = "model")]
use crate::utils::encode_image;

#[derive(Clone, Debug, Serialize)]
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
    /// Sets the channel id of the scheduled event. Required if the [`kind`] of the event is
    /// [`StageInstance`] or [`Voice`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
    /// [`StageInstance`]: ScheduledEventType::StageInstance
    /// [`Voice`]: ScheduledEventType::Voice
    pub fn channel_id<C: Into<ChannelId>>(&mut self, channel_id: C) -> &mut Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Sets the name of the scheduled event. Required to be set for event creation.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the scheduled event.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start time of the scheduled event. Required to be set for event creation.
    #[inline]
    pub fn start_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self.scheduled_start_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the end time of the scheduled event. Required if the [`kind`] of the event is
    /// [`External`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    #[inline]
    pub fn end_time<T: Into<Timestamp>>(&mut self, timestamp: T) -> &mut Self {
        self.scheduled_end_time = Some(timestamp.into().to_string());
        self
    }

    /// Sets the entity type of the scheduled event. Required to be set for event creation.
    pub fn kind(&mut self, kind: ScheduledEventType) -> &mut Self {
        self.entity_type = Some(kind);
        self
    }

    /// Sets the location of the scheduled event. Required to be set and non-empty if the
    /// [`kind`] of the event is [`External`].
    ///
    /// [`kind`]: CreateScheduledEvent::kind
    /// [`External`]: ScheduledEventType::External
    pub fn location(&mut self, location: impl Into<String>) -> &mut Self {
        self.entity_metadata = Some(ScheduledEventMetadata {
            location: location.into(),
        });

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
        self.image = Some(encode_image(&image_data));
        Ok(self)
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
