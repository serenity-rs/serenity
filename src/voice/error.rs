use serde_json::Value;
use std::process::Output;

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
