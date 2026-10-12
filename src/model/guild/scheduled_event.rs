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
    /// The definition for how often this event should recur.
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

/// Represents a definition for how often a scheduled event should recur.
///
/// There are currently many limitations to this system. Check [Discord docs] for further details
/// and examples of common rules.
///
/// [Discord docs]: https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RecurrenceRule {
    /// The starting time of the recurrence interval.
    pub start: Timestamp,
    /// The ending time of the recurrence interval.
    ///
    /// **Note**: Cannot be set externally currently.
    pub end: Option<Timestamp>,
    /// How often the event occurs.
    pub frequency: Frequency,
    /// The spacing between the events, defined by `frequency`.
    ///
    /// **Note**: Can only be set to [`Every`] unless frequency is [`Weekly`], in which case a
    /// value of [`EveryOther`] can be used to define an "every other week" recurrence. Due to the
    /// limitations placed on `by_weekday` this means that if you wish to use "every other week"
    /// functionality you can only do so for a single day.
    ///
    /// [`Weekly`]: Frequency::Weekly
    /// [`EveryOther`]: Interval::EveryOther
    /// [`Every`]: Interval::Every
    pub interval: Interval,
    /// The set of specific days within a week for the event to recur on.
    ///
    /// **Note**: Only valid for [`Daily`] and [`Weekly`] events.
    ///
    /// When used in a [`Daily`] event, the following sets of days are allowed:
    /// - Monday-Friday
    /// - Tuesday-Saturday
    /// - Sunday-Thursday
    /// - Friday-Saturday
    /// - Saturday-Sunday
    /// - Sunday-Monday
    ///
    /// When used in a [`Weekly`] event, the `by_weekday` array currently can only be a length of
    /// 1, meaning you can only select a single day within a week to have a recurring event on.
    /// For multiple days within a week, use [`Daily`].
    ///
    /// [`Daily`]: Frequency::Daily
    /// [`Weekly`]: Frequency::Weekly
    pub by_weekday: Option<FixedArray<Weekday, u8>>,
    /// The list of specific days within a specific week of a month to recur on.
    ///
    /// **Note**: Only valid for [`Monthly`] events. The `by_n_weekday` array currently can only
    /// be a length of 1, meaning you can only select a single day within a month to have a
    /// recurring event on.
    ///
    /// [`Monthly`]: Frequency::Monthly
    pub by_n_weekday: Option<FixedArray<NWeekday, u8>>,
    /// The set of specific months to recur on.
    ///
    /// **Note**: Only valid for [`Yearly`] events. Both `by_month` and `by_month_day` must be
    /// provided, and both arrays must have a length of 1, meaning you can only set a single date
    /// for annual events.
    ///
    /// [`Yearly`]: Frequency::Yearly
    pub by_month: Option<FixedArray<Month, u8>>,
    /// The set of specific dates within a month to recur on.
    ///
    /// **Note**: Only valid for [`Yearly`] events. Both `by_month` and `by_month_day` must be
    /// provided, and both arrays must have a length of 1, meaning you can only set a single date
    /// for annual events.
    ///
    /// [`Yearly`]: Frequency::Yearly
    pub by_month_day: Option<FixedArray<u8, u8>>,
    /// The set of days within a year to recur on (1-364).
    ///
    /// **Note**: Cannot be set externally currently.
    pub by_year_day: Option<FixedArray<u16, u16>>,
    /// The total amount of times that the event is allowed to recur before stopping.
    ///
    /// **Note**: Cannot be set externally currently.
    pub count: Option<NonMaxU32>,
}

enum_number! {
    /// The frequency of recurrence for a scheduled event recurrence rule.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-frequency).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Frequency {
        Yearly = 0,
        Monthly = 1,
        Weekly = 2,
        Daily = 3,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The spacing between events, defined by [`Frequency`].
    ///
    /// Currently, [`Interval::EveryOther`] is only valid for [`Frequency::Weekly`], a combination
    /// used to denote an event that occurs every other week.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Interval {
        Every = 1,
        EveryOther = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Represents a specific day within a week for a scheduled event to recur on.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-weekday).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Weekday {
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

/// Represents a specific day within a specific week of a month for a scheduled event to recur on.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-nweekday-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct NWeekday {
    /// The week to reoccur on.
    pub n: WeekNumber,
    /// The day within the week to reoccur on.
    pub day: Weekday,
}

enum_number! {
    /// Represents a specific week within a month.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum WeekNumber {
        One = 1,
        Two = 2,
        Three = 3,
        Four = 4,
        Five = 5,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Represents a specific month for a scheduled event to recur on.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object-guild-scheduled-event-recurrence-rule-month).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Month {
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
