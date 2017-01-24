use serde_json::builder::ObjectBuilder;
use std::sync::mpsc::{self, Sender as MpscSender};
use super::{AudioReceiver, AudioSource};
use super::connection_info::ConnectionInfo;
use super::{Status as VoiceStatus, Target};
use ::client::gateway::GatewayStatus;
use ::constants::VoiceOpCode;
use ::model::{ChannelId, GuildId, UserId, VoiceState};
use super::threading;

/// The handler is responsible for "handling" a single voice connection, acting
/// as a clean API above the inner connection.
///
/// # Examples
///
/// Assuming that you already have a [`Manager`], most likely retrieved via a
/// [WebSocket connection], you can join a guild's voice channel and deafen
/// yourself like so:
///
/// ```rust,ignore
/// // assuming a `manager` has already been bound, hopefully retrieved through
/// // a websocket's connection.
/// use serenity::model::{ChannelId, GuildId};
///
/// let guild_id = GuildId(81384788765712384);
/// let channel_id = ChannelId(85482585546833920);
///
/// let handler = manager.join(Some(guild_id), channel_id);
/// handler.deafen(true);
/// ```
///
/// [`Manager`]: struct.Manager.html
/// [WebSocket connection]: ../../client/struct.Connection.html
pub struct Handler {
    channel_id: Option<ChannelId>,
    endpoint_token: Option<(String, String)>,
    guild_id: Option<GuildId>,
    self_deaf: bool,
    self_mute: bool,
    sender: MpscSender<VoiceStatus>,
    session_id: Option<String>,
    user_id: UserId,
    ws: MpscSender<GatewayStatus>,
}

impl Handler {
    /// Creates a new Handler.
    ///
    /// **Note**: You should never call this yourself, and should instead use
    /// [`Manager::join`].
    ///
    /// Like, really. Really do not use this. Please.
    ///
    /// [`Manager::join`]: struct.Manager.html#method.join
    #[doc(hidden)]
    pub fn new(target: Target, ws: MpscSender<GatewayStatus>, user_id: UserId)
        -> Self {
        let (tx, rx) = mpsc::channel();

        let (channel_id, guild_id) = match target {
            Target::Channel(channel_id) => (Some(channel_id), None),
            Target::Guild(guild_id) => (None, Some(guild_id)),
        };

        threading::start(target, rx);

        Handler {
            channel_id: channel_id,
            endpoint_token: None,
            guild_id: guild_id,
            self_deaf: false,
            self_mute: false,
            sender: tx,
            session_id: None,
            user_id: user_id,
            ws: ws,
        }
    }

    /// Retrieves the current connected voice channel's `ChannelId`, if connected
    /// to one.
    ///
    /// Note that when connected to a voice channel, while the `ChannelId` will
    /// not be `None`, the [`GuildId`] retrieved via [`guild`] can, in the event
    /// of [`Group`] or 1-on-1 [`Call`]s.
    ///
    /// [`Call`]: ../../model/struct.Call.html
    /// [`Group`]: ../../model/struct.Group.html
    /// [`GuildId`]: ../../model/struct.GuildId.html
    /// [`guild`]: #method.guild
    pub fn channel(&self) -> Option<ChannelId> {
        self.channel_id
    }

    /// Sets whether the current connection to be deafened.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: Unlike in the official client, you _can_ be deafened while
    /// not being muted.
    pub fn deafen(&mut self, deaf: bool) {
        self.self_deaf = deaf;

        // Only send an update if there is currently a connected channel.
        //
        // Otherwise, this can be treated as a "settings" update for a
        // connection.
        if self.channel_id.is_some() {
            self.update();
        }
    }

    /// Retrieves the current connected voice channel's `GuildId`, if connected
    /// to one.
    ///
    /// Note that the `GuildId` can be `None` in the event of [`Group`] or
    /// 1-on-1 [`Call`]s, although when connected to a voice channel, the
    /// [`ChannelId`] retrieved via [`channel`] will be `Some`.
    ///
    /// [`Call`]: ../../model/struct.Call.html
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    /// [`Group`]: ../../model/struct.Group.html
    /// [`channel`]: #method.channel
    pub fn guild(&self) -> Option<GuildId> {
        self.guild_id
    }

    /// Whether the current handler is set to deafen voice connections.
    ///
    /// Use [`deafen`] to modify this configuration.
    ///
    /// [`deafen`]: #method.deafen
    pub fn is_deafened(&self) -> bool {
        self.self_deaf
    }

    /// Whether the current handler is set to mute voice connections.
    ///
    /// Use [`mute`] to modify this configuration.
    ///
    /// [`mute`]: #method.mute
    pub fn is_muted(&self) -> bool {
        self.self_mute
    }

    /// Connect - or switch - to the given voice channel by its Id.
    ///
    /// **Note**: This is not necessary for [`Group`] or direct [call][`Call`]s.
    ///
    /// [`Call`]: ../../model/struct.Call.html
    /// [`Group`]: ../../model/struct.Group.html
    pub fn join(&mut self, channel_id: ChannelId) {
        self.channel_id = Some(channel_id);

        self.connect();
    }

    /// Leaves the current voice channel, disconnecting from it.
    ///
    /// This does _not_ forget settings, like whether to be self-deafened or
    /// self-muted.
    pub fn leave(&mut self) {
        if let Some(_) = self.channel_id {
            self.channel_id = None;

            self.update();
        }
    }

    /// Sets a receiver, i.e. a way to receive audio. Most use cases for bots do
    /// not require this.
    ///
    /// The `receiver` argument can be thought of as an "optional Option". You
    /// can pass in just a boxed receiver, and do not need to specify `Some`.
    ///
    /// Pass `None` to drop the current receiver, if one exists.
    pub fn listen<O: Into<Option<Box<AudioReceiver>>>>(&mut self, receiver: O) {
        self.send(VoiceStatus::SetReceiver(receiver.into()))
    }

    /// Sets whether the current connection is to be muted.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    pub fn mute(&mut self, mute: bool) {
        self.self_mute = mute;

        if self.channel_id.is_some() {
            self.update();
        }
    }

    /// Plays audio from a source. This can be a source created via
    /// [`voice::ffmpeg`] or [`voice::ytdl`].
    ///
    /// [`voice::ffmpeg`]: fn.ffmpeg.html
    /// [`voice::ytdl`]: fn.ytdl.html
    pub fn play(&mut self, source: Box<AudioSource>) {
        self.send(VoiceStatus::SetSender(Some(source)))
    }

    pub fn stop(&mut self) {
        self.send(VoiceStatus::SetSender(None))
    }

    /// Switches the current connected voice channel to the given `channel_id`.
    ///
    /// This has 3 separate behaviors:
    ///
    /// - if the given `channel_id` is equivalent to the current connected
    ///   `channel_id`, then do nothing;
    /// - if the given `channel_id` is _not_ equivalent to the current connected
    ///   `channel_id`, then switch to the given `channel_id`;
    /// - if not currently connected to a voice channel, connect to the given
    /// one.
    ///
    /// **Note**: The given `channel_id`, if in a guild, _must_ be in the
    /// current handler's associated guild.
    ///
    /// If you are dealing with switching from one group to another, then open
    /// another handler, and optionally drop this one via [`Manager::remove`].
    ///
    /// [`Manager::remove`]: struct.Manager.html#method.remove
    pub fn switch_to(&mut self, channel_id: ChannelId) {
        match self.channel_id {
            Some(current_id) if current_id == channel_id => {
                // If already connected to the given channel, do nothing.
                return;
            },
            Some(_) => {
                self.channel_id = Some(channel_id);

                self.update();
            },
            None => {
                self.channel_id = Some(channel_id);

                self.connect();
            },
        }
    }

    fn connect(&self) {
        // Do _not_ try connecting if there is not at least a channel. There
        // does not _necessarily_ need to be a guild.
        if self.channel_id.is_none() {
            return;
        }

        self.update();
    }

    fn connect_with_data(&mut self, session_id: String, endpoint: String, token: String) {
        let target_id = if let Some(guild_id) = self.guild_id {
            guild_id.0
        } else if let Some(channel_id) = self.channel_id {
            channel_id.0
        } else {
            // Theoretically never happens? This needs to be researched more.
            error!("(╯°□°）╯︵ ┻━┻ No guild/channel ID when connecting");

            return;
        };

        let user_id = self.user_id;

        self.send(VoiceStatus::Connect(ConnectionInfo {
            endpoint: endpoint,
            session_id: session_id,
            target_id: target_id,
            token: token,
            user_id: user_id,
        }))
    }

    // Send an update for the current session.
    fn update(&self) {
        let map = ObjectBuilder::new()
            .insert("op", VoiceOpCode::SessionDescription.num())
            .insert_object("d", |o| o
                .insert("channel_id", self.channel_id.map(|c| c.0))
                .insert("guild_id", self.guild_id.map(|g| g.0))
                .insert("self_deaf", self.self_deaf)
                .insert("self_mute", self.self_mute))
            .build();

        let _ = self.ws.send(GatewayStatus::SendMessage(map));
    }

    fn send(&mut self, status: VoiceStatus) {
        // Restart thread if it errored.
        if let Err(mpsc::SendError(status)) = self.sender.send(status) {
            let (tx, rx) = mpsc::channel();

            self.sender = tx;
            self.sender.send(status).unwrap();

            threading::start(Target::Guild(self.guild_id.unwrap()), rx);

            self.update();
        }
    }

    /// You probably shouldn't use this if you're reading the source code.
    #[doc(hidden)]
    pub fn update_server(&mut self, endpoint: &Option<String>, token: &str) {
        if let Some(endpoint) = endpoint.clone() {
            let token = token.to_owned();

            if let Some(session_id) = self.session_id.clone() {
                self.connect_with_data(session_id, endpoint, token);
            } else {
                self.endpoint_token = Some((endpoint, token));
            }
        } else {
            self.leave();
        }
    }

    /// You probably shouldn't use this if you're reading the source code.
    #[doc(hidden)]
    pub fn update_state(&mut self, voice_state: &VoiceState) {
        if self.user_id != voice_state.user_id.0 {
            return;
        }

        self.channel_id = voice_state.channel_id;

        if voice_state.channel_id.is_some() {
            let session_id = voice_state.session_id.clone();

            match self.endpoint_token.take() {
                Some((endpoint, token)) => {
                    self.connect_with_data(session_id, endpoint, token);
                },
                None => {
                    self.session_id = Some(session_id);
                },
            }
        } else {
            self.leave();
        }
    }
}

impl Drop for Handler {
    /// Leaves the current connected voice channel, if connected to one, and
    /// forgets all configurations relevant to this Handler.
    fn drop(&mut self) {
        self.leave();
    }
}
