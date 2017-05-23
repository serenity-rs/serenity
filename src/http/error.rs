use hyper::status::StatusCode;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// When a status code was unexpectedly received for a request's status.
    InvalidRequest(StatusCode),
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64`.
    RateLimitI64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When a status is received, but the verification to ensure the response
    /// is valid does not recognize the status.
    UnknownStatus(u16),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidRequest(_) => "Received an unexpected status code",
            Error::RateLimitI64 => "Error decoding a header into an i64",
            Error::RateLimitUtf8 => "Error decoding a header from UTF-8",
            Error::UnknownStatus(_) => "Verification does not understand status",
        }
    }
}
