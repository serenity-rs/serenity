use serde_json::Value;
use websocket::client::Sender;
use websocket::stream::WebSocketStream;

#[doc(hidden)]
pub enum Status {
    Interval(u64),
    Sender(Sender<WebSocketStream>),
    SendMessage(Value),
    Sequence(u64),
}
