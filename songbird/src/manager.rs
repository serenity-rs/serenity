#[cfg(feature = "driver")]
use crate::error::ConnectionResult;
use crate::{
    error::{JoinError, JoinResult},
    id::{ChannelId, GuildId, UserId},
    shards::Sharder,
    Call,
    ConnectionInfo,
};
#[cfg(feature = "serenity")]
use async_trait::async_trait;
use flume::Receiver;
#[cfg(feature = "serenity")]
use futures::channel::mpsc::UnboundedSender as Sender;
use parking_lot::RwLock as PRwLock;
#[cfg(feature = "serenity")]
use serenity::{
    client::bridge::voice::VoiceGatewayManager,
    gateway::InterMessage,
    model::{
        id::{GuildId as SerenityGuild, UserId as SerenityUser},
        voice::VoiceState,
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
#[cfg(feature = "twilight")]
use twilight_gateway::Cluster;
#[cfg(feature = "twilight")]
use twilight_model::gateway::event::Event as TwilightEvent;

#[derive(Clone, Copy, Debug, Default)]
struct ClientData {
    shard_count: u64,
    initialised: bool,
    user_id: UserId,
}

/// A shard-aware struct responsible for managing [`Call`]s.
///
/// This manager transparently maps guild state and a source of shard information
/// into individual calls, and forwards state updates which affect call state.
///
/// [`Call`]: struct.Call.html
#[derive(Debug)]
pub struct Songbird {
    client_data: PRwLock<ClientData>,
    calls: PRwLock<HashMap<GuildId, Arc<Mutex<Call>>>>,
    sharder: Sharder,
}

impl Songbird {
    #[cfg(feature = "serenity")]
    /// Create a new Songbird instance for serenity.
    ///
    /// This must be [registered] after creation.
    ///
    /// [registered]: serenity/fn.register_with.html
    pub fn serenity() -> Arc<Self> {
        Arc::new(Self {
            client_data: Default::default(),
            calls: Default::default(),
            sharder: Sharder::Serenity(Default::default()),
        })
    }

    #[cfg(feature = "twilight")]
    /// Create a new Songbird instance for twilight.
    ///
    /// Twilight handlers do not need to be registered, but
    /// users are responsible for passing in any events using
    /// [`process`].
    ///
    /// [`process`]: #method.process
    pub fn twilight<U>(cluster: Cluster, shard_count: u64, user_id: U) -> Arc<Self>
    where
        U: Into<UserId>,
    {
        Arc::new(Self {
            client_data: PRwLock::new(ClientData {
                shard_count,
                initialised: true,
                user_id: user_id.into(),
            }),
            calls: Default::default(),
            sharder: Sharder::Twilight(cluster),
        })
    }

    /// Set the bot's user, and the number of shards in use.
    ///
    /// If this struct is already initialised (e.g., from [`::twilight`]),
    /// or a previous call, then this function is a no-op.
    ///
    /// [`::twilight`]: #method.twilight
    pub fn initialise_client_data<U: Into<UserId>>(&self, shard_count: u64, user_id: U) {
        let mut client_data = self.client_data.write();

        if client_data.initialised {
            return;
        }

        client_data.shard_count = shard_count;
        client_data.user_id = user_id.into();
        client_data.initialised = true;
    }

    /// Retreives a [`Call`] for the given guild, if one already exists.
    ///
    /// [`Call`]: struct.Call.html
    pub fn get<G: Into<GuildId>>(&self, guild_id: G) -> Option<Arc<Mutex<Call>>> {
        let map_read = self.calls.read();
        map_read.get(&guild_id.into()).cloned()
    }

    /// Retreives a [`Call`] for the given guild, creating a new one if
    /// none is found.
    ///
    /// This will not join any calls, or cause connection state to change.
    ///
    /// [`Call`]: struct.Call.html
    pub fn get_or_insert(&self, guild_id: GuildId) -> Arc<Mutex<Call>> {
        self.get(guild_id).unwrap_or_else(|| {
            let mut map_read = self.calls.write();

            map_read
                .entry(guild_id)
                .or_insert_with(|| {
                    let info = self.manager_info();
                    let shard = shard_id(guild_id.0, info.shard_count);
                    let shard_handle = self
                        .sharder
                        .get_shard(shard)
                        .expect("Failed to get shard handle: shard_count incorrect?");

                    Arc::new(Mutex::new(Call::new(guild_id, shard_handle, info.user_id)))
                })
                .clone()
        })
    }

    fn manager_info(&self) -> ClientData {
        let client_data = self.client_data.write();

        *client_data
    }

    #[cfg(feature = "driver")]
    /// Connects to a target by retrieving its relevant [`Call`] and
    /// connecting, or creating the handler if required.
    ///
    /// This can also switch to the given channel, if a handler already exists
    /// for the target and the current connected channel is not equal to the
    /// given channel.
    ///
    /// The provided channel ID is used as a connection target. The
    /// channel _must_ be in the provided guild. This is _not_ checked by the
    /// library, and will result in an error. If there is already a connected
    /// handler for the guild, _and_ the provided channel is different from the
    /// channel that the connection is already connected to, then the handler
    /// will switch the connection to the provided channel.
    ///
    /// If you _only_ need to retrieve the handler for a target, then use
    /// [`get`].
    ///
    /// [`Call`]: struct.Call.html
    /// [`get`]: #method.get
    #[inline]
    pub async fn join<C, G>(
        &self,
        guild_id: G,
        channel_id: C,
    ) -> (Arc<Mutex<Call>>, JoinResult<Receiver<ConnectionResult<()>>>)
    where
        C: Into<ChannelId>,
        G: Into<GuildId>,
    {
        self._join(guild_id.into(), channel_id.into()).await
    }

    #[cfg(feature = "driver")]
    async fn _join(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> (Arc<Mutex<Call>>, JoinResult<Receiver<ConnectionResult<()>>>) {
        let call = self.get_or_insert(guild_id);

        let result = {
            let mut handler = call.lock().await;
            handler.join(channel_id).await
        };

        (call, result)
    }

    /// Partially connects to a target by retrieving its relevant [`Call`] and
    /// connecting, or creating the handler if required.
    ///
    /// This method returns the handle and the connection info needed for other libraries
    /// or drivers, such as lavalink, and does not actually start or run a voice call.
    ///
    /// [`Call`]: struct.Call.html
    #[inline]
    pub async fn join_gateway<C, G>(
        &self,
        guild_id: G,
        channel_id: C,
    ) -> (Arc<Mutex<Call>>, JoinResult<Receiver<ConnectionInfo>>)
    where
        C: Into<ChannelId>,
        G: Into<GuildId>,
    {
        self._join_gateway(guild_id.into(), channel_id.into()).await
    }

    async fn _join_gateway(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> (Arc<Mutex<Call>>, JoinResult<Receiver<ConnectionInfo>>) {
        let call = self.get_or_insert(guild_id);

        let result = {
            let mut handler = call.lock().await;
            handler.join_gateway(channel_id).await
        };

        (call, result)
    }

    /// Retrieves the [handler][`Call`] for the given target and leaves the
    /// associated voice channel, if connected.
    ///
    /// This will _not_ drop the handler, and will preserve it and its settings.
    ///
    /// This is a wrapper around [getting][`get`] a handler and calling
    /// [`leave`] on it.
    ///
    /// [`Call`]: struct.Call.html
    /// [`get`]: #method.get
    /// [`leave`]: struct.Call.html#method.leave
    #[inline]
    pub async fn leave<G: Into<GuildId>>(&self, guild_id: G) -> JoinResult<()> {
        self._leave(guild_id.into()).await
    }

    async fn _leave(&self, guild_id: GuildId) -> JoinResult<()> {
        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            handler.leave().await
        } else {
            Err(JoinError::NoCall)
        }
    }

    /// Retrieves the [`Call`] for the given target and leaves the associated
    /// voice channel, if connected.
    ///
    /// The handler is then dropped, removing settings for the target.
    ///
    /// An Err(...) value implies that the gateway could not be contacted,
    /// and that leaving should be attempted again later (i.e., after reconnect).
    ///
    /// [`Call`]: struct.Call.html
    #[inline]
    pub async fn remove<G: Into<GuildId>>(&self, guild_id: G) -> JoinResult<()> {
        self._remove(guild_id.into()).await
    }

    async fn _remove(&self, guild_id: GuildId) -> JoinResult<()> {
        self.leave(guild_id).await?;
        let mut calls = self.calls.write();
        calls.remove(&guild_id);
        Ok(())
    }
}

#[cfg(feature = "twilight")]
impl Songbird {
    /// Handle events received on the cluster.
    ///
    /// When using twilight, you are required to call this with all inbound
    /// (voice) events, *i.e.*, at least `VoiceStateUpdate`s and `VoiceServerUpdate`s.
    pub async fn process(&self, event: &TwilightEvent) {
        match event {
            TwilightEvent::VoiceServerUpdate(v) => {
                let call = v.guild_id.map(GuildId::from).and_then(|id| self.get(id));

                if let Some(call) = call {
                    let mut handler = call.lock().await;
                    if let Some(endpoint) = &v.endpoint {
                        handler.update_server(endpoint.clone(), v.token.clone());
                    }
                }
            },
            TwilightEvent::VoiceStateUpdate(v) => {
                if v.0.user_id.0 != self.client_data.read().user_id.0 {
                    return;
                }

                let call = v.0.guild_id.map(GuildId::from).and_then(|id| self.get(id));

                if let Some(call) = call {
                    let mut handler = call.lock().await;
                    handler.update_state(v.0.session_id.clone());
                }
            },
            _ => {},
        }
    }
}

#[cfg(feature = "serenity")]
#[async_trait]
impl VoiceGatewayManager for Songbird {
    async fn initialise(&self, shard_count: u64, user_id: SerenityUser) {
        self.initialise_client_data(shard_count, user_id);
    }

    async fn register_shard(&self, shard_id: u64, sender: Sender<InterMessage>) {
        self.sharder.register_shard_handle(shard_id, sender);
    }

    async fn deregister_shard(&self, shard_id: u64) {
        self.sharder.deregister_shard_handle(shard_id);
    }

    async fn server_update(&self, guild_id: SerenityGuild, endpoint: &Option<String>, token: &str) {
        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            if let Some(endpoint) = endpoint {
                handler.update_server(endpoint.clone(), token.to_string());
            }
        }
    }

    async fn state_update(&self, guild_id: SerenityGuild, voice_state: &VoiceState) {
        if voice_state.user_id.0 != self.client_data.read().user_id.0 {
            return;
        }

        if let Some(call) = self.get(guild_id) {
            let mut handler = call.lock().await;
            handler.update_state(voice_state.session_id.clone());
        }
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 {
    (guild_id >> 22) % shard_count
}
