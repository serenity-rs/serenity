use crate::gateway::InterMessage;
use crate::model::id::{ChannelId, GuildId, UserId};
use std::{
    collections::HashMap,
    sync::mpsc::Sender as MpscSender
};
use super::Handler;

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
/// [`Group`]: ../../model/channel/struct.Group.html
/// [`Handler`]: struct.Handler.html
/// [guild's channel]: ../../model/channel/enum.ChannelType.html#variant.Voice
/// [`Shard`]: ../gateway/struct.Shard.html
#[derive(Clone, Debug)]
pub struct Manager {
    handlers: HashMap<GuildId, Handler>,
    user_id: UserId,
    ws: MpscSender<InterMessage>,
}

impl Manager {
    pub(crate) fn new(ws: MpscSender<InterMessage>, user_id: UserId) -> Manager {
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
