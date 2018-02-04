use futures::Canceled;
use hyper::Response;
use hyper::error::{Error as HyperError, UriError};
use serde_json::Error as JsonError;
use std::cell::BorrowMutError;
use std::error::Error as StdError;
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::result::Result as StdResult;
use super::ratelimiting::RateLimitError;
use tokio_timer::TimerError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// There was an error mutably borrowing an `std::cell::RefCell`.
    BorrowMut(BorrowMutError),
    /// A future was canceled.
    ///
    /// This most likely occurred during a pre-emptive ratelimit.
    Canceled(Canceled),
    /// An error from the `std::fmt` module.
    Format(FmtError),
    /// An error from the `hyper` crate.
    Hyper(HyperError),
    /// When a status code was unexpectedly received for a request's status.
    InvalidRequest(Response),
    /// An error from the `std::io` module.
    Io(IoError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// An error from the `ratelimiting` module.
    RateLimit(RateLimitError),
    /// An error occurred while creating a timer.
    Timer(TimerError),
    /// When a status is received, but the verification to ensure the response
    /// is valid does not recognize the status.
    UnknownStatus(u16),
    /// A `hyper` error while parsing a Uri.
    Uri(UriError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { f.write_str(self.description()) }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BorrowMut(ref inner) => inner.description(),
            Error::Canceled(ref inner) => inner.description(),
            Error::Format(ref inner) => inner.description(),
            Error::Hyper(ref inner) => inner.description(),
            Error::InvalidRequest(_) => "Received an unexpected status code",
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::RateLimit(ref inner) => inner.description(),
            Error::Timer(ref inner) => inner.description(),
            Error::UnknownStatus(_) => "Verification does not understand status",
            Error::Uri(ref inner) => inner.description(),
        }
    }
}

impl From<BorrowMutError> for Error {
    fn from(err: BorrowMutError) -> Self {
        Error::BorrowMut(err)
    }
}

impl From<Canceled> for Error {
    fn from(err: Canceled) -> Self {
        Error::Canceled(err)
    }
}

impl From<FmtError> for Error {
    fn from(err: FmtError) -> Self {
        Error::Format(err)
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Self {
        Error::Hyper(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json(err)
    }
}

impl From<RateLimitError> for Error {
    fn from(err: RateLimitError) -> Self {
        Error::RateLimit(err)
    }
}

impl From<TimerError> for Error {
    fn from(err: TimerError) -> Self {
        Error::Timer(err)
    }
}
