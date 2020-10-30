// FIXME: this is copied from serenity/src/internal/ws_impl.rs
// To prevent this duplication, we either need to expose this on serenity's API
// (not desirable) or break the common WS elements into a subcrate.
// I believe that decisions is outside of the scope of the voice subcrate PR.

use crate::model::Event;

use async_trait::async_trait;
use async_tungstenite::{
    tokio::ConnectStream,
    tungstenite::{error::Error as TungsteniteError, protocol::CloseFrame, Message},
    WebSocketStream,
};
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde_json::Error as JsonError;
use tokio::time::timeout;
use tracing::{instrument, warn};

pub type WsStream = WebSocketStream<ConnectStream>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Json(JsonError),
    #[cfg(all(feature = "rustls", not(feature = "native")))]
    Tls(RustlsError),

    /// The discord voice gateway does not support or offer zlib compression.
    /// As a result, only text messages are expected.
    UnexpectedBinaryMessage(Vec<u8>),

    Ws(TungsteniteError),

    WsClosed(Option<CloseFrame<'static>>),
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

#[cfg(all(feature = "rustls", not(feature = "native")))]
impl From<RustlsError> for Error {
    fn from(e: RustlsError) -> Error {
        Error::Tls(e)
    }
}

impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Error {
        Error::Ws(e)
    }
}

use futures::stream::SplitSink;
#[cfg(all(feature = "rustls", not(feature = "native")))]
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
};
use url::Url;

#[async_trait]
pub trait ReceiverExt {
    async fn recv_json(&mut self) -> Result<Option<Event>>;
    async fn recv_json_no_timeout(&mut self) -> Result<Option<Event>>;
}

#[async_trait]
pub trait SenderExt {
    async fn send_json(&mut self, value: &Event) -> Result<()>;
}

#[async_trait]
impl ReceiverExt for WsStream {
    async fn recv_json(&mut self) -> Result<Option<Event>> {
        const TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_millis(500);

        let ws_message = match timeout(TIMEOUT, self.next()).await {
            Ok(Some(Ok(v))) => Some(v),
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => None,
        };

        convert_ws_message(ws_message)
    }

    async fn recv_json_no_timeout(&mut self) -> Result<Option<Event>> {
        convert_ws_message(self.try_next().await.ok().flatten())
    }
}

#[async_trait]
impl SenderExt for SplitSink<WsStream, Message> {
    async fn send_json(&mut self, value: &Event) -> Result<()> {
        Ok(serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .map(|m| self.send(m))?
            .await?)
    }
}

#[async_trait]
impl SenderExt for WsStream {
    async fn send_json(&mut self, value: &Event) -> Result<()> {
        Ok(serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .map(|m| self.send(m))?
            .await?)
    }
}

#[inline]
pub(crate) fn convert_ws_message(message: Option<Message>) -> Result<Option<Event>> {
    Ok(match message {
        Some(Message::Text(payload)) =>
            serde_json::from_str(&payload).map(Some).map_err(|why| {
                warn!("Err deserializing text: {:?}; text: {}", why, payload,);

                why
            })?,
        Some(Message::Binary(bytes)) => {
            return Err(Error::UnexpectedBinaryMessage(bytes));
        },
        Some(Message::Close(Some(frame))) => {
            return Err(Error::WsClosed(Some(frame)));
        },
        // Ping/Pong message behaviour is internally handled by tungstenite.
        _ => None,
    })
}

/// An error that occured while connecting over rustls
#[derive(Debug)]
#[non_exhaustive]
#[cfg(all(feature = "rustls", not(feature = "native")))]
pub enum RustlsError {
    /// An error with the handshake in tungstenite
    HandshakeError,
    /// Standard IO error happening while creating the tcp stream
    Io(IoError),
}

#[cfg(all(feature = "rustls", not(feature = "native")))]
impl From<IoError> for RustlsError {
    fn from(e: IoError) -> Self {
        RustlsError::Io(e)
    }
}

#[cfg(all(feature = "rustls", not(feature = "native")))]
impl Display for RustlsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RustlsError::HandshakeError =>
                f.write_str("TLS handshake failed when making the websocket connection"),
            RustlsError::Io(inner) => Display::fmt(&inner, f),
        }
    }
}

#[cfg(all(feature = "rustls", not(feature = "native")))]
impl StdError for RustlsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            RustlsError::Io(inner) => Some(inner),
            _ => None,
        }
    }
}

#[cfg(all(feature = "rustls", not(feature = "native")))]
#[instrument]
pub(crate) async fn create_rustls_client(url: Url) -> Result<WsStream> {
    let (stream, _) = async_tungstenite::tokio::connect_async_with_config::<Url>(
        url,
        Some(async_tungstenite::tungstenite::protocol::WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            max_send_queue: None,
        }),
    )
    .await
    .map_err(|_| RustlsError::HandshakeError)?;

    Ok(stream)
}

#[cfg(feature = "native")]
#[instrument]
pub(crate) async fn create_native_tls_client(url: Url) -> Result<WsStream> {
    let (stream, _) = async_tungstenite::tokio::connect_async_with_config::<Url>(
        url,
        Some(async_tungstenite::tungstenite::protocol::WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            max_send_queue: None,
        }),
    )
    .await?;

    Ok(stream)
}
