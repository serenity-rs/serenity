use serde_json::Value;
use websocket::client::Sender;
use websocket::stream::WebSocketStream;

#[doc(hidden)]
pub enum Status {
    SendMessage(Value),
    Sequence(u64),
    ChangeInterval(u64),
    ChangeSender(Sender<WebSocketStream>),
}
