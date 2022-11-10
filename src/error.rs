use std::error::Error as StdError;
use std::fmt::{self, Error as FormatError};
use std::io::Error as IoError;

#[cfg(feature = "http")]
use reqwest::{header::InvalidHeaderValue, Error as ReqwestError};
use serde_json::Error as JsonError;
#[cfg(feature = "gateway")]
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tracing::instrument;

#[cfg(feature = "client")]
use crate::client::ClientError;
#[cfg(feature = "gateway")]
use crate::gateway::GatewayError;
#[cfg(feature = "http")]
use crate::http::HttpError;
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
    /// Some other error. This is only used for "Expected value \<TYPE\>" errors,
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
    /// An error from the [`gateway`] module.
    ///
    /// [`gateway`]: crate::gateway
    #[cfg(feature = "gateway")]
    Gateway(GatewayError),
    /// An error from the [`http`] module.
    ///
    /// [`http`]: crate::http
    #[cfg(feature = "http")]
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

#[cfg(feature = "http")]
impl From<HttpError> for Error {
    fn from(e: HttpError) -> Error {
        Error::Http(e)
    }
}

#[cfg(feature = "http")]
impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Error {
        HttpError::InvalidHeader(e).into()
    }
}

#[cfg(feature = "http")]
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
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "http")]
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
            #[cfg(feature = "http")]
            Self::Http(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => Some(inner),
            _ => None,
        }
    }
}
