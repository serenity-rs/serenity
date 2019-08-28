use flate2::read::ZlibDecoder;
use crate::gateway::WsClient;
use crate::internal::prelude::*;
use serde_json;
use tungstenite::{
    util::NonBlockingResult,
    Message,
};
use log::warn;

#[cfg(not(feature = "native_tls_backend"))]
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
#[cfg(not(feature = "native_tls_backend"))]
use url::Url;
#[cfg(not(feature = "native_tls_backend"))]
use std::net::ToSocketAddrs;

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
#[cfg(not(feature = "native_tls_backend"))]
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

#[cfg(not(feature = "native_tls_backend"))]
impl From<IoError> for RustlsError {
    fn from(e: IoError) -> Self {
        RustlsError::Io(e)
    }
}

#[cfg(not(feature = "native_tls_backend"))]
impl Display for RustlsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { f.write_str(self.description()) }
}

#[cfg(not(feature = "native_tls_backend"))]
impl StdError for RustlsError {
    fn description(&self) -> &str {
        use self::RustlsError::*;

        match *self {
            WebPKI => "Failed to validate X.509 certificate",
            HandshakeError => "TLS handshake failed when making the websocket connection",
            Io(ref inner) => inner.description(),
            __Nonexhaustive => unreachable!(),
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

    let host = url.host()
        .ok_or_else(|| Error::Url("No host name in the URL.".into()))?;
    let port = url.port_or_known_default()
        .ok_or_else(|| Error::Url("No port number in the URL.".into()))?;
    // We need these to ensure the lifetime is long enough,
    // variables that would live inside the `match` would not live long enough.
    let addr;
    let addrs;
    let addrs = match host {
        url::Host::Domain(domain) => {
            addrs = (domain, port).to_socket_addrs()?;
            addrs.as_slice()
        },
        url::Host::Ipv4(ip) => {
            addr = (ip, port).into();
            std::slice::from_ref(&addr)
        },
        url::Host::Ipv6(ip) => {
            addr = (ip, port).into();
            std::slice::from_ref(&addr)
        },
    };

    let socket = TcpStream::connect(&addrs)?;
    let tls = rustls::StreamOwned::new(session, socket);

    let client = tungstenite::client(url, tls)
        .map_err(|_| RustlsError::HandshakeError)?;

    Ok(client.0)
}
