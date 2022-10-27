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
//! let timestamp: Timestamp = GuildId::new(175928847299117063).created_at();
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
use std::str::FromStr;

#[cfg(feature = "chrono")]
pub use chrono::ParseError as InnerError;
#[cfg(feature = "chrono")]
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
#[cfg(not(feature = "chrono"))]
pub use dep_time::error::Parse as InnerError;
#[cfg(not(feature = "chrono"))]
use dep_time::{format_description::well_known::Rfc3339, serde::rfc3339, Duration, OffsetDateTime};
use serde::{Deserialize, Serialize};

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Timestamp(
    #[cfg(feature = "chrono")] DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    #[serde(with = "rfc3339")]
    OffsetDateTime,
);

impl Timestamp {
    /// Creates a new [`Timestamp`] from the number of milliseconds since 1970.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is invalid.
    pub fn from_millis(millis: i64) -> Result<Self, InvalidTimestamp> {
        #[cfg(feature = "chrono")]
        let x = Utc.timestamp_millis_opt(millis).single();
        #[cfg(not(feature = "chrono"))]
        let x = OffsetDateTime::from_unix_timestamp_nanos(
            Duration::milliseconds(millis).whole_nanoseconds(),
        )
        .ok();
        x.map(Self).ok_or(InvalidTimestamp)
    }

    pub(crate) fn from_discord_id(id: u64) -> Self {
        // This can't fail because of the bit shifting
        // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
        Self::from_millis(((id >> 22) + DISCORD_EPOCH) as i64).expect("can't fail")
    }

    /// Create a new `Timestamp` with the current date and time in UTC.
    #[must_use]
    pub fn now() -> Self {
        #[cfg(feature = "chrono")]
        let x = Utc::now();
        #[cfg(not(feature = "chrono"))]
        let x = OffsetDateTime::now_utc();
        Self(x)
    }

    /// Creates a new [`Timestamp`] from a UNIX timestamp (seconds since 1970).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is invalid.
    pub fn from_unix_timestamp(secs: i64) -> Result<Self, InvalidTimestamp> {
        Self::from_millis(secs * 1000)
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
    #[must_use]
    pub fn unix_timestamp(&self) -> i64 {
        #[cfg(feature = "chrono")]
        let x = self.0.timestamp();
        #[cfg(not(feature = "chrono"))]
        let x = self.0.unix_timestamp();
        x
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
        #[cfg(feature = "chrono")]
        let x = DateTime::parse_from_rfc3339(input).map_err(ParseError)?.with_timezone(&Utc);
        #[cfg(not(feature = "chrono"))]
        let x = OffsetDateTime::parse(input, &Rfc3339).map_err(ParseError)?;
        Ok(Self(x))
    }

    #[must_use]
    pub fn to_rfc3339(&self) -> Option<String> {
        #[cfg(feature = "chrono")]
        let x = self.0.to_rfc3339_opts(SecondsFormat::Millis, true);
        #[cfg(not(feature = "chrono"))]
        let x = self.0.format(&Rfc3339).ok()?;
        Some(x)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_rfc3339().ok_or(std::fmt::Error)?)
    }
}

impl std::ops::Deref for Timestamp {
    #[cfg(feature = "chrono")]
    type Target = DateTime<Utc>;
    #[cfg(not(feature = "chrono"))]
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "chrono")]
impl<Tz: TimeZone> From<DateTime<Tz>> for Timestamp {
    fn from(dt: DateTime<Tz>) -> Self {
        Self(dt.with_timezone(&Utc))
    }
}
#[cfg(not(feature = "chrono"))]
impl From<OffsetDateTime> for Timestamp {
    fn from(dt: OffsetDateTime) -> Self {
        Self(dt)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        #[cfg(feature = "chrono")]
        let x = DateTime::default();
        #[cfg(not(feature = "chrono"))]
        let x = OffsetDateTime::UNIX_EPOCH;
        Self(x)
    }
}

#[derive(Debug)]
pub struct InvalidTimestamp;

impl std::error::Error for InvalidTimestamp {}

impl fmt::Display for InvalidTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid UNIX timestamp value")
    }
}

/// Signifies the failure to parse the `Timestamp` from an RFC 3339 string.
#[derive(Debug)]
pub struct ParseError(InnerError);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for Timestamp {
    type Err = ParseError;

    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Timestamp::parse(s)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for Timestamp {
    type Error = ParseError;

    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Timestamp::parse(s)
    }
}

impl From<&Timestamp> for Timestamp {
    fn from(ts: &Timestamp) -> Self {
        *ts
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn from_unix_timestamp() {
        let timestamp = Timestamp::from_unix_timestamp(1462015105).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1462015105);
        if cfg!(feature = "chrono") {
            assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.000Z");
        } else {
            assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25Z");
        }
    }
}
