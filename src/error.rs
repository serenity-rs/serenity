use std::error::Error as StdError;
use std::fmt;

pub use serenity_core::error::Error as CoreError;
#[cfg(feature = "gateway")]
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;

#[cfg(feature = "gateway")]
use crate::gateway::GatewayError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error from the [`gateway`] module.
    ///
    /// [`gateway`]: crate::gateway
    #[cfg(feature = "gateway")]
    Gateway(GatewayError),
    /// An error from the `tungstenite` crate.
    #[cfg(feature = "gateway")]
    Tungstenite(Box<TungsteniteError>),
    /// An error from serenity's core.
    Core(CoreError),
}

#[cfg(feature = "gateway")]
impl From<GatewayError> for Error {
    fn from(e: GatewayError) -> Error {
        Error::Gateway(e)
    }
}

#[cfg(feature = "gateway")]
impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Error {
        Error::Tungstenite(Box::new(e))
    }
}

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Error {
        Error::Core(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => fmt::Display::fmt(&inner, f),
            Self::Core(inner) => fmt::Display::fmt(&inner, f),
        }
    }
}

impl StdError for Error {
    #[cfg_attr(feature = "tracing_instrument", instrument)]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "gateway")]
            Self::Gateway(inner) => Some(inner),
            #[cfg(feature = "gateway")]
            Self::Tungstenite(inner) => Some(inner),
            Self::Core(inner) => Some(inner),
        }
    }
}
