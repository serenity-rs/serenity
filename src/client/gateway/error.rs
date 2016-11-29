use std::fmt::{self, Display};

/// An error that occurred while attempting to deal with the gateway.
///
/// Note that - from a user standpoint - there should be no situation in which
/// you manually handle these.
#[derive(Clone, Debug)]
pub enum Error {
    /// The connection unexpectedly (read: non-cleanly) closed.
    Closed(Option<u16>, String),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
    /// Failed to reconnect after a number of attempts.
    ReconnectFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Closed(s, ref v) => {
                f.write_str(&format!("Connection closed {:?}: {:?}", s, v))
            },
            Error::ExpectedHello => {
                f.write_str("Expected Hello during handshake")
            },
            Error::InvalidHandshake => {
                f.write_str("Expected Ready or InvalidateSession")
            },
            Error::ReconnectFailure => {
                f.write_str("Failed to Reconnect")
            },
        }
    }
}
