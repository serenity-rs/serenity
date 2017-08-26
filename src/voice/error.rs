use serde_json::{self, Value};
use std::process::Output;
use std;

/// An error returned from the voice module.
// Errors which are not visible to the end user are hidden.
#[derive(Debug)]
pub enum VoiceError {
    /// An indicator that an endpoint URL was invalid.
    EndpointUrl,
    #[doc(hidden)]
    ExpectedHandshake,
    #[doc(hidden)]
    FindingByte,
    #[doc(hidden)]
    HostnameResolve,
    #[doc(hidden)]
    KeyGen,
    /// An error occurred while checking if a path is stereo.
    Streams,
    #[doc(hidden)]
    VoiceModeInvalid,
    #[doc(hidden)]
    VoiceModeUnavailable,
    /// An error occurred while running `youtube-dl`.
    YouTubeDLRun(Output),
    /// An error occurred while processing the JSON output from `youtube-dl`.
    ///
    /// The JSON output is given.
    YouTubeDLProcessing(Value),
    /// The `url` field of the `youtube-dl` JSON output was not present.
    ///
    /// The JSON output is given.
    YouTubeDLUrl(Value),
}

/// An error returned from the dca method.
#[derive(Debug)]
pub enum DcaError {
    IoError(std::io::Error),
    InvalidHeader,
    InvalidMetadata(serde_json::Error),
    InvalidSize(i32),
}
