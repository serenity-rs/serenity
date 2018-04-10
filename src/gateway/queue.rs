use futures::{
    future, 
    Future, 
    Stream,
};
use futures::sync::mpsc::{
        channel, 
        Sender as MpscSender, 
        Receiver as MpscReceiver, 
        SendError,
        TrySendError,
};

pub trait ReconnectQueue {
    type Error;

    fn push_back(&mut self, shard_id: u64) -> Box<Future<Item = (), Error = Self::Error>>;

    fn stream(&mut self) -> Box<Future<Item = u64, Error = ()>>;
}

use std::collections::VecDeque;

type Sender = MpscSender<u64>;
type Receiver = MpscSender<u64>;

struct ShardReconnectQueue {
    queue: VecDeque<u64>,
    pub sender: Sender,
    receiver: Option<Receiver>,
}

impl ShardReconnectQueue {
    pub fn new() -> Self {
        let (sender, receiver) = channel(0);

        Self {
            sender,
            recever: Some(receiver),
            .. Default::default()
        }
    }
}

impl ReconnectQueue for ShardReconnectQueue {
    type Error = ();

    fn push_back(&mut self, shard_id: u64) -> Box<Future<Item = (), Error = Self::Error>> {
        self.queue.push_back(shard_id);
        Box::new(future::ok(()))
    } 

    fn stream(&mut self) -> Box<Future<Item = u64, Error = ()>> {
        Box::new(self.receiver.take().unwrap())
    }
}
