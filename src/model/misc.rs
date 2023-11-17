//! Miscellaneous helper traits, enums, and structs for models.

#[cfg(all(feature = "model", feature = "utils"))]
use std::error::Error as StdError;
use std::fmt::{self, Display};
#[cfg(all(feature = "model", feature = "utils"))]
use std::result::Result as StdResult;
use std::str::FromStr;

use super::prelude::*;
#[cfg(all(feature = "model", any(feature = "cache", feature = "utils")))]
use crate::utils;

/// Hides the implementation detail of ImageHash as an enum.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImageHashInner {
    Normal { hash: [u8; 16], is_animated: bool },
    Clyde,
}

/// An image hash returned from the Discord API.
///
/// This type can be constructed via it's [`FromStr`] implementation, and can be turned into it's
/// cannonical representation via [`std::fmt::Display`] or [`serde::Serialize`].
///
/// # Example
/// ```rust
/// use serenity::model::misc::ImageHash;
///
/// let image_hash: ImageHash = "f1eff024d9c85339c877985229ed8fec".parse().unwrap();
/// assert_eq!(image_hash.to_string(), String::from("f1eff024d9c85339c877985229ed8fec"));
/// ```

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageHash(ImageHashInner);

impl ImageHash {
    /// Returns if the linked image is animated, which means the hash starts with `a_`.
    ///
    /// # Example
    /// ```rust
    /// use serenity::model::misc::ImageHash;
    ///
    /// let animated_hash: ImageHash = "a_e3c0db7f38777778fb43081f8746ebc9".parse().unwrap();
    /// assert!(animated_hash.is_animated());
    /// ```
    #[must_use]
    pub fn is_animated(&self) -> bool {
        match &self.0 {
            ImageHashInner::Normal {
                is_animated, ..
            } => *is_animated,
            ImageHashInner::Clyde => true,
        }
    }
}

impl std::fmt::Debug for ImageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        <Self as std::fmt::Display>::fmt(self, f)?;
        f.write_str("\"")
    }
}

impl serde::Serialize for ImageHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ImageHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let helper = arrayvec::ArrayString::<34>::deserialize(deserializer)?;
        Self::from_str(&helper).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ImageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ImageHashInner::Normal {
            hash,
            is_animated,
        } = &self.0
        else {
            return f.write_str("clyde");
        };

        if *is_animated {
            f.write_str("a_")?;
        }

        for byte in hash {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

/// An error returned when [`ImageHash`] is passed an erronous value.
#[derive(Debug, Clone)]
pub enum ImageHashParseError {
    /// The given hash was not a valid [`ImageHash`] length, containing the invalid length.
    InvalidLength(usize),
}

impl std::error::Error for ImageHashParseError {}

impl std::fmt::Display for ImageHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(f, "Invalid length {length}, expected 32 or 34 characters")
            },
        }
    }
}

impl std::str::FromStr for ImageHash {
    type Err = ImageHashParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let (hex, is_animated) = if s.len() == 34 && s.starts_with("a_") {
            (&s[2..], true)
        } else if s.len() == 32 {
            (s, false)
        } else if s == "clyde" {
            return Ok(Self(ImageHashInner::Clyde));
        } else {
            return Err(Self::Err::InvalidLength(s.len()));
        };

        let mut hash = [0u8; 16];
        for i in (0..hex.len()).step_by(2) {
            let hex_byte = &hex[i..i + 2];
            hash[i / 2] = u8::from_str_radix(hex_byte, 16).unwrap_or_else(|err| {
                tracing::warn!("Invalid byte in ImageHash ({s}): {err}");
                0
            });
        }

        Ok(Self(ImageHashInner::Normal {
            is_animated,
            hash,
        }))
    }
}

/// A version of an emoji used only when solely the animated state, Id, and name are known.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-emoji).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct EmojiIdentifier {
    /// Whether the emoji is animated
    pub animated: bool,
    /// The Id of the emoji.
    pub id: EmojiId,
    /// The name of the emoji. It must be at least 2 characters long and can only contain
    /// alphanumeric characters and underscores.
    pub name: String,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl EmojiIdentifier {
    /// Generates a URL to the emoji's image.
    #[must_use]
    pub fn url(&self) -> String {
        let ext = if self.animated { "gif" } else { "png" };

        cdn!("/emojis/{}.{}", self.id, ext)
    }
}

#[derive(Debug)]
#[cfg(all(feature = "model", feature = "utils"))]
pub struct EmojiIdentifierParseError {
    parsed_string: String,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl fmt::Display for EmojiIdentifierParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is not a valid emoji identifier", self.parsed_string)
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for EmojiIdentifierParseError {}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for EmojiIdentifier {
    type Err = EmojiIdentifierParseError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        utils::parse_emoji(s).ok_or_else(|| EmojiIdentifierParseError {
            parsed_string: s.to_owned(),
        })
    }
}

/// An incident retrieved from the Discord status page.
///
/// This is not necessarily a representation of an ongoing incident.
///
/// [Discord docs](https://discordstatus.com/api) (see "Unresolved incident" example)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Incident {
    pub created_at: String,
    pub id: String,
    pub impact: String,
    pub incident_updates: Vec<IncidentUpdate>,
    pub monitoring_at: Option<String>,
    pub name: String,
    pub page_id: String,
    pub resolved_at: Option<String>,
    pub shortlink: String,
    pub status: String,
    pub updated_at: String,
}

/// An update to an incident from the Discord status page.
///
/// This will typically state what new information has been discovered about an incident.
///
/// [Discord docs](https://discordstatus.com/api) (see "Unresolved incident" example)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IncidentUpdate {
    pub body: String,
    pub created_at: String,
    pub display_at: String,
    pub id: String,
    pub incident_id: String,
    pub status: String,
    pub updated_at: String,
}

/// A Discord status maintenance message. This can be either for active maintenances or for
/// scheduled maintenances.
///
/// [Discord docs](https://discordstatus.com/api) (see "scheduled maintenances" examples)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Maintenance {
    pub created_at: String,
    pub id: String,
    pub impact: String,
    pub incident_updates: Vec<IncidentUpdate>,
    pub monitoring_at: Option<String>,
    pub name: String,
    pub page_id: String,
    pub resolved_at: Option<String>,
    pub scheduled_for: String,
    pub scheduled_until: String,
    pub shortlink: String,
    pub status: String,
    pub updated_at: String,
}

#[cfg(test)]
mod test {
    use crate::model::prelude::*;

    #[test]
    fn test_formatters() {
        assert_eq!(ChannelId::new(1).to_string(), "1");
        assert_eq!(EmojiId::new(2).to_string(), "2");
        assert_eq!(GuildId::new(3).to_string(), "3");
        assert_eq!(RoleId::new(4).to_string(), "4");
        assert_eq!(UserId::new(5).to_string(), "5");
    }
}

/// Represents a combination of a timestamp and a style for formatting time in messages.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg(all(feature = "model", feature = "utils"))]
pub struct FormattedTimestamp(i64, FormattedTimestampStyle);

/// Enum representing various styles for formatting time in messages.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg(all(feature = "model", feature = "utils"))]
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

#[cfg(all(feature = "model", feature = "utils"))]
impl FormattedTimestamp {
    /// Creates a new [`FormattedTimestamp`] instance from the given [`Timestamp`] and
    /// [`FormattedTimestampStyle`].
    #[must_use]
    pub fn new(timestamp: Timestamp, style: FormattedTimestampStyle) -> Self {
        Self(timestamp.timestamp(), style)
    }

    /// Creates a new [`FormattedTimestamp`] instance representing the current timestamp with the
    /// default style.
    #[must_use]
    pub fn now() -> Self {
        Self(Timestamp::now().timestamp(), FormattedTimestampStyle::default())
    }

    /// Returns the timestamp of this [`FormattedTimestamp`].
    #[must_use]
    pub fn timestamp(&self) -> i64 {
        self.0
    }

    /// Returns the style of this [`FormattedTimestamp`].
    #[must_use]
    pub fn style(&self) -> FormattedTimestampStyle {
        self.1
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl From<Timestamp> for FormattedTimestamp {
    /// Creates a new [`FormattedTimestamp`] instance from the given [`Timestamp`] with the default
    /// style.
    fn from(timestamp: Timestamp) -> Self {
        Self(timestamp.timestamp(), FormattedTimestampStyle::default())
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl Display for FormattedTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<t:{}:{}>", self.0, self.1)
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl Display for FormattedTimestampStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = match self {
            Self::ShortTime => "t",
            Self::LongTime => "T",
            Self::ShortDate => "d",
            Self::LongDate => "D",
            Self::ShortDateTime => "f",
            Self::LongDateTime => "F",
            Self::RelativeTime => "R",
        };
        f.write_str(style)
    }
}

/// An error that can occur when parsing a [`FormattedTimestamp`] from a string.
#[cfg(all(feature = "model", feature = "utils"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormattedTimestampParseError {
    InvalidFormat,
    InvalidTimestamp,
    InvalidStyle,
}

#[cfg(all(feature = "model", feature = "utils"))]
impl StdError for FormattedTimestampParseError {}

#[cfg(all(feature = "model", feature = "utils"))]
impl Display for FormattedTimestampParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => f.write_str("Invalid format"),
            Self::InvalidTimestamp => f.write_str("Invalid timestamp"),
            Self::InvalidStyle => f.write_str("Invalid style"),
        }
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
impl FromStr for FormattedTimestamp {
    type Err = FormattedTimestampParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<t:") || !s.ends_with('>') {
            return Err(FormattedTimestampParseError::InvalidFormat);
        }

        let mut parts = s[3..s.len() - 1].split(':');

        let secs = parts
            .next()
            .ok_or(FormattedTimestampParseError::InvalidFormat)?
            .parse()
            .map_err(|_| FormattedTimestampParseError::InvalidTimestamp)?;

        let timestamp = Timestamp::from_unix_timestamp(secs)
            .map_err(|_| FormattedTimestampParseError::InvalidTimestamp)?;

        let style = parts.next().ok_or(FormattedTimestampParseError::InvalidFormat)?.parse()?;

        Ok(Self(timestamp.timestamp(), style))
    }
}

#[cfg(all(feature = "model", feature = "utils"))]
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
            _ => Err(FormattedTimestampParseError::InvalidStyle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_time() {
        let timestamp = Timestamp::now();

        let time = FormattedTimestamp::new(timestamp, FormattedTimestampStyle::ShortDateTime);

        let time_str = time.to_string();

        assert_eq!(
            time_str,
            format!("<t:{}:{}>", timestamp.timestamp(), FormattedTimestampStyle::ShortDateTime)
        );
    }

    #[test]
    fn test_message_time_style() {
        let short_time = FormattedTimestampStyle::ShortTime;
        let long_time = FormattedTimestampStyle::LongTime;
        let short_date = FormattedTimestampStyle::ShortDate;
        let long_date = FormattedTimestampStyle::LongDate;
        let short_date_time = FormattedTimestampStyle::ShortDateTime;
        let long_date_time = FormattedTimestampStyle::LongDateTime;
        let relative_time = FormattedTimestampStyle::RelativeTime;

        assert_eq!(short_time.to_string(), "t");
        assert_eq!(long_time.to_string(), "T");
        assert_eq!(short_date.to_string(), "d");
        assert_eq!(long_date.to_string(), "D");
        assert_eq!(short_date_time.to_string(), "f");
        assert_eq!(long_date_time.to_string(), "F");
        assert_eq!(relative_time.to_string(), "R");
    }

    #[test]
    fn test_message_time_parse() {
        let timestamp = Timestamp::now();

        let time = FormattedTimestamp::new(timestamp, FormattedTimestampStyle::ShortDateTime);

        let time_str =
            format!("<t:{}:{}>", timestamp.timestamp(), FormattedTimestampStyle::ShortDateTime);

        let time_parsed = time_str.parse::<FormattedTimestamp>().unwrap();

        assert_eq!(time, time_parsed);
    }

    #[test]
    fn test_message_time_style_parse() {
        let short_time = FormattedTimestampStyle::ShortTime;
        let long_time = FormattedTimestampStyle::LongTime;
        let short_date = FormattedTimestampStyle::ShortDate;
        let long_date = FormattedTimestampStyle::LongDate;
        let short_date_time = FormattedTimestampStyle::ShortDateTime;
        let long_date_time = FormattedTimestampStyle::LongDateTime;
        let relative_time = FormattedTimestampStyle::RelativeTime;

        assert_eq!("t".parse(), Ok(short_time));
        assert_eq!("T".parse(), Ok(long_time));
        assert_eq!("d".parse(), Ok(short_date));
        assert_eq!("D".parse(), Ok(long_date));
        assert_eq!("f".parse(), Ok(short_date_time));
        assert_eq!("F".parse(), Ok(long_date_time));
        assert_eq!("R".parse(), Ok(relative_time));
    }
}
