use byteorder::{LittleEndian, ReadBytesExt};
use crate::internal::prelude::*;
use audiopus::{
    Channels,
    coder::Decoder as OpusDecoder,
    Result as OpusResult,
};
use parking_lot::Mutex;
use serde_json;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufReader, ErrorKind as IoErrorKind, Read, Result as IoResult},
    process::{Child, Command, Stdio},
    result::Result as StdResult,
    sync::Arc,
};
use super::{AudioSource, AudioType, DcaError, DcaMetadata, VoiceError, audio};
use log::{debug, warn};
use crate::prelude::SerenityError;

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

// Since each audio item needs its own decoder, we need to
// work around the fact that OpusDecoders aint sendable.
struct SendDecoder(OpusDecoder);

impl SendDecoder {
    fn decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> OpusResult<usize> {
        let &mut SendDecoder(ref mut sd) = self;
        sd.decode_float(input, output, fec)
    }
}

unsafe impl Send for SendDecoder {}

struct InputSource<R: Read + Send + 'static> {
    stereo: bool,
    reader: R,
    kind: AudioType,
    decoder: Option<Arc<Mutex<SendDecoder>>>,
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

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize> {
        let decoder_lock = self.decoder.as_mut()?.clone();
        let frame = self.read_opus_frame()?;
        let mut local_buf = [0f32; 960 * 2];

        let count = {
            let mut decoder = decoder_lock.lock();

            decoder.decode_float(frame.as_slice(), &mut local_buf, false).ok()?
        };

        for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
            *float_buffer_element += local_buf[i] * volume;
        }

        Some(count)
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Box<dyn AudioSource>> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Box<dyn AudioSource>> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo { "2" } else { "1" };

    _ffmpeg_optioned(path, &[
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ], Some(is_stereo))
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
) -> Result<Box<dyn AudioSource>> {
    _ffmpeg_optioned(path.as_ref(), args, None)
}

fn _ffmpeg_optioned(path: &OsStr, args: &[&str], is_stereo_known: Option<bool>) -> Result<Box<dyn AudioSource>> {
    let is_stereo = is_stereo_known
        .or_else(|| is_stereo(path).ok())
        .unwrap_or(false);

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
pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<dyn AudioSource>, DcaError> {
    _dca(path.as_ref())
}

fn _dca(path: &OsStr) -> StdResult<Box<dyn AudioSource>, DcaError> {
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
pub fn opus<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Opus,
        decoder: Some(
            Arc::new(Mutex::new(
                // We always want to decode *to* stereo, for mixing reasons.
                SendDecoder(OpusDecoder::new(audio::SAMPLE_RATE, Channels::Stereo).unwrap())
            ))
        ),
    })
}

/// Creates a PCM audio source.
pub fn pcm<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Pcm,
        decoder: None,
    })
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub fn ytdl(uri: &str) -> Result<Box<dyn AudioSource>> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        uri,
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-re")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(pcm(true, ChildContainer(ffmpeg)))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub fn ytdl_search(name: &str) -> Result<Box<dyn AudioSource>> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        &format!("ytsearch1:{}",name),
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-re")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(pcm(true, ChildContainer(ffmpeg)))
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
