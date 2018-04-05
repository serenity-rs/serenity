use futures::{future, Future, Stream, Poll, Sink, StartSend, AsyncSink};
use ::Error;
use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use gateway::shard::Shard;
use tokio_core::reactor::Handle;
use futures::sync::mpsc::{
    unbounded, UnboundedSender, UnboundedReceiver, SendError
};
use tungstenite::{Message, Error as TungsteniteError};

#[derive(Clone, Copy, Debug)]
pub enum ShardingStrategy {
    Autoshard,
    Range(u64, u64, u64),
}

impl ShardingStrategy {
    pub fn auto() -> Self {
        ShardingStrategy::Autoshard
    }

    pub fn multi(count: u64) -> Self {
        ShardingStrategy::Range(0, count, count)
    }

    pub fn simple() -> Self {
        ShardingStrategy::Range(0, 1, 1)
    }

    pub fn range(index: u64, count: u64, total: u64) -> Self {
        ShardingStrategy::Range(index, count, total)
    }
}

impl Default for ShardingStrategy {
    fn default() -> Self {
        ShardingStrategy::Autoshard
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShardManagerOptions {
    pub strategy: ShardingStrategy,
    pub token: Rc<String>,
    pub ws_uri: Rc<String>,
}

pub type MessageStream = UnboundedReceiver<(Rc<RefCell<Shard>>, Message)>;

pub struct ShardManager {
    pub queue: VecDeque<u64>,
    pub shards: Rc<RefCell<HashMap<u64, Rc<RefCell<Shard>>>>>,
    pub strategy: ShardingStrategy,
    pub token: Rc<String>,
    pub ws_uri: Rc<String>,
    non_exhaustive: (),
    handle: Handle,
    message_stream: Option<MessageStream>,
}

impl ShardManager {
    pub fn new(options: ShardManagerOptions, handle: Handle) -> Self {
        Self {
            queue: VecDeque::new(),
            shards: Rc::new(RefCell::new(HashMap::new())),
            strategy: options.strategy,
            token: options.token,
            ws_uri: options.ws_uri,
            non_exhaustive: (),
            handle,
            message_stream: None,
        }
    }

    pub fn start(&mut self) -> Box<Future<Item = (), Error = Error>> {
        let (
            shards_index, 
            shards_count, 
            shards_total
        ) = match self.strategy {
            ShardingStrategy::Autoshard => unimplemented!(),
            ShardingStrategy::Range(i, c, t) => (i, c, t),
        };

        let (sender, receiver) = unbounded();
        self.message_stream = Some(receiver);

        for shard_id in shards_index..shards_count {
            let shards_map = self.shards.clone();
            let sender = sender.clone();
            let handle = self.handle.clone();

            let future: Box<Future<Item = (), Error = ()>> = Box::new(Shard::new(self.token.clone(), [shard_id, shards_total], handle.clone()) 
                .then(move |result| {
                    let shard = match result {
                        Ok(shard) => Rc::new(RefCell::new(shard)),
                        Err(e) => {
                            return future::err(Error::from(e));
                        },
                    };

                    let sink = MessageSink {
                        shard: shard.clone(), 
                        sender,
                    };

                    let future: Box<Future<Item = (), Error = ()>> = Box::new(shard.borrow_mut()
                        .messages()
                        .map_err(MessageSinkError::from)
                        .forward(sink)
                        .map(|_| ())
                        .map_err(|e: MessageSinkError| error!("Error forwarding shard messages to sink: {:?}", e)));

                    handle.spawn(future);
                    shards_map.borrow_mut().insert(shard_id, shard);
                    future::ok(())
                })
                .map_err(|e: Error| error!("Error starting shard: {:?}", e)));

            self.handle.spawn(future);
        }

        Box::new(future::ok(()))
    }

    // takes the message stream
    // panics if already taken
    pub fn messages(&mut self) -> MessageStream {
        self.message_stream.take().unwrap() 
    }
}

pub enum MessageSinkError {
    MpscSend(SendError<(Rc<RefCell<Shard>>, Message)>),
    Tungstenite(TungsteniteError),
}

impl From<SendError<(Rc<RefCell<Shard>>, Message)>> for MessageSinkError {
    fn from(e: SendError<(Rc<RefCell<Shard>>, Message)>) -> Self {
        MessageSinkError::MpscSend(e)
    }
}

impl From<TungsteniteError> for MessageSinkError {
    fn from(e: TungsteniteError) -> Self {
        MessageSinkError::Tungstenite(e)
    }
}

impl ::std::fmt::Debug for MessageSinkError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use std::error::Error;

        write!(f, "{}", match *self {
            MessageSinkError::MpscSend(ref err) => err.description(),
            MessageSinkError::Tungstenite(ref err) => err.description(),
        })
    }
}

struct MessageSink {
    shard: Rc<RefCell<Shard>>,
    sender: UnboundedSender<(Rc<RefCell<Shard>>, Message)>,
}

impl Sink for MessageSink {
    type SinkItem = Message;
    type SinkError = MessageSinkError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(match self.sender.start_send((self.shard.clone(), item))? {
            AsyncSink::NotReady((_, item)) => AsyncSink::NotReady(item),
            AsyncSink::Ready => AsyncSink::Ready,
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.sender.poll_complete()
            .map_err(From::from)
    }
}
