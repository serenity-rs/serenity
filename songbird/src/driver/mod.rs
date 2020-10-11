mod config;
mod connection;
mod crypto;
pub(crate) mod tasks;

pub use config::Config;
pub use connection::error::Error;
pub use crypto::Mode as CryptoMode;

use crate::{
    events::EventData,
    input::Input,
    tracks::{Track, TrackHandle},
    ConnectionInfo,
    Event,
    EventHandler,
};
use audiopus::Bitrate;
use flume::{Receiver, SendError, Sender};
use tasks::message::CoreMessage;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Driver {
    config: Config,
    self_mute: bool,
    sender: Sender<CoreMessage>,
}

impl Driver {
    /// Creates a new voice driver.
    ///
    /// This will create the core voice tasks in the background.
    #[inline]
    pub fn new(config: Config) -> Self {
        let sender = Self::start_inner();

        Driver {
            config,
            self_mute: false,
            sender,
        }
    }

    fn start_inner() -> Sender<CoreMessage> {
        let (tx, rx) = flume::unbounded();

        // FIXME: excise guild id.
        tasks::start(serenity_voice_model::id::GuildId(0), rx, tx.clone());

        tx
    }

    fn restart_inner(&mut self) {
        self.sender = Self::start_inner();

        self.mute(self.self_mute);
    }

    /// Connects to a voice channel using the specified server.
    pub fn connect(&mut self, info: ConnectionInfo) -> Receiver<Result<(), Error>> {
        let (tx, rx) = flume::bounded(1);

        self.raw_connect(info, tx);

        rx
    }

    /// Connects to a voice channel using the specified server.
    pub(crate) fn raw_connect(&mut self, info: ConnectionInfo, tx: Sender<Result<(), Error>>) {
        self.send(CoreMessage::ConnectWithResult(info, tx));
    }

    /// Leaves the current voice channel, disconnecting from it.
    ///
    /// This does *not* forget settings, like whether to be self-deafened or
    /// self-muted.
    pub fn leave(&mut self) {
        self.send(CoreMessage::Disconnect);
    }

    /// Sets whether the current connection is to be muted.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    pub fn mute(&mut self, mute: bool) {
        self.self_mute = mute;
        self.send(CoreMessage::Mute(mute));
    }

    /// Returns whether the driver is muted (i.e., processes audio internally
    /// but submits none).
    pub fn is_mute(&self) -> bool {
        self.self_mute
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
    pub fn stop(&mut self) {
        self.send(CoreMessage::SetTrack(None))
    }

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

    /// Sends a message to the inner tasks, restarting it if necessary.
    fn send(&mut self, status: CoreMessage) {
        // Restart thread if it errored.
        if let Err(SendError(status)) = self.sender.send(status) {
            self.restart_inner();

            self.sender.send(status).unwrap();
        }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new(Default::default())
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
