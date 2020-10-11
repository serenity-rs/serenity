use audiopus::Error as OpusError;
use serde_json::{Error as JsonError, Value};
use std::{io::Error as IoError, process::Output};
use streamcatcher::CatcherError;

/// An error returned from the voice module.
// Errors which are not visible to the end user are hidden.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Dca(DcaError),

    Io(IoError),

    Json(JsonError),

    /// An error occurred within the Opus codec.
    Opus(OpusError),

    /// Apparently failed to create stdout...
    Stdout,

    /// An error occurred while checking if a path is stereo.
    Streams,
    /// Configuration error for a cached Input.
    Streamcatcher(CatcherError),

    /// An error occurred while processing the JSON output from `youtube-dl`.
    ///
    /// The JSON output is given.
    YouTubeDLProcessing(Value),
    /// An error occurred while running `youtube-dl`.
    YouTubeDLRun(Output),
    /// The `url` field of the `youtube-dl` JSON output was not present.
    ///
    /// The JSON output is given.
    YouTubeDLUrl(Value),
}

impl From<CatcherError> for Error {
    fn from(e: CatcherError) -> Self {
        Error::Streamcatcher(e)
    }
}

impl From<DcaError> for Error {
    fn from(e: DcaError) -> Self {
        Error::Dca(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}

impl From<OpusError> for Error {
    fn from(e: OpusError) -> Error {
        Error::Opus(e)
    }
}

/// An error returned from the `dca` method.
#[derive(Debug)]
#[non_exhaustive]
pub enum DcaError {
    IoError(IoError),
    InvalidHeader,
    InvalidMetadata(JsonError),
    InvalidSize(i32),
    Opus(OpusError),
}

pub type Result<T> = std::result::Result<T, Error>;
