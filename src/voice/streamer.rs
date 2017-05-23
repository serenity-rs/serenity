use byteorder::{LittleEndian, ReadBytesExt};
use serde_json;
use std::ffi::OsStr;
use std::io::{ErrorKind as IoErrorKind, Read, Result as IoResult};
use std::process::{Child, Command, Stdio};
use super::{AudioSource, VoiceError};
use ::internal::prelude::*;

struct ChildContainer(Child);

impl Read for ChildContainer {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.0.stdout.as_mut().unwrap().read(buffer)
    }
}

struct PcmSource<R: Read + Send + 'static>(bool, R);

impl<R: Read + Send> AudioSource for PcmSource<R> {
    fn is_stereo(&mut self) -> bool {
        self.0
    }

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        for (i, v) in buffer.iter_mut().enumerate() {
            *v = match self.1.read_i16::<LittleEndian>() {
                Ok(v) => v,
                Err(ref e) => return if e.kind() == IoErrorKind::UnexpectedEof {
                    Some(i)
                } else {
                    None
                },
            }
        }

        Some(buffer.len())
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Box<AudioSource>> {
    let path = path.as_ref();

    /// Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo {
        "2"
    } else {
        "1"
    };

    let args = [
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ];

    let command = Command::new("ffmpeg")
        .arg("-i")
        .arg(path)
        .args(&args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(pcm(is_stereo, ChildContainer(command)))
}

/// Creates a PCM audio source.
pub fn pcm<R: Read + Send + 'static>(is_stereo: bool, reader: R)
    -> Box<AudioSource> {
    Box::new(PcmSource(is_stereo, reader))
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub fn ytdl(uri: &str) -> Result<Box<AudioSource>> {
    let args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "--no-playlist",
        "--print-json",
        "--skip-download",
        uri,
    ];

    let out = Command::new("youtube-dl")
        .args(&args)
        .stdin(Stdio::null())
        .output()?;

    if !out.status.success() {
        return Err(Error::Voice(VoiceError::YouTubeDLRun(out)));
    }

    let value = serde_json::from_reader(&out.stdout[..])?;
    let mut obj = match value {
        Value::Object(obj) => obj,
        other => return Err(Error::Voice(VoiceError::YouTubeDLProcessing(other))),
    };

    let uri = match obj.remove("url") {
        Some(v) => match v {
            Value::String(uri) => uri,
            other => return Err(Error::Voice(VoiceError::YouTubeDLUrl(other))),
        },
        None => return Err(Error::Voice(VoiceError::YouTubeDLUrl(Value::Object(obj)))),
    };

    ffmpeg(&uri)
}

fn is_stereo(path: &OsStr) -> Result<bool> {
    let args = [
        "-v",
        "quiet",
        "-of",
        "json",
        "-show-streams",
        "-i",
    ];

    let out = Command::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let streams = value.as_object()
        .and_then(|m| m.get("streams"))
        .and_then(|v| v.as_array())
        .ok_or(Error::Voice(VoiceError::Streams))?;

    let check = streams.iter()
        .any(|stream| {
            let channels = stream.as_object()
                .and_then(|m| m.get("channels")
                    .and_then(|v| v.as_i64()));

            channels == Some(2)
        });

    Ok(check)
}
