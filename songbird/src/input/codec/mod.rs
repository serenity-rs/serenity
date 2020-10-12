mod opus;

pub use self::opus::OpusDecoderState;

use super::*;
use std::{fmt::Debug, mem};

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
