//! Utilities for parsing and formatting RFC 3339 timestamps.
//!
//! The [`Timestamp`] newtype wraps `jiff::Timestamp` and adds a few useful methods for Discord
//! timestamps.
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

pub use jiff::Error as InnerError;
use jiff::Timestamp as JiffTimestamp;
use serde::{Deserialize, Serialize};

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;
// `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
const MAX_DISCORD_ID_MILLIS: u64 = 5_818_116_911_103;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[serde(transparent)]
pub struct Timestamp(JiffTimestamp);

impl Timestamp {
    /// Creates a new [`Timestamp`] from the number of milliseconds since 1970.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is invalid.
    pub fn from_millis(millis: i64) -> Result<Self, InvalidTimestamp> {
        JiffTimestamp::from_millisecond(millis).map(Self).map_err(|_| InvalidTimestamp)
    }

    pub(crate) fn from_discord_id(id: u64) -> Self {
        // This can't fail because of the bit shifting
        // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
        Self::from_millis(((id >> 22) + DISCORD_EPOCH) as i64).expect("can't fail")
    }

    /// Create a new `Timestamp` with the current date and time in UTC.
    #[must_use]
    pub fn now() -> Self {
        Self(JiffTimestamp::now())
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
        self.0.as_second()
    }

    /// Returns the number of non-leap milliseconds since January 1, 1970 0:00:00 UTC
    #[must_use]
    pub fn unix_timestamp_millis(&self) -> i64 {
        self.0.as_millisecond()
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
        let x = input.parse::<JiffTimestamp>().map_err(ParseError)?;
        Ok(Self(x))
    }

    #[must_use]
    pub fn to_rfc3339(self) -> String {
        self.0.to_string()
    }

    pub(crate) fn try_as_discord_id(self) -> Result<u64, TimestampOutOfRange> {
        let unix_millis = TryInto::<u64>::try_into(self.unix_timestamp_millis())
            .map_err(|_| TimestampOutOfRange)?;
        if !(DISCORD_EPOCH..=MAX_DISCORD_ID_MILLIS).contains(&unix_millis) {
            return Err(TimestampOutOfRange);
        }
        Ok((unix_millis - DISCORD_EPOCH) << 22)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_rfc3339())
    }
}

impl std::ops::Deref for Timestamp {
    type Target = JiffTimestamp;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<JiffTimestamp> for Timestamp {
    fn from(dt: JiffTimestamp) -> Self {
        Self(dt)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        let x = JiffTimestamp::default();
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

#[derive(Debug)]
pub struct TimestampOutOfRange;

impl std::error::Error for TimestampOutOfRange {}

impl fmt::Display for TimestampOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("timestamp is outside the valid range of Discord id timestamps")
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
    use super::{DISCORD_EPOCH, MAX_DISCORD_ID_MILLIS, Timestamp, TimestampOutOfRange};

    #[test]
    fn from_unix_timestamp() {
        let timestamp = Timestamp::from_unix_timestamp(1462015105).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1462015105);
        assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25Z");
    }

    #[test]
    fn test_try_as_discord_id() {
        // https://docs.discord.com/developers/reference#convert-snowflake-to-datetime
        let timestamp = Timestamp::from_millis(1462015105796).unwrap();
        let as_discord_id = timestamp.try_as_discord_id().unwrap();
        assert_eq!(as_discord_id, 175928847298985984);

        let too_early = Timestamp::from_millis((DISCORD_EPOCH - 1) as i64).unwrap();
        let result = too_early.try_as_discord_id();
        assert!(matches!(result, Err(TimestampOutOfRange)));

        let too_late = Timestamp::from_millis((MAX_DISCORD_ID_MILLIS + 1) as i64).unwrap();
        let result = too_late.try_as_discord_id();
        assert!(matches!(result, Err(TimestampOutOfRange)));

        // The Discord epoch itself should be a valid conversion, and should produce an id of 0.
        let just_right = Timestamp::from_millis(DISCORD_EPOCH as i64).unwrap();
        let zero = just_right.try_as_discord_id().unwrap();
        assert_eq!(zero, 0);
    }
}
