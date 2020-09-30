use async_trait::async_trait;
use serenity::client::bridge::voice::VoiceGatewayManager;
use serenity::gateway::InterMessage;
use serenity::model::{
    id::{ChannelId, GuildId, UserId},
    voice::VoiceState,
};
use std::collections::HashMap;
use futures::channel::mpsc::UnboundedSender as Sender;
use super::Handler;
use tokio::sync::{Mutex, MutexGuard};

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
#[derive(Clone)]
pub struct Manager {
    handlers: HashMap<GuildId, Handler>,
    user_id: UserId,
    ws: Sender<InterMessage>,
}

impl Manager {
    pub(crate) fn new(ws: Sender<InterMessage>, user_id: UserId) -> Manager {
        Manager {
            handlers: HashMap::new(),
            user_id,
            ws,
        }
    }

    /// Retrieves an immutable handler for the given target, if one exists.
    #[inline]
    pub fn get<G: Into<GuildId>>(&self, guild_id: G) -> Option<&Handler> {
        self._get(guild_id.into())
    }

    fn _get(&self, guild_id: GuildId) -> Option<&Handler> {
        self.handlers.get(&guild_id)
    }

    /// Retrieves a mutable handler for the given target, if one exists.
    #[inline]
    pub fn get_mut<G: Into<GuildId>>(&mut self, guild_id: G)
        -> Option<&mut Handler> {
        self._get_mut(guild_id.into())
    }

    fn _get_mut(&mut self, guild_id: GuildId) -> Option<&mut Handler> {
        self.handlers.get_mut(&guild_id)
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
    pub fn join<C, G>(&mut self, guild_id: G, channel_id: C) -> &mut Handler
        where C: Into<ChannelId>, G: Into<GuildId> {
        self._join(guild_id.into(), channel_id.into())
    }

    fn _join(
        &mut self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> &mut Handler {
        {
            let mut found = false;

            if let Some(handler) = self.handlers.get_mut(&guild_id) {
                handler.switch_to(channel_id);

                found = true;
            }

            if found {
                // Actually safe, as the key has already been found above.
                return self.handlers.get_mut(&guild_id).unwrap();
            }
        }

        let mut handler = Handler::new(guild_id, self.ws.clone(), self.user_id);
        handler.join(channel_id);

        self.handlers.insert(guild_id, handler);

        // Actually safe, as the key would have been inserted above.
        self.handlers.get_mut(&guild_id).unwrap()
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
    pub fn leave<G: Into<GuildId>>(&mut self, guild_id: G) {
        self._leave(guild_id.into())
    }

    fn _leave(&mut self, guild_id: GuildId) {
        if let Some(handler) = self.handlers.get_mut(&guild_id) {
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
    pub fn remove<G: Into<GuildId>>(&mut self, guild_id: G) {
        self._remove(guild_id.into())
    }

    fn _remove(&mut self, guild_id: GuildId) {
        self.leave(guild_id);
        self.handlers.remove(&guild_id);
    }
}

pub struct ClientVoiceManager {
    inner: Mutex<InnerClientVoiceManager>,
}

impl ClientVoiceManager {
    pub fn new(shard_count: u64, user_id: UserId) -> Self {
        Self {
            inner: Mutex::new(InnerClientVoiceManager::new(shard_count, user_id)),
        }
    }

    pub async fn lock<'a>(&'a self) -> MutexGuard<'a, InnerClientVoiceManager> {
        self.inner.lock().await
    }
}

pub struct InnerClientVoiceManager {
    managers: HashMap<u64, Manager>,
    shard_count: u64,
    user_id: UserId,
}

impl InnerClientVoiceManager {
    pub fn new(shard_count: u64, user_id: UserId) -> Self {
        Self {
            managers: HashMap::default(),
            shard_count,
            user_id,
        }
    }

    pub fn get<G: Into<GuildId>>(&self, guild_id: G) -> Option<&Handler> {
        let (gid, sid) = self.manager_info(guild_id);

        self.managers.get(&sid)?.get(gid)
    }

    pub fn get_mut<G: Into<GuildId>>(&mut self, guild_id: G)
        -> Option<&mut Handler> {
        let (gid, sid) = self.manager_info(guild_id);

        self.managers.get_mut(&sid)?.get_mut(gid)
    }

    /// Refer to [`Manager::join`].
    ///
    /// This is a shortcut to retrieving the inner [`Manager`] and then calling
    /// its `join` method.
    ///
    /// [`Manager`]: ../../../voice/struct.Manager.html
    /// [`Manager::join`]: ../../../voice/struct.Manager.html#method.join
    pub fn join<C, G>(&mut self, guild_id: G, channel_id: C)
        -> Option<&mut Handler> where C: Into<ChannelId>, G: Into<GuildId> {
        let (gid, sid) = self.manager_info(guild_id);

        self.managers.get_mut(&sid).map(|manager| manager.join(gid, channel_id))
    }

    /// Refer to [`Manager::leave`].
    ///
    /// This is a shortcut to retrieving the inner [`Manager`] and then calling
    /// its `leave` method.
    ///
    /// [`Manager`]: ../../../voice/struct.Manager.html
    /// [`Manager::leave`]: ../../../voice/struct.Manager.html#method.leave
    pub fn leave<G: Into<GuildId>>(&mut self, guild_id: G) -> Option<()> {
        let (gid, sid) = self.manager_info(guild_id);

        self.managers.get_mut(&sid).map(|manager| manager.leave(gid))
    }

    /// Refer to [`Manager::remove`].
    ///
    /// This is a shortcut to retrieving the inner [`Manager`] and then calling
    /// its `remove` method.
    ///
    /// [`Manager`]: ../../../voice/struct.Manager.html
    /// [`Manager::remove`]: ../../../voice/struct.Manager.html#method.remove
    pub fn remove<G: Into<GuildId>>(&mut self, guild_id: G) -> Option<()> {
        let (gid, sid) = self.manager_info(guild_id);

        self.managers.get_mut(&sid).map(|manager| manager.remove(gid))
    }

    pub fn set(&mut self, shard_id: u64, sender: Sender<InterMessage>) {
        self.managers.insert(shard_id, Manager::new(sender, self.user_id));
    }

    /// Sets the number of shards for the voice manager to use when calculating
    /// guilds' shard numbers.
    ///
    /// You probably should not call this.
    #[doc(hidden)]
    pub fn set_shard_count(&mut self, shard_count: u64) {
        self.shard_count = shard_count;
    }

    /// Sets the ID of the user for the voice manager.
    ///
    /// You probably _really_ should not call this.
    ///
    /// But it's there if you need it. For some reason.
    #[doc(hidden)]
    pub fn set_user_id(&mut self, user_id: UserId) {
        self.user_id = user_id;
    }

    pub fn manager_get(&self, shard_id: u64) -> Option<&Manager> {
        self.managers.get(&shard_id)
    }

    pub fn manager_get_mut(&mut self, shard_id: u64) -> Option<&mut Manager> {
        self.managers.get_mut(&shard_id)
    }

    pub fn manager_remove(&mut self, shard_id: u64) -> Option<Manager> {
        self.managers.remove(&shard_id)
    }

    fn manager_info<G: Into<GuildId>>(&self, guild_id: G) -> (GuildId, u64) {
        let guild_id = guild_id.into();
        let shard_id = shard_id(guild_id.0, self.shard_count);

        (guild_id, shard_id)
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 { (guild_id >> 22) % shard_count }

#[async_trait]
impl VoiceGatewayManager for ClientVoiceManager {
    async fn initialise(&self, shard_count: u64, user_id: UserId) {
        let mut manager = self.inner.lock().await;

        manager.set_shard_count(shard_count);
        manager.set_user_id(user_id);
    }

    async fn register_shard(&self, shard_id: u64, sender: Sender<InterMessage>) {
        let mut manager = self.inner.lock().await;

        manager.set(shard_id, sender);
    }

    async fn deregister_shard(&self, shard_id: u64) {
        // FIXME: this is now the culprit of the rebalance voice disconnect bug.
        // Double check whether the stored channel for a shard is refreshed,
        // and if so, buffer up messages until it is replaced.
        let mut manager = self.inner.lock().await;
        manager.manager_remove(shard_id);
    }

    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &String) {
        let mut manager = self.inner.lock().await;
        let search = manager.get_mut(guild_id);

        if let Some(handler) = search {
            handler.update_server(endpoint, token);
        }
    }

    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState) {
        let mut manager = self.inner.lock().await;
        let search = manager.get_mut(guild_id);

        if let Some(handler) = search {
            handler.update_state(voice_state);
        }
    }
}
