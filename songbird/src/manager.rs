use super::Handler;
use crate::model::id::{ChannelId, GuildId, UserId};

use async_trait::async_trait;
#[cfg(feature = "serenity")]
use serenity::{
    client::bridge::voice::VoiceGatewayManager,
    gateway::InterMessage,
    model::{
        id::{
            ChannelId as SerenityChannel,
            GuildId as SerenityGuild,
            UserId as SerenityUser,
        },
        voice::VoiceState,
    },
};
use std::{
    collections::HashMap,
    result::Result as StdResult,
    sync::Arc,
};
use tracing::error;
#[cfg(feature = "serenity")]
use futures::channel::mpsc::{
    TrySendError,
    UnboundedSender as Sender,
};
use tokio::sync::{Mutex, MutexGuard, RwLock};

use parking_lot::{
    lock_api::RwLockWriteGuard,
    Mutex as PMutex,
    RwLock as PRwLock,
};

#[cfg(feature = "serenity")]
#[derive(Debug, Default)]
pub(crate) struct ShardHandle {
    sender: PRwLock<Option<Sender<InterMessage>>>,
    queue: PMutex<Vec<InterMessage>>,
}

impl ShardHandle {
    fn register(&self, sender: Sender<InterMessage>) {
        let mut sender_lock = self.sender.write();
        *sender_lock = Some(sender);

        let sender_lock = RwLockWriteGuard::downgrade(sender_lock);
        let mut messages_lock = self.queue.lock();

        if let Some(sender) = &*sender_lock {
            for msg in messages_lock.drain(..) {
                if let Err(e) = sender.unbounded_send(msg) {
                    error!("Error while clearing gateway message queue.");
                    break;
                }
            }
        }
    }

    fn deregister(&self) {
        let mut sender_lock = self.sender.write();
        *sender_lock = None;
    }

    pub(crate) fn send(&self, message: InterMessage) -> StdResult<(), TrySendError<InterMessage>> {
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

#[derive(Clone, Copy, Debug, Default)]
struct ClientData {
    shard_count: u64,
    user_id: UserId,
}

/// A manager is a struct responsible for managing [`Handler`]s which belong to
/// a single [`Shard`]. This is a fairly complex key-value store,
/// with a bit of extra utility for easily joining a "target".
///
/// The "target" used by the Manager is determined based on the `guild_id` and
/// `channel_id` provided. If a `guild_id` is _not_ provided to methods that
/// optionally require it, then the target is a group or 1-on-1 call with a
/// user. The `channel_id` is then used as the target.
///
/// If a `guild_id` is provided, then the target is the guild, as a user
/// can not be connected to two channels within one guild simultaneously.
///
/// [`Handler`]: struct.Handler.html
/// [guild's channel]: ../../model/channel/enum.ChannelType.html#variant.Voice
/// [`Shard`]: ../gateway/struct.Shard.html
#[derive(Debug, Default)]
pub struct Manager {
    client_data: PRwLock<ClientData>,
    calls: PRwLock<HashMap<GuildId, Arc<Mutex<Handler>>>>,
    #[cfg(feature = "serenity")]
    shard_handles: PRwLock<HashMap<u64, Arc<ShardHandle>>>,
}

impl Manager {
    pub fn initialise_client_data<U: Into<UserId>>(&self, shard_count: u64, user_id: U) {
        let mut client_data = self.client_data.write();

        client_data.shard_count = shard_count;
        client_data.user_id = user_id.into();
    }

    #[cfg(feature = "serenity")]
    fn get_or_insert_shard_handle(&self, shard_id: u64) -> Arc<ShardHandle> {
        ({
            let map_read = self.shard_handles.read();
            map_read.get(&shard_id).cloned()
        }).unwrap_or_else(|| {
            let mut map_read = self.shard_handles.write();
            map_read.entry(shard_id)
                .or_default()
                .clone()
        })
    }

    #[cfg(feature = "serenity")]
    pub fn register_shard_handle(&self, shard_id: u64, sender: Sender<InterMessage>) {
        // Write locks are only used to add new entries to the map.
        let handle = self.get_or_insert_shard_handle(shard_id);

        handle.register(sender);
    }

    #[cfg(feature = "serenity")]
    pub fn deregister_shard_handle(&self, shard_id: u64) {
        // Write locks are only used to add new entries to the map.
        let handle = self.get_or_insert_shard_handle(shard_id);

        handle.deregister();
    }

    pub fn get<G: Into<GuildId>>(&self, guild_id: G) -> Option<Arc<Mutex<Handler>>> {
        let map_read = self.calls.read();
        map_read.get(&guild_id.into()).cloned()
    }

    fn get_or_insert_call(&self, guild_id: GuildId) -> Arc<Mutex<Handler>> {
        self.get(guild_id).unwrap_or_else(|| {
            let mut map_read = self.calls.write();

            map_read.entry(guild_id)
                .or_insert_with(|| {
                    let info = self.manager_info();
                    let shard = shard_id(guild_id.0, info.shard_count);

                    Arc::new(Mutex::new(Handler::new(
                        guild_id,
                        self.get_or_insert_shard_handle(shard),
                        info.user_id,
                    )))
                })
                .clone()
        })
    }

    fn manager_info(&self) -> ClientData {
        let mut client_data = self.client_data.write();

        *client_data
    }

    /// Connects to a target by retrieving its relevant [`Handler`] and
    /// connecting, or creating the handler if required.
    ///
    /// This can also switch to the given channel, if a handler already exists
    /// for the target and the current connected channel is not equal to the
    /// given channel.
    ///
    /// In the case of channel targets, the same channel is used to connect to.
    ///
    /// In the case of guilds, the provided channel is used to connect to. The
    /// channel _must_ be in the provided guild. This is _not_ checked by the
    /// library, and will result in an error. If there is already a connected
    /// handler for the guild, _and_ the provided channel is different from the
    /// channel that the connection is already connected to, then the handler
    /// will switch the connection to the provided channel.
    ///
    /// If you _only_ need to retrieve the handler for a target, then use
    /// [`get`].
    ///
    /// [`Handler`]: struct.Handler.html
    /// [`get`]: #method.get
    #[inline]
    pub async fn join<C, G>(&self, guild_id: G, channel_id: C) -> Arc<Mutex<Handler>>
        where C: Into<ChannelId>, G: Into<GuildId> {
        self._join(guild_id.into(), channel_id.into()).await
    }

    async fn _join(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Arc<Mutex<Handler>> {
        let call = self.get_or_insert_call(guild_id);

        {
            let mut handler = call.lock().await;
            handler.join(channel_id)
        }

        call
    }

    /// Retrieves the [handler][`Handler`] for the given target and leaves the
    /// associated voice channel, if connected.
    ///
    /// This will _not_ drop the handler, and will preserve it and its settings.
    ///
    /// This is a wrapper around [getting][`get`] a handler and calling
    /// [`leave`] on it.
    ///
    /// [`Handler`]: struct.Handler.html
    /// [`get`]: #method.get
    /// [`leave`]: struct.Handler.html#method.leave
    #[inline]
    pub async fn leave<G: Into<GuildId>>(&self, guild_id: G) {
        self._leave(guild_id.into()).await;
    }

    async fn _leave(&self, guild_id: GuildId) {
        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            handler.leave();
        }
    }

    /// Retrieves the [`Handler`] for the given target and leaves the associated
    /// voice channel, if connected.
    ///
    /// The handler is then dropped, removing settings for the target.
    ///
    /// [`Handler`]: struct.Handler.html
    #[inline]
    pub async fn remove<G: Into<GuildId>>(&self, guild_id: G) {
        self._remove(guild_id.into()).await
    }

    async fn _remove(&self, guild_id: GuildId) {
        self.leave(guild_id).await;
        let mut calls = self.calls.write();
        calls.remove(&guild_id);
    }
}

#[async_trait]
impl VoiceGatewayManager for Manager {
    async fn initialise(&self, shard_count: u64, user_id: SerenityUser) {
        self.initialise_client_data(shard_count, user_id);
    }

    async fn register_shard(&self, shard_id: u64, sender: Sender<InterMessage>) {
        self.register_shard_handle(shard_id, sender);
    }

    async fn deregister_shard(&self, shard_id: u64) {
        self.deregister_shard_handle(shard_id);
    }

    async fn server_update(&self, guild_id: SerenityGuild, endpoint: &Option<String>, token: &str) {
        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            handler.update_server(endpoint, token);
        }
    }

    async fn state_update(&self, guild_id: SerenityGuild, voice_state: &VoiceState) {
        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            handler.update_state(voice_state);
        }
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 { (guild_id >> 22) % shard_count }
