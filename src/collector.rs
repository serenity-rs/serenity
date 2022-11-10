// Or we'll get deprecation warnings from our own deprecated type (seriously Rust?)
#![allow(deprecated)]

use futures::future::pending;
use futures::{Stream, StreamExt as _};

use crate::client::bridge::gateway::{CollectorCallback, ShardMessenger};
use crate::model::prelude::*;

/// Fundamental collector function. All collector types in this module are just wrappers around this
/// function.
///
/// Example: creating a collector stream over removed reactions
/// ```rust
/// # use std::time::Duration;
/// # use futures::StreamExt as _;
/// # use serenity::model::prelude::Event;
/// # use serenity::client::bridge::gateway::ShardMessenger;
/// # use serenity::collector::collect;
/// # async fn _example(shard: &ShardMessenger) {
/// let stream = collect(shard, |event| match event {
///     Event::ReactionRemove(event) => Some(event.reaction.clone()),
///     _ => None,
/// });
///
/// stream
///     .for_each(|reaction| async move {
///         println!("{}: removed {}", reaction.channel_id, reaction.emoji);
///     })
///     .await;
/// # }
/// ```
pub fn collect<T: Send + 'static>(
    shard: &ShardMessenger,
    extractor: impl Fn(&Event) -> Option<T> + Send + Sync + 'static,
) -> impl Stream<Item = T> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    // Register an event callback in the shard. It's kept alive as long as we return `true`
    shard.add_collector(CollectorCallback(Box::new(move |event| match extractor(event) {
        // If this event matches, we send it to the receiver stream
        Some(item) => sender.send(item).is_ok(),
        None => !sender.is_closed(),
    })));

    // Convert the mpsc Receiver into a Stream
    futures::stream::poll_fn(move |cx| receiver.poll_recv(cx))
}

macro_rules! make_specific_collector {
    (
        $( #[ $($meta:tt)* ] )*
        $collector_type:ident, $item_type:ident,
        $extractor:pat => $extracted_item:ident,
        $( $filter_name:ident: $filter_type:ty => $filter_passes:expr, )*
    ) => {
        #[doc = concat!("A [`", stringify!($collector_type), "`] receives [`", stringify!($item_type), "`]'s match the given filters for a set duration.")]
        $( #[ $($meta)* ] )*
        #[must_use]
        pub struct $collector_type {
            shard: ShardMessenger,
            duration: Option<std::time::Duration>,
            filter: Option<Box<dyn Fn(&$item_type) -> bool + Send + Sync>>,
            $( $filter_name: Option<$filter_type>, )*
        }

        impl $collector_type {
            /// Creates a new collector without any filters configured.
            pub fn new(shard: impl AsRef<ShardMessenger>) -> Self {
                Self {
                    shard: shard.as_ref().clone(),
                    duration: None,
                    filter: None,
                    $( $filter_name: None, )*
                }
            }

            /// Sets a duration for how long the collector shall receive interactions.
            pub fn timeout(mut self, duration: std::time::Duration) -> Self {
                self.duration = Some(duration);
                self
            }

            /// Sets a generic filter function.
            pub fn filter(mut self, filter: impl Fn(&$item_type) -> bool + Send + Sync + 'static) -> Self {
                self.filter = Some(Box::new(filter));
                self
            }

            $(
                #[doc = concat!("Filters [`", stringify!($item_type), "`]'s by a specific [`", stringify!($filter_type), "`].")]
                pub fn $filter_name(mut self, $filter_name: $filter_type) -> Self {
                    self.$filter_name = Some($filter_name);
                    self
                }
            )*

            #[doc = concat!("Returns a [`Stream`] over all collected [`", stringify!($item_type), "`].")]
            pub fn collect_stream(self) -> impl Stream<Item = $item_type> {
                let filters_pass = move |$extracted_item: &$item_type| {
                    // Check each of the built-in filters (author_id, channel_id, etc.)
                    $( if let Some($filter_name) = &self.$filter_name {
                        if !$filter_passes {
                            return false;
                        }
                    } )*
                    // Check the callback-based filter
                    if let Some(custom_filter) = &self.filter {
                        if !custom_filter($extracted_item) {
                            return false;
                        }
                    }
                    true
                };

                // A future that completes once the timeout is triggered
                let timeout = async move { match self.duration {
                    Some(d) => tokio::time::sleep(d).await,
                    None => pending::<()>().await,
                } };

                let stream = collect(&self.shard, move |event| match event {
                    $extractor if filters_pass($extracted_item) => Some($extracted_item.clone()),
                    _ => None,
                });
                // Need to Box::pin this, or else users have to `pin_mut!()` the stream to the stack
                stream.take_until(Box::pin(timeout))
            }

            #[doc = concat!("Returns the next [`", stringify!($item_type), "`] which passes the filters.")]
            pub async fn collect_single(self) -> Option<$item_type> {
                self.collect_stream().next().await
            }
        }
    };
}

make_specific_collector!(
    // First line has name of the collector type, and the type of the collected items.
    ComponentInteractionCollector, ComponentInteraction,
    // This defines the extractor pattern, which extracts the data we want to collect from an Event.
    Event::InteractionCreate(InteractionCreateEvent {
        interaction: Interaction::Component(interaction),
    }) => interaction,
    // All following lines define built-in filters of the collector.
    // Each line consists of:
    // - the filter name (the name of the generated builder-like method on the collector type)
    // - filter argument type (used as argument of the builder-like method on the collector type)
    // - filter expression (this expressoin must return true to let the event through)
    author_id: UserId => interaction.user.id == *author_id,
    channel_id: ChannelId => interaction.channel_id == *channel_id,
    guild_id: GuildId => interaction.guild_id.map_or(true, |x| x == *guild_id),
    message_id: MessageId => interaction.message.id == *message_id,
    custom_ids: Vec<String> => custom_ids.contains(&interaction.data.custom_id),
);
make_specific_collector!(
    ModalInteractionCollector, ModalInteraction,
    Event::InteractionCreate(InteractionCreateEvent {
        interaction: Interaction::Modal(interaction),
    }) => interaction,
    author_id: UserId => interaction.user.id == *author_id,
    channel_id: ChannelId => interaction.channel_id == *channel_id,
    guild_id: GuildId => interaction.guild_id.map_or(true, |g| g == *guild_id),
    message_id: MessageId => interaction.message.as_ref().map_or(true, |m| m.id == *message_id),
    custom_ids: Vec<String> => custom_ids.contains(&interaction.data.custom_id),
);
make_specific_collector!(
    ReactionCollector, Reaction,
    Event::ReactionAdd(ReactionAddEvent { reaction }) => reaction,
    author_id: UserId => reaction.user_id.map_or(true, |a| a == *author_id),
    channel_id: ChannelId => reaction.channel_id == *channel_id,
    guild_id: GuildId => reaction.guild_id.map_or(true, |g| g == *guild_id),
    message_id: MessageId => reaction.message_id == *message_id,
);
make_specific_collector!(
    MessageCollector, Message,
    Event::MessageCreate(MessageCreateEvent { message }) => message,
    author_id: UserId => message.author.id == *author_id,
    channel_id: ChannelId => message.channel_id == *channel_id,
    guild_id: GuildId => message.guild_id.map_or(true, |g| g == *guild_id),
);
make_specific_collector!(
    #[deprecated = "prefer the stand-alone collect() function to collect arbitrary events"]
    EventCollector, Event,
    event => event,
);
