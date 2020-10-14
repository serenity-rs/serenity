mod compressed;
mod hint;
mod memory;
#[cfg(test)]
mod tests;

pub use self::{
    compressed::*,
    hint::*,
    memory::*,
};

use crate::input::utils;
use crate::constants::*;
use audiopus::Bitrate;
use std::{
    mem,
    time::Duration,
};
use streamcatcher::{
    Config,
    GrowthStrategy,
};

pub fn compressed_cost_per_sec(bitrate: Bitrate) -> usize {
    let framing_cost_per_sec = AUDIO_FRAME_RATE * mem::size_of::<u16>();

    let bitrate_raw = match bitrate {
        Bitrate::BitsPerSecond(i) => i,
        Bitrate::Auto => 64_000,
        Bitrate::Max => 512_000,
    } as usize;

    (bitrate_raw / 8) + framing_cost_per_sec
}

pub fn raw_cost_per_sec(stereo: bool) -> usize {
    utils::timestamp_to_byte_count(Duration::from_secs(1), stereo)
}

pub fn default_config(cost_per_sec: usize) -> Config {
    Config::new().chunk_size(GrowthStrategy::Constant(5 * cost_per_sec))
}
