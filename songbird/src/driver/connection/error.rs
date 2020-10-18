//! Connection errors and convenience types.

use crate::{
    driver::tasks::{error::Recipient, message::*},
    ws::Error as WsError,
};
use flume::SendError;
use serde_json::Error as JsonError;
use std::{error::Error as ErrorTrait, fmt, io::Error as IoError};
use xsalsa20poly1305::aead::Error as CryptoError;

/// Errors encountered while connecting to a Discord voice server over the driver.
#[derive(Debug)]
pub enum Error {
    /// An error occurred during [en/de]cryption of voice packets or key generation.
    Crypto(CryptoError),
    /// Server did not return the expected crypto mode during negotiation.
    CryptoModeInvalid,
    /// Selected crypto mode was not offered by server.
    CryptoModeUnavailable,
    /// An indicator that an endpoint URL was invalid.
    EndpointUrl,
    /// Discord hello/ready handshake was violated.
    ExpectedHandshake,
    /// Discord failed to correctly respond to IP discovery.
    IllegalDiscoveryResponse,
    /// Could not parse Discord's view of our IP.
    IllegalIp,
    /// Miscellaneous I/O error.
    Io(IoError),
    /// JSON (de)serialization error.
    Json(JsonError),
    /// Failed to message other background tasks after connection establishment.
    InterconnectFailure(Recipient),
    /// Error communicating with gateway server over WebSocket.
    Ws(WsError),
}

impl From<CryptoError> for Error {
    fn from(e: CryptoError) -> Self {
        Error::Crypto(e)
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

impl From<SendError<WsMessage>> for Error {
    fn from(_e: SendError<WsMessage>) -> Error {
        Error::InterconnectFailure(Recipient::AuxNetwork)
    }
}

impl From<SendError<EventMessage>> for Error {
    fn from(_e: SendError<EventMessage>) -> Error {
        Error::InterconnectFailure(Recipient::Event)
    }
}

impl From<SendError<MixerMessage>> for Error {
    fn from(_e: SendError<MixerMessage>) -> Error {
        Error::InterconnectFailure(Recipient::Mixer)
    }
}

impl From<WsError> for Error {
    fn from(e: WsError) -> Error {
        Error::Ws(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to connect to Discord RTP server: ")?;
        use Error::*;
        match self {
            Crypto(c) => write!(f, "cryptography error {}.", c),
            CryptoModeInvalid => write!(f, "server changed negotiated encryption mode."),
            CryptoModeUnavailable => write!(f, "server did not offer chosen encryption mode."),
            EndpointUrl => write!(f, "endpoint URL received from gateway was invalid."),
            ExpectedHandshake => write!(f, "voice initialisation protocol was violated."),
            IllegalDiscoveryResponse =>
                write!(f, "IP discovery/NAT punching response was invalid."),
            IllegalIp => write!(f, "IP discovery/NAT punching response had bad IP value."),
            Io(i) => write!(f, "I/O failure ({}).", i),
            Json(j) => write!(f, "JSON (de)serialization issue ({}).", j),
            InterconnectFailure(r) => write!(f, "failed to contact other task ({:?})", r),
            Ws(w) => write!(f, "websocket issue ({:?}).", w),
        }
    }
}

impl ErrorTrait for Error {}

/// Convenience type for Discord voice/driver connection error handling.
pub type Result<T> = std::result::Result<T, Error>;
