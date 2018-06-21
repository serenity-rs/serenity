use futures::{future, Future, Stream, Poll, Sink, StartSend, AsyncSink};
use ::Error;
use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};
use gateway::{
    shard::Shard,
    queue::ReconnectQueue,
};
use model::event::{Event, GatewayEvent};
use parking_lot::Mutex;
use tokio::{self, timer::{Delay, Interval}};
use futures::sync::mpsc::{
    unbounded, UnboundedSender, UnboundedReceiver, 
    channel, Sender as MpscSender, Receiver as MpscReceiver,
    SendError,
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
pub struct ShardManagerOptions<T: ReconnectQueue> {
    pub strategy: ShardingStrategy,
    pub token: String,
    pub ws_uri: Rc<String>,
    pub queue: T,
}

pub type WrappedShard = Arc<Mutex<Shard>>;//Rc<RefCell<Shard>>;
pub type Message = (WrappedShard, TungsteniteMessage);
pub type MessageStream = UnboundedReceiver<Message>;
type ShardsMap = Rc<RefCell<HashMap<u64, WrappedShard>>>;

pub struct ShardManager<T: ReconnectQueue> {
    pub queue: VecDeque<u64>,
    reconnect_queue: T,
    shards: ShardsMap,
    pub strategy: ShardingStrategy,
    pub token: String,
    pub ws_uri: Rc<String>,
    message_stream: Option<MessageStream>,
    queue_sender: MpscSender<u64>,
    queue_receiver: Option<MpscReceiver<u64>>,
    #[allow(dead_code)]
    non_exhaustive: (),
}

impl<T: ReconnectQueue> ShardManager<T> {
    pub fn new(options: ShardManagerOptions<T>) -> Self {
        let (queue_sender, queue_receiver) = channel(0);

        Self {
            queue: VecDeque::new(),
            reconnect_queue: options.queue,
            shards: Rc::new(RefCell::new(HashMap::new())),
            strategy: options.strategy,
            token: options.token,
            ws_uri: options.ws_uri,
            message_stream: None,
            queue_sender,
            queue_receiver: Some(queue_receiver),
            non_exhaustive: (),
        }
    }

    pub fn start(&mut self) -> impl Future<Item = (), Error = Error> {
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
            trace!("pushing shard id {} to back of queue", &shard_id);

            let future = self.reconnect_queue.push_back(shard_id)
                .map_err(|_| error!("Error pushing shard to reconnect queue"));

            tokio::spawn(future);
        }
        
        let mut queue_sender = self.queue_sender.clone();
        let queue_receiver = self.queue_receiver.take().unwrap();
        let token = self.token.clone();
        let sender_1 = sender.clone();
        let shards = self.shards.clone();

        let future = self.reconnect_queue.pop_front()
            .and_then(move |shard_id| {
                let shard_id = shard_id.expect("shard start queue is empty");

                queue_sender.try_send(shard_id)
                    .expect("could not send first shard to start");
                
                future::ok(())
            })
            .map_err(|_| error!("error popping front of reconnect queue"))
            .and_then(move |_| {
                process_queue(
                    queue_receiver,
                    token,
                    shards_total,
                    sender_1,
                    shards,
                )
            });

        tokio::spawn(future);

        Box::new(future::ok(()))
    }

    pub fn messages(&mut self) -> MessageStream {
        self.message_stream.take().unwrap() 
    }

    pub fn process(&mut self, event: &GatewayEvent) {
        if let &GatewayEvent::Dispatch(_, Event::Ready(ref event)) = event {
            let shard_id = match event.ready.shard {
                Some(shard) => shard[0],
                None => {
                    error!("ready event has no shard id");
                    return;
                }
            };

            println!("shard id {} has started", &shard_id);

            let mut queue_sender = self.queue_sender.clone();
            let future = self.reconnect_queue.pop_front()
                .and_then(move |shard_id| {
                    if let Some(next_shard_id) = shard_id {
                        if let Err(e) = queue_sender.try_send(next_shard_id) {
                            error!("could not send shard id to queue mpsc receiver: {:?}", e);
                        }
                    }

                    future::ok(())
                })
                .map_err(|_| error!("error popping front of reconnect queue"));

            tokio::spawn(future);
        }
    }
}

fn process_queue(
    queue_receiver: MpscReceiver<u64>,
    token: String,
    shards_total: u64,
    sender: UnboundedSender<Message>,
    shards_map: ShardsMap,
) -> impl Future<Item = (), Error = ()> {
    queue_receiver
        .for_each(move |shard_id| {
            trace!("received message to start shard {}", &shard_id);
            let token = token.clone();
            let sender = sender.clone();
            let shards_map = shards_map.clone();
            let sleep_future = Delay::new(Instant::now() + Duration::from_secs(6));

            sleep_future
                .map_err(|e| error!("Error sleeping before starting next shard: {:?}", e))
                .and_then(move |_| {
                    let future = start_shard(token, shard_id, shards_total,
                     sender)
                        .map(move |shard| {
                            shards_map.borrow_mut().insert(shard_id.clone(), shard);
                        });

                    tokio::spawn(future);
                    future::ok(())
                })
        })
        .map_err(|_| ())
}

fn start_shard(
    token: String, 
    shard_id: u64, 
    shards_total: u64, 
    sender: UnboundedSender<Message>,
) -> Box<Future<Item = WrappedShard, Error = ()>> {
    Box::new(Shard::new(token, [shard_id, shards_total])
        .then(move |result| {
            let shard = match result {
                Ok(shard) => Arc::new(Mutex::new(shard)),
                Err(e) => {
                    return future::err(Error::from(e));
                },
             };

            let sink = MessageSink {
                shard,
                sender,
            };

            let messages = {
                let shard_lock = shard.clone();
                let shard = shard_lock.lock();

                shard.messages()
            };

            let future = messages
                .map_err(MessageSinkError::from)
                .forward(sink)
                .map(|_| ())
                .map_err(|e| error!("Error forwarding shard messages to sink: {:?}", e));

            tokio::spawn(future);
            future::ok(shard)
        })
        .map_err(|e| error!("Error starting shard: {:?}", e)))
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
