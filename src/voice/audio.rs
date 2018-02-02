use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub const HEADER_LEN: usize = 12;
pub const SAMPLE_RATE: u32 = 48_000;

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
    fn speaking_update(&mut self, ssrc: u32, user_id: u64, speaking: bool);

    fn voice_packet(&mut self,
                    ssrc: u32,
                    sequence: u16,
                    timestamp: u32,
                    stereo: bool,
                    data: &[i16]);
}

#[derive(Clone, Copy)]
pub enum AudioType {
    Opus,
    Pcm,
}

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access is
/// always guarded.
pub struct Audio {
    pub playing: bool,
    pub volume: f32,
    pub finished: bool,
    pub source: Box<AudioSource>,
    pub position: Duration,
    pub position_modified: bool,
}

impl Audio {
    pub fn new(source: Box<AudioSource>) -> Self {
        Self {
            playing: true,
            volume: 1.0,
            finished: false,
            source,
            position: Duration::new(0, 0),
            position_modified: false,
        }
    }

    pub fn play(&mut self) -> &mut Self {
        self.playing = true;

        self
    }

    pub fn pause(&mut self) -> &mut Self {
        self.playing = false;

        self
    }

    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;

        self
    }

    pub fn position(&mut self, position: Duration) -> &mut Self {
        self.position = position;
        self.position_modified = true;

        self
    }

    pub fn step_frame(&mut self) {
        self.position += Duration::from_millis(20);
        self.position_modified = false;
    }
}

/// Threadsafe form of an instance of the [`Audio`] struct, locked behind a
/// Mutex.
///
/// [`Audio`]: struct.Audio.html
pub type LockedAudio = Arc<Mutex<Audio>>;
