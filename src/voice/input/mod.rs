//! Raw audio input data streams and sources.

pub mod cached;
pub mod utils;

use audiopus::coder::Decoder as OpusDecoder;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cached::{CompressedSource, MemorySource};
use crate::{
    internal::prelude::*,
    prelude::SerenityError,
    voice::{
        constants::*,
        VoiceError,
    },
};
use log::{debug, warn};
use parking_lot::Mutex;
use std::{
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

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum InputType {
    Opus,
    Pcm,
    FloatPcm,
}

#[non_exhaustive]
pub enum InputTypeData {
    Opus(OpusDecoder),
    Pcm,
    FloatPcm,
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

/// Data and metadata needed to correctly parse a [`Reader`]'s audio bytestream.
///
/// [`Reader`]: enum.Reader.html
#[derive(Debug)]
pub struct Input {
    pub stereo: bool,
    pub reader: Reader,
    pub kind: InputType,
    pub decoder: Option<Arc<Mutex<OpusDecoder>>>,
}

impl Input {
    pub fn float_pcm(is_stereo: bool, reader: Reader) -> Input {
        Input {
            stereo: is_stereo,
            reader,
            kind: InputType::FloatPcm,
            decoder: None,
        }
    }

    pub fn new(stereo: bool, reader: Reader, kind: InputType, decoder: Option<Arc<Mutex<OpusDecoder>>>) -> Self {
        Input {
            stereo,
            reader,
            kind,
            decoder,
        }
    }

    pub fn is_seekable(&self) -> bool {
        self.reader.is_seekable()
    }

    pub fn is_stereo(&self) -> bool {
        self.stereo
    }

    pub fn get_type(&self) -> InputType {
        self.kind
    }

    #[inline]
    pub fn mix(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> usize {
        match self.kind {
            InputType::Opus => unimplemented!(),
                    // if self.reader.decode_and_add_opus_frame(&mut float_buffer, vol).is_some() {
                    //     0 //; opus_frame.len()
                    // } else {
                    //     0
                    // },
            InputType::Pcm => {
                match self.reader.add_pcm_frame(float_buffer, self.stereo, volume) {
                    Some(len) => len,
                    None => 0,
                }
            },
            InputType::FloatPcm => {
                match self.reader.add_float_pcm_frame(float_buffer, self.stereo, volume) {
                    Some(len) => len,
                    None => 0,
                }
            },
        }
    }

    // fixme: make this relative.
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
        // Read::read(&mut self.reader, buffer)
        match self.kind {
            InputType::Opus => unimplemented!(),
            InputType::Pcm => {
                //FIXME: probably stifiling an error.
                let mut buffer = &mut buffer[..];
                while written_floats < float_space {
                    if let Ok(signal) = self.reader.read_i16::<LittleEndian>() {
                        buffer.write_f32::<LittleEndian>(f32::from(signal) / 32768.0)?;
                        written_floats += 1;
                    } else {
                        break;
                    }
                }
                Ok(written_floats)
            },
            InputType::FloatPcm => {
                Read::read(&mut self.reader, buffer)
            },
        }
    }
}

/// Extension trait to pull frames of audio from a byte source.
pub trait ReadAudioExt {
    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize>;

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize>;

    fn consume(&mut self, amt: usize) -> usize where Self: Sized;
}

impl<R: Read + Sized> ReadAudioExt for R {
    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        // Duplicate this to avoid repeating the stereo check.
        // This should let us unconditionally duplicate samples in the main loop body.
        if true_stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let raw = match self.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        warn!("abrupt end? {:?}", e);
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = f32::from(raw) / 32768.0;

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        warn!("abrupt end? {:?}", e);
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * f32::from(raw) / 32768.0;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], stereo: bool, volume: f32) -> Option<usize> {
        if stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let sample = match self.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        warn!("abrupt end? {:?}", e);
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
                        warn!("abrupt end? {:?}", e);
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

// impl AudioSource for Input {
//     // FIXME: COMPLETELY BROKEN
//     // this assumes DCA exculsively.
//     // DOES NOT WORK FOR OPUS IN THE GENERAL CASE.
//     fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
//         match self.reader.read_i16::<LittleEndian>() {
//             Ok(size) => {
//                 if size <= 0 {
//                     warn!("Invalid opus frame size: {}", size);
//                     return None;
//                 }

//                 let mut frame = Vec::with_capacity(size as usize);

//                 {
//                     let reader = self.reader.by_ref();

//                     if reader.take(size as u64).read_to_end(&mut frame).is_err() {
//                         return None;
//                     }
//                 }

//                 Some(frame)
//             },
//             Err(ref e) => if e.kind() == IoErrorKind::UnexpectedEof {
//                 Some(Vec::new())
//             } else {
//                 None
//             },
//         }
//     }

//     fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
//         let decoder_lock = self.decoder.as_mut()?.clone();
//         let frame = self.read_opus_frame()?;
//         let mut local_buf = [0f32; 960 * 2];

//         let count = {
//             let mut decoder = decoder_lock.lock();

//             decoder.decode_float(frame.as_slice(), &mut local_buf[..], false).ok()?
//         };

//         for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
//             *float_buffer_element += local_buf[i] * volume;
//         }

//         Some(count)
//     }
// }


/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Input> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Input> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo { "2" } else { "1" };

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
    is_stereo_known: Option<bool>,
) -> Result<Input> {
    let is_stereo = is_stereo_known
        .or_else(|| is_stereo(path).ok())
        .unwrap_or(false);

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(is_stereo, child_to_reader::<f32>(command), InputType::FloatPcm, None))
}

// /// Creates a streamed audio source from a DCA file.
// /// Currently only accepts the [DCA1 format](https://github.com/bwmarrin/dca).
// pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<dyn AudioSource>, DcaError> {
//     _dca(path.as_ref())
// }

// fn _dca(path: &OsStr) -> StdResult<Box<dyn AudioSource>, DcaError> {
//     let file = File::open(path).map_err(DcaError::IoError)?;

//     let mut reader = BufReader::new(file);

//     let mut header = [0u8; 4];

//     // Read in the magic number to verify it's a DCA file.
//     reader.read_exact(&mut header).map_err(DcaError::IoError)?;

//     if header != b"DCA1"[..] {
//         return Err(DcaError::InvalidHeader);
//     }

//     reader.read_exact(&mut header).map_err(DcaError::IoError)?;

//     let size = (&header[..]).read_i32::<LittleEndian>().unwrap();

//     // Sanity check
//     if size < 2 {
//         return Err(DcaError::InvalidSize(size));
//     }

//     let mut raw_json = Vec::with_capacity(size as usize);

//     {
//         let json_reader = reader.by_ref();
//         json_reader
//             .take(size as u64)
//             .read_to_end(&mut raw_json)
//             .map_err(DcaError::IoError)?;
//     }

//     let metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
//         .map_err(DcaError::InvalidMetadata)?;

//     Ok(opus(metadata.is_stereo(), reader))
// }

// /// Creates an Opus audio source. This makes certain assumptions: namely, that the input stream
// /// is composed ONLY of opus frames of the variety that Discord expects.
// ///
// /// If you want to decode a `.opus` file, use [`ffmpeg`]
// ///
// /// [`ffmpeg`]: fn.ffmpeg.html
// pub fn opus<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource + Send> {
//     Box::new(Input {
//         stereo: is_stereo,
//         reader,
//         kind: InputType::Opus,
//         decoder: Some(
//             Arc::new(Mutex::new(
//                 // We always want to decode *to* stereo, for mixing reasons.
//                 OpusDecoder::new(audio::SAMPLE_RATE, Channels::Stereo).unwrap()
//             ))
//         ),
//     })
// }

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

    Ok(Input::new(true, child_to_reader::<f32>(ffmpeg), InputType::FloatPcm, None))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub fn ytdl_search(name: &str) -> Result<Input> {
    ytdl(&format!("ytsearch1:{}",name))
}

fn is_stereo(path: &OsStr) -> Result<bool> {
    let args = ["-v", "quiet", "-of", "json", "-show_streams", "-i"];

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

/// A wrapper around a method to create a new [`AudioSource`] which
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
/// [`AudioSource`]: trait.AudioSource.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`CompressedSource`]: struct.CompressedSource.html
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

                let is_stereo = is_stereo(path.as_ref()).unwrap_or(false);
                let stereo_val = if is_stereo { "2" } else { "1" };

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
    fn from(src: RestartableSource) -> Self {
        Self {
            stereo: src.source.stereo,
            kind: src.source.kind,
            decoder: None,

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
