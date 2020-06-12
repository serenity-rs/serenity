//! Raw audio input data streams and sources.

pub mod cached;
mod dca;
pub mod utils;

use audiopus::{
    coder::Decoder as OpusDecoder,
    Channels,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cached::{CompressedSource, MemorySource};
use crate::{
    internal::prelude::*,
    prelude::SerenityError,
    voice::{
        constants::*,
        error::DcaError,
        VoiceError,
    },
};
use dca::DcaMetadata;
use log::{debug, warn};
use parking_lot::Mutex;
use std::{
    fs::File,
    ffi::OsStr,
    fmt::{
        Debug,
        Error as FormatError,
        Formatter,
    },
    io::{
        self,
        BufReader,
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
        Seek,
        SeekFrom,
    },
    mem,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

/// Type of data being passed into an [`Input`].
///
/// [`Input`]: struct.Input.html
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum CodecType {
    Opus,
    Pcm,
    FloatPcm,
}

impl CodecType {
    pub fn sample_len(&self) -> usize {
        use CodecType::*;

        match self {
            Opus | FloatPcm => mem::size_of::<f32>(),
            Pcm => mem::size_of::<i16>(),
            _ => unimplemented!(),
        }
    }
}

/// State used to decode input bytes of an [`Input`].
///
/// [`Input`]: struct.Input.html
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Codec {
    Opus(Arc<Mutex<OpusDecoder>>),
    Pcm,
    FloatPcm,
}

impl From<&Codec> for CodecType {
    fn from(f: &Codec) -> Self {
        use Codec::*;

        match f {
            Opus(_) => Self::Opus,
            Pcm => Self::Pcm,
            FloatPcm => Self::FloatPcm,
        }
    }
}

/// Information used in audio frame detection.
pub struct Frame {
    pub header_len: usize,
    pub frame_len: usize,
}

/// Marker for decoding framed input files.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Container {
    Raw,
    Dca,
}

impl Container {
    pub fn next_frame_length(&mut self, mut reader: impl Read, input: CodecType) -> IoResult<Frame> {
        use Container::*;

        match self {
            Raw => Ok(Frame{header_len: 0, frame_len: input.sample_len()}),
            Dca => reader.read_i16::<LittleEndian>().map(|frame_len| Frame {
                header_len: mem::size_of::<i16>(),
                frame_len: frame_len.min(0) as usize,
            }),
        }
    }

    pub fn is_seek_trivial(&self) -> bool {
        use Container::*;

        match self {
            Raw => true,
            _ => false,
        }
    }
}

/// Handle for a child process which ensures that any subprocesses are properly closed
/// on drop.
#[derive(Debug)]
pub struct ChildContainer(Child);

fn child_to_reader<T>(child: Child) -> Reader {
    Reader::Pipe(
        BufReader::with_capacity(
            STEREO_FRAME_SIZE * mem::size_of::<T>() * CHILD_BUFFER_LEN,
            ChildContainer(child),
        )
    )
}

impl From<Child> for Reader {
    fn from(container: Child) -> Self {
        child_to_reader::<f32>(container)
    }
}

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

/// Usable data/byte sources for an audio stream.
///
/// Users may define their own data sources using [`Extension`]
/// and [`ExtensionSeek`].
///
/// [`Extension`]: #variant.Extension
/// [`ExtensionSeek`]: #variant.ExtensionSeek
pub enum Reader {
    Pipe(BufReader<ChildContainer>),
    InMemory(MemorySource),
    Compressed(CompressedSource),
    Restartable(RestartableSource),
    File(BufReader<File>),
    Extension(Box<dyn Read + Send>),
    ExtensionSeek(Box<dyn ReadSeek + Send>),
}

impl Reader {
    fn is_seekable(&self) -> bool {
        use Reader::*;

        match self {
            Restartable(_) | Compressed(_) | InMemory(_) => true,
            Extension(_) => false,
            ExtensionSeek(_) => true,
            _ => false,
        }
    }
}

impl Read for Reader {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        use Reader::*;
        match self {
            Pipe(a) => Read::read(a, buffer),
            InMemory(a) => Read::read(a, buffer),
            Compressed(a) => Read::read(a, buffer),
            Restartable(a) => Read::read(a, buffer),
            File(a) => Read::read(a, buffer),
            Extension(a) => a.read(buffer),
            ExtensionSeek(a) => a.read(buffer),
        }
    }
}

impl Seek for Reader {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        use Reader::*;
        match self {
            Pipe(_) | Extension(_) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "Seeking not supported on Reader of this type.")),
            InMemory(a) => Seek::seek(a, pos),
            Compressed(a) => Seek::seek(a, pos),
            File(a) => Seek::seek(a, pos),
            Restartable(a) => Seek::seek(a, pos),
            ExtensionSeek(a) => a.seek(pos),
        }
    }
}

impl Debug for Reader {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), FormatError> {
        use Reader::*;
        let field = match self {
            Pipe(a) => format!("{:?}", a),
            InMemory(a) => format!("{:?}", a),
            Compressed(a) => format!("{:?}", a),
            Restartable(a) => format!("{:?}", a),
            File(a) => format!("{:?}", a),
            Extension(_) => "Extension".to_string(),
            ExtensionSeek(_) => "ExtensionSeek".to_string(),
        };
        f.debug_tuple("Reader")
            .field(&field)
            .finish()
    }
}

/// Fusion trait for custom input sources which allow seeking.
pub trait ReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>;

    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64>;
}

impl Read for dyn ReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        ReadSeek::read(self, buf)
    }
}

impl Seek for dyn ReadSeek {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        ReadSeek::seek(self, pos)
    }
}

impl<R: Read + Seek> ReadSeek for R {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>{
        Read::read(self, buf)
    }

    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64>{
        Seek::seek(self, pos)
    }
}

/// Information about an [`Input`] source.
///
/// [`Input`]: struct.Input.html
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub date: Option<String>,

    pub channels: Option<u8>,
    pub start_time: Option<Duration>,
    pub duration: Option<Duration>,
    pub sample_rate: Option<u32>,
}

impl Metadata {
    /// Extract metadata and details from the output of
    /// `ffprobe`.
    pub fn from_ffprobe_json(value: &Value) -> Self {
        let format = value
            .as_object()
            .and_then(|m| m.get("format"));

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

        let tags = format
            .and_then(|m| m.get("tags"));

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
            .and_then(|v|
                v.iter()
                    .filter(|line|
                        line.get("codec_type").and_then(Value::as_str) == Some("audio")
                    )
                    .nth(0)
            );

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
    pub fn from_ytdl_uri(uri: &str) -> Self {
        let args = ["-s", "-j"];

        let out: Option<Value> = Command::new("youtube-dl")
            .args(&args)
            .arg(uri)
            .stdin(Stdio::null())
            .output()
            .ok()
            .and_then(|r| serde_json::from_reader(&r.stdout[..]).ok());

        let value = out.unwrap_or_default();
        let obj = value.as_object();

        let track = obj
            .and_then(|m| m.get("track"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let title = track.or_else(|| obj
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .map(str::to_string)
        );

        let artist = obj
            .and_then(|m| m.get("artist"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let r_date = obj
            .and_then(|m| m.get("release_date"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let date = r_date.or_else(|| obj
            .and_then(|m| m.get("upload_date"))
            .and_then(Value::as_str)
            .map(str::to_string)
        );

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

/// Data and metadata needed to correctly parse a [`Reader`]'s audio bytestream.
///
/// [`Reader`]: enum.Reader.html
#[derive(Debug)]
pub struct Input {
    pub metadata: Metadata,
    pub stereo: bool,
    pub reader: Reader,
    pub kind: Codec,
    pub container: Container,
}

impl Input {
    pub fn float_pcm(is_stereo: bool, reader: Reader) -> Input {
        Input {
            metadata: Default::default(),
            stereo: is_stereo,
            reader,
            kind: Codec::FloatPcm,
            container: Container::Raw,
        }
    }

    pub fn new(stereo: bool, reader: Reader, kind: Codec, container: Container, metadata: Option<Metadata>) -> Self {
        Input {
            metadata: metadata.unwrap_or_default(),
            stereo,
            reader,
            kind,
            container,
        }
    }

    pub fn is_seekable(&self) -> bool {
        self.reader.is_seekable()
    }

    pub fn is_stereo(&self) -> bool {
        self.stereo
    }

    pub fn get_type(&self) -> CodecType {
        (&self.kind).into()
    }

    #[inline]
    pub fn mix(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> usize {
        match self.reader.add_float_pcm_frame(float_buffer, self.stereo, volume) {
            Some(len) => len,
            None => 0,
        }
    }

    pub fn seek_time(&mut self, time: Duration) -> Option<Duration> {
        let future_pos = utils::timestamp_to_byte_count(time, self.stereo);
        Seek::seek(&mut self.reader, SeekFrom::Start(future_pos as u64))
            .ok()
            .map(|a| utils::byte_count_to_timestamp(a as usize, self.stereo))
    }
}

impl Read for Input {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        // This implementation of Read converts the input stream
        // to floating point output.
        let float_space = buffer.len() / mem::size_of::<f32>();
        let mut written_floats = 0;

        match &mut self.kind {
            Codec::Opus(decoder) => {
                if matches!(self.container, Container::Raw) {
                    return Err(IoError::new(
                        IoErrorKind::InvalidInput,
                        "Raw container cannot demarcate Opus frames.")
                    );
                }

                let mut opus_data_buffer = [0u8; 4000];
                let mut opus_out_buffer = [0f32; STEREO_FRAME_SIZE];

                let frame = self.container.next_frame_length(&mut self.reader, CodecType::Opus)?;

                let seen = Read::read(&mut self.reader, &mut opus_data_buffer[..frame.frame_len])?;
                let mut decoder = decoder.lock();
                let samples = decoder.decode_float(Some(&opus_data_buffer[..seen]), &mut opus_out_buffer[..], false)
                    .unwrap_or(0);

                let mut buffer = &mut buffer[..];
                while written_floats < float_space.min(samples) {
                    buffer.write_f32::<LittleEndian>(opus_out_buffer[written_floats])?;
                    written_floats += 1;
                }
                Ok(written_floats * mem::size_of::<f32>())
            },
            Codec::Pcm => {
                let mut buffer = &mut buffer[..];
                while written_floats < float_space {
                    if let Ok(signal) = self.reader.read_i16::<LittleEndian>() {
                        buffer.write_f32::<LittleEndian>(f32::from(signal) / 32768.0)?;
                        written_floats += 1;
                    } else {
                        break;
                    }
                }
                Ok(written_floats * mem::size_of::<f32>())
            },
            Codec::FloatPcm => {
                Read::read(&mut self.reader, buffer)
            },
        }
    }
}

/// Extension trait to pull frames of audio from a byte source.
pub trait ReadAudioExt {
    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize>;

    fn consume(&mut self, amt: usize) -> usize where Self: Sized;
}

impl<R: Read + Sized> ReadAudioExt for R {
    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], stereo: bool, volume: f32) -> Option<usize> {
        if stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let sample = match self.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * raw;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn consume(&mut self, amt: usize) -> usize {
        io::copy(&mut self.by_ref().take(amt as u64), &mut io::sink()).unwrap_or(0) as usize
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Input> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Input> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path.as_ref())
        .unwrap_or_else(|_e| (false, Default::default()));
    let stereo_val = if is_stereo.0 { "2" } else { "1" };

    _ffmpeg_optioned(path, &[], &[
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
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
/// use serenity::voice::input;
///
/// let stereo_val = "2";
///
/// let streamer = input::ffmpeg_optioned("./some_file.mp3", &[], &[
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
///```
pub fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    pre_input_args: &[&str],
    args: &[&str],
) -> Result<Input> {
    _ffmpeg_optioned(path.as_ref(), pre_input_args, args, None)
}

fn _ffmpeg_optioned(
    path: &OsStr,
    pre_input_args: &[&str],
    args: &[&str],
    is_stereo_known: Option<(bool, Metadata)>,
) -> Result<Input> {
    let (is_stereo, metadata) = is_stereo_known
        .or_else(|| is_stereo(path).ok())
        .unwrap_or_else(|| (false, Default::default()));

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(is_stereo, child_to_reader::<f32>(command), Codec::FloatPcm, Container::Raw, Some(metadata)))
}

/// Creates a streamed audio source from a DCA file.
/// Currently only accepts the [DCA1 format](https://github.com/bwmarrin/dca).
pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Input, DcaError> {
    _dca(path.as_ref())
}

fn _dca(path: &OsStr) -> StdResult<Input, DcaError> {
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

    let metadata: Metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
        .map_err(DcaError::InvalidMetadata)?
        .into();

    let stereo = metadata.channels == Some(2);

    Ok(Input {
        metadata,
        stereo,
        reader: Reader::File(reader),
        kind: Codec::Opus(
            Arc::new(Mutex::new(OpusDecoder::new(SAMPLE_RATE, Channels::Stereo).map_err(DcaError::Opus)?))
        ),
        container: Container::Dca,
    })
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub fn ytdl(uri: &str) -> Result<Input> {
    _ytdl(uri, &[])
}

fn _ytdl(uri: &str, pre_args: &[&str]) -> Result<Input> {
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
        "pcm_f32le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let metadata = Metadata::from_ytdl_uri(uri);

    debug!("[Voice] ytdl metadata {:?}", metadata);

    Ok(Input::new(true, child_to_reader::<f32>(ffmpeg), Codec::FloatPcm, Container::Raw, Some(metadata)))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub fn ytdl_search(name: &str) -> Result<Input> {
    ytdl(&format!("ytsearch1:{}",name))
}

fn is_stereo(path: &OsStr) -> Result<(bool, Metadata)> {
    let args = ["-v", "quiet", "-of", "json", "-show_format", "-show_streams", "-i"];

    let out = Command::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let metadata = Metadata::from_ffprobe_json(&value);

    debug!("[Voice] FFprobe metadata {:?}", metadata);

    if let Some(count) = metadata.channels {
        Ok((count == 2, metadata))
    } else {
        Err(Error::Voice(VoiceError::Streams))
    }
}

/// A wrapper around a method to create a new [`Input`] which
/// seeks backward by recreating the source.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`CompressedSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with single-use audio tracks which
/// may require looping or seeking, but where additional memory
/// cannot be spared. Forward seeks will drain the track until reaching
/// the desired timestamp.
///
/// [`Input`]: struct.Input.html
/// [`MemorySource`]: cached/struct.MemorySource.html
/// [`CompressedSource`]: cached/struct.CompressedSource.html
pub struct RestartableSource {
    position: usize,
    recreator: Box<dyn Fn(Option<Duration>) -> Result<Input> + Send + 'static>,
    source: Box<Input>,
}

impl RestartableSource {
    /// Create a new source, which can be restarted using a `recreator` function.
    pub fn new(recreator: impl Fn(Option<Duration>) -> Result<Input> + Send + 'static) -> Result<Self> {
        recreator(None)
            .map(move |source| {
                Self {
                    position: 0,
                    recreator: Box::new(recreator),
                    source: Box::new(source),
                }
            })
    }

    /// Create a new restartable ffmpeg source for a local file.
    pub fn ffmpeg<P: AsRef<OsStr> + Send + Clone + Copy + 'static>(path: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {

                let is_stereo = is_stereo(path.as_ref())
                    .unwrap_or_else(|_e| (false, Default::default()));
                let stereo_val = if is_stereo.0 { "2" } else { "1" };

                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());
                _ffmpeg_optioned(path.as_ref(), &[
                    "-ss",
                    &ts,
                    ],

                    &[
                    "-f",
                    "s16le",
                    "-ac",
                    stereo_val,
                    "-ar",
                    "48000",
                    "-acodec",
                    "pcm_f32le",
                    "-",
                ], Some(is_stereo))
            } else {
                ffmpeg(path)
            }
        })
    }

    /// Create a new restartable ytdl source.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl<P: AsRef<str> + Send + Clone + 'static>(uri: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {
                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());

                _ytdl(uri.as_ref(), &["-ss",&ts,])
            } else {
                ytdl(uri.as_ref())
            }
        })
    }

    /// Create a new restartable ytdl source, using the first result of a youtube search.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl_search(name: &str) -> Result<Self> {
       Self::ytdl(format!("ytsearch1:{}",name))
    }
}

impl Debug for RestartableSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), FormatError> {
        f.debug_struct("Reader")
            .field("position", &self.position)
            .field("recreator", &"<fn>")
            .field("source", &self.source)
            .finish()
    }
}

impl From<RestartableSource> for Input {
    fn from(mut src: RestartableSource) -> Self {
        let kind = src.source.kind.clone();
        Self {
            metadata: src.source.metadata.take(),
            stereo: src.source.stereo,
            kind,
            container: src.source.container,

            reader: Reader::Restartable(src), 
        }
    }
}

impl Read for RestartableSource {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.source.read(buffer)
            .map(|a| { self.position += a; a })
    }
}

impl Seek for RestartableSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let _local_pos = self.position as u64;

        use SeekFrom::*;
        match pos {
            Start(offset) => {
                let stereo = self.source.stereo;
                let _current_ts = utils::byte_count_to_timestamp(self.position, stereo);
                let offset = offset as usize;

                if offset < self.position {
                    // FIXME: don't unwrap
                    self.source = Box::new(
                        (self.recreator)(
                            Some(utils::byte_count_to_timestamp(offset, stereo))
                        ).unwrap()
                    );
                    self.position = offset;
                } else {
                    self.position += self.source.consume(offset - self.position);
                }

                Ok(offset as u64)
            },
            End(_offset) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "End point for RestartableSources is not known.")),
            Current(_offset) => unimplemented!(),
        }
    }
}
