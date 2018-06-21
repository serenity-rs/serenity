use futures::{
    future::SharedError,
    Canceled,
    Future
};
use http_lib::{
    uri::InvalidUri as UriError,
    Error as HttpCrateError,
};
use internal::prelude::*;
use model::ModelError;
use serde_json::Error as JsonError;
use std::{
    cell::{BorrowError, BorrowMutError},
    error::Error as StdError,
    fmt::{self, Display, Error as FormatError},
    io::Error as IoError,
    num::ParseIntError,
};

#[cfg(feature = "tungstenite")]
use futures::sync::mpsc::SendError;
#[cfg(feature = "hyper")]
use hyper::Error as HyperError;
#[cfg(feature = "native-tls")]
use native_tls::Error as TlsError;
#[cfg(feature = "voice")]
use opus::Error as OpusError;
#[cfg(feature = "tungstenite")]
use tungstenite::{Error as TungsteniteError, Message as TungsteniteMessage};
#[cfg(feature = "client")]
use client::ClientError;
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
    /// A cell could not be immutably borrowed.
    Borrow(BorrowError),
    /// An error occurred while mutably borrowing from an `std::cell::RefCell`.
    BorrowMut(BorrowMutError),
    /// A future was canceled, most likely reading from an mpsc channel.
    Canceled(Canceled),
    /// An error while decoding a payload.
    Decode(&'static str, Value),
    /// There was an error with a format.
    Format(FormatError),
    /// An error while trying to send over a future-compatible
    /// MPSC.
    FutureMpsc(&'static str),
    /// A `hyper` error relating to Http.
    HttpCrate(HttpCrateError),
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
    /// A `hyper` error while parsing a Uri.
    Uri(UriError),
    /// An error from the `url` crate.
    Url(String),
    /// A [client] error.
    ///
    /// [client]: client/index.html
    #[cfg(feature = "client")]
    Client(ClientError),
    /// An error from the [`http`] module.
    ///
    /// [`http`]: http/index.html
    #[cfg(feature = "http")]
    Http(HttpError),
    /// An error from the `hyper` crate.
    #[cfg(feature = "hyper")]
    Hyper(HyperError),
    /// An error from the `native-tls` crate.
    #[cfg(feature = "native-tls")]
    Tls(TlsError),
    /// An error while sending a message over a WebSocket.
    #[cfg(feature = "tungstenite")]
    WebSocketSend(SendError<TungsteniteMessage>),
    /// An error from the `tungstenite` crate.
    #[cfg(feature = "tungstenite")]
    Tungstenite(TungsteniteError),
    /// An error from the `opus` crate.
    #[cfg(feature = "voice")]
    Opus(OpusError),
    /// Indicating an error within the [voice module].
    ///
    /// [voice module]: voice/index.html
    #[cfg(feature = "voice")]
    Voice(VoiceError),
}

impl From<BorrowError> for Error {
    fn from(e: BorrowError) -> Error { Error::Borrow(e) }
}

impl From<BorrowMutError> for Error {
    fn from(e: BorrowMutError) -> Error { Error::BorrowMut(e) }
}

impl From<Canceled> for Error {
    fn from(e: Canceled) -> Error { Error::Canceled(e) }
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error { Error::Format(e) }
}

#[cfg(feature = "http")]
impl From<HttpError> for Error {
    fn from(e: HttpError) -> Error { Error::Http(e) }
}

#[cfg(feature = "hyper")]
impl From<HyperError> for Error {
    fn from(e: HyperError) -> Error { Error::Hyper(e) }
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

#[cfg(feature = "tungstenite")]
impl From<SendError<TungsteniteMessage>> for Error {
    fn from(err: SendError<TungsteniteMessage>) -> Self {
        Error::WebSocketSend(err)
    }
}

impl<T> From<SharedError<T>> for Error
        where Error: From<T>, T: Clone {
    fn from(e: SharedError<T>) -> Error { Error::from((*e).clone()) }
}

#[cfg(feature = "native-tls")]
impl From<TlsError> for Error {
    fn from(e: TlsError) -> Error { Error::Tls(e) }
}

#[cfg(feature = "tungstenite")]
impl From<TungsteniteError> for Error {
    fn from(err: TungsteniteError) -> Self {
        Error::Tungstenite(err)
    }
}

impl From<UriError> for Error {
    fn from(e: UriError) -> Error { Error::Uri(e) }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Borrow(ref inner) => inner.description(),
            Error::BorrowMut(ref inner) => inner.description(),
            Error::Canceled(ref inner) => inner.description(),
            Error::Decode(msg, _) | Error::Other(msg) => msg,
            Error::ExceededLimit(..) => "Input exceeded a limit",
            Error::Format(ref inner) => inner.description(),
            Error::FutureMpsc(ref inner) => inner,
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::Model(ref inner) => inner.description(),
            Error::Num(ref inner) => inner.description(),
            Error::Url(ref inner) => inner,
            Error::Uri(ref inner) => inner.description(),
            #[cfg(feature = "client")]
            Error::Client(ref inner) => inner.description(),
            #[cfg(feature = "http")]
            Error::Http(ref inner) => inner.description(),
            #[cfg(feature = "http")]
            Error::Hyper(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Opus(ref inner) => inner.description(),
            #[cfg(feature = "native-tls")]
            Error::Tls(ref inner) => inner.description(),
            #[cfg(feature = "tungstenite")]
            Error::Tungstenite(ref inner) => inner.description(),
            #[cfg(feature = "voice")]
            Error::Voice(_) => "Voice error",
            #[cfg(feature = "tungstenite")]
            Error::WebSocketSend(ref inner) => inner.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            #[cfg(feature = "http")]
            Error::Hyper(ref inner) => Some(inner),
            Error::Json(ref inner) => Some(inner),
            Error::Io(ref inner) => Some(inner),
            _ => None,
        }
    }
}
