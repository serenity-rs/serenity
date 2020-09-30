use audiopus::{
    Application,
    Bitrate,
    Channels,
    Error as OpusError,
    ErrorCode as OpusErrorCode,
    coder::Encoder as OpusEncoder,
    SampleRate,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::{
    constants::*,
    Result,
    Error,
};
use tracing::debug;
use std::{
    convert::TryInto,
    io::{
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
    },
    mem,
    result::Result as StdResult,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
    time::Duration,
};
use streamcatcher::{
    Catcher,
    Config,
    GrowthStrategy,
    NeedsBytes,
    Stateful,
    Transform,
    TransformPosition,
    TxCatcher,
};
use super::{
    utils,
    CodecType,
    Container,
    Input,
    Metadata,
    Reader,
};


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

/// A wrapper around an existing [`Input`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`Restartable`]
/// and [`Compressed`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with small, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved in **uncompressed floating point** form to minimise the
/// cost of audio processing. This is a significant *3 Mbps (375 kiB/s)*,
/// or 131 MiB of RAM for a 6 minute song.
///
/// [`Input`]: ../struct.Input.html
/// [`Compressed`]: struct.Compressed.html
/// [`Restartable`]: ../struct.Restartable.html
#[derive(Clone, Debug)]
pub struct Memory {
    pub raw: Catcher<Box<Reader>>,
    pub metadata: Metadata,
    pub kind: CodecType,
    pub stereo: bool,
    pub container: Container,
}

// work out the froms, intos...

// issues: need enough info to reconstruct input from reader.
// ALSO: need to make sure that compressed can only be opus + dca
// and that memory preserves its input format.

impl Memory {
    /// Wrap an existing [`Input`] with an in-memory store with the same codec and framing.
    ///
    /// [`Input`]: ../struct.Input.html
    pub fn new(source: Input) -> Result<Self> {
        Self::with_config(source, None)
    }

    /// Wrap an existing [`Input`] with an in-memory store with the same codec and framing.
    ///
    /// `length_hint` may be used to control the size of the initial chunk, preventing
    /// needless allocations and copies. If this is not present, the value specified in
    /// `source`'s [`Metadata.duration`] will be used, assuming that the source is uncompressed.
    ///
    /// [`Input`]: ../struct.Input.html
    /// [`Metadata.duration`]: ../struct.Metadata.html#structfield.duration
    pub fn with_config(mut source: Input, config: Option<Config>) -> Result<Self> {
        let stereo = source.stereo;
        let kind = (&source.kind).into();
        let container = source.container;
        let metadata = source.metadata.take();

        let cost_per_sec = raw_cost_per_sec(stereo);

        let mut config = config.unwrap_or_else(|| default_config(cost_per_sec));

        // apply length hint.
        if config.length_hint.is_none() {
            if let Some(dur) = metadata.duration {
                apply_length_hint(&mut config, dur, cost_per_sec);
            }
        }

        let raw = config.build(Box::new(source.reader))
            .map_err(Error::Streamcatcher)?;

        Ok(Self {
            raw,
            metadata,
            kind,
            stereo,
            container,
        })
    }

    /// Acquire a new handle to this object, creating a new
    /// view of the existing cached data from the beginning.
    pub fn new_handle(&self) -> Self {
        Self {
            raw: self.raw.new_handle(),
            metadata: self.metadata.clone(),
            kind: self.kind,
            stereo: self.stereo,
            container: self.container,
        }
    }
}

impl From<Memory> for Input {
    fn from(src: Memory) -> Self {
        Input::new(
            src.stereo,
            Reader::Memory(src.raw),
            src.kind.try_into().expect("FIXME: make this a tryinto"),
            src.container,
            Some(src.metadata),
        )
    }
}

/// A wrapper around an existing [`Input`] which compresses
/// the input using the Opus codec before storing it in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`Restartable`]
/// and [`Memory`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with larger, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved as **compressed Opus audio**. There is an associated memory cost,
/// but this is far smaller than using a [`Memory`].
///
/// [`Input`]: ../struct.Input.html
/// [`Memory`]: struct.Memory.html
/// [`Restartable`]: ../struct.Restartable.html
#[derive(Clone, Debug)]
pub struct Compressed {
    pub raw: TxCatcher<Box<Input>, OpusCompressor>,
    pub metadata: Metadata,
    pub stereo: bool,
}

impl Compressed {
    /// Wrap an existing [`Input`] with an in-memory store, compressed using Opus.
    ///
    /// [`Input`]: ../struct.Input.html
    /// [`Metadata.duration`]: ../struct.Metadata.html#structfield.duration
    pub fn new(source: Input, bitrate: Bitrate) -> Result<Self> {
        Self::with_config(source, bitrate, None)
    }

    /// Wrap an existing [`Input`] with an in-memory store, compressed using Opus.
    ///
    /// `config.length_hint` may be used to control the size of the initial chunk, preventing
    /// needless allocations and copies. If this is not present, the value specified in
    /// `source`'s [`Metadata.duration`] will be used.
    ///
    /// [`Input`]: ../struct.Input.html
    /// [`Metadata.duration`]: ../struct.Metadata.html#structfield.duration
    pub fn with_config(source: Input, bitrate: Bitrate, config: Option<Config>) -> Result<Self> {
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
    /// [`Input`]: ../struct.Input.html
    /// [`new`]: #method.new
    pub fn with_encoder(mut source: Input, encoder: OpusEncoder, config: Option<Config>) -> Result<Self> {
        let bitrate = encoder.bitrate()?;
        let cost_per_sec = compressed_cost_per_sec(bitrate);
        let stereo = source.stereo;
        let metadata = source.metadata.take();

        let mut config = config.unwrap_or_else(|| default_config(cost_per_sec));

        // apply length hint.
        if config.length_hint.is_none() {
            if let Some(dur) = metadata.duration {
                apply_length_hint(&mut config, dur, cost_per_sec);
            }
        }

        let raw = config.build_tx(
            Box::new(source),
            OpusCompressor::new(encoder, stereo),
        ).map_err(Error::Streamcatcher)?;

        Ok(Self {
            raw,
            metadata,
            stereo,
        })
    }

    /// Acquire a new handle to this object, creating a new
    /// view of the existing cached data from the beginning.
    pub fn new_handle(&self) -> Self {
        Self {
            raw: self.raw.new_handle(),
            metadata: self.metadata.clone(),
            stereo: self.stereo,
        }
    }
}

impl From<Compressed> for Input {
    fn from(src: Compressed) -> Self {
        Input::new(
            src.stereo,
            Reader::Compressed(src.raw),
            CodecType::Opus.try_into()
                .expect("Default decoder values are known to be valid."),
            Container::Dca{ first_frame: 0 },
            Some(src.metadata),
        )
    }
}

#[derive(Debug)]
pub struct OpusCompressor {
    encoder: OpusEncoder,
    last_frame: Vec<u8>,
    stereo_input: bool,
    frame_pos: usize,
    audio_bytes: AtomicUsize,
}

impl OpusCompressor {
    fn new(encoder: OpusEncoder, stereo_input: bool) -> Self {
        Self {
            encoder,
            last_frame: Vec::with_capacity(4000),
            stereo_input,
            frame_pos: 0,
            audio_bytes: Default::default(),
        }
    }
}

impl<T> Transform<T> for OpusCompressor
where
    T: Read,
{
    fn transform_read(&mut self, src: &mut T, buf: &mut [u8]) -> IoResult<TransformPosition> {
        let output_start = mem::size_of::<u16>();
        let mut eof = false;

        let mut raw_len = 0;
        let mut out = None;
        let mut sample_buf = [0f32; STEREO_FRAME_SIZE];
        let samples_in_frame = if self.stereo_input { STEREO_FRAME_SIZE } else { MONO_FRAME_SIZE };

        // Purge old frame and read new, if needed.
        if self.frame_pos == self.last_frame.len() + output_start || self.last_frame.is_empty() {
            self.last_frame.resize(self.last_frame.capacity(), 0);

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
                    match self.encoder.encode_float(&sample_buf[..], &mut self.last_frame[..]) {
                        Ok(pkt_len) => {
                            debug!("Next packet to write has {:?}", pkt_len);
                            self.frame_pos = 0;
                            self.last_frame.truncate(pkt_len);
                            break;
                        },
                        Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                            // If we need more capacity to encode this frame, then take it.
                            debug!("Resizing inner buffer (+256).");
                            self.last_frame.resize(self.last_frame.len() + 256, 0);
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
            let start = if self.frame_pos < output_start {
                (&mut buf[..output_start]).write_i16::<LittleEndian>(self.last_frame.len() as i16)
                    .expect("Minimum bytes requirement for Opus (2) should mean that an i16 \
                             may always be written.");
                self.frame_pos += output_start;

                debug!("Wrote frame header.");

                output_start
            } else { 0 };

            let out_pos = self.frame_pos - output_start;
            let remaining = self.last_frame.len() - out_pos;
            let write_len = remaining.min(buf.len() - start);
            buf[start..start+write_len].copy_from_slice(&self.last_frame[out_pos..out_pos + write_len]);
            self.frame_pos += write_len;
            debug!("Appended {} to inner store", write_len);
            out = Some(Ok(write_len + start));
        }

        // NOTE: use of raw_len here preserves true sample length even if
        // stream is extended to 20ms boundary.
        out.unwrap_or_else(|| Err(IoError::new(IoErrorKind::Other, "Unclear.")))
            .map(|compressed_sz| {
                self.audio_bytes.fetch_add(raw_len * mem::size_of::<f32>(), Ordering::Release);

                if eof {
                    TransformPosition::Finished
                } else {
                    TransformPosition::Read(compressed_sz)
                }
            })
    }
}

impl NeedsBytes for OpusCompressor {
    fn min_bytes_required(&self) -> usize {
        2
    }
}

impl Stateful for OpusCompressor {
    type State = usize;

    fn state(&self) -> Self::State {
        self.audio_bytes.load(Ordering::Acquire)
    }
}

pub fn compressed_cost_per_sec(bitrate: Bitrate) -> usize {
    let framing_cost_per_sec = AUDIO_FRAME_RATE * mem::size_of::<u16>();

    let bitrate_raw = match bitrate {
        Bitrate::BitsPerSecond(i) => i,
        Bitrate::Auto => 64_000,
        Bitrate::Max => 512_000,
    } as usize;

    (bitrate_raw / 8) + framing_cost_per_sec
}

pub fn raw_cost_per_sec(stereo: bool) -> usize {
    utils::timestamp_to_byte_count(Duration::from_secs(1), stereo)
}

pub fn apply_length_hint<H>(config: &mut Config, hint: H, cost_per_sec: usize)
where
    H: Into<LengthHint>,
{
    config.length_hint = Some(match hint.into() {
        LengthHint::Bytes(a) => a,
        LengthHint::Time(t) => {
            let s = t.as_secs() + if t.subsec_millis() > 0 { 1 } else { 0 };
            (s as usize) * cost_per_sec
        }
    });
}

pub fn default_config(cost_per_sec: usize) -> Config {
    Config::new()
        .chunk_size(GrowthStrategy::Constant(5 * cost_per_sec))
}
