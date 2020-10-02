use crate::tasks::{
    error::Location,
    AuxPacketMessage,
    EventMessage,
    MixerMessage,
};
use serde_json::Error as JsonError;
use std::io::Error as IoError;
use flume::SendError;
use xsalsa20poly1305::aead::Error as CryptoError;
use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use crate::ws::Error as WsError;

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

    IllegalDiscoveryResponse,

    Io(IoError),

    Json(JsonError),

    InterconnectFailure(Location),

    Ws(WsError),
}

impl From<CryptoError> for Error {
    fn from(e: CryptoError) -> Self { Error::Crypto(e) }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error { Error::Io(e) }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error { Error::Json(e) }
}

impl From<SendError<AuxPacketMessage>> for Error {
    fn from(_e: SendError<AuxPacketMessage>) -> Error { Error::InterconnectFailure(Location::AuxNetwork) }
}

impl From<SendError<EventMessage>> for Error {
    fn from(_e: SendError<EventMessage>) -> Error { Error::InterconnectFailure(Location::Event) }
}

impl From<SendError<MixerMessage>> for Error {
    fn from(_e: SendError<MixerMessage>) -> Error { Error::InterconnectFailure(Location::Mixer) }
}

impl From<WsError> for Error {
    fn from(e: WsError) -> Error { Error::Ws(e) }
}

pub type Result<T> = std::result::Result<T, Error>;
