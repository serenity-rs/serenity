use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use serde_json::Value;
use websocket::result::WebSocketError;
use ::client::{ClientError, ConnectionError};

/// The common result type between most library functions.
pub type Result<T> = ::std::result::Result<T, Error>;

/// A common error enum returned by most of the library's functionality within a
/// [`Result`].
///
/// The most common error types, the [`ClientError`] and [`ConnectionError`]
/// enums, are both wrapped around this in the form of the [`Client`] and
/// [`Connection`] variants.
///
/// [`Client`]: #variant.Client
/// [`ClientError`]: client/enum.ClientError.html
/// [`Connection`]: #variant.Connection
/// [`ConnectionError`]: client/enum.ConnectionError.html
/// [`Result`]: type.Result.html
#[derive(Debug)]
pub enum Error {
    /// An Http or Client error.
    Client(ClientError),
    /// An error with the WebSocket connection.
    Connection(ConnectionError),
    /// An error while decoding a payload.
    Decode(&'static str, Value),
    /// An error from the `hyper` crate.
    Hyper(HyperError),
    /// An `std::io` error.
    Io(IoError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// Some other error.
    Other(&'static str),
    /// An error from the `url` crate.
    Url(String),
    /// An error from the `rust-websocket` crate.
    WebSocket(WebSocketError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error::Hyper(err)
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Error {
        Error::Json(err)
    }
}

impl From<WebSocketError> for Error {
    fn from(err: WebSocketError) -> Error {
        Error::WebSocket(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Hyper(ref inner) => inner.fmt(f),
            Error::Json(ref inner) => inner.fmt(f),
            Error::WebSocket(ref inner) => inner.fmt(f),
            Error::Io(ref inner) => inner.fmt(f),
            _ => f.write_str(self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Client(_) => "Client refused a request",
            Error::Connection(ref _inner) => "Connection error",
            Error::Decode(msg, _) | Error::Other(msg) => msg,
            Error::Hyper(ref inner) => inner.description(),
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::Url(ref inner) => inner,
            Error::WebSocket(ref inner) => inner.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Hyper(ref inner) => Some(inner),
            Error::Json(ref inner) => Some(inner),
            Error::WebSocket(ref inner) => Some(inner),
            Error::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}
