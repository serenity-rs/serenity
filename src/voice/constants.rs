/// Sampling rate of audio to be sent to Discord.
pub const SAMPLING_RATE: usize = 48_000;

/// Number of audio frames/packets to be sent per second.
pub const AUDIO_FRAME_RATE: usize = 50;

/// Number of samples in one complete frame of audio per channel.
pub const MONO_FRAME_SIZE: usize = SAMPLING_RATE / AUDIO_FRAME_RATE;

pub const STEREO_FRAME_SIZE: usize = 2 * MONO_FRAME_SIZE;

pub const MONO_FRAME_BYTE_SIZE: usize = MONO_FRAME_SIZE * std::mem::size_of::<f32>();

pub const STEREO_FRAME_BYTE_SIZE: usize = STEREO_FRAME_SIZE * std::mem::size_of::<f32>();

/// Length (in milliseconds) of any audio frame.
pub const FRAME_LEN_MS: usize = 1000 / AUDIO_FRAME_RATE;

/// Maximum number of audio frames/packets to be sent per second to be buffered.
pub const CHILD_BUFFER_LEN: usize = AUDIO_FRAME_RATE / 2;

/// Maximum packet size for a voice packet.
///
/// Set a safe amount below the Ethernet MTU to avoid framgnetation/rejection.
pub const VOICE_PACKET_MAX: usize = 1460;