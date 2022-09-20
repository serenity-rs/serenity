use std::error::Error as StdError;
use std::fmt::{self, Error as FormatError};
use std::io::Error as IoError;

use reqwest::header::InvalidHeaderValue;
use reqwest::Error as ReqwestError;
use serde_json::Error as JsonError;
#[cfg(feature = "gateway")]
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tracing::instrument;
use url::ParseError as UrlError;

#[cfg(feature = "client")]
use crate::client::ClientError;
#[cfg(feature = "collector")]
use crate::collector::CollectorError;
#[cfg(feature = "gateway")]
use crate::gateway::GatewayError;
#[cfg(feature = "http")]
use crate::http::error::ErrorResponse;
use crate::internal::prelude::*;
use crate::model::ModelError;

/// The common result type between most library functions.
///
/// The library exposes functions which, for a result type, exposes only one
/// type, rather than the usual 2 (`Result<T, Error>`). This is because all
/// functions that return a result return serenity's [`Error`], so this is
/// implied, and a "simpler" result is used.
pub type Result<T> = StdResult<T, Error>;

/// A common error enum returned by most of the library's functionality within a
/// custom [`Result`].
///
/// The most common error types, the [`ClientError`] and [`GatewayError`]
/// enums, are both wrapped around this in the form of the [`Self::Client`] and
/// [`Self::Gateway`] variants.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error while decoding a payload.
    Decode(&'static str, Value),
    /// There was an error with a format.
    Format(FormatError),
    /// An [`std::io`] error.
    Io(IoError),
    /// An error from the [`serde_json`] crate.
    Json(JsonError),
    #[cfg(feature = "simd-json")]
    /// An error from the `simd_json` crate.
    SimdJson(simd_json::Error),
    /// An error from the [`model`] module.
    ///
    /// [`model`]: crate::model
    Model(ModelError),
    /// Input exceeded a limit.
    /// Providing the input and the limit that's not supposed to be exceeded.
    ///
    /// *This only exists for the [`GuildId::ban`] and [`Member::ban`] functions. For their cases,
    /// it's the "reason".*
    ///
    /// [`GuildId::ban`]: crate::model::id::GuildId::ban
    /// [`Member::ban`]: crate::model::guild::Member::ban
    ExceededLimit(String, u32),
    /// The input is not in the specified range.
    /// Returned by [`GuildId::members`], [`Guild::members`] and [`PartialGuild::members`]
    ///
    /// (param_name, value, range_min, range_max)
    ///
    /// [`GuildId::members`]: crate::model::id::GuildId::members
    /// [`Guild::members`]: crate::model::guild::Guild::members
    /// [`PartialGuild::members`]: crate::model::guild::PartialGuild::members
    NotInRange(&'static str, u64, u64, u64),
    /// Some other error. This is only used for "Expected value <TYPE>" errors,
    /// when a more detailed error can not be easily provided via the
    /// [`Error::Decode`] variant.
    Other(&'static str),
    /// An error from the [`url`] crate.
    Url(String),
    /// A [client] error.
    ///
    /// [client]: crate::client
    #[cfg(feature = "client")]
    Client(ClientError),
    /// A [collector] error.
    ///
    /// [collector]: crate::collector
    #[cfg(feature = "collector")]
    Collector(CollectorError),
    /// An error from the [`gateway`] module.
    ///
    /// [`gateway`]: crate::gateway
    #[cfg(feature = "gateway")]
    Gateway(GatewayError),
    /// An HTTP error. Mostly from the `http` module, but can also be from `gateway`.
    ///
    /// [`http`]: crate::http
    Http(HttpError),
    /// An error from the `tungstenite` crate.
    #[cfg(feature = "gateway")]
    Tungstenite(TungsteniteError),
}

#[cfg(feature = "simd-json")]
impl From<simd_json::Error> for Error {
    fn from(e: simd_json::Error) -> Self {
        Error::SimdJson(e)
    }
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error {
        Error::Format(e)
    }
}

#[cfg(feature = "gateway")]
impl From<GatewayError> for Error {
    fn from(e: GatewayError) -> Error {
        Error::Gateway(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

impl From<ModelError> for Error {
    fn from(e: ModelError) -> Error {
        Error::Model(e)
    }
}

#[cfg(feature = "gateway")]
impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Error {
        Error::Tungstenite(e)
    }
}

impl From<HttpError> for Error {
    fn from(e: HttpError) -> Error {
        Error::Http(e)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Error {
        HttpError::InvalidHeader(e).into()
    }
}

impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Error {
        HttpError::Request(e).into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(msg, _) | Self::Other(msg) => f.write_str(msg),
            Self::ExceededLimit(..) => f.write_str("Input exceeded a limit"),
            Self::NotInRange(..) => f.write_str("Input is not in the specified range"),
            Self::Format(inner) => fmt::Display::fmt(&inner, f),
            Self::Io(inner) => fmt::Display::fmt(&inner, f),
            Self::Json(inner) => fmt::Display::fmt(&inner, f),
            Self::Model(inner) => fmt::Display::fmt(&inner, f),
            Self::Url(msg) => f.write_str(msg),
            #[cfg(feature = "simd-json")]
            Error::SimdJson(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "client")]
            Self::Client(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "collector")]
            Self::Collector(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => fmt::Display::fmt(&inner, f),
            Self::Http(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => fmt::Display::fmt(&inner, f),
        }
    }
}

impl StdError for Error {
    #[instrument]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Format(inner) => Some(inner),
            Self::Io(inner) => Some(inner),
            Self::Json(inner) => Some(inner),
            Self::Model(inner) => Some(inner),
            #[cfg(feature = "client")]
            Self::Client(inner) => Some(inner),
            #[cfg(feature = "collector")]
            Self::Collector(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => Some(inner),
            #[cfg(feature = "http")]
            Self::Http(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => Some(inner),
            _ => None,
        }
    }
}

// This lives here instead of in http/ because the gateway can throw HTTP errors too
// But the impl is in http/
#[derive(Debug)]
#[non_exhaustive]
pub enum HttpError {
    /// When a non-successful status code was received for a request.
    #[cfg(feature = "http")]
    UnsuccessfulRequest(ErrorResponse),
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64` or `f64`.
    RateLimitI64F64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When parsing an URL failed due to invalid input.
    Url(UrlError),
    /// When parsing a Webhook fails due to invalid input.
    InvalidWebhook,
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    /// When using a proxy with an invalid scheme.
    InvalidScheme,
    /// When using a proxy with an invalid port.
    InvalidPort,
    /// When an application id was expected but missing.
    ApplicationIdMissing,
}

impl From<ReqwestError> for HttpError {
    fn from(error: ReqwestError) -> HttpError {
        HttpError::Request(error)
    }
}

impl From<UrlError> for HttpError {
    fn from(error: UrlError) -> HttpError {
        HttpError::Url(error)
    }
}

impl From<InvalidHeaderValue> for HttpError {
    fn from(error: InvalidHeaderValue) -> HttpError {
        HttpError::InvalidHeader(error)
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "http")]
            Self::UnsuccessfulRequest(e) => {
                f.write_str(&e.error.message)?;

                // Put Discord's human readable error explanations in parentheses
                let mut errors_iter = e.error.errors.iter();
                if let Some(error) = errors_iter.next() {
                    f.write_str(" (")?;
                    f.write_str(&error.path)?;
                    f.write_str(": ")?;
                    f.write_str(&error.message)?;
                    for error in errors_iter {
                        f.write_str(", ")?;
                        f.write_str(&error.path)?;
                        f.write_str(": ")?;
                        f.write_str(&error.message)?;
                    }
                    f.write_str(")")?;
                }

                Ok(())
            },
            Self::RateLimitI64F64 => f.write_str("Error decoding a header into an i64 or f64"),
            Self::RateLimitUtf8 => f.write_str("Error decoding a header from UTF-8"),
            Self::Url(_) => f.write_str("Provided URL is incorrect."),
            Self::InvalidWebhook => f.write_str("Provided URL is not a valid webhook."),
            Self::InvalidHeader(_) => f.write_str("Provided value is an invalid header value."),
            Self::Request(_) => f.write_str("Error while sending HTTP request."),
            Self::InvalidScheme => f.write_str("Invalid Url scheme."),
            Self::InvalidPort => f.write_str("Invalid port."),
            Self::ApplicationIdMissing => f.write_str("Application id was expected but missing."),
        }
    }
}

impl StdError for HttpError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Url(inner) => Some(inner),
            Self::Request(inner) => Some(inner),
            _ => None,
        }
    }
}
