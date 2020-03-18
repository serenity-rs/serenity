use audiopus::{Bitrate, SampleRate};
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
};

pub const HEADER_LEN: usize = 12;
pub const SAMPLE_RATE: SampleRate = SampleRate::Hz48000;
pub const DEFAULT_BITRATE: Bitrate = Bitrate::BitsPerSecond(128_000);

/// A readable audio source.
pub trait AudioSource: Send {
    fn is_stereo(&mut self) -> bool;

    fn get_type(&self) -> AudioType;

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize>;

    fn read_opus_frame(&mut self) -> Option<Vec<u8>>;

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize>;

    fn is_seekable(&self) -> bool;
}

/// A receiver for incoming audio.
pub trait AudioReceiver: Send {
    fn speaking_update(&mut self, _ssrc: u32, _user_id: u64, _speaking: bool) { }

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

#[derive(Clone, Copy)]
pub enum AudioType {
    Opus,
    Pcm,
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access is
/// always guarded. In particular, you should expect to receive
/// a [`LockedAudio`] when calling [`Handler::play_returning`] or
/// [`Handler::play_only`].
///
/// # Example
///
/// ```rust,ignore
/// use serenity::voice::{Handler, LockedAudio, ffmpeg};
///
/// let handler: Handler = /* ... */;
/// let source = ffmpeg("../audio/my-favourite-song.mp3")?;
/// let safe_audio: LockedAudio = handler.play_only(source);
/// {
///     let audio_lock = safe_audio_control.clone();
///     let mut audio = audio_lock.lock();
///
///     audio.volume(0.5);
/// }
/// ```
///
/// [`LockedAudio`]: type.LockedAudio.html
/// [`Handler::play_only`]: struct.Handler.html#method.play_only
/// [`Handler::play_returning`]: struct.Handler.html#method.play_returning
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
    pub source: Box<dyn AudioSource>,

    /// The current position for playback.
    ///
    /// Consider the position fields **read-only** for now.
    pub position: Duration,
    pub position_modified: bool,


    /// List of events attached to this audio track.
    ///
    /// Currently, events are visited by linear scan for eligibility.
    /// This can likely be accelerated.
    pub events: EventStore,

    /// Channel from which commands are received.
    ///
    /// Audio commands are sent in this manner to ensure that access
    /// occurs in a thread-safe manner, without allowing any external
    /// code to lock access to audio objects and block packet generation.
    pub commands: Receiver<AudioCommand>,

    pub handle: AudioHandle,
}

impl Audio {
    pub fn new(source: Box<dyn AudioSource>, commands: Receiver<AudioCommand>, handle: AudioHandle) -> Self {
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

    /// Steps playback location forward by one frame.
    ///
    /// *Used internally*, although in future this might affect seek position.
    pub(crate) fn step_frame(&mut self) {
        self.position += Duration::from_millis(20);
        self.position_modified = false;
    }

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
                        Seek(_time) => unimplemented!(),
                        AddEvent(evt) => self.events.add_event(evt, self.position),
                        Do(action) => action(self),
                        Request(tx) => {let _ = tx.send(Box::new(self.get_state()));},
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

    pub fn get_state(&self) -> AudioState {
        AudioState {
            playing: self.playing,
            volume: self.volume,
            finished: self.finished,
            position: self.position,
        }
    }
}

pub fn create_player(source: Box<dyn AudioSource>) -> (Audio, AudioHandle) {
    let (tx, rx) = mpsc::channel();
    let can_seek = source.is_seekable();
    let player = Audio::new(source, rx, AudioHandle::new(tx.clone(), can_seek));

    (player, AudioHandle::new(tx, can_seek))
}

#[derive(Debug)]
pub struct AudioState {
    pub playing: bool,
    pub volume: f32,
    pub finished: bool,
    pub position: Duration,
}

pub type AudioResult = Result<(), SendError<AudioCommand>>;
pub type AudioQueryResult = Result<Receiver<Box<AudioState>>, SendError<AudioCommand>>;
pub type BlockingAudioQueryResult = Result<Box<AudioState>, SendError<AudioCommand>>;
pub type AudioFn = fn(&mut Audio) -> ();

#[derive(Clone, Debug)]
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

    pub fn play(&self) -> AudioResult {
        self.send(AudioCommand::Play)
    }

    pub fn pause(&self) -> AudioResult {
        self.send(AudioCommand::Pause)
    }

    pub fn stop(&self) -> AudioResult {
        self.send(AudioCommand::Stop)
    }

    pub fn set_volume(&self, volume: f32) -> AudioResult {
        self.send(AudioCommand::Volume(volume))
    }

    pub fn seek(&self, position: Duration) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Seek(position))
        } else {
            Err(SendError(AudioCommand::Seek(position)))
        }
    }

    pub fn add_event<F>(&self, event: Event, action: F) -> AudioResult 
        where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
    {
        self.send(AudioCommand::AddEvent(EventData::new(event, action)))
    }

    /// Warn user of taking too much time here...
    pub fn action(&self, action: AudioFn) -> AudioResult {
        self.send(AudioCommand::Do(action))
    }

    pub fn get_info(&self) -> AudioQueryResult {
        let (tx, rx) = mpsc::channel();
        self.send(AudioCommand::Request(tx))
            .map(move |_| rx)
    }

    pub fn get_info_blocking(&self) -> BlockingAudioQueryResult {
        // FIXME: combine into audio error type.
        self.get_info()
            .map(|c| c.recv().unwrap())
    }

    #[inline]
    pub fn send(&self, cmd: AudioCommand) -> AudioResult {
        self.command_channel.send(cmd)
    }
}

pub enum AudioCommand {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Seek(Duration),
    AddEvent(EventData),
    Do(AudioFn),
    Request(Sender<Box<AudioState>>),
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
        })
    }
}

pub enum PlayMode {
    Play,
    Pause,
    Stop,
}

pub struct TrackQueue {
    inner: Arc<Mutex<TrackQueueCore>>,
}

pub struct TrackQueueCore {
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

    pub fn add_source(&self, source: Box<dyn AudioSource>, handler: &mut Handler) {
        let (audio, audio_handle) = create_player(source);
        self.add_track(audio, audio_handle, handler);
    }

    pub fn add_track(&self, mut audio: Audio, audio_handle: AudioHandle, handler: &mut Handler) {
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

    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    pub fn pause(&self) -> AudioResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.pause()
        } else {
            Ok(())
        }
    }

    pub fn resume(&self) -> AudioResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.play()
        } else {
            Ok(())
        }
    }

    pub fn stop(&self) -> AudioResult {
        let mut inner = self.inner.lock();

        let out = if let Some(handle) = inner.tracks.front() {
            handle.stop()
        } else { Ok(()) };

        inner.tracks.clear();

        out
    }
}

impl Default for TrackQueue {
    fn default() -> Self {
        Self::new()
    }
}