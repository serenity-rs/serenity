use std::{
    sync::Arc,
    time::Duration,
    future::Future,
    pin::Pin,
    task::{Context as FutContext, Poll},
};
use tokio::sync::mpsc::{
    unbounded_channel,
    UnboundedReceiver as Receiver,
    UnboundedSender as Sender,
};
use tokio::{
    sync::Mutex,
    time::timeout,
};
use futures::future::BoxFuture;

use crate::client::bridge::gateway::ShardMessenger;
use crate::model::channel::Message;

type ForeachFunction = for<'fut> fn(&'fut Arc<Message>) -> BoxFuture<'fut, bool>;
type Shard = Arc<Mutex<ShardMessenger>>;

macro_rules! impl_message_collector {
    ($($name:ident;)*) => {
        $(
            impl<'a> $name<'a> {
                /// Limits how many messages will attempt to be filtered.
                ///
                /// The filter checks whether the message has been sent
                /// in the right guild, channel, and by the right author.
                pub fn filter_limit(mut self, limit: u32) -> Self {
                    self.filter.as_mut().unwrap().filter_limit = Some(limit);

                    self
                }

                /// Sets a filter function where messages passed to the `function` must
                /// return `true`, otherwise the message won't be collected and failed the filter
                /// process.
                /// This is the last instance to pass for a message to count as *collected*.
                ///
                /// This function is intended to be a message content filter.
                pub fn filter<F: Fn(&Arc<Message>) -> bool + 'static + Send + Sync>(mut self, function: F) -> Self {
                    self.filter.as_mut().unwrap().filter = Some(Arc::new(function));

                    self
                }

                /// Sets a duration the collector shall await messages.
                /// Once the timeout is reached, the collector will *close* and be unable
                /// to receive any new messages, already received messages will be yielded.
                pub fn timeout(mut self, timeout: Duration) -> Self {
                    self.collector.as_mut().unwrap().timeout = Some(timeout);

                    self
                }

                /// Sets the required author ID of a message.
                /// If a message does not meet this ID, it won't be received.
                pub fn author_id(mut self, author_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().author_id = Some(author_id.into());

                    self
                }

                /// Sets the required channel ID of a message.
                /// If a message does not meet this ID, it won't be received.
                pub fn channel_id(mut self, channel_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().channel_id = Some(channel_id.into());

                    self
                }

                /// Sets the required guild ID of a message.
                /// If a message does not meet this ID, it won't be received.
                pub fn guild_id(mut self, guild_id: impl Into<u64>) -> Self {
                    self.filter.as_mut().unwrap().guild_id = Some(guild_id.into());

                    self
                }

                /// Performs a function whenever a message is received,
                /// if the function returns true, the collector will continue receiving
                /// more messages, if it returns false, it will stop receiving.
                pub fn for_each_received(mut self, function: ForeachFunction) -> Self {
                    self.collector.as_mut().unwrap().for_each = Some(Arc::new(function));

                    self
                }
            }
        )*
    }
}

/// Filters events on the shard's end and sends them to the collector.
#[derive(Clone, Debug)]
pub struct MessageFilter {
    filtered: u32,
    collected: u32,
    options: FilterOptions,
    sender: Sender<Arc<Message>>,
}

impl MessageFilter {
    /// Creates a new filter
    fn new(options: FilterOptions) -> (Self, Receiver<Arc<Message>>) {
        let (sender, receiver) = unbounded_channel();

        let filter = Self {
            filtered: 0,
            collected: 0,
            sender,
            options,
        };

        (filter, receiver)
    }

    /// Sends a `message` to the consuming collector if the `message` conforms
    /// to the constraints and the limits are not reached yet.
    pub(crate) fn send_message(&mut self, message: &Arc<Message>) -> bool {
        if self.is_passing_constraints(&message) {

            if self.options.filter.as_ref().map_or(true, |f| f(&message)) {
                self.collected += 1;

                if let Err(_) = self.sender.send(Arc::clone(message)) {
                    return false;
                }
            }
        }

        self.filtered += 1;

        self.is_within_limits()
    }

    /// Checks if the `message` passes set constraints.
    /// Constraints are optional, as it is possible to limit messages to
    /// be sent by a specific author or in a specifc guild.
    fn is_passing_constraints(&self, message: &Arc<Message>) -> bool {
        self.options.guild_id.map_or(true, |g| { Some(g) == message.guild_id.map(|g| g.0) })
        && self.options.channel_id.map_or(true, |g| { g == message.channel_id.0 })
        && self.options.author_id.map_or(true, |g| { g == message.author.id.0 })
    }

    /// Checks if the filter is within set receive and collect limits.
    /// A message is considered *received* even when it does not meet the
    /// constraints.
    fn is_within_limits(&self) -> bool {
        self.options.filter_limit.as_ref().map_or(true, |limit| { self.filtered < *limit })
        && self.options.collect_limit.as_ref().map_or(true, |limit| { self.collected < *limit })
    }
}

#[derive(Default)]
struct CollectorOptions {
    for_each: Option<Arc<ForeachFunction>>,
    timeout: Option<Duration>,
}

#[derive(Clone, Default)]
struct FilterOptions {
    filter_limit: Option<u32>,
    collect_limit: Option<u32>,
    filter: Option<Arc<dyn Fn(&Arc<Message>) -> bool + 'static + Send + Sync>>,
    channel_id: Option<u64>,
    guild_id: Option<u64>,
    author_id: Option<u64>,
}

// Implement the common setters for all message collector types.
// This avoids using a trait that the user would need to import in
// order to use any of these methods.
impl_message_collector! {
    CollectOneReply;
    CollectNReplies;
    CollectAllReplies;
    MessageCollectorBuilder;
}

pub struct MessageCollectorBuilder<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, MessageCollector>>,
}

impl<'a> MessageCollectorBuilder<'a> {
    /// A future that builds a [`MessageCollector`] based on the settings.
    ///
    /// [`MessageCollector`]: ../struct.MessageCollector.html
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }

    /// Limits how many messages can be collected.
    ///
    /// A message is considered *collected*, if the message
    /// passes all the requirements.
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().collect_limit = Some(limit);

        self
    }
}

impl<'a> Future for MessageCollectorBuilder<'a> {
    type Output = MessageCollector;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = MessageFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_message_filter(filter);

                MessageCollector {
                    received: 0,
                    receiver,
                    timeout,
                    for_each,
                }
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

pub struct CollectOneReply<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, Option<Arc<Message>>>>,
}

impl<'a> CollectOneReply<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }
}

impl<'a> Future for CollectOneReply<'a> {
    type Output = Option<Arc<Message>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = MessageFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_message_filter(filter);

                MessageCollector {
                    received: 0,
                    receiver,
                    timeout,
                    for_each,
                }.receive_one().await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

pub struct CollectNReplies<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard_messenger: Option<Shard>,
    fut: Option<BoxFuture<'a, Vec<Arc<Message>>>>,
}

impl<'a> CollectNReplies<'a> {
    /// A future that will collect as many messages as wanted.
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard_messenger: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }

    /// Limits how many messages can be collected.
    ///
    /// A message is considered *collected*, if the message
    /// passes all the requirements.
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().collect_limit = Some(limit);

        self
    }
}

impl<'a> Future for CollectNReplies<'a> {
    type Output = Vec<Arc<Message>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard_messenger.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let number = self.filter.as_ref().unwrap().collect_limit.unwrap_or(1);
            let (filter, receiver) = MessageFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_message_filter(filter);

                MessageCollector {
                    received: 0,
                    receiver,
                    timeout,
                    for_each,
                }.receive_n(number).await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

/// Future to collect all replies.
pub struct CollectAllReplies<'a> {
    filter: Option<FilterOptions>,
    collector: Option<CollectorOptions>,
    shard: Option<Shard>,
    fut: Option<BoxFuture<'a, Vec<Arc<Message>>>>,
}

impl<'a> CollectAllReplies<'a> {
    pub fn new(shard_messenger: impl AsRef<Shard>) -> Self {
        Self {
            filter: Some(FilterOptions::default()),
            collector: Some(CollectorOptions::default()),
            shard: Some(Arc::clone(shard_messenger.as_ref())),
            fut: None,
        }
    }

    /// Limits how many messages can be collected.
    ///
    /// A message is considered *collected*, if the message
    /// passes all the requirements.
    pub fn collect_limit(mut self, limit: u32) -> Self {
        self.filter.as_mut().unwrap().collect_limit = Some(limit);

        self
    }
}

impl<'a> Future for CollectAllReplies<'a> {
    type Output = Vec<Arc<Message>>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let shard_messenger = self.shard.take().unwrap();
            let timeout = self.collector.as_ref().unwrap().timeout;
            let for_each = self.collector.as_mut().unwrap().for_each.take();
            let (filter, receiver) = MessageFilter::new(self.filter.take().unwrap());

            self.fut = Some(Box::pin(async move {
                shard_messenger.lock().await.set_message_filter(filter);

                MessageCollector {
                    received: 0,
                    receiver,
                    timeout,
                    for_each,
                }.receive_all().await
            }))
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

impl std::fmt::Debug for FilterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageFilter")
            .field("collect_limit", &self.collect_limit)
            .field("filter", &"Option<Arc<dyn Fn(&Arc<Message>) -> bool + 'static + Send + Sync>>")
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .field("author_id", &self.author_id)
            .finish()
    }
}

/// A message collector receives messages matching a the given filter for a
/// set duration.
pub struct MessageCollector {
    timeout: Option<Duration>,
    receiver: Receiver<Arc<Message>>,
    for_each: Option<Arc<ForeachFunction>>,
    received: u32,
}

impl MessageCollector {
    /// Internal method to receive messages until limitations are reached.
    /// The method fills `messages` while it is awaited, you cancel awaiting.
    async fn receive(&mut self, messages: &mut Vec<Arc<Message>>, max_messages: Option<u32>) {
        while let Some(message) = self.receiver.recv().await {
            if max_messages.map_or(false, |max| self.received >= max) {
                break;
            }

            self.received += 1;

            if let Some(for_each) = &self.for_each {

                if !for_each(&message).await {
                    break;
                }
            }

            messages.push(message);
        }
    }

    /// Receives all messages until set limitations are reached and
    /// returns the messages.
    ///
    /// This will consume the collector and end the collection.
    pub async fn receive_all(mut self) -> Vec<Arc<Message>> {
        let mut messages = Vec::new();

        if let Some(time) = self.timeout {
            let _ = timeout(time, self.receive(&mut messages, None)).await;
        } else {
            self.receive(&mut messages, None).await;
        }

        self.receiver.close();

        messages
    }

    /// Receives `number` amount of messages until set limitations are reached
    /// and returns the messages.
    ///
    /// This will consume the collector and end the collection.
    pub async fn receive_n(mut self, number: u32) -> Vec<Arc<Message>> {
        let mut messages = Vec::new();

        if let Some(time) = self.timeout {
            let _ = timeout(time, self.receive(&mut messages, Some(number))).await;
        } else {
            self.receive(&mut messages, Some(number)).await;
        }

        self.receiver.close();

        messages
    }

   /// Receives a single message and returns it.
   /// The timer is applied each time called, the internal message collected
   /// counter carries over.
    pub async fn receive_one(&mut self) -> Option<Arc<Message>> {
        if let Some(time) = self.timeout {
            timeout(time, self.receiver.recv()).await.ok().flatten()
        } else {
            self.receiver.recv().await
        }
    }

    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(mut self) {
        self.receiver.close();
    }
}

impl Drop for MessageCollector {
    fn drop(&mut self) {
        self.receiver.close();
    }
}
