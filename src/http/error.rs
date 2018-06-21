use futures::Canceled;
use http_lib::{
    uri::InvalidUri as UriError,
    Error as HttpCrateError
};
use hyper::{
    error::Error as HyperError,
    Response,
};
use native_tls::Error as TlsError;
use serde_json::Error as JsonError;
use std::{
    cell::BorrowMutError,
    error::Error as StdError,
    fmt::{Display, Error as FmtError, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};
use super::ratelimiting::RateLimitError;
use tokio::timer::Error as TimerError;

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
    Http(HttpCrateError),
    /// An error from the `hyper` crate.
    Hyper(HyperError),
    /// When a status code was unexpectedly received for a request's status.
    InvalidRequest(String),
    /// An error from the `std::io` module.
    Io(IoError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// An error from the `ratelimiting` module.
    RateLimit(RateLimitError),
    /// An error occurred while creating a timer.
    Timer(TimerError),
    /// An error from the `native_tls` crate.
    Tls(TlsError),
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
            Error::Http(ref inner) => inner.description(),
            Error::Hyper(ref inner) => inner.description(),
            Error::InvalidRequest(_) => "Received an unexpected status code",
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::RateLimit(ref inner) => inner.description(),
            Error::Timer(ref inner) => inner.description(),
            Error::Tls(ref inner) => inner.description(),
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

impl From<HttpCrateError> for Error {
    fn from(err: HttpCrateError) -> Self {
        Error::Http(err)
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

impl From<TlsError> for Error {
    fn from(err: TlsError) -> Self {
        Error::Tls(err)
    }
}
