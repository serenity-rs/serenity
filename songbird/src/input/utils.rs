use audiopus::{
    coder::Decoder,
    Channels,
    Result as OpusResult,
    SampleRate,
};
use crate::constants::*;
use std::{
    mem,
    time::Duration,
};

pub fn timestamp_to_sample_count(timestamp: Duration, stereo: bool) -> usize {
    ((timestamp.as_millis() as usize) * (MONO_FRAME_SIZE / FRAME_LEN_MS)) << stereo as usize
}

pub fn sample_count_to_timestamp(amt: usize, stereo: bool) -> Duration {
    Duration::from_millis((((amt * FRAME_LEN_MS) / MONO_FRAME_SIZE) as u64) >> stereo as u64)
}

pub fn timestamp_to_byte_count(timestamp: Duration, stereo: bool) -> usize {
    timestamp_to_sample_count(timestamp, stereo) * mem::size_of::<f32>()
}

pub fn byte_count_to_timestamp(amt: usize, stereo: bool) -> Duration {
    sample_count_to_timestamp(amt / mem::size_of::<f32>(), stereo)
}

/// Create an Opus decoder outputting at a sample rate of 48kHz.
pub fn decoder(stereo: bool) -> OpusResult<Decoder> {
    Decoder::new(
        SampleRate::Hz48000,
        if stereo { Channels::Stereo } else { Channels::Mono },
    )
}
