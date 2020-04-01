/// Sampling rate of audio to be sent to Discord.
pub const SAMPLING_RATE: usize = 48_000;

/// Number of audio frames/packets to be sent per second.
pub const AUDIO_FRAME_RATE: usize = 50;

/// Number of samples in one complete frame of audio per channel.
pub const MONO_FRAME_SIZE: usize = SAMPLING_RATE / AUDIO_FRAME_RATE;

pub const STEREO_FRAME_SIZE: usize = 2 * MONO_FRAME_SIZE;

/// Length (in milliseconds) of any audio frame.
pub const FRAME_LEN_MS: usize = 1000 / AUDIO_FRAME_RATE;