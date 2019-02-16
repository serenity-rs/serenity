use flate2::read::ZlibDecoder;
use crate::gateway::WsClient;
use crate::internal::prelude::*;
use serde_json;
use tungstenite::{
    util::NonBlockingResult,
    Message,
};
use log::warn;

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
