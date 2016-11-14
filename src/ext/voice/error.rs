#[derive(Debug)]
pub enum VoiceError {
    // An indicator that an endpoint URL was invalid.
    EndpointUrl,
    ExpectedHandshake,
    HostnameResolve,
    KeyGen,
    VoiceModeInvalid,
    VoiceModeUnavailable,
}
