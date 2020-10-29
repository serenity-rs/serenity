use super::{apply_length_hint, compressed_cost_per_sec, default_config};
use crate::{
    constants::*,
    input::{
        error::{Error, Result},
        CodecType,
        Container,
        Input,
        Metadata,
        Reader,
    },
};
use audiopus::{
    coder::Encoder as OpusEncoder,
    Application,
    Bitrate,
    Channels,
    Error as OpusError,
    ErrorCode as OpusErrorCode,
    SampleRate,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::TryInto,
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult},
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};
use streamcatcher::{Config, NeedsBytes, Stateful, Transform, TransformPosition, TxCatcher};
use tracing::{debug, trace};

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
    /// Inner shared bytestore.
    pub raw: TxCatcher<Box<Input>, OpusCompressor>,
    /// Metadata moved out of the captured source.
    pub metadata: Metadata,
    /// Stereo-ness of the captured source.
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
        let channels = if source.stereo {
            Channels::Stereo
        } else {
            Channels::Mono
        };
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
    pub fn with_encoder(
        mut source: Input,
        encoder: OpusEncoder,
        config: Option<Config>,
    ) -> Result<Self> {
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

        let raw = config
            .build_tx(Box::new(source), OpusCompressor::new(encoder, stereo))
            .map_err(Error::Streamcatcher)?;

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
            true,
            Reader::Compressed(src.raw),
            CodecType::Opus
                .try_into()
                .expect("Default decoder values are known to be valid."),
            Container::Dca { first_frame: 0 },
            Some(src.metadata),
        )
    }
}

/// Transform applied inside [`Compressed`], converting a floating-point PCM
/// input stream into a DCA-framed Opus stream.
///
/// Created and managed by [`Compressed`].
///
/// [`Compressed`]: struct.Compressed.html
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
        let samples_in_frame = if self.stereo_input {
            STEREO_FRAME_SIZE
        } else {
            MONO_FRAME_SIZE
        };

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
                    },
                }
            }

            if out.is_none() && raw_len > 0 {
                loop {
                    // NOTE: we don't index by raw_len because the last frame can be too small
                    // to occupy a "whole packet". Zero-padding is the correct behaviour.
                    match self
                        .encoder
                        .encode_float(&sample_buf[..samples_in_frame], &mut self.last_frame[..])
                    {
                        Ok(pkt_len) => {
                            trace!("Next packet to write has {:?}", pkt_len);
                            self.frame_pos = 0;
                            self.last_frame.truncate(pkt_len);
                            break;
                        },
                        Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                            // If we need more capacity to encode this frame, then take it.
                            trace!("Resizing inner buffer (+256).");
                            self.last_frame.resize(self.last_frame.len() + 256, 0);
                        },
                        Err(e) => {
                            debug!("Read error {:?} {:?} {:?}.", e, out, raw_len);
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
                (&mut buf[..output_start])
                    .write_i16::<LittleEndian>(self.last_frame.len() as i16)
                    .expect(
                        "Minimum bytes requirement for Opus (2) should mean that an i16 \
                             may always be written.",
                    );
                self.frame_pos += output_start;

                trace!("Wrote frame header: {}.", self.last_frame.len());

                output_start
            } else {
                0
            };

            let out_pos = self.frame_pos - output_start;
            let remaining = self.last_frame.len() - out_pos;
            let write_len = remaining.min(buf.len() - start);
            buf[start..start + write_len]
                .copy_from_slice(&self.last_frame[out_pos..out_pos + write_len]);
            self.frame_pos += write_len;
            trace!("Appended {} to inner store", write_len);
            out = Some(Ok(write_len + start));
        }

        // NOTE: use of raw_len here preserves true sample length even if
        // stream is extended to 20ms boundary.
        out.unwrap_or_else(|| Err(IoError::new(IoErrorKind::Other, "Unclear.")))
            .map(|compressed_sz| {
                self.audio_bytes
                    .fetch_add(raw_len * mem::size_of::<f32>(), Ordering::Release);

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
