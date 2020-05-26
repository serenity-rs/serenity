//! Live, controllable audio instances.

mod queue;

use crate::{
	voice::{
		events::{
	        Event,
	        EventContext,
	        EventData,
	        EventStore,
	    },
	    input::Input,
	},
};
use std::{
    sync::{
        mpsc::{
            self,
            Receiver,
            SendError,
            Sender,
            TryRecvError,
        },
    },
    time::Duration,
};

pub use queue::*;

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access from user code is
/// almost always guarded via an [`TrackHandle`]. You should expect to receive
/// access to a raw object of this type via [`voice::create_player`], for use in
/// [`Handler::play`] or [`Handler::play_only`].
///
/// # Example
///
/// ```rust,ignore
/// use serenity::voice::{Handler, ffmpeg, create_player};
///
/// let handler: Handler = /* ... */;
/// let source = ffmpeg("../audio/my-favourite-song.mp3")?;
/// let (audio, audio_handle) = create_player(source);
///
/// audio.volume(0.5); 
///
/// handler.play_only(audio);
///
/// // Future access occurs via audio_handle.
/// ```
///
/// [`Handler::play_only`]: struct.Handler.html#method.play_only
/// [`Handler::play`]: struct.Handler.html#method.play
/// [`TrackHandle`]: struct.TrackHandle.html
/// [`voice::create_player`]: fn.create_player.html
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
    pub volume: f32,

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
    pub events: EventStore,

    /// Channel from which commands are received.
    ///
    /// Track commands are sent in this manner to ensure that access
    /// occurs in a thread-safe manner, without allowing any external
    /// code to lock access to audio objects and block packet generation.
    pub(crate) commands: Receiver<TrackCommand>,

    /// Handle for safe control of this audio track from other threads.
    ///
    /// Typically, this is used by internal code to supply context information
    /// to event handlers, though more may be cloned from this handle.
    pub handle: TrackHandle,

    /// Count of remaining loops.
    pub loops: LoopState,
}

impl Track {
    pub fn new(source: Input, commands: Receiver<TrackCommand>, handle: TrackHandle) -> Self {
        Self {
            playing: Default::default(),
            volume: 1.0,
            source,
            position: Default::default(),
            play_time: Default::default(),
            events: EventStore::new(),
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
        self.position += Duration::from_millis(20);
        self.play_time += Duration::from_millis(20);
    }

    /// Receives and acts upon any commands forwarded by [`TrackHandle`]s.
    ///
    /// *Used internally*, this should not be exposed to users.
    ///
    /// [`TrackHandle`]: struct.TrackHandle.html
    pub(crate) fn process_commands(&mut self) {
        // Note: disconnection and an empty channel are both valid,
        // and should allow the audio object to keep running as intended.
        //
        // However, a paused and disconnected stream may never be revived,
        // (i.e., if a later event would unpause it, there would still be a handle).
        // We should then delete it TODO.
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => {
                    use TrackCommand::*;
                    match cmd {
                        Play => {self.play();},
                        Pause => {self.pause();},
                        Stop => {self.stop();},
                        Volume(vol) => {self.set_volume(vol);},
                        Seek(time) => {self.seek_time(time);},
                        AddEvent(evt) => self.events.add_event(evt, self.position),
                        Do(action) => action(self),
                        Request(tx) => {let _ = tx.send(Box::new(self.state()));},
                        Loop(loops) => {self.set_loops(loops);},
                    }
                },
                Err(TryRecvError::Disconnected) => {
                    // if self.playing {
                    //     self.finished = true;
                    // }

                    // TODO: issue with keeping the track handle in the struct...
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
    /// threads in response to an [`TrackHandle`].
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

/// Creates an [`Track`] object to pass into the audio context, and an [`TrackHandle`]
/// for safe, lock-free access in external code.
///
/// Typically, this would be used if you wished to directly work on or configure
/// the [`Track`] object before it is passed over to the audio mixing, transmission,
/// and event handling tasks.
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle`]: struct.TrackHandle.html
pub fn create_player(source: Input) -> (Track, TrackHandle) {
    let (tx, rx) = mpsc::channel();
    let can_seek = source.is_seekable();
    let player = Track::new(source, rx, TrackHandle::new(tx.clone(), can_seek));

    (player, TrackHandle::new(tx, can_seek))
}

/// State of an [`Track`] object, designed to be passed to event handlers
/// and retrieved remotely via [`TrackHandle::get_info`] or
/// [`TrackHandle::get_info_blocking`].
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle::get_info`]: struct.TrackHandle.html#method.get_info
/// [`TrackHandle::get_info_blocking`]: struct.TrackHandle.html#method.get_info_blocking
#[derive(Debug, Default)]
pub struct TrackState {
    pub playing: PlayMode,
    pub volume: f32,
    pub position: Duration,
    pub play_time: Duration,
    pub loops: LoopState,
}

/// Alias for most result-free calls to an [`TrackHandle`].
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
pub type TrackQueryResult = Result<Receiver<Box<TrackState>>, SendError<TrackCommand>>;

/// Alias for return value from calls to [`TrackHandle::get_info_blocking`].
///
/// Crucially, the audio thread will respond *at a later time*:
/// in ordinary use, this **will block for up to 20ms**.
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`TrackHandle::get_info_blocking`]: struct.TrackHandle.html#method.get_info_blocking
pub type BlockingTrackQueryResult = Result<Box<TrackState>, SendError<TrackCommand>>;

pub type TrackFn = fn(&mut Track) -> ();

#[derive(Clone, Debug)]
/// Handle for safe control of an [`Track`] track from other threads, outside
/// of the audio mixing and voice handling context.
///
/// Almost all method calls here are fallible; in most cases, this will be because
/// the underlying [`Track`] object has been discarded. Those which aren't refer
/// to immutable properties of the underlying stream.
///
/// [`Track`]: struct.Track.html
pub struct TrackHandle {
    command_channel: Sender<TrackCommand>,
    seekable: bool,
}

impl TrackHandle {
    pub fn new(command_channel: Sender<TrackCommand>, seekable: bool) -> Self {
        Self {
            command_channel,
            seekable,
        }
    }

    /// Unpauses an audio track.
    pub fn play(&self) -> TrackResult {
        self.send(TrackCommand::Play)
    }

    /// Pauses an audio track.
    pub fn pause(&self) -> TrackResult {
        self.send(TrackCommand::Pause)
    }

    /// Stops an audio track.
    ///
    /// This is *final*, and will cause the audio context to fire
    /// a [`TrackEvent::End`] event.
    ///
    /// [`TrackEvent::End`]: ../events/enum.TrackEvent.html#variant.End
    pub fn stop(&self) -> TrackResult {
        self.send(TrackCommand::Stop)
    }

    /// Sets the volume of an audio track.
    pub fn set_volume(&self, volume: f32) -> TrackResult {
        self.send(TrackCommand::Volume(volume))
    }

    /// Denotes whether the underlying [`TrackSource`] stream is compatible with arbitrary seeking.
    ///
    /// If this returns `false`, all calls to [`seek`] will fail, and the track is
    /// incapable of looping.
    ///
    /// [`seek`]: #method.seek
    /// [`TrackSource`]: trait.TrackSource.html
    pub fn is_seekable(&self) -> bool {
        self.seekable
    }

    /// Seeks along the track to the specified position.
    ///
    /// If the underlying [`TrackSource`] does not support this behaviour,
    /// then all calls will fail.
    ///
    /// [`TrackSource`]: trait.TrackSource.html
    pub fn seek_time(&self, position: Duration) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Seek(position))
        } else {
            Err(SendError(TrackCommand::Seek(position)))
        }
    }

    /// Attach an event handler to an audio track. These will receive [`EventContext::Track`].
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: struct.Track.html
    /// [`EventContext::Track`]: enum.EventContext.html#variant.Track
    pub fn add_event<F>(&self, event: Event, action: F) -> TrackResult 
        where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
    {
        self.send(TrackCommand::AddEvent(EventData::new(event, action)))
    }

    /// Perform an arbitrary action on a raw [`Track`] object.
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: struct.Track.html
    pub fn action(&self, action: TrackFn) -> TrackResult {
        self.send(TrackCommand::Do(action))
    }

    /// Request playback information and state from the audio context.
    ///
    /// Crucially, the audio thread will respond *at a later time*:
    /// It is up to the user when or how this should be read from the returned channel.
    pub fn get_info(&self) -> TrackQueryResult {
        let (tx, rx) = mpsc::channel();
        self.send(TrackCommand::Request(tx))
            .map(move |_| rx)
    }

    /// Request playback information and state from the audio context, blocking the current
    /// thread until an answer is received.
    ///
    /// Crucially, the audio thread will respond *at a later time*:
    /// in ordinary use, this may block for up to 20ms.
    pub fn get_info_blocking(&self) -> BlockingTrackQueryResult {
        // FIXME: combine into audio error type.
        self.get_info()
            .map(|c| c.recv().unwrap())
    }

    // Set an audio track to loop indefinitely.
    pub fn enable_loop(&self) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Infinite))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Infinite)))
        }
    }

    // Set an audio track to no longer loop.
    pub fn disable_loop(&self) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Finite(0)))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Finite(0))))
        }
    }

    // Set an audio track to loop a set number of times.
    pub fn loop_for(&self, count: usize) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Finite(count)))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Finite(count))))
        }
    }

    #[inline]
    /// Send a raw command to the [`Track`] object.
    ///
    /// [`Track`]: struct.Track.html
    pub fn send(&self, cmd: TrackCommand) -> TrackResult {
        self.command_channel.send(cmd)
    }
}

#[derive(Clone, Copy, Debug)]
/// Looping behaviour for a [`Track`].
///
/// [`Track`]: struct.Track.html
pub enum LoopState {
	/// Track will loop endlessly until loop state is changed or
	/// manually stopped.
    Infinite,

    /// Track will loop `n` more times.
    ///
    /// `Finite(0)` is the `Default`, stopping the track once its [`Input`] ends.
    ///
    /// [`Input`]: ../input/struct.Input.html
    Finite(usize),
}

impl Default for LoopState {
    fn default() -> Self {
        Self::Finite(0)
    }
}

/// A request from external code using an [`TrackHandle`] to modify
/// or act upon an [`Track`] object.
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle`]: struct.TrackHandle.html
pub enum TrackCommand {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Seek(Duration),
    AddEvent(EventData),
    Do(TrackFn),
    Request(Sender<Box<TrackState>>),
    Loop(LoopState),
}

impl std::fmt::Debug for TrackCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        use TrackCommand::*;
        write!(f, "TrackCommand::{}", match self {
            Play => "Play".to_string(),
            Pause => "Pause".to_string(),
            Stop => "Stop".to_string(),
            Volume(vol) => format!("Volume({})", vol),
            Seek(d) => format!("Seek({:?})", d),
            AddEvent(evt) => format!("AddEvent({:?})", evt),
            Do(_f) => "Do([function])".to_string(),
            Request(tx) => format!("Request({:?})", tx),
            Loop(loops) => format!("Loop({:?})", loops),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Playback status of a track.
pub enum PlayMode {
	/// The track is currently playing.
    Play,

    /// The track is currently paused, and may be resumed.
    Pause,

    /// The track has been manually stopped, and cannot be restarted.
    Stop,

    /// The track has naturally ended, and cannot be restarted.
    End,
}

impl PlayMode {
	/// Returns whether 
	pub fn is_done(self) -> bool {
		matches!(self, PlayMode::Stop | PlayMode::End)
	}

	fn change_to(self, other: Self) -> PlayMode {
		use PlayMode::*;

		// Idea: a finished track cannot be restarted -- this action is final.
		// We may want to change this in future so that seekable tracks can uncancel
		// themselves, perhaps, but this requires a bit more machinery to readd...
		match self {
			Play | Pause => other,
			a@_ => a,
		}
	}
}

impl Default for PlayMode {
	fn default() -> Self {
		PlayMode::Play
	}
}
