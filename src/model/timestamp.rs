//! Utilities for parsing and formatting RFC 3339 timestamps.
//!
//! The [`Timestamp`] newtype wraps `chrono::DateTime<Utc>` or `time::OffsetDateTime` if the `time`
//! feature is enabled.
//!
//! # Formatting
//! ```
//! # use serenity::model::id::GuildId;
//! # use serenity::model::Timestamp;
//! #
//! let timestamp: Timestamp = GuildId(175928847299117063).created_at();
//! assert_eq!(timestamp.unix_timestamp(), 1462015105);
//! assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
//! ```
//!
//! # Parsing RFC 3339 string
//! ```
//! # use serenity::model::Timestamp;
//! #
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
//!
//! let timestamp: Timestamp = "2016-04-30T11:18:25Z".parse().unwrap();
//! let timestamp: Timestamp = "2016-04-30T11:18:25+00:00".parse().unwrap();
//! let timestamp: Timestamp = "2016-04-30T11:18:25.796Z".parse().unwrap();
//!
//! assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
//! assert!(Timestamp::parse("2016-04-30T11:18").is_err());
//! ```

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[cfg(not(feature = "time"))]
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use time::format_description::well_known::Rfc3339;
#[cfg(feature = "time")]
use time::serde::rfc3339;
#[cfg(feature = "time")]
use time::{Duration, OffsetDateTime};

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;

/// Representation of a Unix timestamp.
///
/// The struct implements the `std::fmt::Display` trait to format the underlying type as
/// an RFC 3339 date and string such as `2016-04-30T11:18:25.796Z`.
///
/// ```
/// # use serenity::model::id::GuildId;
/// # use serenity::model::Timestamp;
/// #
/// let timestamp: Timestamp = GuildId(175928847299117063).created_at();
/// assert_eq!(timestamp.unix_timestamp(), 1462015105);
/// assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
/// ```
#[cfg(not(feature = "time"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

/// Representation of a Unix timestamp.
///
/// The struct implements the `std::fmt::Display` trait to format the underlying type as
/// an RFC 3339 date and string such as `2016-04-30T11:18:25.796Z`.
///
/// ```
/// # use serenity::model::id::GuildId;
/// # use serenity::model::Timestamp;
/// #
/// let timestamp: Timestamp = GuildId(175928847299117063).created_at();
/// assert_eq!(timestamp.unix_timestamp(), 1462015105);
/// assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
/// ```
#[cfg(feature = "time")]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Timestamp(#[serde(with = "rfc3339")] OffsetDateTime);

#[cfg(feature = "time")]
impl Timestamp {
    pub(crate) fn from_discord_id(id: u64) -> Timestamp {
        let ns = Duration::milliseconds(((id >> 22) + DISCORD_EPOCH) as i64).whole_nanoseconds();
        // This can't fail because of the bit shifting
        // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
        Self(OffsetDateTime::from_unix_timestamp_nanos(ns).expect("can't fail"))
    }

    /// Create a new `Timestamp` with the current date and time in UTC.
    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
    pub fn unix_timestamp(&self) -> i64 {
        self.0.unix_timestamp()
    }

    /// Parse a timestamp from an RFC 3339 date and time string.
    ///
    /// # Examples
    /// ```
    /// # use serenity::model::Timestamp;
    /// #
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
    ///
    /// assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
    /// assert!(Timestamp::parse("2016-04-30T11:18").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string is not a valid RFC 3339 date and time string.
    pub fn parse(input: &str) -> Result<Timestamp, ParseError> {
        OffsetDateTime::parse(input, &Rfc3339).map(Self).map_err(ParseError)
    }
}

#[cfg(not(feature = "time"))]
impl Timestamp {
    pub(crate) fn from_discord_id(id: u64) -> Timestamp {
        Self(Utc.timestamp_millis(((id >> 22) + DISCORD_EPOCH) as i64))
    }

    /// Create a new `Timestamp` with the current date and time in UTC.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
    pub fn unix_timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    /// Parse a timestamp from an RFC 3339 date and time string.
    ///
    /// # Examples
    /// ```
    /// # use serenity::model::Timestamp;
    /// #
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
    ///
    /// assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
    /// assert!(Timestamp::parse("2016-04-30T11:18").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string is not a valid RFC 3339 date and time string.
    pub fn parse(input: &str) -> Result<Timestamp, ParseError> {
        DateTime::parse_from_rfc3339(input).map(|d| Self(d.with_timezone(&Utc))).map_err(ParseError)
    }
}

#[cfg(not(feature = "time"))]
use chrono::ParseError as InnerError;
#[cfg(feature = "time")]
use time::error::Parse as InnerError;

/// Signifies the failure to parse the `Timestamp` from an RFC 3339 string.
#[derive(Debug)]
pub struct ParseError(InnerError);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "time")]
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.format(&Rfc3339).map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

#[cfg(not(feature = "time"))]
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use chrono::SecondsFormat;
        let s = self.0.to_rfc3339_opts(SecondsFormat::Millis, true);
        f.write_str(&s)
    }
}

#[cfg(not(feature = "time"))]
impl Deref for Timestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "time")]
impl Deref for Timestamp {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Timestamp {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Timestamp::parse(s)
    }
}

impl From<String> for Timestamp {
    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    ///
    /// Panics on invalid value.
    fn from(s: String) -> Self {
        #[allow(clippy::unwrap_used)]
        Timestamp::parse(&s).unwrap()
    }
}

impl<'a> From<&'a str> for Timestamp {
    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    ///
    /// Panics on invalid value.
    fn from(s: &'a str) -> Self {
        #[allow(clippy::unwrap_used)]
        Timestamp::parse(s).unwrap()
    }
}

impl From<&Timestamp> for Timestamp {
    fn from(ts: &Timestamp) -> Self {
        *ts
    }
}

#[cfg(not(feature = "time"))]
impl<Tz: TimeZone> From<DateTime<Tz>> for Timestamp {
    fn from(dt: DateTime<Tz>) -> Self {
        Self(dt.with_timezone(&Utc))
    }
}

#[cfg(feature = "time")]
impl From<OffsetDateTime> for Timestamp {
    fn from(dt: OffsetDateTime) -> Self {
        Self(dt)
    }
}
