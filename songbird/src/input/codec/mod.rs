//! Decoding schemes for input audio bytestreams.

mod opus;

pub use self::opus::OpusDecoderState;

use super::*;
use std::{fmt::Debug, mem};

/// State used to decode input bytes of an [`Input`].
///
/// [`Input`]: ../struct.Input.html
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Codec {
    /// The inner bytestream is encoded using the Opus codec, to be decoded
    /// using the given state.
    ///
    /// Must be combined with a non-[`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
    Opus(OpusDecoderState),
    /// The inner bytestream is encoded using raw `i16` samples.
    ///
    /// Must be combined with a [`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
    Pcm,
    /// The inner bytestream is encoded using raw `f32` samples.
    ///
    /// Must be combined with a [`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
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
/// [`Input`]: ../struct.Input.html
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum CodecType {
    /// The inner bytestream is encoded using the Opus codec.
    ///
    /// Must be combined with a non-[`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
    Opus,
    /// The inner bytestream is encoded using raw `i16` samples.
    ///
    /// Must be combined with a [`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
    Pcm,
    /// The inner bytestream is encoded using raw `f32` samples.
    ///
    /// Must be combined with a [`Raw`] container.
    ///
    /// [`Raw`]: ../enum.Container.html#variant.Raw
    FloatPcm,
}

impl CodecType {
    /// Returns the length of a single output sample, in bytes.
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
