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
    pub playing: bool,

    /// The desired volume for playback.
    ///
    /// Sensible values fall between `0.0` and `1.0`.
    ///
    /// Can be controlled with [`volume`] if chaining is desired.
    ///
    /// [`volume`]: #method.volume
    pub volume: f32,

    /// Whether or not the sound has finished, or reached the end of its stream.
    ///
    /// ***Read-only*** for now.
    pub finished: bool,

    /// Underlying data access object.
    ///
    /// *Calling code is not expected to use this.*
    pub source: Input,

    /// The current position for playback.
    ///
    /// Consider the position fields **read-only** for now.
    pub position: Duration,
    pub position_modified: bool,


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
    pub commands: Receiver<TrackCommand>,

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
            playing: true,
            volume: 1.0,
            finished: false,
            source,
            position: Duration::new(0, 0),
            position_modified: false,
            events: EventStore::new(),
            commands,
            handle,
            loops: LoopState::Finite(0),
        }
    }

    /// Sets [`playing`] to `true` in a manner that allows method chaining.
    ///
    /// [`playing`]: #structfield.playing
    pub fn play(&mut self) -> &mut Self {
        self.playing = true;

        self
    }

    /// Sets [`playing`] to `false` in a manner that allows method chaining.
    ///
    /// [`playing`]: #structfield.playing
    pub fn pause(&mut self) -> &mut Self {
        self.playing = false;

        self
    }

    /// Sets [`finished`] to `true` in a manner that allows method chaining.
    ///
    /// This will cause the audio track to be removed, with any relevant events triggered.
    ///
    /// [`playing`]: #structfield.playing
    pub fn stop(&mut self) -> &mut Self {
        self.finished = true;

        self
    }

    /// Sets [`volume`] in a manner that allows method chaining.
    ///
    /// [`volume`]: #structfield.volume
    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;

        self
    }

    /// Change the position in the stream for subsequent playback.
    ///
    /// Currently a No-op.
    pub fn position(&mut self, position: Duration) -> &mut Self {
        self.position = position;
        self.position_modified = true;

        self
    }

    /// Sets [`loops`] in a manner that allows method chaining.
    ///
    /// [`loops`]: #structfield.loops
    pub fn loops(&mut self, loops: LoopState) -> &mut Self {
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
    ///
    /// *Used internally*, although in future this might affect seek position.
    pub(crate) fn step_frame(&mut self) {
        self.position += Duration::from_millis(20);
        self.position_modified = false;
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
        // However, a paused and disconnected stream MUST be stopped
        // to prevent resource leakage.
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => {
                    use TrackCommand::*;
                    match cmd {
                        Play => {self.play();},
                        Pause => {self.pause();},
                        Stop => {self.stop();},
                        Volume(vol) => {self.volume(vol);},
                        Seek(time) => {self.seek_time(time);},
                        AddEvent(evt) => self.events.add_event(evt, self.position),
                        Do(action) => action(self),
                        Request(tx) => {let _ = tx.send(Box::new(self.get_state()));},
                        Loop(loops) => {self.loops(loops);},
                    }
                },
                Err(TryRecvError::Disconnected) => {
                    if !self.playing {
                        self.finished = true;
                    }
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
    pub fn get_state(&self) -> TrackState {
        TrackState {
            playing: self.playing,
            volume: self.volume,
            finished: self.finished,
            position: self.position,
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
    pub playing: bool,
    pub volume: f32,
    pub finished: bool,
    pub position: Duration,
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
    /// [`TrackEvent::End`]: enum.TrackEvent.html#variant.End
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
    pub fn seek(&self, position: Duration) -> TrackResult {
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
pub enum LoopState {
    Infinite,
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

#[derive(Clone, Copy, Debug)]
pub enum PlayMode {
    Play,
    Pause,
    Stop,
}
