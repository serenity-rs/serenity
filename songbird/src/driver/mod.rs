mod connection;
mod crypto;
pub(crate) mod tasks;

pub use crypto::Mode as CryptoMode;

use audiopus::Bitrate;
use crate::{
    events::EventData,
    id::{ChannelId, GuildId, UserId},
    input::Input,
    tracks::{
        Track,
        TrackHandle,
    },
    ConnectionInfo,
    Event,
    EventHandler,
};
use flume::{
    SendError,
    Sender as FlumeSender,
};
use tasks::message::CoreMessage;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Driver {
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub guild_id: GuildId,
    pub self_deaf: bool,
    pub self_mute: bool,
    sender: FlumeSender<CoreMessage>,
    pub session_id: Option<String>,
    pub token: Option<String>,
    pub user_id: UserId,
}

impl Driver {
    /// Creates a new Handler.
    #[inline]
    pub fn new(
        guild_id: GuildId,
        user_id: UserId,
    ) -> Self {
        let (tx, rx) = flume::unbounded();

        tasks::start(guild_id.into(), rx, tx.clone());

        Driver {
            channel_id: None,
            endpoint: None,
            guild_id,
            self_deaf: false,
            self_mute: false,
            sender: tx,
            session_id: None,
            token: None,
            user_id,
        }
    }

    /// Connects to the voice channel if the following are present:
    ///
    /// - [`endpoint`]
    /// - [`session_id`]
    /// - [`token`]
    ///
    /// If they _are_ all present, then `true` is returned. Otherwise, `false`
    /// is.
    ///
    /// This will automatically be called by [`update_server`] or
    /// [`update_state`] when all three values become present.
    ///
    /// [`endpoint`]: #structfield.endpoint
    /// [`session_id`]: #structfield.session_id
    /// [`token`]: #structfield.token
    /// [`update_server`]: #method.update_server
    /// [`update_state`]: #method.update_state
    pub fn connect(&mut self) -> bool {
        if self.endpoint.is_none() || self.session_id.is_none() || self.token.is_none() {
            return false;
        }

        let endpoint = self.endpoint.clone().unwrap();
        let guild_id = self.guild_id;
        let session_id = self.session_id.clone().unwrap();
        let token = self.token.clone().unwrap();
        let user_id = self.user_id;

        // Safe as all of these being present was already checked.
        self.send(CoreMessage::Connect(ConnectionInfo {
            endpoint,
            guild_id: guild_id.into(),
            session_id,
            token,
            user_id: user_id.into(),
        }));

        true
    }

    /// Leaves the current voice channel, disconnecting from it.
    ///
    /// This does _not_ forget settings, like whether to be self-deafened or
    /// self-muted.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally connected to a
    /// voice channel.
    ///
    /// [`standalone`]: #method.standalone
    pub fn leave(&mut self) {
        // Only send an update if we were in a voice channel.
        if self.channel_id.is_some() {
            self.channel_id = None;
            self.send(CoreMessage::Disconnect);
        }
    }

    /// Sets whether the current connection is to be muted.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally muted.
    ///
    /// [`standalone`]: #method.standalone
    pub fn mute(&mut self, mute: bool) {
        self.self_mute = mute;

        if self.channel_id.is_some() {
            self.send(CoreMessage::Mute(mute));
        }
    }

    /// Plays audio from a source, returning a handle for further control.
    ///
    /// This can be a source created via [`voice::ffmpeg`] or [`voice::ytdl`].
    ///
    /// [`voice::ffmpeg`]: input/fn.ffmpeg.html
    /// [`voice::ytdl`]: input/fn.ytdl.html
    pub fn play_source(&mut self, source: Input) -> TrackHandle {
        let (player, handle) = super::create_player(source);
        self.send(CoreMessage::AddTrack(player));

        handle
    }

    /// Plays audio from a source, returning a handle for further control.
    ///
    /// Unlike [`play_only_source`], this stops all other sources attached
    /// to the channel.
    ///
    /// [`play_only_source`]: #method.play_only_source
    pub fn play_only_source(&mut self, source: Input) -> TrackHandle {
        let (player, handle) = super::create_player(source);
        self.send(CoreMessage::SetTrack(Some(player)));

        handle
    }

    /// Plays audio from a [`Track`] object.
    ///
    /// This will be one half of the return value of [`voice::create_player`].
    /// The main difference between this function and [`play_source`] is
    /// that this allows for direct manipulation of the [`Track`] object
    /// before it is passed over to the voice and mixing contexts.
    ///
    /// [`voice::create_player`]: tracks/fn.create_player.html
    /// [`Track`]: tracks/struct.Track.html
    /// [`play_source`]: #method.play_source
    pub fn play(&mut self, track: Track) {
        self.send(CoreMessage::AddTrack(track));
    }

    /// Exclusively plays audio from a [`Track`] object.
    ///
    /// This will be one half of the return value of [`voice::create_player`].
    /// As in [`play_only_source`], this stops all other sources attached to the
    /// channel. Like [`play`], however, this allows for direct manipulation of the
    /// [`Track`] object before it is passed over to the voice and mixing contexts.
    ///
    /// [`voice::create_player`]: tracks/fn.create_player.html
    /// [`Track`]: tracks/struct.Track.html
    /// [`play_only_source`]: #method.play_only_source
    /// [`play`]: #method.play
    pub fn play_only(&mut self, track: Track) {
        self.send(CoreMessage::SetTrack(Some(track)));
    }

    /// Sets the bitrate for encoding Opus packets sent along
    /// the channel being managed.
    ///
    /// The default rate is 128 kbps.
    /// Sensible values range between `Bits(512)` and `Bits(512_000)`
    /// bits per second.
    /// Alternatively, `Auto` and `Max` remain available.
    pub fn set_bitrate(&mut self, bitrate: Bitrate) {
        self.send(CoreMessage::SetBitrate(bitrate))
    }

    /// Stops playing audio from all sources, if any are set.
    pub fn stop(&mut self) { self.send(CoreMessage::SetTrack(None)) }

    /// Attach a global event handler to an audio context. Global events may receive
    /// any [`EventContext`].
    ///
    /// Global timing events will tick regardless of whether audio is playing,
    /// so long as the bot is connected to a voice channel, and have no tracks.
    /// [`TrackEvent`]s will respond to all relevant tracks, giving some audio elements.
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: tracks/struct.Track.html
    /// [`TrackEvent`]: events/enum.TrackEvent.html
    /// [`EventContext`]: events/enum.EventContext.html
    pub fn add_global_event<F: EventHandler + 'static>(&mut self, event: Event, action: F) {
        self.send(CoreMessage::AddEvent(EventData::new(event, action)));
    }

    /// Updates the voice server data.
    ///
    /// You should only need to use this if you initialized the `Handler` via
    /// [`standalone`].
    ///
    /// Refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    #[instrument(skip(self, token))]
    pub fn update_server(&mut self, endpoint: &Option<String>, token: &str) {
        self.token = Some(token.to_string());

        if let Some(endpoint) = endpoint.clone() {
            self.endpoint = Some(endpoint);

            if self.session_id.is_some() {
                self.connect();
            }
        } else {
            self.leave();
        }
    }

    /// Updates the internal voice state of the current user.
    ///
    /// You should only need to use this if you initialized the `Handler` via
    /// [`standalone`].
    ///
    /// refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    // pub fn update_state(&mut self, voice_state: &VoiceState) {
    //     if self.user_id != voice_state.user_id.into() {
    //         return;
    //     }

    //     self.channel_id = voice_state.channel_id.map(Into::into);

    //     if voice_state.channel_id.is_some() {
    //         self.session_id = Some(voice_state.session_id.clone());

    //         if self.endpoint.is_some() && self.token.is_some() {
    //             self.connect();
    //         }
    //     } else {
    //         self.leave();
    //     }
    // }

    // FIXME: rethink above. Pass in full COnnecitonInfo? Or require update of all?

    /// Sends a message to the inner tasks, restarting it if necessary.
    fn send(&mut self, status: CoreMessage) {
        // Restart thread if it errored.
        if let Err(SendError(status)) = self.sender.send(status) {
            let (tx, rx) = flume::unbounded();

            self.sender = tx.clone();
            self.sender.send(status).unwrap();

            tasks::start(self.guild_id.into(), rx, tx);
        }
    }
}

impl Drop for Driver {
    /// Leaves the current connected voice channel, if connected to one, and
    /// forgets all configurations relevant to this Handler.
    fn drop(&mut self) {
        self.leave();
        let _ = self.sender.send(CoreMessage::Poison);
    }
}
