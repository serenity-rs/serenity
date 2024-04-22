use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

#[cfg(feature = "http")]
use reqwest::{header::InvalidHeaderValue, Error as ReqwestError};
#[cfg(feature = "gateway")]
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

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
/// The library exposes functions which, for a result type, exposes only one type, rather than the
/// usual 2 (`Result<T, Error>`). This is because all functions that return a result return
/// serenity's [`Error`], so this is implied, and a "simpler" result is used.
pub type Result<T, E = Error> = StdResult<T, E>;

/// A common error enum returned by most of the library's functionality within a custom [`Result`].
///
/// The most common error types, the [`ClientError`] and [`GatewayError`] enums, are both wrapped
/// around this in the form of the [`Self::Client`] and [`Self::Gateway`] variants.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An [`std::io`] error.
    Io(IoError),
    /// An error from the [`serde_json`] crate.
    Json(serde_json::Error),
    /// An error from the [`model`] module.
    ///
    /// [`model`]: crate::model
    Model(ModelError),
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
    Tungstenite(Box<TungsteniteError>),
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
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
        Error::Tungstenite(Box::new(e))
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
            Self::Io(inner) => fmt::Display::fmt(&inner, f),
            Self::Json(inner) => fmt::Display::fmt(&inner, f),
            Self::Model(inner) => fmt::Display::fmt(&inner, f),
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
    #[cfg_attr(feature = "tracing_instrument", instrument)]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(inner) => Some(inner),
            Self::Json(inner) => Some(inner),
            Self::Model(inner) => Some(inner),
            #[cfg(feature = "client")]
            Self::Client(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => Some(inner),
            #[cfg(feature = "http")]
            Self::Http(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => Some(inner),
        }
    }
}
