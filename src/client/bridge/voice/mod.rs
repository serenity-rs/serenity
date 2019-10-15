use crate::gateway::InterMessage;
use std::collections::HashMap;
use std::sync::mpsc::Sender as MpscSender;
use crate::model::id::{ChannelId, GuildId, UserId};
use crate::voice::{Handler, Manager};
use crate::utils;

pub struct ClientVoiceManager {
    managers: HashMap<u64, Manager>,
    shard_count: u64,
    user_id: UserId,
}

impl ClientVoiceManager {
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

    pub fn set(&mut self, shard_id: u64, sender: MpscSender<InterMessage>) {
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
        let shard_id = utils::shard_id(guild_id.0, self.shard_count);

        (guild_id, shard_id)
    }
}
