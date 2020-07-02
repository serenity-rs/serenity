use flate2::read::ZlibDecoder;
use crate::gateway::{GatewayError, WsStream};
use crate::internal::prelude::*;
use serde_json;
use async_tungstenite::tungstenite::Message;
use async_trait::async_trait;
use log::warn;
use futures::{SinkExt, StreamExt, TryStreamExt};
use tokio::time::timeout;

#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    io::Error as IoError,
};
use url::Url;
use futures::stream::SplitSink;

#[async_trait]
pub trait ReceiverExt {
    async fn recv_json(&mut self) -> Result<Option<Value>>;
    async fn try_recv_json(&mut self) -> Result<Option<Value>>;
}

#[async_trait]
pub trait SenderExt {
    async fn send_json(&mut self, value: &Value) -> Result<()>;
}

#[async_trait]
impl ReceiverExt for WsStream {
    async fn recv_json(&mut self) -> Result<Option<Value>> {
        const TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_millis(500);

        let ws_message = match timeout(TIMEOUT, self.next()).await {
            Ok(v) => v.map(|v| v.ok()).flatten(),
            Err(_) => None,
        };

        convert_ws_message(ws_message)
    }

    async fn try_recv_json(&mut self) -> Result<Option<Value>> {
        convert_ws_message(self.try_next().await.ok().flatten())
    }
}

#[async_trait]
impl SenderExt for SplitSink<WsStream, Message> {
    async fn send_json(&mut self, value: &Value) -> Result<()> {
        Ok(serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .and_then(|m| {
                Ok(self.send(m))
            })?
            .await?)
    }
}

#[async_trait]
impl SenderExt for WsStream {
    async fn send_json(&mut self, value: &Value) -> Result<()> {
        Ok(serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .and_then(|m| {
                Ok(self.send(m))
            })?
            .await?)
    }
}

#[inline]
pub(crate) fn convert_ws_message(message: Option<Message>) -> Result<Option<Value>> {
    Ok(match message {
        Some(Message::Binary(bytes)) => {
            serde_json::from_reader(ZlibDecoder::new(&bytes[..]))
                .map(Some)
                .map_err(|why| {
                    warn!("Err deserializing bytes: {:?}; bytes: {:?}", why, bytes);

                    why
                })?
        },
        Some(Message::Text(payload)) => {
            serde_json::from_str(&payload).map(Some).map_err(|why| {
                warn!(
                    "Err deserializing text: {:?}; text: {}",
                    why,
                    payload,
                );

                why
            })?
        },
        Some(Message::Close(Some(frame))) => {
            return Err(Error::Gateway(GatewayError::Closed(Some(frame))));
        },
        // Ping/Pong message behaviour is internally handled by tungstenite.
        _ => None,
    })
}

/// An error that occured while connecting over rustls
#[derive(Debug)]
#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
pub enum RustlsError {
    /// WebPKI X.509 Certificate Validation Error.
    WebPKI,
    /// An error with the handshake in tungstenite
    HandshakeError,
    /// Standard IO error happening while creating the tcp stream
    Io(IoError),
    #[doc(hidden)]
    __Nonexhaustive,
}

#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
impl From<IoError> for RustlsError {
    fn from(e: IoError) -> Self {
        RustlsError::Io(e)
    }
}

#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
impl Display for RustlsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RustlsError::WebPKI => f.write_str("Failed to validate X.509 certificate"),
            RustlsError::HandshakeError => f.write_str("TLS handshake failed when making the websocket connection"),
            RustlsError::Io(inner) => Display::fmt(&inner, f),
            RustlsError::__Nonexhaustive => unreachable!(),
        }
    }
}

#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
impl StdError for RustlsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            RustlsError::Io(inner) => Some(inner),
            _ => None,
        }
    }
}

// Create a tungstenite client with a rustls stream.
#[cfg(not(feature = "native_tls_backend"))]
pub(crate) fn create_rustls_client(url: Url) -> Result<WsClient> {
    let mut config = rustls::ClientConfig::new();
    config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let base_host = if let Some(h) = url.host_str() {
        let (dot, _) = h.rmatch_indices('.').nth(1).unwrap_or((0, ""));
        // We do not want the leading '.', but if there is no leading '.' we do
        // not want to remove the leading character.
        let split_at_index = if dot == 0 { 0 } else { dot + 1 };
        let (_, base) = h.split_at(split_at_index);
        base.to_owned()
    } else { "discord.gg".to_owned() };

    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&base_host)
        .map_err(|_| RustlsError::WebPKI)?;

    let session = rustls::ClientSession::new(&Arc::new(config), dns_name);

    let port = url.port_or_known_default()
        .ok_or_else(|| Error::Url("No port number in the URL.".into()))?;

    let addrs = url.socket_addrs(|| Some(port))?;

    let socket = TcpStream::connect(addrs.as_slice())?;
    let tls = rustls::StreamOwned::new(session, socket);

    let client = tungstenite::client(url, tls)
        .map_err(|_| RustlsError::HandshakeError)?;

    Ok(client.0)
}


#[cfg(all(feature = "rustls_backend", not(feature = "native_tls_backend")))]
pub(crate) async fn create_rustls_client(url: Url) -> Result<WsStream> {
    let (stream, _) = async_tungstenite::tokio::connect_async_with_config::<Url>(
        url.into(),
        Some(async_tungstenite::tungstenite::protocol::WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            max_send_queue: None,
        }))
        .await
        .map_err(|_| RustlsError::HandshakeError)?;

    Ok(stream)
}

#[cfg(feature = "native_tls_backend")]
pub(crate) async fn create_native_tls_client(url: Url) -> Result<WsStream> {
    let (stream, _) = async_tungstenite::tokio::connect_async_with_config::<Url>(
        url.into(),
        Some(async_tungstenite::tungstenite::protocol::WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            max_send_queue: None,
        }))
        .await?;

    Ok(stream)
}
