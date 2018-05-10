use flate2::read::ZlibDecoder;
use futures::{
    future::{
        result,
        ok,
    },
    Future,
    Sink,
    Stream,
};
use internal::{
    either_n::Either4,
    prelude::*,
};
use parking_lot::Mutex;
use serde_json;
use std::sync::{mpsc::Sender, Arc};
use tokio_core::net::TcpStream;
use tokio_tls::TlsStream;
use tokio_tungstenite::{
    stream::Stream as StreamSwitcher,
    WebSocketStream,
};
use tungstenite::{
    error::Error as TungsteniteError,
    Message
};

pub type WsClient = WebSocketStream<StreamSwitcher<TcpStream, TlsStream<TcpStream>>>;

pub trait ReceiverExt {
    fn recv_json(self) -> Box<Future<Item = (Option<Value>, Self), Error = Error>>;
}

pub trait SenderExt {
    fn send_json(self, value: &Value) -> Box<Future<Item = Self, Error = Error>>;
}

impl ReceiverExt for WsClient {
    fn recv_json(self) -> Box<Future<Item = (Option<Value>, WsClient), Error = Error>> {
        let out = self.into_future()
            .map_err(|(e, _)| e.into())
            .and_then(|(value, ws)| match value {
                Some(message) => Ok((message, ws)),
                None => Err(Error::Tungstenite(TungsteniteError::ConnectionClosed(None)))
            })
            .and_then(|(message, ws)| {
                match message {
                    Message::Binary(bytes) => {
                        let done = result(serde_json::from_reader(ZlibDecoder::new(&bytes[..])).map(Some))
                            .map_err(Error::from)
                            .map(move |val| (val, ws));

                        Either4::One(done)
                    },
                    Message::Text(payload) => {
                        let done = result(serde_json::from_str(&payload).map(Some))
                            .map_err(Error::from)
                            .map(move |val| (val, ws));

                        Either4::Two(done)
                    },
                    Message::Ping(x) => {
                        let done = ws.send(Message::Pong(x))
                            .map_err(Error::from)
                            .map(|ws| (None, ws));

                        Either4::Three(done)
                    },
                    Message::Pong(_) => Either4::Four(ok((None, ws))),
                }
            });

        Box::new(out)
    }
}

pub fn message_to_json(message: Message, notifier_lock: Arc<Mutex<Sender<Vec<u8>>>>) -> Result<Option<Value>> {
    // This is like the above, except in the case where the sender and receiver have been split.
    // It doesn't seem like Stream + Sink allows .shared() to be called, so here we are...
    // Telling the holder of the send side that they're obliged to Pong.
    match message {
        Message::Binary(bytes) => serde_json::from_reader(ZlibDecoder::new(&bytes[..])).map(Some).map_err(Error::from),
        Message::Text(payload) => serde_json::from_str(&payload).map(Some).map_err(Error::from),
        Message::Ping(x) => {
            let notifier = notifier_lock.lock();
            notifier.send(x);

            Ok(None)
        },
        Message::Pong(_) => Ok(None),
    }
}

impl<T: 'static> SenderExt for T 
        where T: Sink<SinkItem = Message, SinkError = TungsteniteError> {
    fn send_json(self, value: &Value) -> Box<Future<Item = Self, Error = Error>> {
        let text = serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from);

        let out = result(text)
            .and_then(|data| self.send(data).map_err(Error::from));

        Box::new(out)
    }
}