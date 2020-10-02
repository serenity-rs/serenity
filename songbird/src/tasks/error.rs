use super::{
	AuxPacketMessage,
    EventMessage,
    MixerMessage,
};
use crate::ws::Error as WsError;
use audiopus::Error as OpusError;
use flume::SendError;
use std::io::Error as IoError;
use xsalsa20poly1305::aead::Error as CryptoError;

#[derive(Debug)]
pub enum Location {
	AuxNetwork,
	Event,
	Mixer,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Crypto(CryptoError),
	/// Received an illegal voice packet on the voice UDP socket.
    IllegalVoicePacket,
    InterconnectFailure(Location),
    Io(IoError),
    Opus(OpusError),
    Ws(WsError),
}

impl From<CryptoError> for Error {
    fn from(e: CryptoError) -> Self { Error::Crypto(e) }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error { Error::Io(e) }
}

impl From<OpusError> for Error {
    fn from(e: OpusError) -> Error { Error::Opus(e) }
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
