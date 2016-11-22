use flate2::read::ZlibDecoder;
use serde_json;
use websocket::client::{Receiver, Sender};
use websocket::message::{Message as WsMessage, Type as WsType};
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WsReceiver;
use websocket::ws::sender::Sender as WsSender;
use ::client::gateway::GatewayError;
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
        let message: WsMessage = try!(self.recv_message());

        if message.opcode == WsType::Close {
            let representation = String::from_utf8_lossy(&message.payload)
                .into_owned();

            Err(Error::Gateway(GatewayError::Closed(message.cd_status_code,
                                                    representation)))
        } else if message.opcode == WsType::Binary || message.opcode == WsType::Text {
            let json: Value = if message.opcode == WsType::Binary {
                try!(serde_json::from_reader(ZlibDecoder::new(&message.payload[..])))
            } else {
                try!(serde_json::from_reader(&message.payload[..]))
            };

            decode(json).map_err(|err| {
                warn!("Error decoding: {}",
                      String::from_utf8_lossy(&message.payload));

                err
            })
        } else {
            let representation = String::from_utf8_lossy(&message.payload)
                .into_owned();

            Err(Error::Gateway(GatewayError::Closed(None, representation)))
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
