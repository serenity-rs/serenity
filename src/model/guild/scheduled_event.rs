use nonmax::{NonMaxU32, NonMaxU64};

use crate::model::prelude::*;

/// Information about a guild scheduled event.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-object).
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
    /// The recurrence rule of the event, if the event recurs.
    pub recurrence_rule: Option<RecurrenceRule>,
}

impl extract_map::ExtractKey<ScheduledEventId> for ScheduledEvent {
    fn extract_key(&self) -> &ScheduledEventId {
        &self.id
    }
}

enum_number! {
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-status).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-entity-types).
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

/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-entity-metadata).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScheduledEventMetadata {
    #[serde(default)]
    pub location: Option<FixedString>,
}

/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-user-object).
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
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-privacy-level).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum ScheduledEventPrivacyLevel {
        GuildOnly = 2,
        _ => Unknown(u8),
    }
}

/// Defines how a scheduled event recurs.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RecurrenceRule {
    /// Starting time of the recurrence interval.
    pub start: Timestamp,
    /// Ending time of the recurrence interval, if any.
    pub end: Option<Timestamp>,
    /// How often the event occurs.
    pub frequency: RecurrenceRuleFrequency,
    /// Spacing between recurrences, in units of [`Self::frequency`]. Must be `1` except for
    /// weekly events, where `2` is also valid to indicate bi-weekly recurrence.
    pub interval: u32,
    /// Weekdays the event recurs on.
    #[serde(default)]
    pub by_weekday: Option<FixedArray<RecurrenceRuleWeekday, u8>>,
    /// Specific weekdays within a month the event recurs on.
    #[serde(default)]
    pub by_n_weekday: Option<FixedArray<RecurrenceRuleNWeekday, u8>>,
    /// Months within a year the event recurs on (1-12).
    #[serde(default)]
    pub by_month: Option<FixedArray<RecurrenceRuleMonth, u8>>,
    /// Days within a month the event recurs on (1-31).
    #[serde(default)]
    pub by_month_day: Option<FixedArray<u8, u8>>,
    /// Days within a year the event recurs on (1-364).
    #[serde(default)]
    pub by_year_day: Option<FixedArray<u16, u16>>,
    /// Total amount of times the event is allowed to recur before stopping.
    pub count: Option<NonMaxU32>,
}

enum_number! {
    /// How often a scheduled event recurs.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-frequency).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum RecurrenceRuleFrequency {
        Yearly = 0,
        Monthly = 1,
        Weekly = 2,
        Daily = 3,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Weekday for a recurrence rule.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-weekday).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum RecurrenceRuleWeekday {
        Monday = 0,
        Tuesday = 1,
        Wednesday = 2,
        Thursday = 3,
        Friday = 4,
        Saturday = 5,
        Sunday = 6,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Month within a year for a recurrence rule.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-month).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum RecurrenceRuleMonth {
        January = 1,
        February = 2,
        March = 3,
        April = 4,
        May = 5,
        June = 6,
        July = 7,
        August = 8,
        September = 9,
        October = 10,
        November = 11,
        December = 12,
        _ => Unknown(u8),
    }
}

/// Specific weekday within a month for a recurrence rule (e.g. "the 2nd Tuesday").
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-nweekday-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RecurrenceRuleNWeekday {
    /// The week to recur on (1-5).
    pub n: u8,
    /// The day within the week to recur on.
    pub day: RecurrenceRuleWeekday,
}
