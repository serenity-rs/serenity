use std::borrow::Cow;

use arrayvec::ArrayVec;

use super::DataUri;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#create-guild-scheduled-event)
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
    image: Option<DataUri<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_rule: Option<CreateRecurrenceRule<'a>>,

    #[serde(skip)]
    audit_log_reason: Option<Cow<'a, str>>,
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
            recurrence_rule: None,

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
    pub fn image(mut self, image: DataUri<'a>) -> Self {
        self.image = Some(image);
        self
    }

    /// Sets the recurrence rule for the scheduled event.
    pub fn recurrence_rule(mut self, rule: CreateRecurrenceRule<'a>) -> Self {
        self.recurrence_rule = Some(rule);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.audit_log_reason = Some(reason.into());
        self
    }

    pub fn into_owned(self) -> CreateScheduledEvent<'static> {
        let Self {
            channel_id,
            entity_metadata,
            name,
            privacy_level,
            scheduled_start_time,
            scheduled_end_time,
            description,
            entity_type,
            image,
            recurrence_rule,
            audit_log_reason,
        } = self;
        CreateScheduledEvent {
            channel_id,
            entity_metadata: entity_metadata.map(CreateScheduledEventMetadata::into_owned),
            name: name.into_owned().into(),
            privacy_level,
            scheduled_start_time,
            scheduled_end_time,
            description: description.map(|d| d.into_owned().into()),
            entity_type,
            image: image.map(DataUri::into_owned),
            recurrence_rule: recurrence_rule.map(CreateRecurrenceRule::into_owned),
            audit_log_reason: audit_log_reason.map(|r| Cow::Owned(r.into_owned())),
        }
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
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<ScheduledEvent> {
        http.create_scheduled_event(guild_id, &self, self.audit_log_reason.as_deref()).await
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub(crate) struct CreateScheduledEventMetadata<'a> {
    pub(crate) location: Option<Cow<'a, str>>,
}

impl CreateScheduledEventMetadata<'_> {
    pub fn into_owned(self) -> CreateScheduledEventMetadata<'static> {
        CreateScheduledEventMetadata {
            location: self.location.map(|l| l.into_owned().into()),
        }
    }
}

/// [Discord docs](https://docs.discord.com/developers/resources/guild-scheduled-event#guild-scheduled-event-recurrence-rule-object)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateRecurrenceRule<'a> {
    start: Timestamp,
    // `end` cannot currently be set externally.
    frequency: Frequency,
    interval: Interval,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    by_weekday: Cow<'a, [Weekday]>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    by_n_weekday: ArrayVec<NWeekday, 1>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    by_month: ArrayVec<Month, 1>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    by_month_day: ArrayVec<u8, 1>,
    // `by_year_day` cannot currently be set externally.
    // `count` cannot currently be set externally.
}

impl<'a> CreateRecurrenceRule<'a> {
    /// Creates a builder with the start, frequency, and interval, leaving all other fields empty.
    ///
    /// **Note**: An interval of [`EveryOther`] is only valid for [`Frequency::Weekly`]. If any
    /// other frequency is selected, `interval` will be set to [`Every`]. See [`Self::interval`]
    /// for details.
    ///
    /// [`EveryOther`]: Interval::EveryOther
    /// [`Every`]: Interval::Every
    pub fn new(start: impl Into<Timestamp>, frequency: Frequency, interval: Interval) -> Self {
        Self {
            start: start.into(),
            frequency,
            interval: if frequency == Frequency::Weekly { interval } else { Interval::Every },
            by_weekday: Cow::default(),
            by_n_weekday: ArrayVec::default(),
            by_month: ArrayVec::default(),
            by_month_day: ArrayVec::default(),
        }
    }

    /// Sets the starting time of the recurrence interval, replacing the current value as set in
    /// [`Self::new`].
    ///
    /// **Note**: This determines the starting time of all recurrences and is separate from the
    /// starting time of the event. It can be the same time or a different time.
    pub fn start(mut self, start: impl Into<Timestamp>) -> Self {
        self.start = start.into();
        self
    }

    /// Sets how often the event occurs, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Setting this to any frequency other than [`Weekly`] will result in `interval`
    /// being set to [`Every`], which is the only valid value for other frequencies.
    ///
    /// [`Weekly`]: Frequency::Weekly
    /// [`Every`]: Interval::Every
    pub fn frequency(mut self, frequency: Frequency) -> Self {
        if frequency != Frequency::Weekly {
            self.interval = Interval::Every;
        }
        self.frequency = frequency;
        self
    }

    /// Sets the spacing between the events, defined by frequency, replacing the current value
    /// as set in [`Self::new`].
    ///
    /// **Note**: Can only be set to [`Every`] unless frequency is [`Weekly`], in which case a
    /// value of [`EveryOther`] can be used to define an "every other week" recurrence. Due to the
    /// limitations placed on `by_weekday` this means that if you wish to use "every other week"
    /// functionality you can only do so for a single day.
    ///
    /// [`Weekly`]: Frequency::Weekly
    /// [`EveryOther`]: Interval::EveryOther
    /// [`Every`]: Interval::Every
    pub fn interval(mut self, interval: Interval) -> Self {
        // Checking against frequency could cause some shenanigans depending on the order of the
        // setters used, so just leave validation to the API.
        self.interval = interval;
        self
    }

    /// Sets the specific days within a week for the event to recur on.
    ///
    /// Only valid for [`Daily`] and [`Weekly`] events.
    ///
    /// When used in a [`Daily`] event, the following sets of days are allowed:
    /// - Monday-Friday
    /// - Tuesday-Saturday
    /// - Sunday-Thursday
    /// - Friday-Saturday
    /// - Saturday-Sunday
    /// - Sunday-Monday
    ///
    /// For example, the following rule would recur every Saturday and Sunday:
    ///
    /// ```rust
    /// # use serenity::all::*;
    /// # let start = Timestamp::now();
    /// let rule = CreateRecurrenceRule::new(start, Frequency::Daily, Interval::Every)
    ///     .by_weekday(&[Weekday::Saturday, Weekday::Sunday]);
    /// ```
    ///
    /// When used in a [`Weekly`] event, the provided array currently can only be a length of 1,
    /// meaning you can only select a single day within a week to have a recurring event on. For
    /// multiple days within a week, use [`Daily`].
    ///
    /// For example, the following rule would recur every other Thursday:
    ///
    /// ```rust
    /// # use serenity::all::*;
    /// # let start = Timestamp::now();
    /// let rule = CreateRecurrenceRule::new(start, Frequency::Weekly, Interval::EveryOther)
    ///     .by_weekday(&[Weekday::Thursday]);
    /// ```
    ///
    /// [`Daily`]: Frequency::Daily
    /// [`Weekly`]: Frequency::Weekly
    pub fn by_weekday(mut self, weekday: impl Into<Cow<'a, [Weekday]>>) -> Self {
        self.by_weekday = weekday.into();
        self
    }

    /// Sets the specific day within a specific week of the month to recur on.
    ///
    /// Frequency will be set to [`Monthly`], which is the only valid value for this type of rule.
    /// You can only select a single day within a month to have a recurring event on.
    ///
    /// For example, the following rule would recur monthly on the second Friday:
    ///
    /// ```rust
    /// # use serenity::all::*;
    /// # let start = Timestamp::now();
    /// let rule = CreateRecurrenceRule::new(start, Frequency::Monthly, Interval::Every)
    ///     .by_n_weekday(WeekNumber::Two, Weekday::Friday);
    /// ```
    ///
    /// [`Monthly`]: Frequency::Monthly
    pub fn by_n_weekday(mut self, n: WeekNumber, day: Weekday) -> Self {
        self.frequency = Frequency::Monthly;
        self.by_n_weekday = ArrayVec::from([NWeekday {
            n,
            day,
        }]);
        self
    }

    /// Sets the specific month and date within that month to recur on.
    ///
    /// Frequency will be set to [`Yearly`], which is the only valid value for this type of rule.
    /// You can only set a single date for annual events.
    ///
    /// For example, the following rule would recur annually on 22 July:
    ///
    /// ```rust
    /// # use serenity::all::*;
    /// # let start = Timestamp::now();
    /// let rule = CreateRecurrenceRule::new(start, Frequency::Yearly, Interval::Every)
    ///     .by_month(Month::July, 22);
    /// ```
    ///
    /// [`Yearly`]: Frequency::Yearly
    pub fn by_month(mut self, month: Month, day: u8) -> Self {
        self.frequency = Frequency::Yearly;
        self.by_month = ArrayVec::from([month]);
        self.by_month_day = ArrayVec::from([day]);
        self
    }

    pub fn into_owned(self) -> CreateRecurrenceRule<'static> {
        CreateRecurrenceRule {
            start: self.start,
            frequency: self.frequency,
            interval: self.interval,
            by_weekday: self.by_weekday.into_owned().into(),
            by_n_weekday: self.by_n_weekday,
            by_month: self.by_month,
            by_month_day: self.by_month_day,
        }
    }
}
