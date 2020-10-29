use super::{apply_length_hint, default_config, raw_cost_per_sec};
use crate::input::{
    error::{Error, Result},
    CodecType,
    Container,
    Input,
    Metadata,
    Reader,
};
use std::convert::{TryFrom, TryInto};
use streamcatcher::{Catcher, Config};

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
    /// Inner shared bytestore.
    pub raw: Catcher<Box<Reader>>,
    /// Metadata moved out of the captured source.
    pub metadata: Metadata,
    /// Codec used to read the inner bytestore.
    pub kind: CodecType,
    /// Stereo-ness of the captured source.
    pub stereo: bool,
    /// Framing mechanism for the inner bytestore.
    pub container: Container,
}

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

        let raw = config
            .build(Box::new(source.reader))
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

impl TryFrom<Memory> for Input {
    type Error = Error;

    fn try_from(src: Memory) -> Result<Self> {
        Ok(Input::new(
            src.stereo,
            Reader::Memory(src.raw),
            src.kind.try_into()?,
            src.container,
            Some(src.metadata),
        ))
    }
}
