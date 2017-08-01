use flate2::read::ZlibDecoder;
use serde_json;
use websocket::message::OwnedMessage;
use websocket::sync::stream::{TcpStream, TlsStream};
use websocket::sync::Client as WsClient;
use gateway::GatewayError;
use internal::prelude::*;

pub trait ReceiverExt {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T> where F: Fn(Value) -> Result<T>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for WsClient<TlsStream<TcpStream>> {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T>
        where F: Fn(Value) -> Result<T> {
        let message = self.recv_message()?;

        let res = match message {
            OwnedMessage::Binary(bytes) => {
                let value = serde_json::from_reader(ZlibDecoder::new(&bytes[..]))?;

                Some(decode(value).map_err(|why| {
                    let s = String::from_utf8_lossy(&bytes);

                    warn!("(╯°□°）╯︵ ┻━┻ Error decoding: {}", s);

                    why
                }))
            },
            OwnedMessage::Close(data) => Some(Err(Error::Gateway(GatewayError::Closed(data)))),
            OwnedMessage::Text(payload) => {
                let value = serde_json::from_str(&payload)?;

                Some(decode(value).map_err(|why| {
                    warn!("(╯°□°）╯︵ ┻━┻ Error decoding: {}", payload);

                    why
                }))
            },
            OwnedMessage::Ping(x) => {
                self.send_message(&OwnedMessage::Pong(x))
                    .map_err(Error::from)?;

                None
            },
            OwnedMessage::Pong(_) => None,
        };

        // As to ignore the `None`s returned from `Ping` and `Pong`.
        // Since they're essentially useless to us anyway.
        match res {
            Some(data) => data,
            None => self.recv_json(decode),
        }
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
