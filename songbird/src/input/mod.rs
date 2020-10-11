//! Raw audio input data streams and sources.
//!
//! [`Input`] is handled in songbird by combining metadata with:
//!  * A 48kHz audio bytestream, via [`Reader`],
//!  * A [`Container`] describing the framing mechanism of the bytestream,
//!  * A [`Codec`], defining the format of audio frames.
//!
//! [`Input`]: struct.Input.html
//! [`Container`]: struct.Container.html
//! [`Codec`]: struct.Codec.html

pub mod cached;
mod dca;
pub mod error;
pub mod utils;

use crate::constants::*;
use audiopus::{
    coder::{Decoder as OpusDecoder, GenericCtl},
    Channels,
    Error as OpusError,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cached::OpusCompressor;
use dca::DcaMetadata;
use error::{DcaError, Error, Result};
use futures::executor;
use parking_lot::Mutex;
use serde_json::Value;
use std::{
    convert::TryFrom,
    ffi::OsStr,
    fmt::{Debug, Error as FormatError, Formatter},
    fs::File,
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
    result::Result as StdResult,
    sync::Arc,
    time::Duration,
};
use streamcatcher::{Catcher, TxCatcher};
use tokio::{fs::File as TokioFile, io::AsyncReadExt, process::Command as TokioCommand};
use tracing::{debug, error};

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
        }
    }
}

impl TryFrom<CodecType> for Codec {
    type Error = Error;

    fn try_from(f: CodecType) -> Result<Self> {
        use CodecType::*;

        match f {
            Opus => Ok(Codec::Opus(OpusDecoderState::new()?)),
            Pcm => Ok(Codec::Pcm),
            FloatPcm => Ok(Codec::FloatPcm),
        }
    }
}

/// State used to decode input bytes of an [`Input`].
///
/// [`Input`]: struct.Input.html
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Codec {
    Opus(OpusDecoderState),
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

#[derive(Clone, Debug)]
pub struct OpusDecoderState {
    pub decoder: Arc<Mutex<OpusDecoder>>,
    /// Controls whether this source allows direct Opus frame passthrough.
    /// Defaults to `true`.
    ///
    /// Enabling this flag is a promise from the programmer to the audio core
    /// that the source has been encoded at 48kHz, using 20ms long frames.
    /// If you cannot guarantee this, disable this flag (or else risk nasal demons)
    /// and bizarre audio behaviour.
    pub allow_passthrough: bool,
    current_frame: Vec<f32>,
    frame_pos: usize,
    should_reset: bool,
}

impl OpusDecoderState {
    pub fn new() -> StdResult<Self, OpusError> {
        Ok(Self::from_decoder(OpusDecoder::new(
            SAMPLE_RATE,
            Channels::Stereo,
        )?))
    }

    pub fn from_decoder(decoder: OpusDecoder) -> Self {
        Self {
            decoder: Arc::new(Mutex::new(decoder)),
            allow_passthrough: true,
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            should_reset: false,
        }
    }
}

/// Information used in audio frame detection.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub header_len: usize,
    pub frame_len: usize,
}

/// Marker for decoding framed input files.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Container {
    Raw,
    Dca { first_frame: usize },
}

impl Container {
    pub fn next_frame_length(
        &mut self,
        mut reader: impl Read,
        input: CodecType,
    ) -> IoResult<Frame> {
        use Container::*;

        match self {
            Raw => Ok(Frame {
                header_len: 0,
                frame_len: input.sample_len(),
            }),
            Dca { .. } => reader.read_i16::<LittleEndian>().map(|frame_len| Frame {
                header_len: mem::size_of::<i16>(),
                frame_len: frame_len.max(0) as usize,
            }),
        }
    }

    pub fn try_seek_trivial(&self, input: CodecType) -> Option<usize> {
        use Container::*;

        match self {
            Raw => Some(input.sample_len()),
            _ => None,
        }
    }

    pub fn input_start(&self) -> usize {
        use Container::*;

        match self {
            Raw => 0,
            Dca { first_frame } => *first_frame,
        }
    }
}

/// Handle for a child process which ensures that any subprocesses are properly closed
/// on drop.
#[derive(Debug)]
pub struct ChildContainer(Child);

fn child_to_reader<T>(child: Child) -> Reader {
    Reader::Pipe(BufReader::with_capacity(
        STEREO_FRAME_SIZE * mem::size_of::<T>() * CHILD_BUFFER_LEN,
        ChildContainer(child),
    ))
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
    fn drop(&mut self) {
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
    Memory(Catcher<Box<Reader>>),
    Compressed(TxCatcher<Box<Input>, OpusCompressor>),
    Restartable(Restartable),
    File(BufReader<File>),
    Extension(Box<dyn Read + Send>),
    ExtensionSeek(Box<dyn ReadSeek + Send>),
}

impl Reader {
    fn is_seekable(&self) -> bool {
        use Reader::*;

        match self {
            Restartable(_) | Compressed(_) | Memory(_) => true,
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
            Memory(a) => Read::read(a, buffer),
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
                "Seeking not supported on Reader of this type.",
            )),
            Memory(a) => Seek::seek(a, pos),
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
            Memory(a) => format!("{:?}", a),
            Compressed(a) => format!("{:?}", a),
            Restartable(a) => format!("{:?}", a),
            File(a) => format!("{:?}", a),
            Extension(_) => "Extension".to_string(),
            ExtensionSeek(_) => "ExtensionSeek".to_string(),
        };
        f.debug_tuple("Reader").field(&field).finish()
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
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        Read::read(self, buf)
    }

    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
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
    pub async fn from_ytdl_uri(uri: &str) -> Self {
        let args = ["-s", "-j"];

        let out: Option<Value> = TokioCommand::new("youtube-dl")
            .args(&args)
            .arg(uri)
            .stdin(Stdio::null())
            .output()
            .await
            .ok()
            .and_then(|r| serde_json::from_reader(&r.stdout[..]).ok());

        let value = out.unwrap_or_default();
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

        let artist = obj
            .and_then(|m| m.get("artist"))
            .and_then(Value::as_str)
            .map(str::to_string);

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
    pos: usize,
}

impl Input {
    pub fn float_pcm(is_stereo: bool, reader: Reader) -> Input {
        Input {
            metadata: Default::default(),
            stereo: is_stereo,
            reader,
            kind: Codec::FloatPcm,
            container: Container::Raw,
            pos: 0,
        }
    }

    pub fn new(
        stereo: bool,
        reader: Reader,
        kind: Codec,
        container: Container,
        metadata: Option<Metadata>,
    ) -> Self {
        Input {
            metadata: metadata.unwrap_or_default(),
            stereo,
            reader,
            kind,
            container,
            pos: 0,
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
        match self.add_float_pcm_frame(float_buffer, self.stereo, volume) {
            Some(len) => len,
            None => 0,
        }
    }

    pub fn seek_time(&mut self, time: Duration) -> Option<Duration> {
        let future_pos = utils::timestamp_to_byte_count(time, self.stereo);
        Seek::seek(self, SeekFrom::Start(future_pos as u64))
            .ok()
            .map(|a| utils::byte_count_to_timestamp(a as usize, self.stereo))
    }

    fn read_inner(&mut self, buffer: &mut [u8], ignore_decode: bool) -> IoResult<usize> {
        // This implementation of Read converts the input stream
        // to floating point output.
        let sample_len = mem::size_of::<f32>();
        let float_space = buffer.len() / sample_len;
        let mut written_floats = 0;

        // FIXME: this is a little bit backwards, and assumes the bottom cases are always raw..
        let out = match &mut self.kind {
            Codec::Opus(decoder_state) => {
                if matches!(self.container, Container::Raw) {
                    return Err(IoError::new(
                        IoErrorKind::InvalidInput,
                        "Raw container cannot demarcate Opus frames.",
                    ));
                }

                if ignore_decode {
                    // If we're less than one frame away from the end of cheap seeking,
                    // then we must decode to make sure the next starting offset is correct.

                    // Step one: use up the remainder of the frame.
                    let mut aud_skipped =
                        decoder_state.current_frame.len() - decoder_state.frame_pos;

                    decoder_state.frame_pos = 0;
                    decoder_state.current_frame.truncate(0);

                    // Step two: take frames if we can.
                    while buffer.len() - aud_skipped >= STEREO_FRAME_BYTE_SIZE {
                        decoder_state.should_reset = true;

                        let frame = self
                            .container
                            .next_frame_length(&mut self.reader, CodecType::Opus)?;
                        self.reader.consume(frame.frame_len);

                        aud_skipped += STEREO_FRAME_BYTE_SIZE;
                    }

                    Ok(aud_skipped)
                } else {
                    // get new frame *if needed*
                    if decoder_state.frame_pos == decoder_state.current_frame.len() {
                        let mut decoder = decoder_state.decoder.lock();

                        if decoder_state.should_reset {
                            decoder
                                .reset_state()
                                .expect("Critical failure resetting decoder.");
                            decoder_state.should_reset = false;
                        }
                        let frame = self
                            .container
                            .next_frame_length(&mut self.reader, CodecType::Opus)?;

                        let mut opus_data_buffer = [0u8; 4000];

                        decoder_state
                            .current_frame
                            .resize(decoder_state.current_frame.capacity(), 0.0);

                        let seen =
                            Read::read(&mut self.reader, &mut opus_data_buffer[..frame.frame_len])?;

                        let samples = decoder
                            .decode_float(
                                Some(&opus_data_buffer[..seen]),
                                &mut decoder_state.current_frame[..],
                                false,
                            )
                            .unwrap_or(0);

                        decoder_state.current_frame.truncate(2 * samples);
                        decoder_state.frame_pos = 0;
                    }

                    // read from frame which is present.
                    let mut buffer = &mut buffer[..];

                    let start = decoder_state.frame_pos;
                    let to_write = float_space.min(decoder_state.current_frame.len() - start);
                    for val in &decoder_state.current_frame[start..start + float_space] {
                        buffer.write_f32::<LittleEndian>(*val)?;
                    }
                    decoder_state.frame_pos += to_write;
                    written_floats = to_write;

                    Ok(written_floats * mem::size_of::<f32>())
                }
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
            Codec::FloatPcm => Read::read(&mut self.reader, buffer),
        };

        out.map(|v| {
            self.pos += v;
            v
        })
    }

    fn cheap_consume(&mut self, count: usize) -> IoResult<usize> {
        let mut scratch = [0u8; STEREO_FRAME_BYTE_SIZE * 4];
        let len = scratch.len();
        let mut done = 0;

        loop {
            let read = self.read_inner(&mut scratch[..len.min(count - done)], true)?;
            if read == 0 {
                break;
            }
            done += read;
        }

        Ok(done)
    }

    pub(crate) fn supports_passthrough(&self) -> bool {
        match &self.kind {
            Codec::Opus(state) => state.allow_passthrough,
            _ => false,
        }
    }

    pub(crate) fn read_opus_frame(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        // Called in event of opus passthrough.
        if let Codec::Opus(state) = &mut self.kind {
            // step 1: align to frame.
            self.pos += state.current_frame.len() - state.frame_pos;

            state.frame_pos = 0;
            state.current_frame.truncate(0);

            // step 2: read new header.
            let frame = self
                .container
                .next_frame_length(&mut self.reader, CodecType::Opus)?;

            // step 3: read in bytes.
            self.reader
                .read_exact(&mut buffer[..frame.frame_len])
                .map(|_| {
                    self.pos += STEREO_FRAME_BYTE_SIZE;
                    frame.frame_len
                })
        } else {
            Err(IoError::new(
                IoErrorKind::InvalidInput,
                "Frame passthrough not supported for this file.",
            ))
        }
    }
}

impl Read for Input {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.read_inner(buffer, false)
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let mut target = self.pos;
        match pos {
            SeekFrom::Start(pos) => {
                target = pos as usize;
            },
            SeekFrom::Current(rel) => {
                target = target.wrapping_add(rel as usize);
            },
            SeekFrom::End(_pos) => unimplemented!(),
        }

        debug!("Seeking to {:?}", pos);

        (if target == self.pos {
            Ok(0)
        } else if let Some(conversion) = self.container.try_seek_trivial(self.get_type()) {
            let inside_target = (target * conversion) / mem::size_of::<f32>();
            Seek::seek(&mut self.reader, SeekFrom::Start(inside_target as u64)).map(|inner_dest| {
                let outer_dest = ((inner_dest as usize) * mem::size_of::<f32>()) / conversion;
                self.pos = outer_dest;
                outer_dest
            })
        } else if target > self.pos {
            // seek in the next amount, disabling decoding if need be.
            let shift = target - self.pos;
            self.cheap_consume(shift)
        } else {
            // start from scratch, then seek in...
            Seek::seek(
                &mut self.reader,
                SeekFrom::Start(self.container.input_start() as u64),
            )?;

            self.cheap_consume(target)
        })
        .map(|_| self.pos as u64)
    }
}

/// Extension trait to pull frames of audio from a byte source.
pub(crate) trait ReadAudioExt {
    fn add_float_pcm_frame(
        &mut self,
        float_buffer: &mut [f32; STEREO_FRAME_SIZE],
        true_stereo: bool,
        volume: f32,
    ) -> Option<usize>;

    fn consume(&mut self, amt: usize) -> usize
    where
        Self: Sized;
}

impl<R: Read + Sized> ReadAudioExt for R {
    fn add_float_pcm_frame(
        &mut self,
        float_buffer: &mut [f32; STEREO_FRAME_SIZE],
        stereo: bool,
        volume: f32,
    ) -> Option<usize> {
        // IDEA: Read in 8 floats at a time, then use iterator code
        // to gently nudge the compiler into vectorising for us.
        // Max SIMD float32 lanes is 8 on AVX, older archs use a divisor of this
        // e.g., 4.
        const SAMPLE_LEN: usize = mem::size_of::<f32>();
        let mut simd_float_bytes = [0u8; 8 * SAMPLE_LEN];
        let mut simd_float_buf = [0f32; 8];

        let mut frame_pos = 0;

        // Code duplication here is because unifying these codepaths
        // with a dynamic chunk size is not zero-cost.
        if stereo {
            let mut max_bytes = STEREO_FRAME_BYTE_SIZE;

            while frame_pos < float_buffer.len() {
                let progress = self
                    .read(&mut simd_float_bytes[..max_bytes.min(8 * SAMPLE_LEN)])
                    .and_then(|byte_len| {
                        let target = byte_len / SAMPLE_LEN;
                        (&simd_float_bytes[..byte_len])
                            .read_f32_into::<LittleEndian>(&mut simd_float_buf[..target])
                            .map(|_| target)
                    })
                    .map(|f32_len| {
                        let new_pos = frame_pos + f32_len;
                        for (el, new_el) in float_buffer[frame_pos..new_pos]
                            .iter_mut()
                            .zip(&simd_float_buf[..f32_len])
                        {
                            *el += volume * new_el;
                        }
                        (new_pos, f32_len)
                    });

                match progress {
                    Ok((new_pos, delta)) => {
                        frame_pos = new_pos;
                        max_bytes -= delta * SAMPLE_LEN;

                        if delta == 0 {
                            break;
                        }
                    },
                    Err(ref e) =>
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(frame_pos)
                        } else {
                            error!("Input died unexpectedly: {:?}", e);
                            None
                        },
                }
            }
        } else {
            let mut max_bytes = MONO_FRAME_BYTE_SIZE;

            while frame_pos < float_buffer.len() {
                let progress = self
                    .read(&mut simd_float_bytes[..max_bytes.min(8 * SAMPLE_LEN)])
                    .and_then(|byte_len| {
                        let target = byte_len / SAMPLE_LEN;
                        (&simd_float_bytes[..byte_len])
                            .read_f32_into::<LittleEndian>(&mut simd_float_buf[..target])
                            .map(|_| target)
                    })
                    .map(|f32_len| {
                        let new_pos = frame_pos + (2 * f32_len);
                        for (els, new_el) in float_buffer[frame_pos..new_pos]
                            .chunks_exact_mut(2)
                            .zip(&simd_float_buf[..f32_len])
                        {
                            let sample = volume * new_el;
                            els[0] += sample;
                            els[1] += sample;
                        }
                        (new_pos, f32_len)
                    });

                match progress {
                    Ok((new_pos, delta)) => {
                        frame_pos = new_pos;
                        max_bytes -= delta * SAMPLE_LEN;

                        if delta == 0 {
                            break;
                        }
                    },
                    Err(ref e) =>
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(frame_pos)
                        } else {
                            error!("Input died unexpectedly: {:?}", e);
                            None
                        },
                }
            }
        }

        Some(frame_pos * SAMPLE_LEN)
    }

    fn consume(&mut self, amt: usize) -> usize {
        io::copy(&mut self.by_ref().take(amt as u64), &mut io::sink()).unwrap_or(0) as usize
    }
}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub async fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Input> {
    _ffmpeg(path.as_ref()).await
}

async fn _ffmpeg(path: &OsStr) -> Result<Input> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path)
        .await
        .unwrap_or_else(|_e| (false, Default::default()));
    let stereo_val = if is_stereo.0 { "2" } else { "1" };

    _ffmpeg_optioned(
        path,
        &[],
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
        ],
        Some(is_stereo),
    )
    .await
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
/// use songbird::input;
///
/// let stereo_val = "2";
///
/// let streamer = futures::executor::block_on(input::ffmpeg_optioned("./some_file.mp3", &[], &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ]));
///```
pub async fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    pre_input_args: &[&str],
    args: &[&str],
) -> Result<Input> {
    _ffmpeg_optioned(path.as_ref(), pre_input_args, args, None).await
}

async fn _ffmpeg_optioned(
    path: &OsStr,
    pre_input_args: &[&str],
    args: &[&str],
    is_stereo_known: Option<(bool, Metadata)>,
) -> Result<Input> {
    let (is_stereo, metadata) = if let Some(vals) = is_stereo_known {
        vals
    } else {
        is_stereo(path)
            .await
            .ok()
            .unwrap_or_else(|| (false, Default::default()))
    };

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(
        is_stereo,
        child_to_reader::<f32>(command),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))
}

/// Creates a streamed audio source from a DCA file.
/// Currently only accepts the [DCA1 format](https://github.com/bwmarrin/dca).
pub async fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Input, DcaError> {
    _dca(path.as_ref()).await
}

async fn _dca(path: &OsStr) -> StdResult<Input, DcaError> {
    let mut reader = TokioFile::open(path).await.map_err(DcaError::IoError)?;

    let mut header = [0u8; 4];

    // Read in the magic number to verify it's a DCA file.
    reader
        .read_exact(&mut header)
        .await
        .map_err(DcaError::IoError)?;

    if header != b"DCA1"[..] {
        return Err(DcaError::InvalidHeader);
    }

    let size = reader
        .read_i32_le()
        .await
        .map_err(|_| DcaError::InvalidHeader)?;

    // Sanity check
    if size < 2 {
        return Err(DcaError::InvalidSize(size));
    }

    let mut raw_json = Vec::with_capacity(size as usize);

    let mut json_reader = reader.take(size as u64);

    json_reader
        .read_to_end(&mut raw_json)
        .await
        .map_err(DcaError::IoError)?;

    let reader = BufReader::new(json_reader.into_inner().into_std().await);

    let metadata: Metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
        .map_err(DcaError::InvalidMetadata)?
        .into();

    let stereo = metadata.channels == Some(2);

    Ok(Input::new(
        stereo,
        Reader::File(reader),
        Codec::Opus(OpusDecoderState::new().map_err(DcaError::Opus)?),
        Container::Dca {
            first_frame: (size as usize) + mem::size_of::<i32>() + header.len(),
        },
        Some(metadata),
    ))
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub async fn ytdl(uri: &str) -> Result<Input> {
    _ytdl(uri, &[]).await
}

async fn _ytdl(uri: &str, pre_args: &[&str]) -> Result<Input> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        uri,
        "-o",
        "-",
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
        .stdin(youtube_dl.stdout.ok_or(Error::Stdout)?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let metadata = Metadata::from_ytdl_uri(uri).await;

    debug!("[Voice] ytdl metadata {:?}", metadata);

    Ok(Input::new(
        true,
        child_to_reader::<f32>(ffmpeg),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub async fn ytdl_search(name: &str) -> Result<Input> {
    ytdl(&format!("ytsearch1:{}", name)).await
}

async fn is_stereo(path: &OsStr) -> Result<(bool, Metadata)> {
    let args = [
        "-v",
        "quiet",
        "-of",
        "json",
        "-show_format",
        "-show_streams",
        "-i",
    ];

    let out = TokioCommand::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .await?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let metadata = Metadata::from_ffprobe_json(&value);

    debug!("[Voice] FFprobe metadata {:?}", metadata);

    if let Some(count) = metadata.channels {
        Ok((count == 2, metadata))
    } else {
        Err(Error::Streams)
    }
}

/// A wrapper around a method to create a new [`Input`] which
/// seeks backward by recreating the source.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`Compressed`]
/// and [`Memory`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with single-use audio tracks which
/// may require looping or seeking, but where additional memory
/// cannot be spared. Forward seeks will drain the track until reaching
/// the desired timestamp.
///
/// [`Input`]: struct.Input.html
/// [`Memory`]: cached/struct.Memory.html
/// [`Compressed`]: cached/struct.Compressed.html
pub struct Restartable {
    position: usize,
    recreator: Box<dyn Restart + Send + 'static>,
    source: Box<Input>,
}

impl Restartable {
    /// Create a new source, which can be restarted using a `recreator` function.
    pub fn new(mut recreator: impl Restart + Send + 'static) -> Result<Self> {
        recreator.call_restart(None).map(move |source| Self {
            position: 0,
            recreator: Box::new(recreator),
            source: Box::new(source),
        })
    }

    /// Create a new restartable ffmpeg source for a local file.
    pub fn ffmpeg<P: AsRef<OsStr> + Send + Clone + 'static>(path: P) -> Result<Self> {
        Self::new(FfmpegRestarter { path: path.clone() })
    }

    /// Create a new restartable ytdl source.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl<P: AsRef<str> + Send + Clone + 'static>(uri: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {
                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());

                executor::block_on(_ytdl(uri.as_ref(), &["-ss", &ts]))
            } else {
                executor::block_on(ytdl(uri.as_ref()))
            }
        })
    }

    /// Create a new restartable ytdl source, using the first result of a youtube search.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl_search(name: &str) -> Result<Self> {
        Self::ytdl(format!("ytsearch1:{}", name))
    }
}

pub trait Restart {
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input>;
}

struct FfmpegRestarter<P>
where
    P: AsRef<OsStr> + Send,
{
    path: P,
}

impl<P> Restart for FfmpegRestarter<P>
where
    P: AsRef<OsStr> + Send,
{
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        executor::block_on(async {
            if let Some(time) = time {
                let is_stereo = is_stereo(self.path.as_ref())
                    .await
                    .unwrap_or_else(|_e| (false, Default::default()));
                let stereo_val = if is_stereo.0 { "2" } else { "1" };

                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());
                _ffmpeg_optioned(
                    self.path.as_ref(),
                    &["-ss", &ts],
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
                    ],
                    Some(is_stereo),
                )
                .await
            } else {
                ffmpeg(self.path.as_ref()).await
            }
        })
    }
}

impl<P> Restart for P
where
    P: FnMut(Option<Duration>) -> Result<Input> + Send + 'static,
{
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        (self)(time)
    }
}

impl Debug for Restartable {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), FormatError> {
        f.debug_struct("Reader")
            .field("position", &self.position)
            .field("recreator", &"<fn>")
            .field("source", &self.source)
            .finish()
    }
}

impl From<Restartable> for Input {
    fn from(mut src: Restartable) -> Self {
        let kind = src.source.kind.clone();
        let meta = Some(src.source.metadata.take());
        let stereo = src.source.stereo;
        let container = src.source.container;
        Input::new(stereo, Reader::Restartable(src), kind, container, meta)
    }
}

impl Read for Restartable {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        Read::read(&mut self.source, buffer).map(|a| {
            self.position += a;
            a
        })
    }
}

impl Seek for Restartable {
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
                        self.recreator
                            .call_restart(Some(utils::byte_count_to_timestamp(offset, stereo)))
                            .unwrap(),
                    );
                    self.position = offset;
                } else {
                    self.position += self.source.consume(offset - self.position);
                }

                Ok(offset as u64)
            },
            End(_offset) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "End point for Restartables is not known.",
            )),
            Current(_offset) => unimplemented!(),
        }
    }
}
