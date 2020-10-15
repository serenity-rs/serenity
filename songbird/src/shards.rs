//! Handlers for sending packets over sharded connections.

use crate::error::{JoinError, JoinResult};
#[cfg(feature = "serenity")]
use futures::channel::mpsc::{TrySendError, UnboundedSender as Sender};
#[cfg(feature = "serenity")]
use parking_lot::{lock_api::RwLockWriteGuard, Mutex as PMutex, RwLock as PRwLock};
use serde_json::Value;
#[cfg(feature = "serenity")]
use serenity::gateway::InterMessage;
#[cfg(feature = "serenity")]
use std::{collections::HashMap, result::Result as StdResult, sync::Arc};
use tracing::error;
#[cfg(feature = "twilight")]
use twilight_gateway::{Cluster, Shard as TwilightShard};

#[derive(Debug)]
#[non_exhaustive]
/// Source of individual shard connection handles.
pub enum Sharder {
    #[cfg(feature = "serenity")]
    /// Serenity-specific wrapper for sharder state initialised by the library.
    Serenity(SerenitySharder),
    #[cfg(feature = "twilight")]
    /// Twilight-specific wrapper for sharder state initialised by the user.
    Twilight(Cluster),
}

impl Sharder {
    #[allow(unreachable_patterns)]
    /// Returns a new handle to the required inner shard.
    pub fn get_shard(&self, shard_id: u64) -> Option<Shard> {
        match self {
            #[cfg(feature = "serenity")]
            Sharder::Serenity(s) => Some(Shard::Serenity(s.get_or_insert_shard_handle(shard_id))),
            #[cfg(feature = "twilight")]
            Sharder::Twilight(t) => t.shard(shard_id).map(Shard::Twilight),
            _ => None,
        }
    }
}

#[cfg(feature = "serenity")]
impl Sharder {
    #[allow(unreachable_patterns)]
    pub(crate) fn register_shard_handle(&self, shard_id: u64, sender: Sender<InterMessage>) {
        match self {
            Sharder::Serenity(s) => s.register_shard_handle(shard_id, sender),
            _ => error!("Called serenity management function on a non-serenity Songbird instance."),
        }
    }

    #[allow(unreachable_patterns)]
    pub(crate) fn deregister_shard_handle(&self, shard_id: u64) {
        match self {
            Sharder::Serenity(s) => s.deregister_shard_handle(shard_id),
            _ => error!("Called serenity management function on a non-serenity Songbird instance."),
        }
    }
}

#[cfg(feature = "serenity")]
#[derive(Debug, Default)]
/// Serenity-specific wrapper for sharder state initialised by the library.
///
/// This is updated and maintained by the library, and is designed to prevent
/// message loss during rebalances and reconnects.
pub struct SerenitySharder(PRwLock<HashMap<u64, Arc<SerenityShardHandle>>>);

#[cfg(feature = "serenity")]
impl SerenitySharder {
    fn get_or_insert_shard_handle(&self, shard_id: u64) -> Arc<SerenityShardHandle> {
        ({
            let map_read = self.0.read();
            map_read.get(&shard_id).cloned()
        })
        .unwrap_or_else(|| {
            let mut map_read = self.0.write();
            map_read.entry(shard_id).or_default().clone()
        })
    }

    fn register_shard_handle(&self, shard_id: u64, sender: Sender<InterMessage>) {
        // Write locks are only used to add new entries to the map.
        let handle = self.get_or_insert_shard_handle(shard_id);

        handle.register(sender);
    }

    fn deregister_shard_handle(&self, shard_id: u64) {
        // Write locks are only used to add new entries to the map.
        let handle = self.get_or_insert_shard_handle(shard_id);

        handle.deregister();
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
/// A reference to an individual websocket connection.
pub enum Shard {
    #[cfg(feature = "serenity")]
    /// Handle to one of serenity's shard runners.
    Serenity(Arc<SerenityShardHandle>),
    #[cfg(feature = "twilight")]
    /// Handle to a twilight shard spawned from a cluster.
    Twilight(TwilightShard),
}

impl Shard {
    #[allow(unreachable_patterns)]
    /// Send a JSON message to the inner shard handle.
    pub async fn send(&mut self, msg: Value) -> JoinResult<()> {
        match self {
            #[cfg(feature = "serenity")]
            Shard::Serenity(s) => s.send(InterMessage::Json(msg))?,
            #[cfg(feature = "twilight")]
            Shard::Twilight(t) => t.command(&msg).await?,
            _ => return Err(JoinError::NoSender),
        }
        Ok(())
    }
}

#[cfg(feature = "serenity")]
/// Handle to an individual shard designed to buffer unsent messages while
/// a reconnect/rebalance is ongoing.
#[derive(Debug, Default)]
pub struct SerenityShardHandle {
    sender: PRwLock<Option<Sender<InterMessage>>>,
    queue: PMutex<Vec<InterMessage>>,
}

#[cfg(feature = "serenity")]
impl SerenityShardHandle {
    fn register(&self, sender: Sender<InterMessage>) {
        let mut sender_lock = self.sender.write();
        *sender_lock = Some(sender);

        let sender_lock = RwLockWriteGuard::downgrade(sender_lock);
        let mut messages_lock = self.queue.lock();

        if let Some(sender) = &*sender_lock {
            for msg in messages_lock.drain(..) {
                if let Err(e) = sender.unbounded_send(msg) {
                    error!("Error while clearing gateway message queue: {:?}", e);
                    break;
                }
            }
        }
    }

    fn deregister(&self) {
        let mut sender_lock = self.sender.write();
        *sender_lock = None;
    }

    fn send(&self, message: InterMessage) -> StdResult<(), TrySendError<InterMessage>> {
        let sender_lock = self.sender.read();
        if let Some(sender) = &*sender_lock {
            sender.unbounded_send(message)
        } else {
            let mut messages_lock = self.queue.lock();
            messages_lock.push(message);
            Ok(())
        }
    }
}
