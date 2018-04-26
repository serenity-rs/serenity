use flate2::read::ZlibDecoder;
use gateway::GatewayError;
use internal::prelude::*;
use serde_json;
use websocket::{
    message::OwnedMessage,
    sync::stream::{TcpStream, TlsStream},
    sync::Client as WsClient
};

pub trait ReceiverExt {
    fn recv_json(&mut self) -> Result<Option<Value>>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for WsClient<TlsStream<TcpStream>> {
    fn recv_json(&mut self) -> Result<Option<Value>> {
        Ok(match self.recv_message()? {
            OwnedMessage::Binary(bytes) => {
                serde_json::from_reader(ZlibDecoder::new(&bytes[..]))
                    .map(Some)
                    .map_err(|why| {
                        warn!("Err deserializing bytes: {:?}; bytes: {:?}", why, bytes);

                        why
                    })?
            },
            OwnedMessage::Close(data) => return Err(Error::Gateway(GatewayError::Closed(data))),
            OwnedMessage::Text(payload) => {
                serde_json::from_str(&payload).map(Some).map_err(|why| {
                    warn!(
                        "Err deserializing text: {:?}; text: {}",
                        why,
                        payload,
                    );

                    why
                })?
            },
            OwnedMessage::Ping(x) => {
                self.send_message(&OwnedMessage::Pong(x))
                    .map_err(Error::from)?;

                None
            },
            OwnedMessage::Pong(_) => None,
        })
    }
}

impl SenderExt for WsClient<TlsStream<TcpStream>> {
    fn send_json(&mut self, value: &Value) -> Result<()> {
        serde_json::to_string(value)
            .map(OwnedMessage::Text)
            .map_err(Error::from)
            .and_then(|m| self.send_message(&m).map_err(Error::from))
    }
}
