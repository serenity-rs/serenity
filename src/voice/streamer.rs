use byteorder::{LittleEndian, ReadBytesExt};
use internal::prelude::*;
use serde_json;
use std::{
    ffi::OsStr,
    fs::File,
    io::{
        BufReader,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult
    },
    process::{
        Child,
        Command,
        Stdio
    },
    result::Result as StdResult
};
use super::{
    AudioSource,
    AudioType,
    DcaError,
    DcaMetadata,
    VoiceError
};

struct ChildContainer(Child);

impl Read for ChildContainer {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.0.stdout.as_mut().unwrap().read(buffer)
    }
}

impl Drop for ChildContainer {
    fn drop (&mut self) {
        if let Err(e) = self.0.kill() {
            debug!("[Voice] Error awaiting child process: {:?}", e);
        }
    }
}

struct InputSource<R: Read + Send + 'static> {
    stereo: bool,
    reader: R,
    kind: AudioType,
}

impl<R: Read + Send> AudioSource for InputSource<R> {
    fn is_stereo(&mut self) -> bool { self.stereo }

    fn get_type(&self) -> AudioType { self.kind }

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        for (i, v) in buffer.iter_mut().enumerate() {
            *v = match self.reader.read_i16::<LittleEndian>() {
                Ok(v) => v,
                Err(ref e) => {
                    return if e.kind() == IoErrorKind::UnexpectedEof {
                        Some(i)
                    } else {
                        None
                    }
                },
            }
        }

        Some(buffer.len())
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        match self.reader.read_i16::<LittleEndian>() {
            Ok(size) => {
                if size <= 0 {
                    warn!("Invalid opus frame size: {}", size);
                    return None;
                }

                let mut frame = Vec::with_capacity(size as usize);

                {
                    let reader = self.reader.by_ref();

                    if reader.take(size as u64).read_to_end(&mut frame).is_err() {
                        return None;
                    }
                }

                Some(frame)
            },
            Err(ref e) => if e.kind() == IoErrorKind::UnexpectedEof {
                Some(Vec::new())
            } else {
                None
            },
        }
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Box<AudioSource>> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Box<AudioSource>> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo { "2" } else { "1" };

    ffmpeg_optioned(path, &[
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ])
}

/// Opens an audio file through `ffmpeg` and creates an audio source, with
/// user-specified arguments to pass to ffmpeg.
///
/// Note that this does _not_ build on the arguments passed by the [`ffmpeg`]
/// function.
///
/// # Examples
///
/// Pass options to create a custom ffmpeg streamer:
///
/// ```rust,no_run
/// use serenity::voice;
///
/// let stereo_val = "2";
///
/// let streamer = voice::ffmpeg_optioned("./some_file.mp3", &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ]);
pub fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    args: &[&str],
) -> Result<Box<AudioSource>> {
    _ffmpeg_optioned(path.as_ref(), args)
}

fn _ffmpeg_optioned(path: &OsStr, args: &[&str]) -> Result<Box<AudioSource>> {
    let is_stereo = is_stereo(path).unwrap_or(false);

    let command = Command::new("ffmpeg")
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(pcm(is_stereo, ChildContainer(command)))
}

/// Creates a streamed audio source from a DCA file.
/// Currently only accepts the DCA1 format.
pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<AudioSource>, DcaError> {
    _dca(path.as_ref())
}

fn _dca(path: &OsStr) -> StdResult<Box<AudioSource>, DcaError> {
    let file = File::open(path).map_err(DcaError::IoError)?;

    let mut reader = BufReader::new(file);

    let mut header = [0u8; 4];

    // Read in the magic number to verify it's a DCA file.
    reader.read_exact(&mut header).map_err(DcaError::IoError)?;

    if header != b"DCA1"[..] {
        return Err(DcaError::InvalidHeader);
    }

    reader.read_exact(&mut header).map_err(DcaError::IoError)?;

    let size = (&header[..]).read_i32::<LittleEndian>().unwrap();

    // Sanity check
    if size < 2 {
        return Err(DcaError::InvalidSize(size));
    }

    let mut raw_json = Vec::with_capacity(size as usize);

    {
        let json_reader = reader.by_ref();
        json_reader
            .take(size as u64)
            .read_to_end(&mut raw_json)
            .map_err(DcaError::IoError)?;
    }

    let metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
        .map_err(DcaError::InvalidMetadata)?;

    Ok(opus(metadata.is_stereo(), reader))
}

/// Creates an Opus audio source. This makes certain assumptions: namely, that the input stream
/// is composed ONLY of opus frames of the variety that Discord expects.
///
/// If you want to decode a `.opus` file, use [`ffmpeg`]
///
/// [`ffmpeg`]: fn.ffmpeg.html
pub fn opus<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<AudioSource> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Opus,
    })
}

/// Creates a PCM audio source.
pub fn pcm<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<AudioSource> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Pcm,
    })
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
    let args = ["-v", "quiet", "-of", "json", "-show-streams", "-i"];

    let out = Command::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let streams = value
        .as_object()
        .and_then(|m| m.get("streams"))
        .and_then(|v| v.as_array())
        .ok_or(Error::Voice(VoiceError::Streams))?;

    let check = streams.iter().any(|stream| {
        let channels = stream
            .as_object()
            .and_then(|m| m.get("channels").and_then(|v| v.as_i64()));

        channels == Some(2)
    });

    Ok(check)
}
