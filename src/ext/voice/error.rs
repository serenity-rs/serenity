use serde_json::Value;
use std::process::Output;

#[derive(Debug)]
pub enum VoiceError {
    // An indicator that an endpoint URL was invalid.
    EndpointUrl,
    ExpectedHandshake,
    FindingByte,
    HostnameResolve,
    KeyGen,
    Streams,
    VoiceModeInvalid,
    VoiceModeUnavailable,
    YouTubeDLRun(Output),
    YouTubeDLProcessing(Value),
    YouTubeDLUrl(Value),
}
