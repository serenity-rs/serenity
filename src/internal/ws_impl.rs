use flate2::read::ZlibDecoder;
use crate::gateway::WsClient;
use crate::internal::prelude::*;
use serde_json;
use tungstenite::{
    util::NonBlockingResult,
    Message,
};

#[cfg(feature = "rustls_support")]
use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    io::Error as IoError,
    net::TcpStream,
    sync::Arc,
};
#[cfg(feature = "rustls_support")]
use url::Url;

pub trait ReceiverExt {
    fn recv_json(&mut self) -> Result<Option<Value>>;
    fn try_recv_json(&mut self) -> Result<Option<Value>>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for WsClient {
    fn recv_json(&mut self) -> Result<Option<Value>> {
        convert_ws_message(Some(self.read_message()?))
    }

    fn try_recv_json(&mut self) -> Result<Option<Value>> {
        convert_ws_message(self.read_message().no_block()?)
    }
}

impl SenderExt for WsClient {
    fn send_json(&mut self, value: &Value) -> Result<()> {
        serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .and_then(|m| self.write_message(m).map_err(Error::from))
    }
}

#[inline]
fn convert_ws_message(message: Option<Message>) -> Result<Option<Value>>{
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
        // Ping/Pong message behaviour is internally handled by tungstenite.
        _ => None,
    })
}

/// An error that occured while connecting over rustls
#[derive(Debug)]
#[cfg(feature = "rustls_support")]
pub enum RustlsError {
    /// WebPKI X.509 Certificate Validation Error.
    WebPKI,
    /// An error with the handshake in tungstenite
    HandshakeError,
    /// Standard IO error happening while creating the tcp stream
    Io(IoError),
}

#[cfg(feature = "rustls_support")]
impl From<IoError> for RustlsError {
    fn from(e: IoError) -> Self {
        RustlsError::Io(e)
    }
}

#[cfg(feature = "rustls_support")]
impl Display for RustlsError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { f.write_str(self.description()) }
}

#[cfg(feature = "rustls_support")]
impl StdError for RustlsError {
    fn description(&self) -> &str {
        use self::RustlsError::*;

        match *self {
            WebPKI => "Failed to validate X.509 certificate",
            HandshakeError => "TLS handshake failed when making the websocket connection",
            Io(ref inner) => inner.description(),
        }
    }
}

// Create a tungstenite client with a rustls stream.
#[cfg(feature = "rustls_support")]
pub(crate) fn create_rustls_client(url: Url) -> Result<WsClient> {
    let mut config = rustls::ClientConfig::new();
    config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let base_host = if let Some(h) = url.host_str() {
        let (dot, _) = h.rmatch_indices('.').skip(1).next().unwrap_or((0, ""));
        // We do not want the leading '.', but if there is no leading '.' we do
        // not want to remove the leading character.
        let split_at_index = if dot == 0 { 0 } else { dot + 1 };
        let (_, base) = h.split_at(split_at_index);
        base.to_owned()
    } else { "discord.gg".to_owned() };

    let dns_name = webpki::DNSNameRef::try_from_ascii_str(&base_host)
        .map_err(|_| RustlsError::WebPKI)?;

    let session = rustls::ClientSession::new(&Arc::new(config), dns_name);
    let socket = TcpStream::connect(&url)?;
    let tls = rustls::StreamOwned::new(session, socket);

    let client = tungstenite::client(url, tls)
        .map_err(|_| RustlsError::HandshakeError)?;

    Ok(client.0)
}
