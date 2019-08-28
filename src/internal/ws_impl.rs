use flate2::read::ZlibDecoder;
use crate::gateway::{WsClient, WsStream};
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

const ZLIB_SUFFIX: [u8; 4] = [0x00,0x00,0xff,0xff];

pub trait ReceiverExt {
    fn recv_json(&mut self) -> Result<Option<Value>>;
    fn try_recv_json(&mut self) -> Result<Option<Value>>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for WsClient {
    fn recv_json(&mut self) -> Result<Option<Value>> {
        dbg!(convert_ws_message(Some(self.read_message()?)))
    }

    fn try_recv_json(&mut self) -> Result<Option<Value>> {
        dbg!(convert_ws_message(Some(self.read_message()?)))
    }
}

impl SenderExt for WsClient {
    fn send_json(&mut self, value: &Value) -> Result<()> {
        serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .and_then(|m| self.stream.write_message(m).map_err(Error::from))
    }
}

impl WsClient {
    pub(crate) fn read_message(&mut self) -> Result<Message> {
        let message;
        loop {
            match self.stream.read_message()? {
                Message::Binary(bin) => {
                    let len = bin.len();
                    let has_suffix = bin[len-4..] == ZLIB_SUFFIX;
                    self.buffer.extend(bin);
                    if len < 4 || !has_suffix {
                        continue;
                    }
                    message = Message::Text(decode_buffer(&self.buffer)?);
                    self.buffer.clear();
                    break;
                },
                _ => unreachable!(),                    
            }
        }
        Ok(message)
    }
}

#[inline]
fn decode_buffer(bytes: &[u8]) -> Result<String> {
    use flate2::bufread::ZlibDecoder as ZlibBufDecoder;
    use std::io::Read;
    let mut z = ZlibBufDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
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
pub(crate) fn create_rustls_client(url: Url) -> Result<WsStream> {
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
    let socket = TcpStream::connect(&url)?;
    let tls = rustls::StreamOwned::new(session, socket);

    let client = tungstenite::client(url, tls)
        .map_err(|_| RustlsError::HandshakeError)?;

    Ok(client.0)
}
