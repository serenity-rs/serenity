use std::error::Error as StdError;
use std::fmt::{self, Display};

/// An error that occurred while attempting to deal with the gateway.
///
/// Note that - from a user standpoint - there should be no situation in which
/// you manually handle these.
#[derive(Clone, Debug)]
pub enum Error {
    /// There was an error building a URL.
    BuildingUrl,
    /// The connection closed, potentially uncleanly.
    Closed(Option<u16>, String),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
    /// An indicator that an unknown opcode was received from the gateway.
    InvalidOpCode,
    /// When a session Id was expected (for resuming), but was not present.
    NoSessionId,
    /// Failed to reconnect after a number of attempts.
    ReconnectFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BuildingUrl => "Error building url",
            Error::Closed(_, _) => "Connection closed",
            Error::ExpectedHello => "Expected a Hello",
            Error::InvalidHandshake => "Expected a valid Handshake",
            Error::InvalidOpCode => "Invalid OpCode",
            Error::NoSessionId => "No Session Id present when required",
            Error::ReconnectFailure => "Failed to Reconnect",
        }
    }
}
