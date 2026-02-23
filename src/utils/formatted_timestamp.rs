use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use aformat::{ArrayString, ToArrayString, aformat_into};

use crate::model::Timestamp;

/// Represents a combination of a timestamp and a style for formatting time in messages.
///
/// [Discord docs](https://docs.discord.com/developers/reference#message-formatting-formats).
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct FormattedTimestamp {
    timestamp: i64,
    style: Option<FormattedTimestampStyle>,
}

/// Enum representing various styles for formatting time in messages.
///
/// [Discord docs](https://docs.discord.com/developers/reference#message-formatting-timestamp-styles).
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FormattedTimestampStyle {
    /// Represents a short time format, e.g., "12:34 PM".
    ShortTime,
    /// Represents a medium time format, e.g., "12:34:56 PM".
    MediumTime,
    /// Represents a short date format, e.g., "2023-11-17".
    ShortDate,
    /// Represents a long date format, e.g., "November 17, 2023".
    LongDate,
    /// Represents a long date and short time format, e.g., "November 17, 2023 at 12:34 PM".
    #[default]
    LongDateShortTime,
    /// Represents a full date and short time format, e.g., "Thursday, November 17, 2023 at 12:34
    /// PM".
    FullDateShortTime,
    /// Represents a short date and time format, e.g., "2023-11-17, 12:34 PM".
    ShortDateShortTime,
    /// Represents a short date and medium time format, e.g., "2023-11-17, 12:34:56 PM".
    ShortDateMediumTime,
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
            Self::MediumTime => "T",
            Self::ShortDate => "d",
            Self::LongDate => "D",
            Self::LongDateShortTime => "f",
            Self::FullDateShortTime => "F",
            Self::ShortDateShortTime => "s",
            Self::ShortDateMediumTime => "S",
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
            "T" => Ok(Self::MediumTime),
            "d" => Ok(Self::ShortDate),
            "D" => Ok(Self::LongDate),
            "f" => Ok(Self::LongDateShortTime),
            "F" => Ok(Self::FullDateShortTime),
            "s" => Ok(Self::ShortDateShortTime),
            "S" => Ok(Self::ShortDateMediumTime),
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

        let time =
            FormattedTimestamp::new(timestamp, Some(FormattedTimestampStyle::LongDateShortTime));
        let time_str = time.to_arraystring();

        assert_eq!(
            time_str,
            aformat!(
                "<t:{}:{}>",
                timestamp.unix_timestamp(),
                FormattedTimestampStyle::LongDateShortTime
            )
        );

        let unstyled = FormattedTimestamp::new(timestamp, None);

        let unstyled_str = unstyled.to_arraystring();

        assert_eq!(&*unstyled_str, &*aformat!("<t:{}>", timestamp.unix_timestamp()));
    }

    #[test]
    fn test_message_time_style() {
        assert_eq!(&*FormattedTimestampStyle::ShortTime.to_arraystring(), "t");
        assert_eq!(&*FormattedTimestampStyle::MediumTime.to_arraystring(), "T");
        assert_eq!(&*FormattedTimestampStyle::ShortDate.to_arraystring(), "d");
        assert_eq!(&*FormattedTimestampStyle::LongDate.to_arraystring(), "D");
        assert_eq!(&*FormattedTimestampStyle::LongDateShortTime.to_arraystring(), "f");
        assert_eq!(&*FormattedTimestampStyle::FullDateShortTime.to_arraystring(), "F");
        assert_eq!(&*FormattedTimestampStyle::ShortDateShortTime.to_arraystring(), "s");
        assert_eq!(&*FormattedTimestampStyle::ShortDateMediumTime.to_arraystring(), "S");
        assert_eq!(&*FormattedTimestampStyle::RelativeTime.to_arraystring(), "R");
    }

    #[test]
    fn test_message_time_parse() {
        let timestamp = Timestamp::now();

        let time =
            FormattedTimestamp::new(timestamp, Some(FormattedTimestampStyle::LongDateShortTime));

        let time_str = aformat!(
            "<t:{}:{}>",
            timestamp.unix_timestamp(),
            FormattedTimestampStyle::LongDateShortTime
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
        assert!(matches!("T".parse(), Ok(FormattedTimestampStyle::MediumTime)));
        assert!(matches!("d".parse(), Ok(FormattedTimestampStyle::ShortDate)));
        assert!(matches!("D".parse(), Ok(FormattedTimestampStyle::LongDate)));
        assert!(matches!("f".parse(), Ok(FormattedTimestampStyle::LongDateShortTime)));
        assert!(matches!("F".parse(), Ok(FormattedTimestampStyle::FullDateShortTime)));
        assert!(matches!("s".parse(), Ok(FormattedTimestampStyle::ShortDateShortTime)));
        assert!(matches!("S".parse(), Ok(FormattedTimestampStyle::ShortDateMediumTime)));
        assert!(matches!("R".parse(), Ok(FormattedTimestampStyle::RelativeTime)));
    }
}
