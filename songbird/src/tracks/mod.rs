//! Live, controllable audio instances.
//!
//! Tracks add control and event data around the bytestreams offered by [`Input`],
//! where each represents a live audio source inside of the driver's mixer.
//!
//! To prevent locking and stalling of the driver, tracks are controlled from your bot using a
//! [`TrackHandle`]. These handles remotely send commands from your bot's (a)sync
//! context to control playback, register events, and execute synchronous closures.
//!
//! If you want a new track from an [`Input`], i.e., for direct control before
//! playing your source on the driver, use [`create_player`].
//!
//! [`Input`]: ../input/struct.Input.html
//! [`TrackHandle`]: struct.TrackHandle.html
//! [`create_player`]: fn.create_player.html

mod command;
mod handle;
mod looping;
mod mode;
mod queue;
mod state;

pub use self::{command::*, handle::*, looping::*, mode::*, queue::*, state::*};

use crate::{constants::*, driver::tasks::message::*, events::EventStore, input::Input};
use std::time::Duration;
use tokio::sync::{
    mpsc::{
        self,
        error::{SendError, TryRecvError},
        UnboundedReceiver,
    },
    oneshot::Receiver as OneshotReceiver,
};

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access from user code is
/// almost always guarded via a [`TrackHandle`]. You should expect to receive
/// access to a raw object of this type via [`create_player`], for use in
/// [`Driver::play`] or [`Driver::play_only`].
///
/// # Example
///
/// ```rust,no_run
/// use songbird::{driver::Driver, ffmpeg, tracks::create_player};
///
/// # async {
/// // A Call is also valid here!
/// let mut handler: Driver = Default::default();
/// let source = ffmpeg("../audio/my-favourite-song.mp3")
///     .await
///     .expect("This might fail: handle this error!");
/// let (mut audio, audio_handle) = create_player(source);
///
/// audio.set_volume(0.5);
///
/// handler.play_only(audio);
///
/// // Future access occurs via audio_handle.
/// # };
/// ```
///
/// [`Driver::play_only`]: ../struct.Driver.html#method.play_only
/// [`Driver::play`]: ../struct.Driver.html#method.play
/// [`TrackHandle`]: struct.TrackHandle.html
/// [`create_player`]: fn.create_player.html
#[derive(Debug)]
pub struct Track {
    /// Whether or not this sound is currently playing.
    ///
    /// Can be controlled with [`play`] or [`pause`] if chaining is desired.
    ///
    /// [`play`]: #method.play
    /// [`pause`]: #method.pause
    pub(crate) playing: PlayMode,

    /// The desired volume for playback.
    ///
    /// Sensible values fall between `0.0` and `1.0`.
    ///
    /// Can be controlled with [`volume`] if chaining is desired.
    ///
    /// [`volume`]: #method.volume
    pub(crate) volume: f32,

    /// Underlying data access object.
    ///
    /// *Calling code is not expected to use this.*
    pub(crate) source: Input,

    /// The current playback position in the track.
    pub(crate) position: Duration,

    /// The total length of time this track has been active.
    pub(crate) play_time: Duration,

    /// List of events attached to this audio track.
    ///
    /// This may be used to add additional events to a track
    /// before it is sent to the audio context for playing.
    pub events: Option<EventStore>,

    /// Channel from which commands are received.
    ///
    /// Track commands are sent in this manner to ensure that access
    /// occurs in a thread-safe manner, without allowing any external
    /// code to lock access to audio objects and block packet generation.
    pub(crate) commands: UnboundedReceiver<TrackCommand>,

    /// Handle for safe control of this audio track from other threads.
    ///
    /// Typically, this is used by internal code to supply context information
    /// to event handlers, though more may be cloned from this handle.
    pub handle: TrackHandle,

    /// Count of remaining loops.
    pub loops: LoopState,
}

impl Track {
    /// Create a new track directly from an input, command source,
    /// and handle.
    ///
    /// In general, you should probably use [`create_player`].
    ///
    /// [`create_player`]: fn.create_player.html
    pub fn new_raw(
        source: Input,
        commands: UnboundedReceiver<TrackCommand>,
        handle: TrackHandle,
    ) -> Self {
        Self {
            playing: Default::default(),
            volume: 1.0,
            source,
            position: Default::default(),
            play_time: Default::default(),
            events: Some(EventStore::new_local()),
            commands,
            handle,
            loops: LoopState::Finite(0),
        }
    }

    /// Sets a track to playing if it is paused.
    pub fn play(&mut self) -> &mut Self {
        self.set_playing(PlayMode::Play)
    }

    /// Pauses a track if it is playing.
    pub fn pause(&mut self) -> &mut Self {
        self.set_playing(PlayMode::Pause)
    }

    /// Manually stops a track.
    ///
    /// This will cause the audio track to be removed, with any relevant events triggered.
    /// Stopped/ended tracks cannot be restarted.
    pub fn stop(&mut self) -> &mut Self {
        self.set_playing(PlayMode::Stop)
    }

    pub(crate) fn end(&mut self) -> &mut Self {
        self.set_playing(PlayMode::End)
    }

    #[inline]
    fn set_playing(&mut self, new_state: PlayMode) -> &mut Self {
        self.playing = self.playing.change_to(new_state);

        self
    }

    /// Returns the current play status of this track.
    pub fn playing(&self) -> PlayMode {
        self.playing
    }

    /// Sets [`volume`] in a manner that allows method chaining.
    ///
    /// [`volume`]: #structfield.volume
    pub fn set_volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;

        self
    }

    /// Returns the current playback position.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Returns the current playback position.
    pub fn position(&self) -> Duration {
        self.position
    }

    /// Returns the total length of time this track has been active.
    pub fn play_time(&self) -> Duration {
        self.play_time
    }

    /// Sets [`loops`] in a manner that allows method chaining.
    ///
    /// [`loops`]: #structfield.loops
    pub fn set_loops(&mut self, loops: LoopState) -> &mut Self {
        self.loops = loops;
        self
    }

    pub(crate) fn do_loop(&mut self) -> bool {
        match self.loops {
            LoopState::Infinite => true,
            LoopState::Finite(0) => false,
            LoopState::Finite(ref mut n) => {
                *n -= 1;
                true
            },
        }
    }

    /// Steps playback location forward by one frame.
    pub(crate) fn step_frame(&mut self) {
        self.position += TIMESTEP_LENGTH;
        self.play_time += TIMESTEP_LENGTH;
    }

    /// Receives and acts upon any commands forwarded by [`TrackHandle`]s.
    ///
    /// *Used internally*, this should not be exposed to users.
    ///
    /// [`TrackHandle`]: struct.TrackHandle.html
    pub(crate) fn process_commands(&mut self, index: usize, ic: &Interconnect) {
        // Note: disconnection and an empty channel are both valid,
        // and should allow the audio object to keep running as intended.

        // Note that interconnect failures are not currently errors.
        // In correct operation, the event thread should never panic,
        // but it receiving status updates is secondary do actually
        // doing the work.
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => {
                    use TrackCommand::*;
                    match cmd {
                        Play => {
                            self.play();
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Mode(self.playing),
                            ));
                        },
                        Pause => {
                            self.pause();
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Mode(self.playing),
                            ));
                        },
                        Stop => {
                            self.stop();
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Mode(self.playing),
                            ));
                        },
                        Volume(vol) => {
                            self.set_volume(vol);
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Volume(self.volume),
                            ));
                        },
                        Seek(time) => {
                            self.seek_time(time);
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Position(self.position),
                            ));
                        },
                        AddEvent(evt) => {
                            let _ = ic.events.send(EventMessage::AddTrackEvent(index, evt));
                        },
                        Do(action) => {
                            action(self);
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Total(self.state()),
                            ));
                        },
                        Request(tx) => {
                            let _ = tx.send(Box::new(self.state()));
                        },
                        Loop(loops) => {
                            self.set_loops(loops);
                            let _ = ic.events.send(EventMessage::ChangeState(
                                index,
                                TrackStateChange::Loops(self.loops, true),
                            ));
                        },
                    }
                },
                Err(TryRecvError::Closed) => {
                    // this branch will never be visited.
                    break;
                },
                Err(TryRecvError::Empty) => {
                    break;
                },
            }
        }
    }

    /// Creates a read-only copy of the audio track's state.
    ///
    /// The primary use-case of this is sending information across
    /// threads in response to a [`TrackHandle`].
    ///
    /// [`TrackHandle`]: struct.TrackHandle.html
    pub fn state(&self) -> TrackState {
        TrackState {
            playing: self.playing,
            volume: self.volume,
            position: self.position,
            play_time: self.play_time,
            loops: self.loops,
        }
    }

    /// Seek to a specific point in the track.
    ///
    /// Returns `None` if unsupported.
    pub fn seek_time(&mut self, pos: Duration) -> Option<Duration> {
        let out = self.source.seek_time(pos);

        if let Some(t) = out {
            self.position = t;
        }

        out
    }
}

/// Creates a [`Track`] object to pass into the audio context, and a [`TrackHandle`]
/// for safe, lock-free access in external code.
///
/// Typically, this would be used if you wished to directly work on or configure
/// the [`Track`] object before it is passed over to the driver.
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle`]: struct.TrackHandle.html
pub fn create_player(source: Input) -> (Track, TrackHandle) {
    let (tx, rx) = mpsc::unbounded_channel();
    let can_seek = source.is_seekable();
    let player = Track::new_raw(source, rx, TrackHandle::new(tx.clone(), can_seek));

    (player, TrackHandle::new(tx, can_seek))
}

/// Alias for most result-free calls to a [`TrackHandle`].
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`TrackHandle`]: struct.TrackHandle.html
pub type TrackResult = Result<(), SendError<TrackCommand>>;

/// Alias for return value from calls to [`TrackHandle::get_info`].
///
/// Crucially, the audio thread will respond *at a later time*:
/// It is up to the user when or how this should be read from the returned channel.
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`TrackHandle::get_info`]: struct.TrackHandle.html#method.get_info
pub type TrackQueryResult = Result<OneshotReceiver<Box<TrackState>>, SendError<TrackCommand>>;
