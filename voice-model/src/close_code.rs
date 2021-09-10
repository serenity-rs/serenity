use enum_primitive::*;

enum_from_primitive! {
/// Discord Voice Gateway Websocket close codes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CloseCode {
    /// Invalid Voice OP Code.
    UnknownOpcode = 4001,

    /// Invalid identification payload sent.
    InvalidPayload = 4002,

    /// A payload was sent prior to identifying.
    NotAuthenticated = 4003,

    /// The account token sent with the identify payload was incorrect.
    AuthenticationFailed = 4004,

    /// More than one identify payload was sent.
    AlreadyAuthenticated = 4005,

    /// The session is no longer valid.
    SessionInvalid = 4006,

    /// A session timed out.
    SessionTimeout = 4009,

    /// The server for the last connection attempt could not be found.
    ServerNotFound = 4011,

    /// Discord did not recognise the voice protocol chosen.
    UnknownProtocol = 4012,

    /// Disconnected, either due to channel closure/removal
    /// or kicking.
    ///
    /// Should not reconnect.
    Disconnected = 4014,

    /// Connected voice server crashed.
    ///
    /// Should resume.
    VoiceServerCrash = 4015,

    /// Discord didn't recognise the encryption scheme.
    UnknownEncryptionMode = 4016,
}
}

impl CloseCode {
    /// Indicates whether a voice client should attempt to reconnect in response to this close code.
    ///
    /// Otherwise, the connection should be closed.
    pub fn should_resume(&self) -> bool {
        matches!(self, CloseCode::VoiceServerCrash | CloseCode::SessionTimeout)
    }
}
