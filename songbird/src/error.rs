// #[cfg(feature = "driver")]
// use crate::tasks::{AuxPacketMessage, EventMessage, MixerMessage};
// #[cfg(feature = "driver")]
// use audiopus::Error as OpusError;
// #[cfg(feature = "driver")]
// use flume::SendError;
// use serde_json::{Error as JsonError, Value};
// use std::{
//     io::Error as IoError,
//     process::Output
// };
// #[cfg(feature = "driver")]
// use streamcatcher::CatcherError;
// #[cfg(feature = "driver")]
// use xsalsa20poly1305::aead::Error as CryptoError;

// #[cfg(feature = "driver")]
// use async_tungstenite::tungstenite::error::Error as TungsteniteError;
// #[cfg(all(feature = "driver", feature = "rustls", not(feature = "native")))]
// use crate::ws::RustlsError;

// use async_tungstenite::tungstenite::protocol::CloseFrame;

// #[cfg(feature = "driver")]
// pub enum ConnectionError {
//     /// An error occurred during [en/de]cryption of voice packets or key generation.
//     Crypto(CryptoError),
//     /// Server did not return the expected crypto mode during negotiation.
//     CryptoModeInvalid,
//     /// Selected crypto mode was not offered by server.
//     CryptoModeUnavailable,
//     /// An indicator that an endpoint URL was invalid.
//     EndpointUrl,
//     /// Discord hello/ready handshake was violated.
//     ExpectedHandshake,

//     IllegalDiscoveryResponse,

//     /// Received an illegal voice packet on the voice UDP socket.
//     IllegalVoicePacket,

//     InterconnectFailure,

//     Tls(RustlsError),

//     Ws(TungsteniteError),

//     WsClosed(Option<CloseFrame<'static>>),
// }

// #[cfg(feature = "driver")]
// pub enum DriverError {
//     InterconnectFailure,
// }

// /// An error returned from the voice module.
// // Errors which are not visible to the end user are hidden.
// // #[derive(Debug)]
// // #[non_exhaustive]
// // pub enum Error {

    

// //     Io(IoError),

// //     Json(JsonError),

// //     /// An error occurred within the Opus codec.
// //     Opus(OpusError),

// //     /// Apparently failed to create stdout...
// //     Stdout,

// //     /// An error occurred while checking if a path is stereo.
// //     Streams,
// //     /// Configuration error for a cached Input.
// //     Streamcatcher(CatcherError),

    
// // }

// impl From<CatcherError> for Error {
//     fn from(e: CatcherError) -> Self { Error::Streamcatcher(e) }
// }

// impl From<CryptoError> for Error {
//     fn from(e: CryptoError) -> Self { Error::Crypto(e) }
// }

// impl From<IoError> for Error {
//     fn from(e: IoError) -> Error { Error::Io(e) }
// }

// impl From<JsonError> for Error {
//     fn from(e: JsonError) -> Self { Error::Json(e) }
// }

// impl From<OpusError> for Error {
//     fn from(e: OpusError) -> Error { Error::Opus(e) }
// }

// impl From<SendError<AuxPacketMessage>> for Error {
//     fn from(_e: SendError<AuxPacketMessage>) -> Error { Error::InterconnectFailure }
// }

// impl From<SendError<EventMessage>> for Error {
//     fn from(_e: SendError<EventMessage>) -> Error { Error::InterconnectFailure }
// }

// impl From<SendError<MixerMessage>> for Error {
//     fn from(_e: SendError<MixerMessage>) -> Error { Error::InterconnectFailure }
// }

// impl From<RustlsError> for Error {
//     fn from(e: RustlsError) -> Error { Error::Tls(e) }
// }

// impl From<TungsteniteError> for Error {
//     fn from(e: TungsteniteError) -> Error { Error::Ws(e) }
// }

// /// An error returned from the dca method.
// #[derive(Debug)]
// #[non_exhaustive]
// pub enum DcaError {
//     IoError(IoError),
//     InvalidHeader,
//     InvalidMetadata(JsonError),
//     InvalidSize(i32),
//     Opus(OpusError),
// }

// pub type Result<T> = std::result::Result<T, Error>;
