use flate2::read::ZlibDecoder;
use gateway::WsClient;
use internal::prelude::*;
use serde_json;
use tungstenite::Message;

pub trait ReceiverExt {
    fn recv_json(&mut self) -> Result<Option<Value>>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for WsClient {
    fn recv_json(&mut self) -> Result<Option<Value>> {
        Ok(match self.read_message()? {
            Message::Binary(bytes) => {
                serde_json::from_reader(ZlibDecoder::new(&bytes[..]))
                    .map(Some)
                    .map_err(|why| {
                        warn!("Err deserializing bytes: {:?}; bytes: {:?}", why, bytes);

                        why
                    })?
            },
            Message::Text(payload) => {
                serde_json::from_str(&payload).map(Some).map_err(|why| {
                    warn!(
                        "Err deserializing text: {:?}; text: {}",
                        why,
                        payload,
                    );

                    why
                })?
            },
            Message::Ping(x) => {
                self.write_message(Message::Pong(x)).map_err(Error::from)?;

                None
            },
            Message::Pong(_) => None,
        })
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
