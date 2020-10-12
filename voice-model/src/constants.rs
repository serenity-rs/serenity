//! A set of constants used by the library.

/// Gateway version of the Voice API which this library encodes.
pub const GATEWAY_VERSION: u8 = 4;

pub mod voice_close_codes {
    /// Invalid Voice OP Code.
    pub const UNKNOWN_OPCODE: u16 = 4001;

    /// A payload was sent prior to identifying.
    pub const NOT_AUTHENTICATED: u16 = 4003;

    /// The account token sent with the identify payload was incorrect.
    pub const AUTHENTICATION_FAILED: u16 = 4004;

    /// More than one identify payload was sent.
    pub const ALREADY_AUTHENTICATED: u16 = 4005;

    /// The session is no longer valid.
    pub const SESSION_INVALID: u16 = 4006;

    /// A session timed out.
    pub const SESSION_TIMEOUT: u16 = 4009;

    /// The server for the last connection attempt could not be found.
    pub const SERVER_NOT_FOUND: u16 = 4011;

    /// Discord did not recognise the voice protocol chosen.
    pub const UNKNOWN_PROTOCOL: u16 = 4012;

    /// Disconnected, either due to channel closure/removal
    /// or kicking.
    ///
    /// Should not reconnect.
    pub const DISCONNECTED: u16 = 4014;

    /// Connected voice server crashed.
    ///
    /// Should resume.
    pub const VOICE_SERVER_CRASH: u16 = 4015;

    /// Discord didn't recognise the encrytpion scheme.
    pub const UNKNOWN_ENCRYPTION_MODE: u16 = 4016;
}
