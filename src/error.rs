use internal::prelude::*;
use model::ModelError;
use serde_json::Error as JsonError;
use std::{
    error::Error as StdError,
    fmt::{
        self,
        Display,
        Error as FormatError
    },
    io::Error as IoError,
    num::ParseIntError
};

#[cfg(feature = "reqwest")]
use reqwest::{Error as ReqwestError, header::InvalidHeaderValue};
#[cfg(feature = "voice")]
use opus::Error as OpusError;
#[cfg(feature = "websocket")]
use websocket::result::WebSocketError;
#[cfg(feature = "client")]
use client::ClientError;
#[cfg(feature = "gateway")]
use gateway::GatewayError;
#[cfg(feature = "http")]
use http::HttpError;
#[cfg(feature = "voice")]
use voice::VoiceError;

/// The common result type between most library functions.
///
/// The library exposes functions which, for a result type, exposes only one
/// type, rather than the usual 2 (`Result<T, Error>`). This is because all
/// functions that return a result return serenity's [`Error`], so this is
/// implied, and a "simpler" result is used.
///
/// [`Error`]: enum.Error.html
pub type Result<T> = StdResult<T, Error>;

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
/// [`GatewayError`]: gateway/enum.GatewayError.html
/// [`Result`]: type.Result.html
#[derive(Debug)]
pub enum Error {
    /// An error while decoding a payload.
    Decode(&'static str, Value),
    /// There was an error with a format.
    Format(FormatError),
    /// An `std::io` error.
    Io(IoError),
    /// An error from the `serde_json` crate.
    Json(JsonError),
    /// An error from the [`model`] module.
    ///
    /// [`model`]: model/index.html
    Model(ModelError),
    /// An error occurred while parsing an integer.
    Num(ParseIntError),
    /// Input exceeded a limit.
    /// Providing the input and the limit that's not supposed to be exceeded.
    ///
    /// *This only exists for the `GuildId::ban` and `Member::ban` functions. For their cases,
    /// it's the "reason".*
    ExceededLimit(String, u32),
    /// Some other error. This is only used for "Expected value <TYPE>" errors,
    /// when a more detailed error can not be easily provided via the
    /// [`Error::Decode`] variant.
    ///
    /// [`Error::Decode`]: #variant.Decode
    Other(&'static str),
    /// An error from the `url` crate.
    Url(String),
    /// A [client] error.
    ///
    /// [client]: client/index.html
    #[cfg(feature = "client")]
    Client(ClientError),
    /// An error from the `gateway` module.
    #[cfg(feature = "gateway")]
    Gateway(GatewayError),
    /// An error from the [`http`] module.
    ///
    /// [`http`]: http/index.html
    #[cfg(feature = "http")]
    Http(HttpError),
    /// An error from the `rust-websocket` crate.
    #[cfg(feature = "gateway")]
    WebSocket(WebSocketError),
    /// An error from the `opus` crate.
    #[cfg(feature = "voice")]
    Opus(OpusError),
    /// Indicating an error within the [voice module].
    ///
    /// [voice module]: voice/index.html
    #[cfg(feature = "voice")]
    Voice(VoiceError),
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error { Error::Format(e) }
}

#[cfg(feature = "gateway")]
impl From<GatewayError> for Error {
    fn from(e: GatewayError) -> Error { Error::Gateway(e) }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error { Error::Io(e) }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error { Error::Json(e) }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error { Error::Num(e) }
}

impl From<ModelError> for Error {
    fn from(e: ModelError) -> Error { Error::Model(e) }
}

#[cfg(feature = "voice")]
impl From<OpusError> for Error {
    fn from(e: OpusError) -> Error { Error::Opus(e) }
}

#[cfg(feature = "gateway")]
impl From<WebSocketError> for Error {
    fn from(e: WebSocketError) -> Error { Error::WebSocket(e) }
}

#[cfg(feature = "http")]
impl From<HttpError> for Error {
    fn from(e: HttpError) -> Error { Error::Http(e) }
}

#[cfg(feature = "http")]
impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Error { HttpError::InvalidHeader(e).into() }
}

#[cfg(feature = "http")]
impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Error { HttpError::Request(e).into() }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Decode(msg, _) | Error::Other(msg) => msg,
            Error::ExceededLimit(..) => "Input exceeded a limit",
            Error::Format(ref inner) => inner.description(),
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::Model(ref inner) => inner.description(),
            Error::Num(ref inner) => inner.description(),
            Error::Url(ref inner) => inner,
            #[cfg(feature = "client")]
            Error::Client(ref inner) => inner.description(),
            #[cfg(feature = "gateway")]
            Error::Gateway(ref inner) => inner.description(),
            #[cfg(feature = "http")]
            Error::Http(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Opus(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Voice(_) => "Voice error",
            #[cfg(feature = "gateway")]
            Error::WebSocket(ref inner) => inner.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Json(ref inner) => Some(inner),
            Error::Io(ref inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Error::WebSocket(ref inner) => Some(inner),
            _ => None,
        }
    }
}
