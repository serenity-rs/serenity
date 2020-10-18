use crate::constants::*;
use serde_json::Value;
use std::time::Duration;

/// Information about an [`Input`] source.
///
/// [`Input`]: struct.Input.html
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    /// The title of this stream.
    pub title: Option<String>,
    /// The main artist of this stream.
    pub artist: Option<String>,
    /// The date of creation of this stream.
    pub date: Option<String>,

    /// The number of audio channels in this stream.
    ///
    /// Any number `>= 2` is treated as stereo.
    pub channels: Option<u8>,
    /// The time at which the first true sample is played back.
    ///
    /// This occurs as an artefact of coder delay.
    pub start_time: Option<Duration>,
    /// The reported duration of this stream.
    pub duration: Option<Duration>,
    /// The sample rate of this stream.
    pub sample_rate: Option<u32>,
}

impl Metadata {
    /// Extract metadata and details from the output of
    /// `ffprobe`.
    pub fn from_ffprobe_json(value: &Value) -> Self {
        let format = value.as_object().and_then(|m| m.get("format"));

        let duration = format
            .and_then(|m| m.get("duration"))
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<f64>().ok())
            .map(Duration::from_secs_f64);

        let start_time = format
            .and_then(|m| m.get("start_time"))
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<f64>().ok())
            .map(Duration::from_secs_f64);

        let tags = format.and_then(|m| m.get("tags"));

        let title = tags
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let artist = tags
            .and_then(|m| m.get("artist"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let date = tags
            .and_then(|m| m.get("date"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let stream = value
            .as_object()
            .and_then(|m| m.get("streams"))
            .and_then(|v| v.as_array())
            .and_then(|v| {
                v.iter()
                    .find(|line| line.get("codec_type").and_then(Value::as_str) == Some("audio"))
            });

        let channels = stream
            .and_then(|m| m.get("channels"))
            .and_then(Value::as_u64)
            .map(|v| v as u8);

        let sample_rate = stream
            .and_then(|m| m.get("sample_rate"))
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<u64>().ok())
            .map(|v| v as u32);

        Self {
            title,
            artist,
            date,

            channels,
            start_time,
            duration,
            sample_rate,
        }
    }

    /// Use `youtube-dl` to extract metadata for an online resource.
    pub fn from_ytdl_output(value: Value) -> Self {
        let obj = value.as_object();

        let track = obj
            .and_then(|m| m.get("track"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let title = track.or_else(|| {
            obj.and_then(|m| m.get("title"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

        let true_artist = obj
            .and_then(|m| m.get("artist"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let artist = true_artist.or_else(|| {
            obj.and_then(|m| m.get("uploader"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

        let r_date = obj
            .and_then(|m| m.get("release_date"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let date = r_date.or_else(|| {
            obj.and_then(|m| m.get("upload_date"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

        let duration = obj
            .and_then(|m| m.get("duration"))
            .and_then(Value::as_f64)
            .map(Duration::from_secs_f64);

        Self {
            title,
            artist,
            date,

            channels: Some(2),
            duration,
            sample_rate: Some(SAMPLE_RATE_RAW as u32),

            ..Default::default()
        }
    }

    /// Move all fields from a `Metadata` object into a new one.
    pub fn take(&mut self) -> Self {
        Self {
            title: self.title.take(),
            artist: self.artist.take(),
            date: self.date.take(),

            channels: self.channels.take(),
            start_time: self.start_time.take(),
            duration: self.duration.take(),
            sample_rate: self.sample_rate.take(),
        }
    }
}
