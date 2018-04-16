use futures::{future, Future};
use std::collections::VecDeque;

pub trait ReconnectQueue {
    type Error: 'static;

    fn push_back(&mut self, shard_id: u64) -> Box<Future<Item = (), Error = Self::Error>>;

    fn pop_front(&mut self) -> Box<Future<Item = Option<u64>, Error = Self::Error>>;
}

pub struct SimpleReconnectQueue {
    queue: VecDeque<u64>,
}

impl SimpleReconnectQueue {
    pub fn new(shard_total: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(shard_total),
        }
    }
}

impl ReconnectQueue for SimpleReconnectQueue {
    type Error = ();

    fn push_back(&mut self, shard_id: u64) -> Box<Future<Item = (), Error = Self::Error>> {
        self.queue.push_back(shard_id);
        Box::new(future::ok(()))
    } 

    fn pop_front(&mut self) -> Box<Future<Item = Option<u64>, Error = Self::Error>> {
        Box::new(future::ok(self.queue.pop_front()))
    }
}
