use flate2::read::ZlibDecoder;
use serde_json;
use websocket::client::{Receiver, Sender};
use websocket::message::{Message as WsMessage, Type as WsType};
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WsReceiver;
use websocket::ws::sender::Sender as WsSender;
use ::gateway::GatewayError;
use ::internal::prelude::*;

pub trait ReceiverExt {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T>
        where F: FnOnce(Value) -> Result<T>;
}

pub trait SenderExt {
    fn send_json(&mut self, value: &Value) -> Result<()>;
}

impl ReceiverExt for Receiver<WebSocketStream> {
    fn recv_json<F, T>(&mut self, decode: F) -> Result<T> where F: FnOnce(Value) -> Result<T> {
        let message: WsMessage = self.recv_message()?;

        if message.opcode == WsType::Close {
            let r = String::from_utf8_lossy(&message.payload).into_owned();

            Err(Error::Gateway(GatewayError::Closed(message.cd_status_code, r)))
        } else if message.opcode == WsType::Binary || message.opcode == WsType::Text {
            let json: Value = if message.opcode == WsType::Binary {
                serde_json::from_reader(ZlibDecoder::new(&message.payload[..]))?
            } else {
                serde_json::from_reader(&message.payload[..])?
            };

            match decode(json) {
                Ok(v) => Ok(v),
                Err(why) => {
                    let s = String::from_utf8_lossy(&message.payload);

                    warn!("(╯°□°）╯︵ ┻━┻ Error decoding: {}", s);

                    Err(why)
                }
            }
        } else {
            let r = String::from_utf8_lossy(&message.payload).into_owned();

            Err(Error::Gateway(GatewayError::Closed(None, r)))
        }
    }
}

impl SenderExt for Sender<WebSocketStream> {
    fn send_json(&mut self, value: &Value) -> Result<()> {
        serde_json::to_string(value)
            .map(WsMessage::text)
            .map_err(Error::from)
            .and_then(|m| self.send_message(&m).map_err(Error::from))
    }
}
