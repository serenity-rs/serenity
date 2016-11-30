use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use serde_json::Value;
use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt::{self, Display, Error as FormatError};
use websocket::result::WebSocketError;
use ::client::gateway::GatewayError;
use ::client::ClientError;
#[cfg(feature = "opus")]
use opus::Error as OpusError;
#[cfg(feature="voice")]
use ::ext::voice::VoiceError;

/// The common result type between most library functions.
///
/// The library exposes functions which, for a result type, exposes only one
/// type, rather than the usual 2 (`Result<T, Error>`). This is because all
/// functions that return a result return serenity's [`Error`], so this is
/// implied, and a "simpler" result is used.
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;

/// A common error enum returned by most of the library's functionality within a
/// custom [`Result`].
///
/// The most common error types, the [`ClientError`] and [`GatewayError`]
/// enums, are both wrapped around this in the form of the [`Client`] and
/// [`Gateway`] variants.
///
/// [`Client`]: #variant.Client
/// [`ClientError`]: client/enum.ClientError.html
/// [`Gateway`]: #variant.Gateway
/// [`GatewayError`]: client/enum.GatewayError.html
/// [`Result`]: type.Result.html
#[derive(Debug)]
pub enum Error {
    /// A [rest] or [client] error.
    ///
    /// [client]: client/index.html
    /// [rest]: client/rest/index.html
    Client(ClientError),
    /// An error with the WebSocket [`Gateway`].
    ///
    /// [`Gateway`]: client/gateway/index.html
    Gateway(GatewayError),
    /// An error while decoding a payload.
    Decode(&'static str, Value),
    /// There was an error with a format.
    Format(FormatError),
    /// An error from the `hyper` crate.
    Hyper(HyperError),
    /// An `std::io` error.
    Io(IoError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// Some other error. This is only used for "Expected value <TYPE>" errors,
    /// when a more detailed error can not be easily provided via the
    /// [`Error::Decode`] variant.
    ///
    /// [`Error::Decode`]: #variant.Decode
    Other(&'static str),
    /// An error from the `url` crate.
    Url(String),
    /// An error from the `rust-websocket` crate.
    WebSocket(WebSocketError),
    /// An error from the `opus` crate.
    #[cfg(feature = "voice")]
    Opus(OpusError),
    /// Indicating an error within the [voice module].
    ///
    /// [voice module]: ext/voice/index.html
    #[cfg(feature = "voice")]
    Voice(VoiceError),
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error {
        Error::Format(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Error {
        Error::Hyper(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

#[cfg(feature = "voice")]
impl From<OpusError> for Error {
    fn from(e: OpusError) -> Error {
        Error::Opus(e)
    }
}

impl From<WebSocketError> for Error {
    fn from(e: WebSocketError) -> Error {
        Error::WebSocket(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Hyper(ref inner) => inner.fmt(f),
            Error::Io(ref inner) => inner.fmt(f),
            Error::Json(ref inner) => inner.fmt(f),
            Error::WebSocket(ref inner) => inner.fmt(f),
            #[cfg(feature = "voice")]
            Error::Opus(ref inner) => inner.fmt(f),
            _ => f.write_str(self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Client(_) => "Client refused a request",
            Error::Decode(msg, _) | Error::Other(msg) => msg,
            Error::Format(ref inner) => inner.description(),
            Error::Gateway(ref _inner) => "Gateway error",
            Error::Hyper(ref inner) => inner.description(),
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::Url(ref inner) => inner,
            Error::WebSocket(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Opus(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Voice(_) => "Voice error",
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
