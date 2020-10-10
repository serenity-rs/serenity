use crate::{
    error::{JoinError, JoinResult},
    id::{ChannelId, GuildId, UserId},
    info::{ConnectionInfo, ConnectionProgress},
    shards::Shard,
};
#[cfg(feature = "driver")]
use crate::driver::{
    Driver,
    Error,
};
use flume::{Receiver, Sender};
use serde_json::json;
use tracing::instrument;

#[cfg(feature = "driver")]
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
enum Return {
    Info(Sender<ConnectionInfo>),
    #[cfg(feature = "driver")]
    Conn(Sender<Result<(), Error>>),
}

/// The handler is responsible for "handling" a single voice connection, acting
/// as a clean API above the inner connection.
///
/// If the `"driver"` feature is enabled, then 
///
/// [`Driver`]: struct.Dri.html
/// [`Shard`]: ../gateway/struct.Shard.html
#[derive(Clone, Debug)]
pub struct Call {
    connection: Option<(ChannelId, ConnectionProgress, Return)>,

    #[cfg(feature = "driver")]
    /// The internal controller of the voice connection monitor thread.
    driver: Driver,

    guild_id: GuildId,
    /// Whether the current handler is set to deafen voice connections.
    self_deaf: bool,
    /// Whether the current handler is set to mute voice connections.
    self_mute: bool,
    user_id: UserId,
    /// Will be set when a `Call` is made via the [`new`][`Call::new`]
    /// method.
    ///
    /// When set via [`standalone`][`Call::standalone`], it will not be
    /// present.
    ws: Option<Shard>,
}

impl Call {
    /// Creates a new Call, which will send out WebSocket messages via
    /// the given shard.
    #[inline]
    pub fn new(
        guild_id: GuildId,
        ws: Shard,
        user_id: UserId,
    ) -> Self {
        Self::new_raw(guild_id, Some(ws), user_id)
    }

    /// Creates a new, standalone Call which is not connected via
    /// WebSocket to the Gateway.
    ///
    /// Actions such as muting, deafening, and switching channels will not
    /// function through this Call and must be done through some other
    /// method, as the values will only be internally updated.
    ///
    /// For most use cases you do not want this.
    #[inline]
    pub fn standalone(guild_id: GuildId, user_id: UserId) -> Self {
        Self::new_raw(guild_id, None, user_id)
    }

    fn new_raw(
        guild_id: GuildId,
        ws: Option<Shard>,
        user_id: UserId,
    ) -> Self {
        Call {
            connection: None,
            #[cfg(feature = "driver")]
            driver: Default::default(),
            guild_id,
            self_deaf: false,
            self_mute: false,
            user_id,
            ws,
        }
    }

    fn do_connect(&mut self) {
        match &self.connection {
            Some((_, ConnectionProgress::Complete(c), Return::Info(tx))) => {
                // It's okay if the receiver hung up.
                let _ = tx.send(c.clone());
            },
            #[cfg(feature = "driver")]
            Some((_, ConnectionProgress::Complete(c), Return::Conn(tx))) => {
                self.driver.raw_connect(c.clone(), tx.clone());
            },
            _ => {},
        }
    }

    /// Sets whether the current connection to be deafened.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: Unlike in the official client, you _can_ be deafened while
    /// not being muted.
    ///
    /// **Note**: If the `Call` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally deafened.
    ///
    /// [`standalone`]: #method.standalone
    pub async fn deafen(&mut self, deaf: bool) -> JoinResult<()> {
        self.self_deaf = deaf;

        self.update().await
    }

    pub fn is_deaf(&self) -> bool {
        self.self_deaf
    }

    #[cfg(feature = "driver")]
    /// Connect or switch to the given voice channel by its Id.
    pub async fn join(&mut self, channel_id: ChannelId) -> JoinResult<Receiver<Result<(), Error>>> {
        let (tx, rx) = flume::unbounded();

        self.connection = Some((
            channel_id,
            ConnectionProgress::new(self.guild_id.into(), self.user_id.into()),
            Return::Conn(tx),
        ));

        self.update()
            .await
            .map(|_| rx)
    }

    /// Join the selected voice channel, *without* running/starting an RTP
    /// session or running the driver.
    ///
    /// Use this if you require connection info for lavalink,
    /// or some other voice implementation.
    pub async fn join_gateway(&mut self, channel_id: ChannelId) -> JoinResult<Receiver<ConnectionInfo>> {
        let (tx, rx) = flume::unbounded();

        self.connection = Some((
            channel_id,
            ConnectionProgress::new(self.guild_id.into(), self.user_id.into()),
            Return::Info(tx),
        ));

        self.update()
            .await
            .map(|_| rx)
    }

    /// Leaves the current voice channel, disconnecting from it.
    ///
    /// This does _not_ forget settings, like whether to be self-deafened or
    /// self-muted.
    ///
    /// **Note**: If the `Call` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally connected to a
    /// voice channel.
    ///
    /// [`standalone`]: #method.standalone
    pub async fn leave(&mut self) -> JoinResult<()> {
        // Only send an update if we were in a voice channel.
        self.connection = None;

        #[cfg(feature = "driver")]
        self.driver.leave();

        self.update().await
    }

    /// Sets whether the current connection is to be muted.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: If the `Call` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally muted.
    ///
    /// [`standalone`]: #method.standalone
    pub async fn mute(&mut self, mute: bool) -> JoinResult<()> {
        self.self_mute = mute;

        #[cfg(feature = "driver")]
        self.driver.mute(mute);

        self.update().await
    }

    pub fn is_mute(&self) -> bool {
        self.self_mute
    }

    /// Updates the voice server data.
    ///
    /// You should only need to use this if you initialized the `Call` via
    /// [`standalone`].
    ///
    /// Refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    #[instrument(skip(self, token))]
    pub fn update_server(&mut self, endpoint: String, token: String) {
        let try_conn = if let Some((_, ref mut progress, _)) = self.connection.as_mut() {
            progress.apply_server_update(endpoint, token)
        } else { false };

        if try_conn {
            self.do_connect();
        }
    }

    /// Updates the internal voice state of the current user.
    ///
    /// You should only need to use this if you initialized the `Call` via
    /// [`standalone`].
    ///
    /// refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    pub fn update_state(&mut self, session_id: String) {
        let try_conn = if let Some((_, ref mut progress, _)) = self.connection.as_mut() {
            progress.apply_state_update(session_id)
        } else { false };

        if try_conn {
            self.do_connect();
        }
    }

    /// Send an update for the current session over WS.
    ///
    /// Does nothing if initialized via [`standalone`].
    ///
    /// [`standalone`]: #method.standalone
    async fn update(&mut self) -> JoinResult<()> {
        if let Some(ws) = self.ws.as_mut() {
            let map = json!({
                "op": 4,
                "d": {
                    "channel_id": self.connection.as_ref().map(|c| c.0.0),
                    "guild_id": self.guild_id.0,
                    "self_deaf": self.self_deaf,
                    "self_mute": self.self_mute,
                }
            });

            ws.send(map).await
        } else {
            Err(JoinError::NoSender)
        }
    }
}

#[cfg(feature = "driver")]
impl Deref for Call {
    type Target = Driver;

    fn deref(&self) -> &Self::Target {
        &self.driver
    }
}

#[cfg(feature = "driver")]
impl DerefMut for Call {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.driver
    }
}
