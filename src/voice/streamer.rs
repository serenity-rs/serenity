use byteorder::{ByteOrder, LittleEndian};
use crate::internal::prelude::*;
use audiopus::{
    Channels,
    coder::Decoder as OpusDecoder,
    Result as OpusResult,
};
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use serde_json;
use std::{
    ffi::OsStr,
    io::ErrorKind as IoErrorKind,
    marker::Unpin,
    pin::Pin,
    process::Stdio,
    result::Result as StdResult,
    sync::Arc,
    task::{Context, Poll},
};
use super::{AudioSource, AudioType, DcaError, DcaMetadata, VoiceError, audio};
use tracing::{debug, warn, instrument};
use crate::prelude::SerenityError;
use async_trait::async_trait;

struct ChildContainer(Child);

impl AsyncRead for ChildContainer {
    #[instrument(skip(self))]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut [u8]
    ) -> Poll<tokio::io::Result<usize>> {
        let stdout = unsafe {
            self.map_unchecked_mut(|s| { s.0.stdout.as_mut().unwrap() })
        };
        stdout.poll_read(cx, buffer)
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
    #[instrument(skip(self))]
    fn decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> OpusResult<usize> {
        let &mut SendDecoder(ref mut sd) = self;
        sd.decode_float(Some(input), output, fec)
    }
}

unsafe impl Send for SendDecoder {}

struct InputSource<R: AsyncRead + Unpin + Send + Sync + 'static> {
    stereo: bool,
    reader: R,
    kind: AudioType,
    decoder: Option<Arc<Mutex<SendDecoder>>>,
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + Sync> AudioSource for InputSource<R> {
    #[instrument(skip(self))]
    async fn is_stereo(&mut self) -> bool {
        self.stereo
    }

    #[instrument(skip(self))]
    async fn get_type(&self) -> AudioType {
        self.kind
    }

    #[instrument(skip(self))]
    async fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        let mut buf: [u8; 2] = [0, 0];
        for (i, v) in buffer.iter_mut().enumerate() {
            let result = self.reader.read_exact(&mut buf).await;
            *v = match result.map(|_| LittleEndian::read_i16(&buf)) {
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

    #[instrument(skip(self))]
    async fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        let mut buf: [u8; 2] = [0, 0];
        let result = self.reader.read_exact(&mut buf).await;
        match result.map(|_| LittleEndian::read_i16(&buf)) {
            Ok(size) => {
                if size <= 0 {
                    warn!("Invalid opus frame size: {}", size);
                    return None;
                }

                let mut frame = Vec::with_capacity(size as usize);

                {
                    let reader = &mut self.reader;

                    if reader.take(size as u64).read_to_end(&mut frame).await.is_err() {
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

    #[instrument(skip(self, float_buffer))]
    async fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize> {
        let decoder_lock = self.decoder.as_mut()?.clone();
        let frame = self.read_opus_frame().await?;
        let mut local_buf = [0f32; 960 * 2];

        let count = {
            let mut decoder = decoder_lock.lock().await;

            decoder.decode_float(frame.as_slice(), &mut local_buf, false).ok()?
        };

        for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
            *float_buffer_element += local_buf[i] * volume;
        }

        Some(count)
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub async fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Box<dyn AudioSource>> {
    _ffmpeg(path.as_ref()).await
}

#[instrument]
async fn _ffmpeg(path: &OsStr) -> Result<Box<dyn AudioSource>> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).await.unwrap_or(false);
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
    ], Some(is_stereo)).await
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
/// # async fn test() {
/// let stereo_val = "2";
///
/// let options = &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ];
///
/// let streamer = voice::ffmpeg_optioned("./some_file.mp3", options).await;
/// # }
pub async fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    args: &[&str],
) -> Result<Box<dyn AudioSource>> {
    _ffmpeg_optioned(path.as_ref(), args, None).await
}

#[instrument]
async fn _ffmpeg_optioned(path: &OsStr, args: &[&str], is_stereo_known: Option<bool>) -> Result<Box<dyn AudioSource>> {
    let is_stereo_known = match is_stereo_known {
        None => is_stereo(path).await.ok(),
        others => others,
    };
    let is_stereo = is_stereo_known.unwrap_or(false);

    let command = Command::new("ffmpeg")
        .kill_on_drop(true)
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
pub async fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<dyn AudioSource>, DcaError> {
    _dca(path.as_ref()).await
}

#[instrument]
async fn _dca(path: &OsStr) -> StdResult<Box<dyn AudioSource>, DcaError> {
    let mut reader = File::open(path).await.map_err(DcaError::IoError)?;

    let mut header = [0u8; 4];

    // Read in the magic number to verify it's a DCA file.
    reader.read_exact(&mut header).await.map_err(DcaError::IoError)?;

    if header != b"DCA1"[..] {
        return Err(DcaError::InvalidHeader);
    }

    reader.read_exact(&mut header).await.map_err(DcaError::IoError)?;

    let size = LittleEndian::read_i32(&header[..]);

    // Sanity check
    if size < 2 {
        return Err(DcaError::InvalidSize(size));
    }

    let mut raw_json = Vec::with_capacity(size as usize);

    {
        let json_reader = &mut reader;
        json_reader
            .take(size as u64)
            .read_to_end(&mut raw_json)
            .await
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
#[instrument(skip(reader))]
pub fn opus<R: AsyncRead + Unpin + Send + Sync + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource> {
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
#[instrument(skip(reader))]
pub fn pcm<R: AsyncRead + Unpin + Send + Sync + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Pcm,
        decoder: None,
    })
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
#[instrument]
pub async fn ytdl(uri: &str) -> Result<Box<dyn AudioSource>> {
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

    let youtube_dl = std::process::Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .kill_on_drop(true)
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
#[instrument]
pub async fn ytdl_search(name: &str) -> Result<Box<dyn AudioSource>> {
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

    let youtube_dl = std::process::Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .kill_on_drop(true)
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

#[instrument]
async fn is_stereo(path: &OsStr) -> Result<bool> {
    let args = ["-v", "quiet", "-of", "json", "-show-streams", "-i"];

    let out = Command::new("ffprobe")
        .kill_on_drop(true)
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output().await?;

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
