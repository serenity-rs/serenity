use audiopus::{Bitrate, SampleRate};

/// Sample rate of audio to be sent to Discord.
pub const SAMPLE_RATE: SampleRate = SampleRate::Hz48000;

/// Sample rate of audio to be sent to Discord.
pub const SAMPLE_RATE_RAW: usize = 48_000;

/// Number of audio frames/packets to be sent per second.
pub const AUDIO_FRAME_RATE: usize = 50;

/// Default bitrate for audio.
pub const DEFAULT_BITRATE: Bitrate = Bitrate::BitsPerSecond(128_000);

/// Number of samples in one complete frame of audio per channel.
pub const MONO_FRAME_SIZE: usize = SAMPLE_RATE_RAW / AUDIO_FRAME_RATE;

/// Number of samples in one complete frame of stereo audio.
pub const STEREO_FRAME_SIZE: usize = 2 * MONO_FRAME_SIZE;

/// Number of bytes in one complete frame of raw `f32`-encoded mono audio.
pub const MONO_FRAME_BYTE_SIZE: usize = MONO_FRAME_SIZE * std::mem::size_of::<f32>();

/// Number of bytes in one complete frame of raw `f32`-encoded stereo audio.
pub const STEREO_FRAME_BYTE_SIZE: usize = STEREO_FRAME_SIZE * std::mem::size_of::<f32>();

/// Length (in milliseconds) of any audio frame.
pub const FRAME_LEN_MS: usize = 1000 / AUDIO_FRAME_RATE;

/// Maximum number of audio frames/packets to be sent per second to be buffered.
pub const CHILD_BUFFER_LEN: usize = AUDIO_FRAME_RATE / 2;

/// Maximum packet size for a voice packet.
///
/// Set a safe amount below the Ethernet MTU to avoid framgnetation/rejection.
pub const VOICE_PACKET_MAX: usize = 1460;