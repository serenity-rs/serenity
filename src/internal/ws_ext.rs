use flate2::read::ZlibDecoder;
use future_utils::mpsc::UnboundedSender;
use futures::{
    future::{
        result,
        ok,
    },
    sync::mpsc::Sender as FutureMpscSender,
    Future,
    Poll,
    IntoFuture,
    Sink,
    Stream,
};
use internal::prelude::*;
use serde_json;
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
    fn recv_json(self) -> Box<Future<Item = (Option<Value>, Self), Error = (Error, Self)>>;
}

pub trait SenderExt {
    fn send_json(self, value: &Value) -> Box<Future<Item = Self, Error = Error>>;
}

pub enum Either4<A, B, C, D> {
    One(A),
    Two(B),
    Three(C),
    Four(D),
}

impl<A, B, C, D> Future for Either4<A, B, C, D>
    where A: Future,
        B: Future<Item = A::Item, Error = A::Error>,
        C: Future<Item = A::Item, Error = A::Error>,
        D: Future<Item = A::Item, Error = A::Error> {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<A::Item, A::Error> {
        match *self {
            Either4::One(ref mut a) => a.poll(),
            Either4::Two(ref mut b) => b.poll(),
            Either4::Three(ref mut c) => c.poll(),
            Either4::Four(ref mut d) => d.poll(),
        }
    }
}

impl ReceiverExt for WsClient {
    fn recv_json(self) -> Box<Future<Item = (Option<Value>, WsClient), Error = (Error, WsClient)>> {
        let out = self.into_future()
            .map_err(|(err, ws)| (err.into(), ws))
            .and_then(|(value, ws)| match value {
                Some(message) => Ok((message, ws)),
                None => Err((Error::Tungstenite(TungsteniteError::ConnectionClosed(None)), ws))
            })
            .and_then(|(message, ws)| {
                match message {
                    Message::Binary(bytes) => {
                        let done = result(serde_json::from_reader(ZlibDecoder::new(&bytes[..])).map(Some))
                            .map_err(move |err| (err.into(), ws))
                            .map(move |val| (val, ws));

                        Either4::One(done)
                    },
                    Message::Text(payload) => {
                        let done = result(serde_json::from_str(&payload).map(Some))
                            .map_err(move |err| (err.into(), ws))
                            .map(move |val| (val, ws));

                        Either4::Two(done)
                    },
                    Message::Ping(x) => {
                        let done = ws.send(Message::Pong(x))
                            .map_err(move |err| (err.into(), ws))
                            .map(|ws| (None, ws));

                        Either4::Three(done)
                    },
                    Message::Pong(_) => Either4::Four(ok((None, ws))),
                }
            })
            .map_err(|(err, ws)| (err.into(), ws));

        Box::new(out)
    }
}

pub fn message_to_json(message: Message, notifier: UnboundedSender<Vec<u8>>) -> Result<Option<Value>> {
    // This is like the above, except in the case where the sender and receiver have been split.
    // It doesn't seem like Stream + Sink allows .shared() to be called, so here we are...
    // Telling the holder of the send side that they're obliged to Pong.
    match message {
        Message::Binary(bytes) => serde_json::from_reader(ZlibDecoder::new(&bytes[..])).map(Some).map_err(Error::from),
        Message::Text(payload) => serde_json::from_str(&payload).map(Some).map_err(Error::from),
        Message::Ping(x) => {
            notifier.unbounded_send(x);

            Ok(None)
        },
        Message::Pong(_) => Ok(None),
    }
}

impl SenderExt for WsClient {
    fn send_json(self, value: &Value) -> Box<Future<Item = Self, Error = Error>> {
        let text = serde_json::to_string(value)
            .map(Message::Text)
            .map_err(Error::from);

        let out = result(text)
            .and_then(|data| self.send(data).map_err(Error::from));

        Box::new(out)
    }
}