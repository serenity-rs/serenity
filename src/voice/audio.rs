use audiopus::{
    coder::Decoder as OpusDecoder,
    Bitrate,
    SampleRate,
};
use crate::model::event::VoiceSpeakingState;
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    sync::{
        mpsc::{
            self,
            Receiver,
            SendError,
            Sender,
            TryRecvError,
        },
        Arc,
    },
    time::Duration,
};
use super::{
    events::{
        Event,
        EventContext,
        EventData,
        EventStore,
        TrackEvent,
    },
    Handler,
    input::Input,
};

pub const HEADER_LEN: usize = 12;
pub const SAMPLE_RATE: SampleRate = SampleRate::Hz48000;
pub const DEFAULT_BITRATE: Bitrate = Bitrate::BitsPerSecond(128_000);

/// A receiver for incoming audio.
pub trait AudioReceiver: Send {
    fn speaking_update(&mut self, _ssrc: u32, _user_id: u64, _speaking: VoiceSpeakingState) { }

    #[allow(clippy::too_many_arguments)]
    fn voice_packet(&mut self,
                    _ssrc: u32,
                    _sequence: u16,
                    _timestamp: u32,
                    _stereo: bool,
                    _data: &[i16],
                    _compressed_size: usize) { }

    fn client_connect(&mut self, _ssrc: u32, _user_id: u64) { }

    fn client_disconnect(&mut self, _user_id: u64) { }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum AudioType {
    Opus,
    Pcm,
    FloatPcm,
}

#[non_exhaustive]
pub enum AudioTypeData {
    Opus(OpusDecoder),
    Pcm,
    FloatPcm,
}

#[non_exhaustive]
pub enum AudioContainer {
    Raw,
    Dca1,
}

#[non_exhaustive]
pub enum AudioContainerData {
    Raw,
    Dca1(),
}

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access from user code is
/// almost always guarded via an [`AudioHandle`]. You should expect to receive
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
/// [`AudioHandle`]: struct.AudioHandle.html
/// [`voice::create_player`]: fn.create_player.html
pub struct Audio {

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
    /// Audio commands are sent in this manner to ensure that access
    /// occurs in a thread-safe manner, without allowing any external
    /// code to lock access to audio objects and block packet generation.
    pub commands: Receiver<AudioCommand>,

    /// Handle for safe control of this audio track from other threads.
    ///
    /// Typically, this is used by internal code to supply context information
    /// to event handlers, though more may be cloned from this handle.
    pub handle: AudioHandle,

    /// Count of remaining loops.
    pub loops: LoopState,
}

impl Audio {
    pub fn new(source: Input, commands: Receiver<AudioCommand>, handle: AudioHandle) -> Self {
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

    /// Receives and acts upon any commands forwarded by [`AudioHandle`]s.
    ///
    /// *Used internally*, this should not be exposed to users.
    ///
    /// [`AudioHandle`]: struct.AudioHandle.html
    pub(crate) fn process_commands(&mut self) {
        // Note: disconnection and an empty channel are both valid,
        // and should allow the audio object to keep running as intended.
        //
        // However, a paused and disconnected stream MUST be stopped
        // to prevent resource leakage.
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => {
                    use AudioCommand::*;
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
    /// threads in response to an [`AudioHandle`].
    ///
    /// [`AudioHandle`]: struct.AudioHandle.html
    pub fn get_state(&self) -> AudioState {
        AudioState {
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

/// Creates an [`Audio`] object to pass into the audio context, and an [`AudioHandle`]
/// for safe, lock-free access in external code.
///
/// Typically, this would be used if you wished to directly work on or configure
/// the [`Audio`] object before it is passed over to the audio mixing, transmission,
/// and event handling tasks.
///
/// [`Audio`]: struct.Audio.html
/// [`AudioHandle`]: struct.AudioHandle.html
pub fn create_player(source: Input) -> (Audio, AudioHandle) {
    let (tx, rx) = mpsc::channel();
    let can_seek = source.is_seekable();
    let player = Audio::new(source, rx, AudioHandle::new(tx.clone(), can_seek));

    (player, AudioHandle::new(tx, can_seek))
}

/// State of an [`Audio`] object, designed to be passed to event handlers
/// and retrieved remotely via [`AudioHandle::get_info`] or
/// [`AudioHandle::get_info_blocking`].
///
/// [`Audio`]: struct.Audio.html
/// [`AudioHandle::get_info`]: struct.AudioHandle.html#method.get_info
/// [`AudioHandle::get_info_blocking`]: struct.AudioHandle.html#method.get_info_blocking
#[derive(Debug, Default)]
pub struct AudioState {
    pub playing: bool,
    pub volume: f32,
    pub finished: bool,
    pub position: Duration,
    pub loops: LoopState,
}

/// Alias for most result-free calls to an [`AudioHandle`].
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`AudioHandle`]: struct.AudioHandle.html
pub type AudioResult = Result<(), SendError<AudioCommand>>;

/// Alias for return value from calls to [`AudioHandle::get_info`].
///
/// Crucially, the audio thread will respond *at a later time*:
/// It is up to the user when or how this should be read from the returned channel.
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`AudioHandle::get_info`]: struct.AudioHandle.html#method.get_info
pub type AudioQueryResult = Result<Receiver<Box<AudioState>>, SendError<AudioCommand>>;

/// Alias for return value from calls to [`AudioHandle::get_info_blocking`].
///
/// Crucially, the audio thread will respond *at a later time*:
/// in ordinary use, this **will block for up to 20ms**.
///
/// Failure indicates that the accessed audio object has been
/// removed or deleted by the audio context.
///
/// [`AudioHandle::get_info_blocking`]: struct.AudioHandle.html#method.get_info_blocking
pub type BlockingAudioQueryResult = Result<Box<AudioState>, SendError<AudioCommand>>;

pub type AudioFn = fn(&mut Audio) -> ();

#[derive(Clone, Debug)]
/// Handle for safe control of an [`Audio`] track from other threads, outside
/// of the audio mixing and voice handling context.
///
/// Almost all method calls here are fallible; in most cases, this will be because
/// the underlying [`Audio`] object has been discarded. Those which aren't refer
/// to immutable properties of the underlying stream.
///
/// [`Audio`]: struct.Audio.html
pub struct AudioHandle {
    command_channel: Sender<AudioCommand>,
    seekable: bool,
}

impl AudioHandle {
    pub fn new(command_channel: Sender<AudioCommand>, seekable: bool) -> Self {
        Self {
            command_channel,
            seekable,
        }
    }

    /// Unpauses an audio track.
    pub fn play(&self) -> AudioResult {
        self.send(AudioCommand::Play)
    }

    /// Pauses an audio track.
    pub fn pause(&self) -> AudioResult {
        self.send(AudioCommand::Pause)
    }

    /// Stops an audio track.
    ///
    /// This is *final*, and will cause the audio context to fire
    /// a [`TrackEvent::End`] event.
    ///
    /// [`TrackEvent::End`]: enum.TrackEvent.html#variant.End
    pub fn stop(&self) -> AudioResult {
        self.send(AudioCommand::Stop)
    }

    /// Sets the volume of an audio track.
    pub fn set_volume(&self, volume: f32) -> AudioResult {
        self.send(AudioCommand::Volume(volume))
    }

    /// Denotes whether the underlying [`AudioSource`] stream is compatible with arbitrary seeking.
    ///
    /// If this returns `false`, all calls to [`seek`] will fail, and the track is
    /// incapable of looping.
    ///
    /// [`seek`]: #method.seek
    /// [`AudioSource`]: trait.AudioSource.html
    pub fn is_seekable(&self) -> bool {
        self.seekable
    }

    /// Seeks along the track to the specified position.
    ///
    /// If the underlying [`AudioSource`] does not support this behaviour,
    /// then all calls will fail.
    ///
    /// [`AudioSource`]: trait.AudioSource.html
    pub fn seek(&self, position: Duration) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Seek(position))
        } else {
            Err(SendError(AudioCommand::Seek(position)))
        }
    }

    /// Attach an event handler to an audio track. These will receive [`EventContext::Track`].
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Audio`]: struct.Audio.html
    /// [`EventContext::Track`]: enum.EventContext.html#variant.Track
    pub fn add_event<F>(&self, event: Event, action: F) -> AudioResult 
        where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
    {
        self.send(AudioCommand::AddEvent(EventData::new(event, action)))
    }

    /// Perform an arbitrary action on a raw [`Audio`] object.
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Audio`]: struct.Audio.html
    pub fn action(&self, action: AudioFn) -> AudioResult {
        self.send(AudioCommand::Do(action))
    }

    /// Request playback information and state from the audio context.
    ///
    /// Crucially, the audio thread will respond *at a later time*:
    /// It is up to the user when or how this should be read from the returned channel.
    pub fn get_info(&self) -> AudioQueryResult {
        let (tx, rx) = mpsc::channel();
        self.send(AudioCommand::Request(tx))
            .map(move |_| rx)
    }

    /// Request playback information and state from the audio context, blocking the current
    /// thread until an answer is received.
    ///
    /// Crucially, the audio thread will respond *at a later time*:
    /// in ordinary use, this may block for up to 20ms.
    pub fn get_info_blocking(&self) -> BlockingAudioQueryResult {
        // FIXME: combine into audio error type.
        self.get_info()
            .map(|c| c.recv().unwrap())
    }

    // Set an audio track to loop indefinitely.
    pub fn enable_loop(&self) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Loop(LoopState::Infinite))
        } else {
            Err(SendError(AudioCommand::Loop(LoopState::Infinite)))
        }
    }

    // Set an audio track to no longer loop.
    pub fn disable_loop(&self) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Loop(LoopState::Finite(0)))
        } else {
            Err(SendError(AudioCommand::Loop(LoopState::Finite(0))))
        }
    }

    // Set an audio track to loop a set number of times.
    pub fn loop_for(&self, count: usize) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Loop(LoopState::Finite(count)))
        } else {
            Err(SendError(AudioCommand::Loop(LoopState::Finite(count))))
        }
    }

    #[inline]
    /// Send a raw command to the [`Audio`] object.
    ///
    /// [`Audio`]: struct.Audio.html
    pub fn send(&self, cmd: AudioCommand) -> AudioResult {
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

/// A request from external code using an [`AudioHandle`] to modify
/// or act upon an [`Audio`] object.
///
/// [`Audio`]: struct.Audio.html
/// [`AudioHandle`]: struct.AudioHandle.html
pub enum AudioCommand {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Seek(Duration),
    AddEvent(EventData),
    Do(AudioFn),
    Request(Sender<Box<AudioState>>),
    Loop(LoopState),
}

impl std::fmt::Debug for AudioCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        use AudioCommand::*;
        write!(f, "AudioCommand::{}", match self {
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

#[derive(Default)]
/// A simple queue for several audio sources, designed to
/// play in sequence.
///
/// This makes use of [`TrackEvent`]s to determine when the current
/// song or audio file has finished before playing the next entry.
///
/// `examples/13_voice_events` demonstrates how a user might manage,
/// track and use this to run a song queue in many guilds in parallel.
/// This code is trivial to extend if extra functionality is needed.
///
/// # Example
/// 
/// ```rust,ignore
/// use serenity::model::id::GuildId;
/// use serenity::voice::{Handler, LockedAudio, ffmpeg, create_player};
/// use std::collections::HashMap;
///
/// let mut manager: ClientVoiceManager = /* ... */;
/// let queues: HashMap<GuildId, TrackQueue> = Default::default();
///
/// let guild_id: GuildId = /* ... */;
///
/// let source = ffmpeg("../audio/my-favourite-song.mp3")?;
/// if let Some(handler) = manager.get_mut(guild_id) {
///     // We need to ensure that this guild has a TrackQueue created for it.
///     let queue = queues.entry(guild_id)
///         .or_default();
///
///     // Queueing a track is this easy!
///     queue.add_source(source, handler);
/// } else {
///     panic!("No voice manager for this guild!");
/// }
/// ```
///
/// [`TrackEvent`]: enum.TrackEvent.html
pub struct TrackQueue {
    inner: Arc<Mutex<TrackQueueCore>>,
}

#[derive(Default)]
/// Inner portion of a [`TrackQueue`].
///
/// This abstracts away thread-safety from the user,
/// and offers a convenient location to store further state if required.
///
/// [`TrackQueue`]: struct.TrackQueue.html
struct TrackQueueCore {
    tracks: VecDeque<AudioHandle>,
}

impl TrackQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TrackQueueCore {
                tracks: VecDeque::new(),
            }))
        }
    }

    /// Adds an audio source to the queue, to be played in the channel managed by `handler`.
    pub fn add_source(&self, source: Input, handler: &mut Handler) {
        let (audio, audio_handle) = create_player(source);
        self.add(audio, audio_handle, handler);
    }

    /// Adds an [`Audio`] object to the queue, to be played in the channel managed by `handler`.
    ///
    /// This is used with [`voice::create_player`] if additional configuration or event handlers
    /// are required before enqueueing the audio track.
    ///
    /// [`Audio`]: struct.Audio.html
    /// [`voice::create_player`]: fn.create_player.html
    pub fn add(&self, mut audio: Audio, audio_handle: AudioHandle, handler: &mut Handler) {
        let remote_lock = self.inner.clone();
        let mut inner = self.inner.lock();

        if !inner.tracks.is_empty() {
            audio.pause();
        }

        audio.events.add_event(
            EventData::new(
                Event::Track(TrackEvent::End),
                move |_ctx| {
                    let mut inner = remote_lock.lock();
                    let _old = inner.tracks.pop_front();

                    // If any audio files die unexpectedly, then keep going until we
                    // find one which works, or we run out.
                    let mut keep_looking = true;
                    while keep_looking && !inner.tracks.is_empty() {
                        if let Some(new) = inner.tracks.front() {
                            keep_looking = new.play().is_err();

                            // Discard files which cannot be used for whatever reason.
                            if keep_looking {
                                let _ = inner.tracks.pop_front();
                            }
                        }
                    }

                    None
                }),
            audio.position,
        );

        handler.play(audio);
        inner.tracks.push_back(audio_handle);
    }

    /// Returns the number of tracks currently in the queue.
    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    /// Returns whether there are no tracks currently in the queue.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    /// Pause the track at the head of the queue.
    pub fn pause(&self) -> AudioResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.pause()
        } else {
            Ok(())
        }
    }

    /// Resume the track at the head of the queue.
    pub fn resume(&self) -> AudioResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.play()
        } else {
            Ok(())
        }
    }

    /// Stop the currently playing track, and clears the queue.
    pub fn stop(&self) -> AudioResult {
        let mut inner = self.inner.lock();

        let out = inner.stop_current();

        inner.tracks.clear();

        out
    }

    /// Skip to the next track in the queue, if it exists.
    pub fn skip(&self) -> AudioResult {
        let mut inner = self.inner.lock();

        let out = inner.stop_current();

        let _ = inner.tracks.pop_front();

        out
    }
}

impl TrackQueueCore {
    /// Skip to the next track in the queue, if it exists.
    fn stop_current(&self) -> AudioResult {
        if let Some(handle) = self.tracks.front() {
            handle.stop()
        } else { Ok(()) }
    }
}
