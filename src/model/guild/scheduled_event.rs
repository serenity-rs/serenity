use nonmax::NonMaxU64;

use crate::model::prelude::*;

/// Information about a guild scheduled event.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEvent {
    /// The Id of the scheduled event.
    pub id: ScheduledEventId,
    /// The Id of the guild that the event belongs to.
    pub guild_id: GuildId,
    /// The Id of the channel that the event belongs to, if any.
    pub channel_id: Option<ChannelId>,
    /// The Id of the User that created the scheduled event.
    ///
    /// Only `None` for events created before October 25th, 2021.
    pub creator_id: Option<UserId>,
    /// The name of the scheduled event.
    pub name: FixedString,
    /// The description of the scheduled event, if any.
    pub description: Option<FixedString>,
    /// The event's starting time.
    #[serde(rename = "scheduled_start_time")]
    pub start_time: Timestamp,
    /// The event's ending time; optional.
    #[serde(rename = "scheduled_end_time")]
    pub end_time: Option<Timestamp>,
    /// The privacy level of the scheduled event.
    pub privacy_level: ScheduledEventPrivacyLevel,
    /// The event's status; either Scheduled, Active, Completed, or Canceled.
    pub status: ScheduledEventStatus,
    /// The User that created the event.
    ///
    /// Only `None` for events created before October 25th, 2021.
    pub creator: Option<User>,
    /// The type of the event, indicating if it will take place in a Stage Instance, a Voice
    /// Channel, or at some External location.
    #[serde(rename = "entity_type")]
    pub kind: ScheduledEventType,
    /// The id of an entity associated with a guild scheduled event.
    pub entity_id: Option<GenericId>,
    /// Optional event location, only required for External events.
    #[serde(rename = "entity_metadata")]
    pub metadata: Option<ScheduledEventMetadata>,
    /// Number of users interested in the event.
    ///
    /// Only populated if `with_user_count` is set to true provided when calling
    /// [`GuildId::scheduled_event`] or [`GuildId::scheduled_events`].
    pub user_count: Option<NonMaxU64>,
    /// The hash of the event's cover image, if present.
    pub image: Option<ImageHash>,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-status).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ScheduledEventStatus {
        Scheduled = 1,
        Active = 2,
        Completed = 3,
        Canceled = 4,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-entity-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ScheduledEventType {
        StageInstance = 1,
        Voice = 2,
        External = 3,
        _ => Unknown(u8),
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-entity-metadata).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEventMetadata {
    #[serde(default)]
    pub location: Option<FixedString>,
}

/// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-user-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEventUser {
    #[serde(rename = "guild_scheduled_event_id")]
    pub event_id: ScheduledEventId,
    pub user: User,
    pub member: Option<Member>,
}

enum_number! {
    /// See [`ScheduledEvent::privacy_level`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-privacy-level).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ScheduledEventPrivacyLevel {
        GuildOnly = 2,
        _ => Unknown(u8),
    }
}
