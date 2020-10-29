use crate::constants::*;
use audiopus::{coder::Decoder as OpusDecoder, Channels, Error as OpusError};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone, Debug)]
/// Inner state
pub struct OpusDecoderState {
    /// Inner decoder used to convert opus frames into a stream of samples.
    pub decoder: Arc<Mutex<OpusDecoder>>,
    /// Controls whether this source allows direct Opus frame passthrough.
    /// Defaults to `true`.
    ///
    /// Enabling this flag is a promise from the programmer to the audio core
    /// that the source has been encoded at 48kHz, using 20ms long frames.
    /// If you cannot guarantee this, disable this flag (or else risk nasal demons)
    /// and bizarre audio behaviour.
    pub allow_passthrough: bool,
    pub(crate) current_frame: Vec<f32>,
    pub(crate) frame_pos: usize,
    pub(crate) should_reset: bool,
}

impl OpusDecoderState {
    /// Creates a new decoder, having stereo output at 48kHz.
    pub fn new() -> Result<Self, OpusError> {
        Ok(Self::from_decoder(OpusDecoder::new(
            SAMPLE_RATE,
            Channels::Stereo,
        )?))
    }

    /// Creates a new decoder pre-configured by the user.
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
