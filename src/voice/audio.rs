use parking_lot::Mutex;
use audiopus::{Bitrate, SampleRate};
use std::{
    sync::Arc,
    time::Duration,
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
/// let safe_audio: LockedAudio = handler.play_only();
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
}

impl Audio {
    pub fn new(source: Box<dyn AudioSource>) -> Self {
        Self {
            playing: true,
            volume: 1.0,
            finished: false,
            source,
            position: Duration::new(0, 0),
            position_modified: false,
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

}

/// Threadsafe form of an instance of the [`Audio`] struct, locked behind a
/// Mutex.
///
/// [`Audio`]: struct.Audio.html
pub type LockedAudio = Arc<Mutex<Audio>>;
