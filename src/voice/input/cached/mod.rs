//! Thread-safe, shareable, and reusable input wrappers which store an audio stream in memory.
//!
//! These wrap an [`Input`] stream with a cache to:
//! * Allow low runtime-cost seeking on one-way streams such as [`ffmpeg`].
//! * Share audio data between calls and threads, for commonly reused sound effects or
//!   interactive experiences.
//!
//! Cached sources share the [same underlying design](https://mcfelix.me/blog/shared-buffers/),
//! where reads which do not exceed the current stored length are lock-free. New data enters a rope
//! data structure, and is copied to contiguous storage on completion.
//!
//! [`Input`]: ../struct.Input.html
//! [`ffmpeg`]: ../fn.ffmpeg.html

#[cfg(test)]
mod tests;

use audiopus::{
    Application,
    Bitrate,
    Channels,
    Error as OpusError,
    ErrorCode as OpusErrorCode,
    coder::GenericCtl,
    coder::Decoder as OpusDecoder,
    coder::Encoder as OpusEncoder,
    Result as OpusResult,
    SampleRate,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::{
    internal::prelude::*,
    voice::{
        constants::*,
        input::InputType,
    },
};
use log::{debug, info, warn};
use parking_lot::{
    lock_api::MutexGuard,
    Mutex,
};
use std::{
    cell::UnsafeCell,
    collections::LinkedList,    
    io::{
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
        Seek,
        SeekFrom,
    },
    mem::{
        self,
        ManuallyDrop,
    },
    ops::{Add, AddAssign, Sub, SubAssign},
    sync::{
        atomic::{
            AtomicU8,
            AtomicUsize, 
            Ordering,
        },
        Arc,
    },
    time::Duration,
};
use super::{utils, Input, ReadAudioExt, Reader};

/// Expected amount of time that an input should last.
#[derive(Copy, Clone, Debug)]
pub enum LengthHint {
    Bytes(usize),
    Time(Duration),
}

impl From<usize> for LengthHint {
    fn from(size: usize) -> Self {
        LengthHint::Bytes(size)
    }
}

impl From<Duration> for LengthHint {
    fn from(size: Duration) -> Self {
        LengthHint::Time(size)
    }
}

/// Configuration for the internals of cached [`Input`]s, such as
/// [`MemorySource`].
///
/// [`Input`]: ../struct.Input.html
/// [`MemorySource`]: struct.MemorySource.html
#[derive(Copy, Clone, Debug)]
pub struct CacheConfig {
    chunk_size: usize,
    spawn_finaliser: bool,
    use_backing: bool,
    length_hint: Option<LengthHint>,
}

impl CacheConfig {
    pub fn new() -> Self {
        Self {
            chunk_size: 0,
            spawn_finaliser: true,
            use_backing: true,
            length_hint: None,
        }
    }

    /// The amount of bytes to allocate any time more space is required to
    /// store the stream.
    ///
    /// A larger value is generally preferred for minimising locking and allocations
    /// but may reserve too much space before the struct is finalised.
    ///
    /// If this is smaller than the minimum contiguous bytes needed for a coding type,
    /// or unspecified, then this will default to an estimated 5 seconds.
    pub fn chunk_size(&mut self, size: usize) -> &mut Self {
        self.chunk_size = size;
        self
    }

    /// Allocate a contiguous backing store to speed up reads after the stream ends.
    ///
    /// Defaults to `true`.
    pub fn use_backing(&mut self, val: bool) -> &mut Self {
        self.use_backing = val;
        self
    }

    /// Spawn a new thread to move contents of the rope into backing storage once
    /// a stream completes.
    ///
    /// Disabling this may negatively impact audio mixing performance.
    ///
    /// Defaults to `true`.
    pub fn spawn_finaliser(&mut self, val: bool) -> &mut Self {
        self.spawn_finaliser = val;
        self
    }

    /// Estimate for the amount of data required to store the completed stream.
    ///
    /// On `None`, this will default to `Bytes(chunk_size)`, or use the track time supplied
    /// by metadata if available.
    ///
    /// Defaults to `None`.
    pub fn length_hint(&mut self, hint: Option<LengthHint>) -> &mut Self {
        self.length_hint = hint;
        self
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper around an existing [`Input`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`CompressedSource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with small, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved in **uncompressed floating point** form to minimise the
/// cost of audio processing. This is a significant *3 Mbps (375 kiB/s)*,
/// or 131 MiB of RAM for a 6 minute song.
///
/// [`Input`]: ../struct.Input.html
/// [`CompressedSource`]: struct.CompressedSource.html
/// [`RestartableSource`]: ../struct.RestartableSource.html
#[derive(Clone, Debug)]
pub struct MemorySource {
    cache: AudioCache,
}

impl MemorySource {
    /// Wrap an existing [`Input`] with an in-memory store of raw 32-bit floating point audio.
    ///
    /// `length_hint` may be used to control the size of the initial chunk, preventing
    /// needless allocations and copies.
    ///
    /// [`Input`]: struct.Input.html
    pub fn new(source: Input, config: Option<CacheConfig>) -> Self {
        let core_raw = RawStore::new(source, EncodingData::FloatPcm, config)
            .expect("This should only be fallible for Opus caches.");

        Self {
            cache: AudioCache::new(core_raw),
        }
    }

    /// Acquire a new handle to this object, to begin a new
    /// source from the exsting cached data.
    pub fn new_handle(&self) -> Self {
        Self {
            cache: self.cache.new_handle(),
        }
    }

    /// Block the current thread to read all bytes from the underlying stream
    /// into the backing store.
    pub fn load_file(&mut self) {
        self.cache.load_file();
    }

    /// Spawn a new thread to read all bytes from the underlying stream
    /// into the backing store.
    pub fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        self.cache.spawn_loader()
    }
}

impl From<MemorySource> for Input {
    fn from(src: MemorySource) -> Self {
        Self {
            stereo: src.cache.core.is_stereo(),
            kind: InputType::FloatPcm,
            decoder: None,

            reader: Reader::InMemory(src),
        }
    }
}

impl Read for MemorySource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.cache.read(buf)
    }
}

impl Seek for MemorySource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let old_pos = self.cache.pos as u64;

        let (valid, new_pos) = match pos {
            SeekFrom::Current(adj) => {
                // overflow expected in many cases.
                let new_pos = old_pos.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= old_pos, new_pos)
            }
            SeekFrom::End(adj) => {
                // FIXME: make this check for metadata as the basis?
                self.load_file();

                let len = self.cache.core.len() as u64;
                let new_pos = len.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= len, new_pos)
            }
            SeekFrom::Start(new_pos) => {
                (true, new_pos)
            }
        };

        if valid {
            if new_pos > old_pos {
                self.cache.consume((new_pos - old_pos) as usize);
            }

            let len = self.cache.core.len() as u64;

            self.cache.pos = new_pos.min(len) as usize;
            Ok(self.cache.pos as u64)
        } else {
            Err(IoError::new(IoErrorKind::InvalidInput, "Tried to seek before start of stream."))
        }
    }
}

/// A wrapper around an existing [`Input`] which compresses
/// the input using the Opus codec before storing it in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with larger, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved as **compressed Opus audio**. There is an associated memory cost,
/// but this is far smaller than using a [`MemorySource`].
///
/// [`Input`]: ../struct.Input.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`RestartableSource`]: ../struct.RestartableSource.html
#[derive(Debug)]
pub struct CompressedSource {
    cache: AudioCache,
    decoder: OpusDecoder,
    current_frame: Vec<f32>,
    frame_pos: usize,
    audio_pos: usize,
    remaining_lookahead: Option<usize>,
    frame_pos_override: Option<usize>,
}

impl CompressedSource {
    /// Wrap an existing `Input` with an in-memory store, compressed using Opus.
    ///
    /// `length_hint` may be used to control the size of the initial chunk, preventing
    /// needless allocations and copies.
    ///
    /// [`Input`]: struct.Input.html
    pub fn new(source: Input, bitrate: Bitrate, config: Option<CacheConfig>) -> Result<Self> {
        let channels = if source.stereo { Channels::Stereo } else { Channels::Mono };
        let mut encoder = OpusEncoder::new(SampleRate::Hz48000, channels, Application::Audio)?;

        encoder.set_bitrate(bitrate)?;

        Self::with_encoder(source, encoder, config)
    }

    /// Wrap an existing [`Input`] with an in-memory store, compressed using a user-defined
    /// Opus encoder.
    ///
    /// `length_hint` functions as in [`new`]. This function's behaviour is undefined if your encoder
    /// has a different sample rate than 48kHz, and if the decoder has a different channel count from the source.
    ///
    /// [`Input`]: struct.Input.html
    /// [`new`]: #method.new
    pub fn with_encoder(source: Input, encoder: OpusEncoder, config: Option<CacheConfig>) -> Result<Self> {
        let encoder_data = EncodingData::Opus{
            encoder,
            last_frame: Vec::with_capacity(4000),//256 + est_cost_per_sec / AUDIO_FRAME_RATE),
            frame_pos: 0,
        };

        let stereo = source.stereo;
        let core_raw = RawStore::new(source, encoder_data, config)?;
        let decoder = utils::decoder(stereo)?;

        Ok(Self {
            cache: AudioCache::new(core_raw),
            decoder,
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        })
    }

    /// Acquire a new handle to this object, to begin a new
    /// source from the existing cached data.
    pub fn new_handle(&self) -> Result<Self> {
        Ok(Self {
            cache: self.cache.new_handle(),
            decoder: utils::decoder(self.cache.core.is_stereo())?,
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        })
    }

    /// Drop all decoder/position state to allow this handle
    /// to be sent across thread boundaries.
    pub fn into_sendable(self) -> CompressedSourceBase {
        CompressedSourceBase {
            cache: self.cache,
        }
    }

    /// Block the current thread to read all bytes from the underlying stream
    /// into the backing store.
    pub fn load_file(&mut self) {
        self.cache.load_file();
    }

    /// Spawn a new thread to read all bytes from the underlying stream
    /// into the backing store.
    pub fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        self.cache.spawn_loader()
    }

    /// Completely resets decoder state and reading position.
    pub fn reset(&mut self) -> OpusResult<()> {
        self.remaining_lookahead = None;
        self.cache.pos = 0;
        self.audio_pos = 0;
        self.frame_pos = 0;
        self.current_frame.truncate(0);
        self.decoder.reset_state()
    }
}

impl From<CompressedSource> for Input {
    fn from(src: CompressedSource) -> Self {
        Input {
            stereo: src.cache.core.is_stereo(),
            kind: InputType::FloatPcm,
            decoder: None,

            reader: Reader::Compressed(src),
        }   
    }
}

impl Read for CompressedSource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let sample_len = mem::size_of::<f32>();

        // We need to discard this many samples of payload
        // after any seek or fresh start.
        if self.remaining_lookahead.is_none() {
            self.remaining_lookahead = Some(
                self.cache.core.lookahead().unwrap_or(0)
                    * if self.cache.core.is_stereo() { 2 } else { 1 }
            );
        }

        if self.frame_pos == self.current_frame.len() {
            if self.cache.core.is_finalised() && self.cache.core.len() == self.cache.pos {
                return Ok(0);
            }
            let mut temp_buf = [0u8; STEREO_FRAME_BYTE_SIZE];
            self.current_frame.resize(self.current_frame.capacity(), 0.0);

            let sz = (self.cache.read_u16::<LittleEndian>()?) as usize;
            self.cache.read_exact(&mut temp_buf[..sz])?;

            let decode_len = self.decoder.decode_float(Some(&temp_buf[..sz]), &mut self.current_frame, false)
                .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

            self.current_frame.truncate(sample_len * decode_len);
            if let Some(pos) = self.frame_pos_override {
                self.frame_pos = pos;
            } else {
                self.frame_pos = 0;
            }
            self.frame_pos_override = None;
        }

        let buf_float_space = buf.len() / sample_len;
        let unwritten_floats = self.current_frame.len() - self.frame_pos;
        let floats_to_write = buf_float_space.min(unwritten_floats);

        if let Some(ref mut bad_sample_count) = self.remaining_lookahead {
            let old_count = *bad_sample_count;
            let to_drain = unwritten_floats.min(old_count);

            *bad_sample_count -= to_drain;
            self.frame_pos += to_drain;

            if old_count > unwritten_floats {
                return Err(IoError::new(
                    IoErrorKind::Interrupted,
                    "Not enough samples in packet to drain lookahead -- keep reading.",
                ));
            }
        }

        {
            let mut buf = &mut buf[..];

            for val in &self.current_frame[self.frame_pos..self.frame_pos + floats_to_write] {
                buf.write_f32::<LittleEndian>(*val)?;
            }
        }

        self.frame_pos += floats_to_write;
        Ok(floats_to_write * sample_len)
    }
}

impl Seek for CompressedSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        // BE AWARE: this refers to a seek in audio space, and NOT in backing store.
        let old_pos = self.audio_pos as u64;

        // FIXME: duped from above.
        let (valid, new_pos) = match pos {
            SeekFrom::Current(adj) => {
                // overflow expected in many cases.
                let new_pos = old_pos.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= old_pos, new_pos)
            }
            SeekFrom::End(adj) => {
                // FIXME: make this check for metadata as the basis?
                self.load_file();

                let len = self.cache.core.len() as u64;
                let new_pos = len.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= len, new_pos)
            }
            SeekFrom::Start(new_pos) => {
                (true, new_pos)
            }
        };

        let new_pos = new_pos as usize;

        if valid {
            loop {
                if let Some(new_backing_pos) = self.cache.audio_to_backing_pos(new_pos) {
                    // We now have the start of the frame which includes the desired pos.
                    // NOTE: we hit this branch once finalised, too.
                    self.reset()
                        .map_err(|e|
                            IoError::new(IoErrorKind::Other, e)
                        )?;
                    self.cache.pos = new_backing_pos.backing_pos;
                    if self.cache.pos != self.cache.core.store_len() {
                        self.frame_pos_override = Some(new_pos - new_backing_pos.audio_pos);
                    }
                    break;
                } else {
                    let sz = (self.cache.read_u16::<LittleEndian>()?) as usize;
                    self.cache.consume(sz);
                }
            }

            Ok(self.cache.pos as u64)
        } else {
            Err(IoError::new(IoErrorKind::InvalidInput, "Tried to seek before start of stream."))
        }
    }
}

/// Handle to a [`CompressedSource`] which is safe to pass between threads.
///
/// This strips all instance-specific state, including position and decoder information.
///
/// [`CompressedSource`]: struct.CompressedSource.html
#[derive(Clone, Debug)]
pub struct CompressedSourceBase {
    cache: AudioCache,
}

impl CompressedSourceBase {
    /// Create a new handle, suitable for conversion to an [`Input`] or [`Audio`].
    ///
    /// [`Input`]: struct.Input.html
    /// [`Audio`]: struct.Audio.html
    pub fn new_handle(&self) -> Result<CompressedSource> {
        Ok(CompressedSource {
            cache: self.cache.new_handle(),
            decoder: utils::decoder(self.cache.core.is_stereo())?,
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        })
    }
}

#[derive(Debug)]
struct SharedStore {
    raw: UnsafeCell<RawStore>,
}

impl SharedStore {
    // The main reason for employing `unsafe` here is *shared mutability*:
    // due to the granularity of the locks we need, (i.e., a moving critical
    // section otherwise lock-free), we need to assert that these operations
    // are safe.
    //
    // Note that only our code can use this, so that we can ensure correctness
    // and concurrent safety.
    #[allow(clippy::mut_from_ref)]
    fn get_mut_ref(&self) -> &mut RawStore {
        unsafe { &mut *self.raw.get() }
    }

    fn read_from_pos(&self, pos: usize, buffer: &mut [u8]) -> (IoResult<usize>, bool) {
        self.get_mut_ref()
            .read_from_pos(pos, buffer)
    }

    fn len(&self) -> usize {
        self.get_mut_ref()
            .audio_len()
    }

    fn store_len(&self) -> usize {
        self.get_mut_ref()
            .len()
    }

    fn is_stereo(&self) -> bool {
        self.get_mut_ref()
            .stereo
    }

    fn is_finalised(&self) -> bool {
        self.get_mut_ref()
            .finalised()
            .is_source_finished()
    }

    fn lookahead(&self) -> IoResult<usize> {
        self.get_mut_ref()
            .inner_type
            .lookahead()
    }

    fn audio_to_backing_pos(&self, audio_byte_pos: usize) -> Option<ChunkPosition> {
        self.get_mut_ref()
            .audio_to_backing_pos(audio_byte_pos)
    }

    fn do_finalise(&self) {
        self.get_mut_ref()
            .do_finalise()
    }
}

#[derive(Debug)]
enum EncodingData {
    FloatPcm,
    Opus{encoder: OpusEncoder, last_frame: Vec<u8>, frame_pos: usize}
}

impl EncodingData {
    fn cost_per_second(&self, stereo: bool) -> Result<usize> {
        match self {
            Self::FloatPcm => Ok(utils::timestamp_to_byte_count(Duration::from_secs(1), stereo)),
            Self::Opus{ encoder, .. } => {
                let framing_cost_per_sec = AUDIO_FRAME_RATE * mem::size_of::<u16>();
                let bitrate = encoder.bitrate()?;

                let bitrate_raw = match bitrate {
                    Bitrate::BitsPerSecond(i) => i,
                    Bitrate::Auto => 64_000,
                    Bitrate::Max => 512_000,
                } as usize;

                Ok((bitrate_raw / 8) + framing_cost_per_sec)
            },
        }
    }

    fn lookahead(&self) -> IoResult<usize> {
        match self {
            Self::FloatPcm => Ok(0),
            Self::Opus{encoder, ..} => encoder.lookahead()
                .map(|n| n as usize)
                .map_err(|e| IoError::new(IoErrorKind::Other, e)),
        }
    }
    fn min_bytes_required(&self) -> usize {
        match self {
            Self::FloatPcm => 1,
            Self::Opus{..} => 2,
        }
    }

    fn read_from_source(&mut self, buf: &mut [u8], src: &mut Option<Box<Input>>, stereo: bool) -> (IoResult<ChunkPosition>, bool) {
        let src = src
            .as_mut()
            .expect("Source MUST exist while not finalised.");

        match self {
            Self::FloatPcm => {
                let out = src.read(buf)
                    .map(|sz| ChunkPosition::new(sz, sz));

                let eof = match out {
                    Ok(ChunkPosition{backing_pos:0 , ..}) => true,
                    _ => false,
                };

                (out, eof)
            },
            Self::Opus{ref mut encoder, ref mut last_frame, ref mut frame_pos} => {
                let output_start = mem::size_of::<u16>();
                let mut eof = false;

                let mut raw_len = 0;
                let mut out = None;
                let mut sample_buf = [0f32; STEREO_FRAME_SIZE];
                let samples_in_frame = if stereo { STEREO_FRAME_SIZE } else { MONO_FRAME_SIZE };

                // Purge old frame and read new, if needed.
                if *frame_pos == last_frame.len() + output_start || last_frame.is_empty() {
                    last_frame.resize(last_frame.capacity(), 0);

                    // We can't use `read_f32_into` because we can't guarantee the buffer will be filled.
                    for el in sample_buf[..samples_in_frame].iter_mut() {
                        match src.read_f32::<LittleEndian>() {
                            Ok(sample) => {
                                *el = sample;
                                raw_len += 1;
                            },
                            Err(e) if e.kind() == IoErrorKind::UnexpectedEof => {
                                eof = true;
                                break;
                            },
                            Err(e) => {
                                out = Some(Err(e));
                                break;
                            }
                        }
                    }

                    if out.is_none() && raw_len > 0 {
                        loop {
                            match encoder.encode_float(&sample_buf[..], &mut last_frame[..]) {
                                Ok(pkt_len) => {
                                    *frame_pos = 0;
                                    last_frame.truncate(pkt_len);
                                    break;
                                },
                                Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                                    // If we need more capacity to encode this frame, then take it.
                                    last_frame.resize(last_frame.len() + 256, 0);
                                },
                                Err(e) => {
                                    out = Some(Err(IoError::new(IoErrorKind::Other, e)));
                                    break;
                                },
                            }
                        }
                    }
                }

                if out.is_none() {
                    // Write from frame we have.
                    let start = if *frame_pos < output_start {
                        (&mut buf[..output_start]).write_u16::<LittleEndian>(last_frame.len() as u16)
                            .expect("Minimum bytes requirement for Opus (2) should mean that a u16 \
                                     may always be written.");
                        *frame_pos += output_start;

                        output_start
                    } else { 0 };

                    let out_pos = *frame_pos - output_start;
                    let remaining = last_frame.len() - out_pos;
                    let write_len = remaining.min(buf.len() - start);
                    buf[start..start+write_len].copy_from_slice(&last_frame[out_pos..out_pos + write_len]);
                    *frame_pos += write_len;
                    out = Some(Ok(write_len + start));
                }

                // NOTE: use of raw_len here preserves true sample length even if
                // stream is extended to 20ms boundary.
                (out.unwrap_or_else(|| Err(IoError::new(IoErrorKind::Other, "Unclear.")))
                    .map(|compressed_sz| ChunkPosition::new(compressed_sz, raw_len * mem::size_of::<f32>())), eof)
            },
        }
    }
}

impl From<&EncodingData> for InputType {
    fn from(a: &EncodingData) -> Self {
        match a {
            EncodingData::FloatPcm => InputType::FloatPcm,
            EncodingData::Opus{ .. } => InputType::Opus,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FinaliseState {
    Live,
    Finalising,
    Finalised,
}

impl From<u8> for FinaliseState {
    fn from(val: u8) -> Self {
        use FinaliseState::*;
        match val {
            0 => Live,
            1 => Finalising,
            2 => Finalised,
            _ => unreachable!(),
        }
    }
}

impl From<FinaliseState> for u8 {
    fn from(val: FinaliseState) -> Self {
        use FinaliseState::*;
        match val {
            Live => 0,
            Finalising => 1,
            Finalised => 2,
        }
    }
}

impl FinaliseState {
    fn is_source_live(self) -> bool {
        matches!(self, FinaliseState::Live)
    }

    fn is_source_finished(self) -> bool {
        !self.is_source_live()
    }

    fn is_backing_ready(self) -> bool {
        matches!(self, FinaliseState::Finalised)
    }
}

// Shared basis for the below cache-based seekables.
#[derive(Debug)]
struct RawStore {
    config: CacheConfig,

    len: AtomicUsize,
    audio_len: AtomicUsize,
    finalised: AtomicU8,

    inner_type: EncodingData,

    source: Option<Box<Input>>,
    stereo: bool,

    backing_store: Option<Vec<u8>>,
    rope: Option<LinkedList<BufferChunk>>,
    rope_users: AtomicUsize,
    lock: Mutex<()>,
}

impl RawStore {
    fn new(source: Input, inner_type: EncodingData, config: Option<CacheConfig>) -> Result<Self> {
        let mut config = config.unwrap_or_else(Default::default);
        let stereo = source.stereo;
        let cost_per_sec = inner_type.cost_per_second(stereo)?;
        let min_bytes = inner_type.min_bytes_required();

        if config.chunk_size < min_bytes {
            config.chunk_size = cost_per_sec * 5;
        };

        let mut start_size = if let Some(length) = config.length_hint {
            match length {
                LengthHint::Bytes(a) => a,
                LengthHint::Time(t) => {
                    let s = t.as_secs() + if t.subsec_millis() > 0 { 1 } else { 0 };
                    (s as usize) * cost_per_sec
                }
            }
        } else {
            config.chunk_size
        };

        if start_size < min_bytes {
            start_size = cost_per_sec * 5;
        }

        let mut list = LinkedList::new();
        list.push_back(BufferChunk::new(Default::default(), start_size));

        Ok(Self {
            config,

            len: Default::default(),
            audio_len: Default::default(),
            finalised: AtomicU8::new(FinaliseState::Live.into()),

            inner_type,

            source: Some(Box::new(source)),
            stereo,

            backing_store: None,
            rope: Some(list),
            rope_users: AtomicUsize::new(1),
            lock: Mutex::new(()),
        })
    }

    fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }

    fn audio_len(&self) -> usize {
        self.audio_len.load(Ordering::Acquire)
    }

    fn finalised(&self) -> FinaliseState {
        self.finalised.load(Ordering::Acquire).into()
    }

    /// Marks stream as finished.
    ///
    /// Returns `true` if a new handle must be spawned by the parent
    /// to finalise in another thread.
    fn finalise(&mut self) -> bool {
        let state_on_call: FinaliseState = self.finalised.compare_and_swap(
            FinaliseState::Live.into(),
            FinaliseState::Finalising.into(),
            Ordering::AcqRel
        ).into();
        
        if state_on_call.is_source_live() {
            if self.config.spawn_finaliser {
                true
            } else {
                self.do_finalise();
                false
            }
        } else {
            false
        }
    }

    fn do_finalise(&mut self) {
        if !self.config.use_backing {
            // If we don't want to use backing, then still remove the source.
            // This state will prevent anyone from trying to use the backing store.
            self.source = None;
            self.finalised.store(FinaliseState::Finalising.into(), Ordering::Release);
            info!("[VoiceCached] Source finalised -- no backing.");
            return;
        }

        let backing_len = self.len();

        // Move the rope of bytes into the backing store.
        let rope = self.rope.as_mut()
            .expect("Writes should only occur while the rope exists.");

        if rope.len() > 1 {
            // Allocate one big store, then start moving entries over
            // chunk-by-chunk.
            let mut back = vec![0u8; backing_len];

            for el in rope.iter() {
                let start = el.start_pos.backing_pos;
                let end = el.end_pos.backing_pos;
                back[start..end]
                    .copy_from_slice(&el.data[..end-start]);
            }

            // Insert the new backing store, but DO NOT purge the old.
            // This is left to the last Arc<> holder of the rope.
            self.backing_store = Some(back);
        } else {
            // Least work, but unsafe.
            // We move the first chunk's buffer to become the backing store,
            // temporarily aliasing it until the list is destroyed.
            // In this case, when the list is destroyed, the first element
            // MUST be leaked to keep the backing store memory valid.
            //
            // (see remove_rope for this leakage)
            //
            // The alternative (write first chunk into always-present
            // backing store) mandates a lock for the final expansion, because
            // the backing store is IN USE. Thus, we can't employ it.
            if let Some(el) = rope.front_mut() {
                // We can be certain that this pointer is not invalidated because:
                // * All writes to the rope/rope are finished. Thus, no
                //   reallocations/moves.
                // * The Vec will live exactly as long as the RawStore, pointer never escapes.
                // Likewise, we knoe that it is safe to build the new vector as:
                // * The stored type and pointer do not change, so alignment is preserved.
                // * The data pointer is created by an existing Vec<T>.
                self.backing_store = Some(unsafe {
                    let data = el.data.as_mut_ptr();
                    Vec::from_raw_parts(data, el.data.len(), el.data.capacity())
                })
            }
        }

        // Drop the old input.
        self.source = None;

        // It's crucial that we do this *last*, as this is the signal
        // for other threads to migrate from rope to backing store.
        self.finalised.store(FinaliseState::Finalised.into(), Ordering::Release);
        info!("[VoiceCached] Source finalised.");
    }

    fn add_rope(&mut self) {
        self.rope_users.fetch_add(1, Ordering::AcqRel);
    }

    fn remove_rope_ref(&mut self, finished: FinaliseState) {
        // We can only remove the rope if the core holds the last reference.
        // Given that the number of active handles at this moment is returned,
        // we also know the amount *after* subtraction.
        let remaining = self.rope_users.fetch_sub(1, Ordering::AcqRel) - 1;

        if finished.is_backing_ready() {
            self.try_delete_rope(remaining);
        }
    }

    fn try_delete_rope(&mut self, seen_count: usize) {
        // This branch will only be visited if BOTH the rope and
        // backing store exist simultaneously.
        if seen_count == 1 {
            // In worst case, several upgrades might pile up.
            // Only one holder should concern itself with drop logic,
            // the rest should carry on and start using the backing store.
            let maybe_lock = self.lock.try_lock();
            if maybe_lock.is_none() {
                return;
            }

            if let Some(rope) = &mut self.rope {
                // Prevent the backing store from being wiped out
                // if the first link in the rope sufficed.
                // This ensures safety as we undo the aliasing
                // in the above special case.
                if rope.len() == 1 {
                    let el = rope.pop_front().expect("Length of rope was established as >= 1.");
                    ManuallyDrop::new(el.data);
                }
            }

            // Drop everything else.
            self.rope = None;
            self.rope_users.store(0, Ordering::Release);

            info!("[VoiceCached] Rope dropped!")
        }
    }

    // Note: if you get a Rope, you need to later call remove_rope to remain sound.
    // This call has the side effect of trying to safely delete the rope.
    fn get_location(&mut self) -> (CacheReadLocation, FinaliseState) {
        let mut finalised = self.finalised();

        let loc = if finalised.is_backing_ready() {
            // try to remove rope.
            let remaining_users = self.rope_users.load(Ordering::Acquire);
            self.try_delete_rope(remaining_users);
            CacheReadLocation::Backed
        } else {
            self.add_rope();
            CacheReadLocation::Roped
        };

        (loc, finalised)
    }

    /// Returns read count, should_upgrade, should_finalise_external
    fn read_from_pos(&mut self, pos: usize, buf: &mut [u8]) -> (IoResult<usize>, bool) {
        // Place read of finalised first to be certain that if we see finalised,
        // then backing_len *must* be the true length.
        let (loc, mut finalised) = self.get_location();

        let mut backing_len = self.len();

        let mut should_finalise_external = false;

        let target_len = pos + buf.len();

        let out = if finalised.is_source_finished() || target_len <= backing_len {
            // If finalised, there is zero risk of triggering more writes.
            let read_amt = buf.len().min(backing_len - pos);
            Ok(self.read_from_local(pos, loc, buf, read_amt))
        } else {
            let mut read = 0;
            let mut base_result = None;

            loop {
                finalised = self.finalised();
                backing_len = self.len();
                let mut remaining_in_store = backing_len - pos - read;

                if remaining_in_store == 0 {
                    // Need to do this to trigger the lock
                    // while holding mutability to the other members.
                    let lock: *mut Mutex<()> = &mut self.lock;
                    let guard = unsafe {
                        let lock = & *lock;
                        lock.lock()
                    };

                    finalised = self.finalised();
                    backing_len = self.len();

                    // If length changed between our check and
                    // acquiring the lock, then drop it -- we don't need new bytes *yet*
                    // and might not!
                    remaining_in_store = backing_len - pos - read;
                    if remaining_in_store == 0 && finalised.is_source_live() {
                        let read_count = self.fill_from_source(buf.len() - read);
                        if let Ok((read_count, finalise_elsewhere)) = read_count {
                            remaining_in_store += read_count;
                            should_finalise_external |= finalise_elsewhere;
                        }
                        base_result = Some(read_count.map(|a| a.0));

                        finalised = self.finalised();
                    }

                    // Unlocked here.
                    MutexGuard::unlock_fair(guard);
                }

                if remaining_in_store > 0 {
                    let count = remaining_in_store.min(buf.len() - read);
                    read += self.read_from_local(pos, loc, &mut buf[read..], count);
                }

                // break out if:
                // * no space in reader's buffer
                // * hit an error
                // * or nothing remaining, AND finalised
                if matches!(base_result, Some(Err(_)))
                    || read == buf.len()
                    || (finalised.is_source_finished() && backing_len == pos + read) {
                    break;
                }
            }

            base_result
                .unwrap_or(Ok(0))
                .map(|_| read)
        };

        if loc == CacheReadLocation::Roped {
            self.remove_rope_ref(finalised);
        }

        (out, should_finalise_external)
    }

    // ONLY SAFE TO CALL WITH LOCK.
    // The critical section concerns:
    // * adding new elements to the rope
    // * drawing bytes from the source
    // * modifying len
    // * modifying encoder state
    fn fill_from_source(&mut self, mut bytes_needed: usize) -> IoResult<(usize, bool)> {
        let minimum_to_write = self.inner_type.min_bytes_required();
        // Round up to the next full audio frame.
        // FIXME: cache this.
        let frame_len = utils::timestamp_to_byte_count(Duration::from_millis(20), self.stereo);

        let overspill = bytes_needed % frame_len;
        if overspill != 0 {
            bytes_needed += frame_len - overspill;
        }

        let mut remaining_bytes = bytes_needed;
        let mut recorded_error = None;

        let mut spawn_new_finaliser = false;

        loop {
            let rope = self.rope.as_mut()
                .expect("Writes should only occur while the rope exists.");

            let rope_el = rope.back_mut()
                .expect("There will always be at least one element in rope.");

            let old_len = rope_el.data.len();
            let cap = rope_el.data.capacity();
            let space = cap - old_len;

            let new_len = old_len + space.min(remaining_bytes);

            if space < minimum_to_write {
                let end = rope_el.end_pos;
                // Make a new chunk!
                rope.push_back(BufferChunk::new(
                    end,
                    self.config.chunk_size,
                ));
            } else {
                rope_el.data.resize(new_len, 0);
                let (pos, eofd) = self.inner_type.read_from_source(&mut rope_el.data[old_len..], &mut self.source, self.stereo);
                match pos {
                    Ok(len) => {
                        // When to not write this?
                        if len.audio_pos > 0 {
                            rope_el.first_frame_head.get_or_insert(old_len);
                        }

                        rope_el.end_pos += len;
                        let store_len = len.backing_pos;

                        rope_el.data.truncate(old_len + store_len);

                        remaining_bytes -= store_len;
                        self.audio_len.fetch_add(len.audio_pos, Ordering::Release);
                        self.len.fetch_add(len.backing_pos, Ordering::Release);
                    },
                    Err(e) if e.kind() == IoErrorKind::Interrupted => {
                        // DO nothing, so try again.
                    },
                    Err(e) => {
                        recorded_error = Some(Err(e));
                    }
                }

                if eofd {
                    spawn_new_finaliser = self.finalise();
                }

                if self.finalised().is_source_finished() || remaining_bytes < minimum_to_write || recorded_error.is_some() {
                    break;
                }
            }
            }

        recorded_error.unwrap_or(Ok((bytes_needed - remaining_bytes, spawn_new_finaliser)))
    }

    #[inline]
    fn read_from_local(&self, mut pos: usize, loc: CacheReadLocation, buf: &mut [u8], count: usize) -> usize {
        use CacheReadLocation::*;
        match loc {
            Backed => {
                let store = self.backing_store
                    .as_ref()
                    .expect("Reader should not attempt to use a backing store before it exists");

                if pos < self.audio_len() {
                    buf[..count].copy_from_slice(&store[pos..pos + count]);

                    count
                } else {
                    0
                }
            },
            Roped => {
                let rope = self.rope
                    .as_ref()
                    .expect("Rope should still exist while any handles hold a ::Roped(_) \
                             (and thus an Arc)");

                let mut written = 0;

                for link in rope.iter() {
                    // Although this isn't atomic, Release on store to .len ensures that
                    // all writes made before setting len STAY before len.
                    // backing_pos might be larger than len, and fluctuates
                    // due to resizes, BUT we're gated by the atomically written len,
                    // via count, which gives us a safe bound on accessible bytes this call.
                    if pos >= link.start_pos.backing_pos && pos < link.end_pos.backing_pos {
                        let local_available = link.end_pos.backing_pos - pos;
                        let to_write = (count - written).min(local_available);

                        let first_el = pos - link.start_pos.backing_pos;

                        let next_len = written + to_write;

                        buf[written..next_len].copy_from_slice(&link.data[first_el..first_el + to_write]);

                        written = next_len;
                        pos += to_write;
                    }

                    if written >= buf.len() {
                        break;
                    }
                }

                count
            }
        }
    }

    fn audio_to_backing_pos(&mut self, audio_byte_pos: usize) -> Option<ChunkPosition> {
        let (loc, finalised) = self.get_location();

        let out = if audio_byte_pos > self.audio_len() {
            if self.finalised().is_source_finished() {
                Some(ChunkPosition::new(self.len(), self.audio_len()))
            } else {
                None
            }
        } else {
            let audio_bytes_per_frame = if self.stereo { STEREO_FRAME_BYTE_SIZE } else { MONO_FRAME_BYTE_SIZE };
            match self.inner_type {
                EncodingData::FloatPcm => {
                    Some(ChunkPosition::new(audio_byte_pos, audio_byte_pos))
                },
                EncodingData::Opus{..} => {
                    match loc {
                        CacheReadLocation::Backed => {
                            let back = self.backing_store.as_ref()
                                .expect("Can't access this code path unless nacking store exists.");

                            // Walk the backing store.
                            // FIXME: accelerate this...
                            let mut pos = ChunkPosition::default();
                            let mut frame_head = 0;

                            while audio_byte_pos - pos.audio_pos >= audio_bytes_per_frame {
                                let next = (&back[frame_head..]).read_u16::<LittleEndian>()
                                    .expect("Frame head should point to u16...") as usize;
                                frame_head += mem::size_of::<u16>() + next;
                                pos.audio_pos += audio_bytes_per_frame;
                            }

                            pos.backing_pos += frame_head;

                            Some(pos)
                        },
                        CacheReadLocation::Roped => {
                            let rope = self.rope
                                .as_ref()
                                .expect("Rope should still exist while any handles hold a ::Roped(_) \
                                         (and thus an Arc)");

                            let mut out = None;
                            for el in rope.iter() {
                                if audio_byte_pos >= el.start_pos.audio_pos && audio_byte_pos < el.end_pos.audio_pos {
                                    let mut pos = el.start_pos;
                                    let mut frame_head = el.first_frame_head
                                        .expect("First frame location must be defined if we're able to read the chunk.");

                                    while audio_byte_pos - pos.audio_pos >= audio_bytes_per_frame {
                                        let next = (&el.data[frame_head..]).read_u16::<LittleEndian>()
                                            .expect("Frame head should point to u16...") as usize;
                                        frame_head += mem::size_of::<u16>() + next;
                                        pos.audio_pos += audio_bytes_per_frame;
                                    }

                                    pos.backing_pos += frame_head;

                                    out = Some(pos);
                                    break;
                                }
                            }
                            out
                        },
                    }
                }
            }
        };

        if loc == CacheReadLocation::Roped {
            self.remove_rope_ref(finalised);
        }

        out
    }
}

impl Drop for RawStore {
    fn drop(&mut self) {
        // This is necesary to prevent unsoundness.
        // I.e., 1-chunk case after finalisation if
        // one handle is left in Rope, then dropped last
        // would cause a double free due to aliased chunk.
        let remaining_users = self.rope_users.load(Ordering::Acquire);
        self.try_delete_rope(remaining_users);
    }
}

// We need to declare these as thread-safe, since we don't have a mutex around
// several raw fields. However, the way that they are used should remain
// consistent.
unsafe impl Sync for SharedStore {}
unsafe impl Send for SharedStore {}

#[derive(Debug)]
struct BufferChunk {
    data: Vec<u8>,

    start_pos: ChunkPosition,
    end_pos: ChunkPosition,

    first_frame_head: Option<usize>,
}

impl BufferChunk {
    fn new(start_pos: ChunkPosition, chunk_len: usize) -> Self {
        BufferChunk {
            data: Vec::with_capacity(chunk_len),

            start_pos,
            end_pos: start_pos,

            first_frame_head: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ChunkPosition {
    /// Marks the byte count as seen by the backing store.
    backing_pos: usize,

    /// Marks the byte count as seen by a reader (i.e., audio bytes).
    audio_pos: usize,
}

impl ChunkPosition {
    fn new(backing_pos: usize, audio_pos: usize) -> Self {
        Self {
            backing_pos,
            audio_pos,
        }
    }
}

impl Add for ChunkPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            backing_pos: self.backing_pos + other.backing_pos,
            audio_pos: self.audio_pos + other.audio_pos,
        }
    }
}

impl AddAssign for ChunkPosition {
    fn add_assign(&mut self, other: Self) {
        self.backing_pos += other.backing_pos;
        self.audio_pos += other.audio_pos;
    }
}

impl Sub for ChunkPosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            backing_pos: self.backing_pos - other.backing_pos,
            audio_pos: self.audio_pos - other.audio_pos,
        }
    }
}

impl SubAssign for ChunkPosition {
    fn sub_assign(&mut self, other: Self) {
        self.backing_pos -= other.backing_pos;
        self.audio_pos -= other.audio_pos;   
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq,)]
enum CacheReadLocation {
    Roped,
    Backed,
}

#[derive(Clone, Debug)]
struct AudioCache {
    core: Arc<SharedStore>,
    pos: usize,
}

impl AudioCache {
    fn new(core: RawStore) -> Self {
        AudioCache {
            core: Arc::new(SharedStore{ raw: UnsafeCell::new(core) }),
            pos: 0,
        }
    }

    fn new_handle(&self) -> Self {
        Self {
            core: self.core.clone(),
            pos: 0,
        }
    }

    fn load_file(&mut self) {
        let pos = self.pos;
        while self.consume(1920 * mem::size_of::<f32>()) > 0 && !self.is_finalised() {}
        self.pos = pos;
    }

    fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        let mut handle = self.new_handle();
        std::thread::spawn(move || {
            handle.load_file();
        })
    }

    fn is_finalised(&self) -> bool {
        self.core.is_finalised()
    }

    fn audio_to_backing_pos(&self, audio_byte_pos: usize) -> Option<ChunkPosition> {
        self.core.audio_to_backing_pos(audio_byte_pos)
    }
}

// Read and Seek on the audio operate on byte positions
// of the output FloatPcm stream.
impl Read for AudioCache {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let (bytes_read, should_finalise_here) = self.core.read_from_pos(self.pos, buf);

        if should_finalise_here {
            let handle = self.core.clone();
            std::thread::spawn(move || handle.do_finalise());
        }

        if let Ok(size) = bytes_read {
            self.pos += size;
        }

        bytes_read
    }
}
