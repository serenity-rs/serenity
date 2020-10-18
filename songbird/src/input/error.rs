//! Errors caused by input creation.

use audiopus::Error as OpusError;
use serde_json::{Error as JsonError, Value};
use std::{io::Error as IoError, process::Output};
use streamcatcher::CatcherError;

/// An error returned when creating a new [`Input`].
///
/// [`Input`]: ../struct.Input.html
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred while opening a new DCA source.
    Dca(DcaError),
    /// An error occurred while reading, or opening a file.
    Io(IoError),
    /// An error occurred while parsing JSON (i.e., during metadata/stereo detection).
    Json(JsonError),
    /// An error occurred within the Opus codec.
    Opus(OpusError),
    /// Failed to extract metadata from alternate pipe.
    Metadata,
    /// Apparently failed to create stdout.
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

/// An error returned from the [`dca`] method.
///
/// [`dca`]: ../fn.dca.html
#[derive(Debug)]
#[non_exhaustive]
pub enum DcaError {
    /// An error occurred while reading, or opening a file.
    IoError(IoError),
    /// The file opened did not have a valid DCA JSON header.
    InvalidHeader,
    /// The file's metadata block was invalid, or could not be parsed.
    InvalidMetadata(JsonError),
    /// The file's header reported an invalid metadata block size.
    InvalidSize(i32),
    /// An error was encountered while creating a new Opus decoder.
    Opus(OpusError),
}

/// Convenience type for fallible return of [`Input`]s.
///
/// [`Input`]: ../struct.Input.html
pub type Result<T> = std::result::Result<T, Error>;
