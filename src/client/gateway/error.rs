use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub enum Error {
    /// The connection closed
    Closed(Option<u16>, String),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
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
        }
    }
}
