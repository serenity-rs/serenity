use futures::{future, Future, Stream, Poll, Sink, StartSend, AsyncSink};
use ::Error;
use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use gateway::shard::Shard;
use model::event::{Event, GatewayEvent};
use tokio_core::reactor::Handle;
use futures::sync::mpsc::{
    unbounded, UnboundedSender, UnboundedReceiver, SendError
};
use tungstenite::{Message as TungsteniteMessage, Error as TungsteniteError};

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

pub type WrappedShard = Rc<RefCell<Shard>>;
pub type Message = (WrappedShard, TungsteniteMessage);
pub type MessageStream = UnboundedReceiver<Message>;
type ShardsMap = Rc<RefCell<HashMap<u64, WrappedShard>>>;

pub struct ShardManager {
    pub queue: VecDeque<u64>,
    shards: ShardsMap,
    pub strategy: ShardingStrategy,
    pub token: Rc<String>,
    pub ws_uri: Rc<String>,
    handle: Handle,
    message_stream: Option<MessageStream>,
    #[allow(dead_code)]
    non_exhaustive: (),
}

impl ShardManager {
    pub fn new(options: ShardManagerOptions, handle: Handle) -> Self {
        Self {
            queue: VecDeque::new(),
            shards: Rc::new(RefCell::new(HashMap::new())),
            strategy: options.strategy,
            token: options.token,
            ws_uri: options.ws_uri,
            handle,
            message_stream: None,
            non_exhaustive: (),
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
            let future = start_shard(
                self.token.clone(),
                shard_id,
                shards_total,
                self.handle.clone(),
                self.shards.clone(),
                sender.clone(),
            );

            self.handle.spawn(future);
        }

        Box::new(future::ok(()))
    }

    pub fn messages(&mut self) -> MessageStream {
        self.message_stream.take().unwrap() 
    }

    pub fn process(&mut self, event: &GatewayEvent) {
        if let GatewayEvent::Dispatch(_, Event::Ready(event)) = event {
            let shard_id = match &event.ready.shard {
                Some(shard) => shard[0],
                None => {
                    error!("ready event has no shard id");
                    return;
                }
            };

            println!("shard id {} has started", shard_id);
        }
    }
}

fn start_shard(
    token: Rc<String>, 
    shard_id: u64, 
    shards_total: u64, 
    handle: Handle, 
    shards_map: ShardsMap,
    sender: UnboundedSender<Message>,
) -> impl Future<Item = (), Error = ()> {
    Shard::new(token, [shard_id, shards_total], handle.clone())
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

            let future = Box::new(shard.borrow_mut()
                .messages()
                .map_err(MessageSinkError::from)
                .forward(sink)
                .map(|_| ())
                .map_err(|e| error!("Error forwarding shard messages to sink: {:?}", e)));

            handle.spawn(future);
            shards_map.borrow_mut().insert(shard_id, shard);
            future::ok(())
        })
        .map_err(|e| error!("Error starting shard: {:?}", e))
}

pub enum MessageSinkError {
    MpscSend(SendError<Message>),
    Tungstenite(TungsteniteError),
}

impl From<SendError<Message>> for MessageSinkError {
    fn from(e: SendError<Message>) -> Self {
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
    shard: WrappedShard,
    sender: UnboundedSender<Message>,
}

impl Sink for MessageSink {
    type SinkItem = TungsteniteMessage;
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
