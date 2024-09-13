use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use aformat::{aformat_into, ArrayString, ToArrayString};

use crate::model::Timestamp;

/// Represents a combination of a timestamp and a style for formatting time in messages.
///
/// [Discord docs](https://discord.com/developers/docs/reference#message-formatting-formats).
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct FormattedTimestamp {
    timestamp: i64,
    style: Option<FormattedTimestampStyle>,
}

/// Enum representing various styles for formatting time in messages.
///
/// [Discord docs](https://discord.com/developers/docs/reference#message-formatting-timestamp-styles).
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FormattedTimestampStyle {
    /// Represents a short time format, e.g., "12:34 PM".
    ShortTime,
    /// Represents a long time format, e.g., "12:34:56 PM".
    LongTime,
    /// Represents a short date format, e.g., "2023-11-17".
    ShortDate,
    /// Represents a long date format, e.g., "November 17, 2023".
    LongDate,
    /// Represents a short date and time format, e.g., "November 17, 2023 12:34 PM".
    #[default]
    ShortDateTime,
    /// Represents a long date and time format, e.g., "Thursday, November 17, 2023 12:34 PM".
    LongDateTime,
    /// Represents a relative time format, indicating the time relative to the current moment,
    /// e.g., "2 hours ago" or "in 2 hours".
    RelativeTime,
}

impl FormattedTimestamp {
    /// Creates a new [`FormattedTimestamp`] instance from the given [`Timestamp`] and
    /// [`FormattedTimestampStyle`].
    #[must_use]
    pub fn new(timestamp: Timestamp, style: Option<FormattedTimestampStyle>) -> Self {
        Self {
            timestamp: timestamp.unix_timestamp(),
            style,
        }
    }

    /// Creates a new [`FormattedTimestamp`] instance representing the current timestamp with the
    /// default style.
    #[must_use]
    pub fn now() -> Self {
        Self {
            timestamp: Timestamp::now().unix_timestamp(),
            style: None,
        }
    }

    /// Returns the timestamp of this [`FormattedTimestamp`].
    #[must_use]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the style of this [`FormattedTimestamp`].
    #[must_use]
    pub fn style(&self) -> Option<FormattedTimestampStyle> {
        self.style
    }
}

impl From<Timestamp> for FormattedTimestamp {
    /// Creates a new [`FormattedTimestamp`] instance from the given [`Timestamp`] with the default
    /// style.
    fn from(timestamp: Timestamp) -> Self {
        Self {
            timestamp: timestamp.unix_timestamp(),
            style: None,
        }
    }
}

impl ToArrayString for FormattedTimestamp {
    const MAX_LENGTH: usize = 27;
    type ArrayString = ArrayString<27>;

    fn to_arraystring(self) -> Self::ArrayString {
        let mut out = Self::ArrayString::new();
        if let Some(style) = self.style {
            aformat_into!(out, "<t:{}:{}>", self.timestamp, style);
        } else {
            aformat_into!(out, "<t:{}>", self.timestamp);
        }

        out
    }
}

impl fmt::Display for FormattedTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_arraystring())
    }
}

impl ToArrayString for FormattedTimestampStyle {
    const MAX_LENGTH: usize = 1;
    type ArrayString = ArrayString<1>;

    fn to_arraystring(self) -> Self::ArrayString {
        let style = match self {
            Self::ShortTime => "t",
            Self::LongTime => "T",
            Self::ShortDate => "d",
            Self::LongDate => "D",
            Self::ShortDateTime => "f",
            Self::LongDateTime => "F",
            Self::RelativeTime => "R",
        };

        ArrayString::from(style)
            .expect("One ASCII character should fit into an ArrayString of one capacity")
    }
}

impl fmt::Display for FormattedTimestampStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_arraystring())
    }
}

/// An error that can occur when parsing a [`FormattedTimestamp`] from a string.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FormattedTimestampParseError {
    string: String,
}

impl StdError for FormattedTimestampParseError {}

impl fmt::Display for FormattedTimestampParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid formatted timestamp {:?}", self.string)
    }
}

fn parse_formatted_timestamp(s: &str) -> Option<FormattedTimestamp> {
    // A formatted timestamp looks like: <t:TIMESTAMP> or <t:TIMESTAMP:STYLE>
    let inner = s.strip_prefix("<t:")?.strip_suffix('>')?;

    Some(match inner.split_once(':') {
        Some((timestamp, style)) => FormattedTimestamp {
            timestamp: timestamp.parse().ok()?,
            style: Some(style.parse().ok()?),
        },
        None => FormattedTimestamp {
            timestamp: inner.parse().ok()?,
            style: None,
        },
    })
}

impl FromStr for FormattedTimestamp {
    type Err = FormattedTimestampParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_formatted_timestamp(s) {
            Some(x) => Ok(x),
            None => Err(FormattedTimestampParseError {
                string: s.into(),
            }),
        }
    }
}

impl FromStr for FormattedTimestampStyle {
    type Err = FormattedTimestampParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" => Ok(Self::ShortTime),
            "T" => Ok(Self::LongTime),
            "d" => Ok(Self::ShortDate),
            "D" => Ok(Self::LongDate),
            "f" => Ok(Self::ShortDateTime),
            "F" => Ok(Self::LongDateTime),
            "R" => Ok(Self::RelativeTime),
            _ => Err(FormattedTimestampParseError {
                string: s.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use aformat::aformat;

    use super::*;

    #[test]
    fn test_message_time() {
        let timestamp = Timestamp::now();

        let time = FormattedTimestamp::new(timestamp, Some(FormattedTimestampStyle::ShortDateTime));
        let time_str = time.to_arraystring();

        assert_eq!(
            time_str,
            aformat!(
                "<t:{}:{}>",
                timestamp.unix_timestamp(),
                FormattedTimestampStyle::ShortDateTime
            )
        );

        let unstyled = FormattedTimestamp::new(timestamp, None);

        let unstyled_str = unstyled.to_arraystring();

        assert_eq!(&*unstyled_str, &*aformat!("<t:{}>", timestamp.unix_timestamp()));
    }

    #[test]
    fn test_message_time_style() {
        assert_eq!(&*FormattedTimestampStyle::ShortTime.to_arraystring(), "t");
        assert_eq!(&*FormattedTimestampStyle::LongTime.to_arraystring(), "T");
        assert_eq!(&*FormattedTimestampStyle::ShortDate.to_arraystring(), "d");
        assert_eq!(&*FormattedTimestampStyle::LongDate.to_arraystring(), "D");
        assert_eq!(&*FormattedTimestampStyle::ShortDateTime.to_arraystring(), "f");
        assert_eq!(&*FormattedTimestampStyle::LongDateTime.to_arraystring(), "F");
        assert_eq!(&*FormattedTimestampStyle::RelativeTime.to_arraystring(), "R");
    }

    #[test]
    fn test_message_time_parse() {
        let timestamp = Timestamp::now();

        let time = FormattedTimestamp::new(timestamp, Some(FormattedTimestampStyle::ShortDateTime));

        let time_str = aformat!(
            "<t:{}:{}>",
            timestamp.unix_timestamp(),
            FormattedTimestampStyle::ShortDateTime
        );

        let time_parsed = time_str.parse::<FormattedTimestamp>().unwrap();

        assert_eq!(time, time_parsed);

        let unstyled = FormattedTimestamp::new(timestamp, None);

        let unstyled_str = aformat!("<t:{}>", timestamp.unix_timestamp());

        let unstyled_parsed = unstyled_str.parse::<FormattedTimestamp>().unwrap();

        assert_eq!(unstyled, unstyled_parsed);
    }

    #[test]
    fn test_message_time_style_parse() {
        assert!(matches!("t".parse(), Ok(FormattedTimestampStyle::ShortTime)));
        assert!(matches!("T".parse(), Ok(FormattedTimestampStyle::LongTime)));
        assert!(matches!("d".parse(), Ok(FormattedTimestampStyle::ShortDate)));
        assert!(matches!("D".parse(), Ok(FormattedTimestampStyle::LongDate)));
        assert!(matches!("f".parse(), Ok(FormattedTimestampStyle::ShortDateTime)));
        assert!(matches!("F".parse(), Ok(FormattedTimestampStyle::LongDateTime)));
        assert!(matches!("R".parse(), Ok(FormattedTimestampStyle::RelativeTime)));
    }
}
